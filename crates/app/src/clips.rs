//! Koppelt de ringbuffer-recorder (`fitcom_video::opname`) aan de motor (fase 15).
//!
//! Bewust een eigen module en niet verspreid over `engine.rs`: alles hier is
//! Windows-only — de recorder leunt op WASAPI-loopback voor het bureaubladgeluid, en
//! die heeft geen macOS-tegenhanger in deze fase (zie ROADMAP, fase 15). Op andere
//! platforms is dit een leeg beheerobject dat niets doet en zichzelf als afwezig meldt;
//! de motor maakt dezelfde aanroepen op beide platforms en de UI verbergt alles wat met
//! clips te maken heeft zodra de snapshot zegt dat ze er niet zijn.
//!
//! De globale hotkey hoort hier thuis en nergens anders: hij doet precies één ding
//! ("bewaar nu"), is instelbaar en kan herregistreerd worden zonder herstart.

use anyhow::{bail, Context, Result};

/// De lokale "Share"-kaart in het algemene kanaal voor de laatste clip (beslissing 44).
/// Geen op en nooit over de draad; de motor vult hem in de momentopname.
#[derive(Debug, Clone, PartialEq)]
pub struct ClipKaart {
    /// Wanneer de clip klaarkwam, in milliseconden sinds de epoch (zoals `created_at`).
    pub at: i64,
    pub grootte: u64,
    /// Al aangeboden (automatisch of met de knop): de kaart toont dat, zonder knop.
    pub gedeeld: bool,
}

/// Een scherm voor clips: zijn naam uit de bronlijst en zijn maat, als die bekend is.
pub type ClipMonitor = (String, Option<(u32, u32)>);

/// Wat de UI over de clipopname weet. `Option<ClipsWeergave> == None` in de snapshot
/// betekent: dit platform ondersteunt geen clips, verberg alles.
#[derive(Debug, Clone)]
pub struct ClipsWeergave {
    pub aanwezig: bool,
    pub aan: bool,
    pub venster_sec: u32,
    /// Welk scherm er opgenomen wordt (naam uit de bronlijst).
    pub monitor: Option<String>,
    /// De ingestelde sneltoets, zoals hij in de config staat.
    pub hotkey: String,
    pub map: String,
    pub laatste: Option<String>,
    pub fout: Option<String>,
    /// De instelling "clips meteen in de chat zetten".
    pub auto_delen: bool,
    /// Beeldtempo, resolutie en kwaliteit zoals in de config (`ClipsConfig`).
    pub fps: u32,
    pub resolutie: String,
    pub kwaliteit: String,
    /// De knop "Share last clip" mag werken: er is een clip, hij staat nog op schijf en
    /// is niet al aangeboden. Door de motor ingevuld; het beheer weet het niet.
    pub kan_delen: bool,
    pub kaart: Option<ClipKaart>,
}

/// Ontleedt een sneltoets zoals `F9`, `ctrl+alt+c` of `shift+f2` naar
/// `(modifier-bits, virtual key)`. De modifier-bits zijn precies Windows'
/// `HOT_KEY_MODIFIERS`: alt=1, ctrl=2, shift=4, win=8. Platform-onafhankelijk zodat
/// de config-validatie ook op een mac kan draaien.
pub fn ontled_hotkey(spec: &str) -> Result<(u32, u32)> {
    let mut mods = 0u32;
    let mut vk: Option<u32> = None;
    for deel in spec.split('+') {
        match deel.trim().to_ascii_lowercase().as_str() {
            "" => continue,
            "ctrl" | "control" => mods |= 0x2,
            "alt" => mods |= 0x1,
            "shift" => mods |= 0x4,
            "win" | "windows" => mods |= 0x8,
            toets if is_f_toets(toets) => {
                let n: u32 = toets[1..].parse().context("f-toetsnummer")?;
                vk = Some(0x70 + n - 1); // VK_F1 = 0x70, oplopend
            }
            letter if letter.chars().count() == 1 => {
                let ch = letter.to_uppercase().chars().next().unwrap();
                let code = u32::from(ch);
                if !(code as u8).is_ascii_alphanumeric() {
                    bail!("onbekende toets '{deel}'");
                }
                vk = Some(code);
            }
            _ => bail!("onbekende toets '{deel}'"),
        }
    }
    let vk = vk.context("geen toets in de sneltoets (bijv. \"F9\")")?;
    Ok((mods, vk))
}

fn is_f_toets(d: &str) -> bool {
    d.len() >= 2 && d.starts_with('f') && d[1..].parse::<u32>().is_ok_and(|n| (1..=24).contains(&n))
}

// ---------------------------------------------------------------- clipbeeld (beslissing 57)
//
// Eén plek voor wat een keuze kost: de motor zet er de encoder mee op, het scherm rekent
// er de schatting naast elke optie mee uit. Platformonafhankelijk, zoals `ontled_hotkey`.

/// Beeldtempo's voor clips. Boven de verversing van het scherm levert de opname gewoon
/// de verversing; WGC levert alleen wat het scherm tekent.
pub const CLIP_FPS: [u32; 4] = [30, 60, 120, 144];

/// Resoluties: naam in de config, label, doelhoogte (`None` = die van het scherm).
pub const RESOLUTIES: [(&str, &str, Option<u32>); 4] = [
    ("native", "Native", None),
    ("1440p", "1440p", Some(1440)),
    ("1080p", "1080p", Some(1080)),
    ("720p", "720p", Some(720)),
];

/// Kwaliteiten: naam in de config, label, Mbit/s bij 1080p60. De bitrate schaalt lineair
/// mee met pixels per seconde, zodat een keuze bij elke maat en elk tempo even scherp is.
pub const KWALITEITEN: [(&str, &str, u32); 4] = [
    ("low", "Low", 8),
    ("medium", "Medium", 15),
    ("high", "High", 30),
    ("very_high", "Very high", 50),
];

/// Plafond van de clipbitrate: ruim onder de 240 Mbit/s van H.264 High@5.2, en al
/// ~1,1 GB per minuut. 4K op "very high" loopt hier tegenaan; hoger kan pas met HEVC-clips.
const MAX_CLIP_BITRATE: u64 = 150_000_000;

/// De AAC-track van elke clip: 192 kbit/s, zoals `opname::AAC_BYTES_PER_SEC`.
const CLIP_AUDIO_BITRATE: u64 = 192_000;

/// De maat waarop een clip gecodeerd wordt: op hoogte geschaald met de beeldverhouding
/// van het scherm, nooit groter dan het scherm, en even (H.264 met 4:2:0).
pub fn doel_afmeting(scherm: (u32, u32), resolutie: &str) -> (u32, u32) {
    let doel = RESOLUTIES
        .iter()
        .find(|r| r.0 == resolutie)
        .and_then(|r| r.2)
        .filter(|&h| h < scherm.1);
    let (b, h) = match doel {
        Some(h) => (
            (u64::from(scherm.0) * u64::from(h) / u64::from(scherm.1)) as u32,
            h,
        ),
        None => scherm,
    };
    ((b & !1).max(2), (h & !1).max(2))
}

/// Bits per seconde voor een clip van deze maat en dit tempo. Een onbekende kwaliteit
/// (kan alleen van buiten `herstel` komen) telt als "high".
pub fn clip_bitrate(afmeting: (u32, u32), fps: u32, kwaliteit: &str) -> u32 {
    let mbit = KWALITEITEN
        .iter()
        .find(|k| k.0 == kwaliteit)
        .map_or(30, |k| k.2);
    let pixels_per_sec = u64::from(afmeting.0) * u64::from(afmeting.1) * u64::from(fps.max(1));
    let bits = u64::from(mbit) * 1_000_000 * pixels_per_sec / (1920 * 1080 * 60);
    bits.min(MAX_CLIP_BITRATE) as u32
}

/// Megabytes per minuut clip, beeld plus geluid. Een bovengrens in de praktijk: een
/// stilstaand bureaublad haalt de bitrate niet, een druk spel wel.
pub fn mb_per_minuut(bitrate: u32) -> u64 {
    (u64::from(bitrate) + CLIP_AUDIO_BITRATE) * 60 / 8 / 1_000_000
}

// ---------------------------------------------------------------- windows

#[cfg(windows)]
mod backend {
    use super::{ontled_hotkey, ClipsWeergave};
    use crate::config::ClipsConfig;
    use anyhow::{bail, Context, Result};
    use fitcom_audio::loopback::LoopbackTap;
    use fitcom_audio::microfoon::MicrofoonTap;
    use fitcom_video::capture::{afmeting_van, beschikbare_bronnen, Bron, BronSoort};
    use fitcom_video::opname::{AudioBronnen, ClipGebeurtenis, ClipInstellingen, OpnameHandle};
    use fitcom_video::D3dContext;
    use std::path::PathBuf;
    use std::sync::mpsc;

    pub struct ClipsBeheer {
        handle: Option<OpnameHandle>,
        gebeurtenissen: mpsc::Receiver<ClipGebeurtenis>,
        /// Houdt de geluids-taps levend zolang de opname loopt; hun Drop stopt de
        /// capturedraden. Los van `handle` omdat het mislukken van het geluid de
        /// videoketen niet hoeft te blokkeren.
        tap_systeem: Option<LoopbackTap>,
        tap_microfoon: Option<MicrofoonTap>,
        laatste: Option<PathBuf>,
        fout: Option<String>,
        map: PathBuf,
    }

    impl ClipsBeheer {
        pub fn nieuw(map: PathBuf) -> Self {
            let (_dode_tx, rx) = mpsc::channel();
            Self {
                handle: None,
                gebeurtenissen: rx,
                tap_systeem: None,
                tap_microfoon: None,
                laatste: None,
                fout: None,
                map,
            }
        }

        pub fn aanwezig(&self) -> bool {
            true
        }

        /// Waar clips en de ring vanaf nu heen gaan. De beller (de motor) herstart een
        /// lopende opname zelf — deze klasse verhuist alleen het doel, want de
        /// opnamedraad heeft zijn paden al bij het starten gekregen.
        ///
        /// De ring van de oude map wordt hier opgeruimd: die segmenten dragen tijden van
        /// deze sessie en worden nooit meer gelezen, dus ze zouden voorgoed blijven
        /// liggen. Alleen `seg-*` gaat weg; clips die er al staan blijven staan.
        pub fn zet_map(&mut self, map: PathBuf) {
            if self.map != map {
                fitcom_video::opname::leeg_ring(&self.map.join("ring"));
            }
            self.map = map;
        }

        /// De huidige map, zodat de motor de oude ring kan opruimen vóór een verhuizing.
        pub fn map(&self) -> &std::path::Path {
            &self.map
        }

        pub fn aan(&self) -> bool {
            self.handle.is_some()
        }

        /// Aan of uit, idempotent. Uitzetten gaat via de Drop van de handle; wat er in
        /// de ringmap blijft liggen wordt bij de volgende start weggegooid — de tijden
        /// in die bestanden komen van een klok die dan opnieuw op nul begint. Zie de
        /// moduledoc van `fitcom_video::opname`.
        ///
        /// `cfg.monitor` kiest welk scherm; `None` of een onbekende naam valt terug
        /// op het eerste gevonden scherm. `mic_naam` is de microfoon uit de
        /// instellingen — dezelfde die het gesprek gebruikt, want je verwacht in je clip
        /// terug te horen wat de anderen ook horen.
        pub fn zet(
            &mut self,
            aan: bool,
            cfg: &ClipsConfig,
            mic_naam: Option<&str>,
            d3d: Option<&D3dContext>,
        ) -> Result<()> {
            match (aan, self.handle.is_some()) {
                (false, false) | (true, true) => return Ok(()),
                (false, true) => {
                    tracing::info!("clipopname uit");
                    self.handle = None; // Drop stopt de keten netjes achteraan
                    self.tap_systeem = None;
                    self.tap_microfoon = None;
                    return Ok(());
                }
                (true, false) => {}
            }
            let Some(d3d) = d3d else {
                bail!("geen grafische kaart voor de clipopname");
            };

            let bron = kies_monitor(cfg.monitor.as_deref())?;
            let scherm = afmeting_van(&bron).context("schermmaat opvragen")?;
            let afmeting = super::doel_afmeting(scherm, &cfg.resolution);
            let instellingen = ClipInstellingen {
                fps: cfg.fps,
                bitrate: super::clip_bitrate(afmeting, cfg.fps, &cfg.quality),
                afmeting: Some(afmeting),
                venster_sec: cfg.venster_sec,
            };

            // Geluid erbij als het kan — systeem/spel via de loopback, je eigen stem
            // via de microfoon. Elke bron mag apart falen: een clip zonder microfoon
            // is nog steeds een clip met spelgeluid.
            self.tap_systeem = None;
            self.tap_microfoon = None;
            let mut audio = AudioBronnen::default();
            match LoopbackTap::start() {
                Ok((tap, _rate, ontvangen)) => {
                    audio.systeem = Some(ontvangen);
                    self.tap_systeem = Some(tap);
                }
                Err(e) => tracing::warn!(
                    error = %format!("{e:#}"),
                    "systeemgeluid voor clips niet beschikbaar"
                ),
            }
            match MicrofoonTap::start(mic_naam) {
                Ok((tap, ontvangen)) => {
                    audio.microfoon = Some(ontvangen);
                    self.tap_microfoon = Some(tap);
                }
                Err(e) => tracing::info!(
                    error = %format!("{e:#}"),
                    "microfoon voor clips niet beschikbaar"
                ),
            }
            if !audio.heeft_bron() {
                tracing::warn!("geen geluidsbron voor clips; alleen beeld");
            }

            tracing::info!(
                ?scherm,
                ?afmeting,
                fps = instellingen.fps,
                bitrate = instellingen.bitrate,
                "clipbeeld"
            );
            let (tx, rx) = mpsc::channel();
            self.gebeurtenissen = rx;
            self.fout = None;
            let ring_dir = self.map.join("ring");
            let handle = fitcom_video::opname::start_opname(
                d3d,
                &bron,
                instellingen,
                ring_dir,
                self.map.clone(),
                Some(audio),
                tx,
            )
            .context("clipopname starten")?;
            tracing::info!(bron = %bron.naam, "clipopname aan");
            self.handle = Some(handle);
            Ok(())
        }

        pub fn bewaar_nu(&mut self) {
            match &self.handle {
                Some(h) => h.bewaar_nu(),
                None => self.fout = Some("Clips staan uit.".into()),
            }
        }

        /// De laatste opgetreden clipfout, als die er nog staat.
        pub fn fout(&self) -> Option<String> {
            self.fout.clone()
        }

        /// Even vaak als de motortik: events binnenhalen en een doodlopende keten
        /// signaleren. Levert het pad van een clip die déze tik klaarkwam, zodat de
        /// motor daar zijn geluidje aan kan hangen. Fouten blijven staan tot de
        /// volgende keer aanzetten.
        pub fn tik(&mut self) -> Option<PathBuf> {
            let mut net_klaar = None;
            while let Ok(ev) = self.gebeurtenissen.try_recv() {
                match ev {
                    ClipGebeurtenis::Klaar { pad } => {
                        tracing::debug!(pad = %pad.display(), "clip klaar");
                        net_klaar = Some(pad.clone());
                        self.laatste = Some(pad);
                    }
                    ClipGebeurtenis::Mislukt { reden } => {
                        tracing::warn!(reden = %reden, "clip mislukt");
                        self.fout = Some(reden);
                    }
                }
            }
            if let Some(h) = &self.handle {
                if h.gestopt() {
                    if let Some(f) = h.fout() {
                        self.fout = Some(f);
                    }
                    self.handle = None;
                    self.tap_systeem = None;
                    self.tap_microfoon = None;
                }
            }
            net_klaar
        }

        pub fn weergave(&self, cfg: &ClipsConfig) -> ClipsWeergave {
            ClipsWeergave {
                aanwezig: true,
                aan: self.aan(),
                venster_sec: cfg.venster_sec,
                monitor: cfg.monitor.clone(),
                hotkey: cfg.hotkey.clone(),
                map: self.map.display().to_string(),
                laatste: self.laatste.as_ref().map(|p| p.display().to_string()),
                fout: self.fout.clone(),
                auto_delen: cfg.auto_delen,
                fps: cfg.fps,
                resolutie: cfg.resolution.clone(),
                kwaliteit: cfg.quality.clone(),
                kan_delen: false,
                kaart: None,
            }
        }

        /// De laatste clip van deze sessie die klaar (volledig geschreven) is.
        pub fn laatste(&self) -> Option<PathBuf> {
            self.laatste.clone()
        }
    }

    /// Het gekozen scherm, op naam; zonder keuze (of bij een naam die nergens meer
    /// bij hoort — monitors veranderen) het eerste gevonden.
    fn kies_monitor(naam: Option<&str>) -> Result<Bron> {
        let bronnen = beschikbare_bronnen().context("bronnen opvragen")?;
        let monitoren: Vec<Bron> = bronnen
            .into_iter()
            .filter(|b| b.soort == BronSoort::Monitor)
            .collect();
        monitoren
            .iter()
            .find(|b| Some(b.naam.as_str()) == naam)
            .or_else(|| monitoren.first())
            .cloned()
            .context("geen scherm gevonden om op te nemen")
    }

    /// Een lopende hotkey-registratie. Vallen laten heft de toets op — de berichtenlus
    /// krijgt WM_QUIT en doet zelf zijn UnregisterHotKey op de juiste thread.
    ///
    /// De Drop *wacht* op die draad. Zonder dat wachten mislukt het opnieuw instellen
    /// van dezelfde toets: `RegisterHotKey` weigert zolang de oude registratie nog
    /// leeft, en dan heeft niemand meer een sneltoets.
    pub struct HotkeyDraad {
        thread_id: u32,
        draad: Option<std::thread::JoinHandle<()>>,
    }

    impl HotkeyDraad {
        pub fn wissel_naar(
            nieuw_spec: &str,
            op_te_doen: impl Fn() + Send + 'static,
        ) -> Result<Self> {
            // Eerst ontleden; een typefout mag de oude toets niet slopen.
            let (mods_raw, vk) = ontled_hotkey(nieuw_spec)?;
            Self::start_met(mods_raw, vk, op_te_doen)
        }

        fn start_met(
            mods_raw: u32,
            vk: u32,
            op_te_doen: impl Fn() + Send + 'static,
        ) -> Result<Self> {
            use windows::Win32::System::Threading::GetCurrentThreadId;
            // De draad meldt terug of de registratie lukte: een toets die al door een
            // ander programma bezet is moet je zien, niet alleen in de log vinden.
            let (id_tx, id_rx) = mpsc::sync_channel::<(u32, Option<String>)>(1);

            let draad = std::thread::Builder::new()
                .name("fitcom-hotkey".into())
                .spawn(move || {
                    // SAFETY: thread-gebonden aanroep vóór de lus; geen parameters.
                    let thread_id = unsafe { GetCurrentThreadId() };
                    hotkey_lus(mods_raw, vk, ID_HOTKEY, op_te_doen, &|fout| {
                        let _ = id_tx.send((thread_id, fout));
                    });
                })
                .context("hotkey-draad starten")?;

            let (thread_id, fout) = id_rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .context("geen antwoord van de hotkey-draad")?;
            if let Some(f) = fout {
                let _ = draad.join();
                bail!("{f}");
            }
            Ok(Self {
                thread_id,
                draad: Some(draad),
            })
        }

        /// Een handle zonder draad: de hotkey staat niet aan, maar alles om hem heen
        /// (config, UI) blijft gewoon werken.
        pub fn dode() -> Self {
            Self {
                thread_id: 0,
                draad: None,
            }
        }
    }

    impl Drop for HotkeyDraad {
        fn drop(&mut self) {
            let Some(draad) = self.draad.take() else {
                return;
            };
            use windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW;
            use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;
            // SAFETY: WM_QUIT naar de hotkey-draad; haar GetMessageW valt daarop terug
            // en unregister't haar eigen hotkey vóór ze stopt.
            unsafe {
                let _ = PostThreadMessageW(
                    self.thread_id,
                    WM_QUIT,
                    windows::Win32::Foundation::WPARAM(0),
                    windows::Win32::Foundation::LPARAM(0),
                );
            }
            // Wachten tot de UnregisterHotKey er echt doorheen is: de volgende
            // registratie kan dezelfde toets willen.
            let _ = draad.join();
        }
    }

    const ID_HOTKEY: i32 = 1;

    fn hotkey_lus(
        mods_raw: u32,
        vk: u32,
        id: i32,
        op_te_doen: impl Fn() + Send + 'static,
        meld: &dyn Fn(Option<String>),
    ) {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY,
        };
        // SAFETY: thread-gebonden hotkey met uniek id binnen dit proces.
        let geregistreerd =
            unsafe { RegisterHotKey(None, id, HOT_KEY_MODIFIERS(mods_raw), vk).is_ok() };
        if !geregistreerd {
            let naam = super::hotkey_naam(mods_raw, vk);
            tracing::warn!(spec = %naam, "de clip-sneltoets is al ergens anders in gebruik");
            meld(Some(format!("{naam} is al ergens anders in gebruik")));
            return;
        }
        meld(None);
        let mut msg = MSG::default();
        // SAFETY: standaard berichtenlus; WM_QUIT (van de Drop van HotkeyDraad) maakt
        // hem netjes af, mét unregister op deze thread.
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                if msg.message == WM_HOTKEY && msg.wParam.0 == id as usize {
                    op_te_doen();
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let _ = UnregisterHotKey(None, id);
        }
    }

    pub fn start_hotkey(spec: &str, op_te_doen: impl Fn() + Send + 'static) -> Result<HotkeyDraad> {
        HotkeyDraad::wissel_naar(spec, op_te_doen)
    }

    /// Monitoren die voor clips gekozen kunnen worden, in bronvolgorde, met hun maat voor
    /// de schatting in het scherm (`None` als Windows hem niet gaf).
    pub fn monitoren() -> Result<Vec<super::ClipMonitor>> {
        let bronnen = beschikbare_bronnen().context("bronnen opvragen")?;
        Ok(bronnen
            .into_iter()
            .filter(|b| b.soort == BronSoort::Monitor)
            .map(|b| {
                let maat = afmeting_van(&b).ok();
                (b.naam, maat)
            })
            .collect())
    }
}

#[cfg(windows)]
pub use backend::{monitoren, start_hotkey, ClipsBeheer, HotkeyDraad};

// ---------------------------------------------------------------- andere platforms

#[cfg(not(windows))]
mod backend {
    use super::ClipsWeergave;
    use crate::config::ClipsConfig;
    use anyhow::Result;
    use fitcom_video::D3dContext;
    use std::path::PathBuf;

    pub struct ClipsBeheer {
        map: PathBuf,
    }

    impl ClipsBeheer {
        pub fn nieuw(map: PathBuf) -> Self {
            Self { map }
        }
        pub fn aanwezig(&self) -> bool {
            false
        }
        pub fn zet_map(&mut self, map: PathBuf) {
            self.map = map;
        }
        #[allow(clippy::unused_self)]
        pub fn map(&self) -> &std::path::Path {
            &self.map
        }
        #[allow(clippy::unused_self)]
        pub fn aan(&self) -> bool {
            false
        }
        #[allow(clippy::needless_pass_by_ref_mut)]
        pub fn zet(
            &mut self,
            _aan: bool,
            _cfg: &ClipsConfig,
            _mic_naam: Option<&str>,
            _d3d: Option<&D3dContext>,
        ) -> Result<()> {
            Ok(())
        }
        #[allow(clippy::unused_self)]
        pub fn bewaar_nu(&mut self) {}
        /// Er is hier geen opnameketen, dus er valt ook nooit iets te melden.
        #[allow(clippy::unused_self)]
        pub fn fout(&self) -> Option<String> {
            None
        }
        pub fn tik(&mut self) -> Option<std::path::PathBuf> {
            None
        }
        pub fn weergave(&self, cfg: &ClipsConfig) -> ClipsWeergave {
            ClipsWeergave {
                aanwezig: false,
                aan: false,
                venster_sec: cfg.venster_sec,
                monitor: cfg.monitor.clone(),
                hotkey: cfg.hotkey.clone(),
                map: self.map.display().to_string(),
                laatste: None,
                fout: None,
                auto_delen: cfg.auto_delen,
                fps: cfg.fps,
                resolutie: cfg.resolution.clone(),
                kwaliteit: cfg.quality.clone(),
                kan_delen: false,
                kaart: None,
            }
        }
        pub fn laatste(&self) -> Option<std::path::PathBuf> {
            None
        }
    }

    pub struct HotkeyDraad;

    impl HotkeyDraad {
        pub fn dode() -> Self {
            Self
        }
    }

    impl HotkeyDraad {
        pub fn wissel_naar(
            _nieuw_spec: &str,
            _op_te_doen: impl Fn() + Send + 'static,
        ) -> Result<Self> {
            Ok(Self)
        }
    }

    pub fn start_hotkey(
        _spec: &str,
        _op_te_doen: impl Fn() + Send + 'static,
    ) -> Result<HotkeyDraad> {
        Ok(HotkeyDraad)
    }

    pub fn monitoren() -> Result<Vec<super::ClipMonitor>> {
        Ok(Vec::new())
    }
}

#[cfg(not(windows))]
pub use backend::{monitoren, start_hotkey, ClipsBeheer, HotkeyDraad};

/// Leesbare vorm voor in logs: welke toets bedoelden we. Alleen voor diagnostiek.
#[allow(dead_code)]
fn hotkey_naam(mods_raw: u32, vk: u32) -> String {
    let mut delen: Vec<String> = Vec::new();
    if mods_raw & 0x2 != 0 {
        delen.push("ctrl".to_string());
    }
    if mods_raw & 0x1 != 0 {
        delen.push("alt".to_string());
    }
    if mods_raw & 0x4 != 0 {
        delen.push("shift".to_string());
    }
    if mods_raw & 0x8 != 0 {
        delen.push("win".to_string());
    }
    if (0x70..=0x87).contains(&vk) {
        delen.push(format!("F{}", vk - 0x70 + 1)); // al String
    } else if let Some(c) = char::from_u32(vk) {
        delen.push(c.to_string());
    }
    delen.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn een_clip_schaalt_op_hoogte_nooit_omhoog_en_altijd_even() {
        assert_eq!(doel_afmeting((2560, 1440), "1080p"), (1920, 1080));
        assert_eq!(doel_afmeting((3440, 1440), "1080p"), (2580, 1080));
        assert_eq!(doel_afmeting((3840, 2160), "native"), (3840, 2160));
        // Een kleiner scherm wordt niet opgeblazen.
        assert_eq!(doel_afmeting((1920, 1080), "1440p"), (1920, 1080));
        // 1366 × 720 / 768 = 1280,6: naar beneden, en even.
        assert_eq!(doel_afmeting((1366, 768), "720p"), (1280, 720));
        assert_eq!(doel_afmeting((1365, 767), "native"), (1364, 766));
    }

    #[test]
    fn de_bitrate_volgt_de_pixels_per_seconde_tot_het_plafond() {
        assert_eq!(clip_bitrate((1920, 1080), 60, "high"), 30_000_000);
        assert_eq!(clip_bitrate((1920, 1080), 30, "high"), 15_000_000);
        assert_eq!(clip_bitrate((3840, 2160), 60, "medium"), 60_000_000);
        assert_eq!(clip_bitrate((3840, 2160), 60, "very_high"), 150_000_000);
        assert_eq!(clip_bitrate((1920, 1080), 60, "onbekend"), 30_000_000);
        // 30 Mbit beeld plus 192 kbit geluid, een minuut lang.
        assert_eq!(mb_per_minuut(30_000_000), 226);
    }

    #[test]
    fn elke_standaard_staat_in_zijn_tabel() {
        let c = crate::config::ClipsConfig::default();
        assert!(CLIP_FPS.contains(&c.fps));
        assert!(RESOLUTIES.iter().any(|r| r.0 == c.resolution));
        assert!(KWALITEITEN.iter().any(|k| k.0 == c.quality));
    }
}
