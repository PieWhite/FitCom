//! The Clips tab: the recording switch, which screen, frame rate, resolution and quality
//! (each option with its disk cost), the global shortcut, the folder, and saving a clip now.

use super::controls::{
    auto_width, code_inline, control_row, help, input_box, label, select, switch, Body,
};
use super::{SettingsPane, AUTO_MONITOR};
use crate::clips::{
    clip_bitrate, doel_afmeting, mb_per_minuut, ClipMonitor, CLIP_FPS, KWALITEITEN, RESOLUTIES,
};
use crate::ui::commands;
use crate::ui::icons::Icon;
use crate::ui::state::ClipsState;
use crate::ui::theme;
use crate::ui::view::widgets::{Btn, BtnKind};
use crate::ui::view::AppSnapshot;
use gpui_kit::{
    div, prelude::FluentBuilder, px, Context, InteractiveElement, KeyDownEvent, Modifiers,
    ModifiersChangedEvent, ParentElement, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

/// The clip lengths on offer, in seconds. Up to two hours; the buffer on disk holds as
/// much as the clip is long.
const LENGTHS: [u32; 10] = [30, 60, 120, 300, 600, 900, 1800, 3600, 5400, 7200];

/// "30 seconds", "1 minute", "15 minutes".
pub(super) fn length_label(sec: u32) -> String {
    match sec {
        s if !s.is_multiple_of(60) => format!("{s} seconds"),
        60 => "1 minute".into(),
        s => format!("{} minutes", s / 60),
    }
}

/// Between an option's name and its estimate. A choice is read back from the part before it,
/// so a label may change with the other settings without losing what it names.
const SEP: &str = " \u{b7} ";

fn name_part(label: &str) -> &str {
    label.split(SEP).next().unwrap_or(label)
}

/// "225 MB", "2.7 GB".
fn size_label(mb: u64) -> String {
    if mb < 1000 {
        format!("{mb} MB")
    } else {
        format!("{:.1} GB", mb as f64 / 1000.)
    }
}

fn per_minute(bitrate: u32) -> String {
    format!("~{}/min", size_label(mb_per_minuut(bitrate)))
}

/// The screen the engine will record, for the estimates: the chosen one, else the first
/// (as the engine falls back), else 1080p while the list or its size is unknown.
fn screen_size(clips: &ClipsState, monitors: Option<&Vec<ClipMonitor>>) -> (u32, u32) {
    monitors
        .and_then(|list| {
            list.iter()
                .find(|(name, _)| Some(name) == clips.monitor.as_ref())
                .or_else(|| list.first())
        })
        .and_then(|(_, size)| *size)
        .unwrap_or((1920, 1080))
}

/// "60 fps · ~225 MB/min" back to 60.
pub(super) fn fps_of(label: &str) -> Option<u32> {
    name_part(label).strip_suffix(" fps")?.parse().ok()
}

/// "1080p · 1920×1080 · ~225 MB/min" back to the config name.
pub(super) fn resolution_of(label: &str) -> Option<&'static str> {
    let name = name_part(label);
    RESOLUTIES.iter().find(|r| r.1 == name).map(|r| r.0)
}

/// "High · 30 Mbit/s · ~225 MB/min" back to the config name.
pub(super) fn quality_of(label: &str) -> Option<&'static str> {
    let name = name_part(label);
    KWALITEITEN.iter().find(|k| k.1 == name).map(|k| k.0)
}

/// The other way round, for the select's choice. Accepts any label `length_label` makes,
/// so a length set by hand in `config.toml` survives being re-chosen.
pub(super) fn length_seconds(label: &str) -> Option<u32> {
    let (n, unit) = name_part(label).split_once(' ')?;
    let n: u32 = n.parse().ok()?;
    Some(if unit.starts_with("second") {
        n
    } else {
        n * 60
    })
}

/// "30s", "5 min": short enough for the button.
fn length_short(sec: u32) -> String {
    if sec.is_multiple_of(60) {
        format!("{} min", sec / 60)
    } else {
        format!("{sec}s")
    }
}

/// The modifiers held, in the order the engine's shortcut format writes them. `platform`
/// (Cmd on a Mac, the Windows key on Windows) is `win` on the wire.
fn modifier_names(m: &Modifiers) -> Vec<&'static str> {
    [
        (m.control, "ctrl"),
        (m.alt, "alt"),
        (m.shift, "shift"),
        (m.platform, "win"),
    ]
    .into_iter()
    .filter_map(|(held, name)| held.then_some(name))
    .collect()
}

/// A key press as a shortcut string, or `None` for a key the wire format does not know
/// (arrows, punctuation, a lone modifier): the box keeps waiting instead of saving
/// something unusable. `key` is GPUI's unshifted lower-case key name.
pub(super) fn hotkey_name(m: &Modifiers, key: &str) -> Option<String> {
    let key = key.to_ascii_lowercase();
    let key = key.as_str();
    let function = key
        .strip_prefix('f')
        .and_then(|n| n.parse::<u8>().ok())
        .is_some_and(|n| (1..=24).contains(&n) && key.len() <= 3);
    let single = key.len() == 1
        && key
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit());
    if !(function || single) {
        return None;
    }
    let mut parts = modifier_names(m);
    parts.push(key);
    Some(parts.join("+"))
}

impl SettingsPane {
    pub(super) fn clips(
        &mut self,
        snap: &AppSnapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Body {
        let Some(clips) = snap.s.clips.as_ref() else {
            return Body::new()
                .h2("Clips")
                .p("Clip recording is not available on this platform.");
        };
        let hotkey = clips.hotkey.to_uppercase();

        // Before the list of screens arrived the select shows just the chosen one.
        let (items, wanted) = match &snap.v.clip_monitors {
            None => {
                let only = clips.monitor.clone().unwrap_or_else(|| "automatic".into());
                (vec![only.clone()], only)
            }
            Some(monitors) => {
                let mut items = vec![AUTO_MONITOR.to_string()];
                items.extend(monitors.iter().map(|(name, _)| name.clone()));
                let chosen = clips
                    .monitor
                    .as_ref()
                    .filter(|m| monitors.iter().any(|(name, _)| name == *m));
                let wanted = chosen.cloned().unwrap_or_else(|| AUTO_MONITOR.to_string());
                (items, wanted)
            }
        };
        let monitor_width = auto_width(&items, 280.);
        self.monitor.sync(items, &wanted, window, cx);

        // Every option carries what it costs at the other two settings, on the screen that
        // gets recorded; the same policy the engine sets the encoder with.
        let screen = screen_size(clips, snap.v.clip_monitors.as_ref());
        let size = doel_afmeting(screen, &clips.resolution);
        let bitrate = clip_bitrate(size, clips.fps, &clips.quality);
        let per_min = mb_per_minuut(bitrate);

        let fps_label = |f: u32| {
            format!(
                "{f} fps{SEP}{}",
                per_minute(clip_bitrate(size, f, &clips.quality))
            )
        };
        let fps_items: Vec<String> = CLIP_FPS.iter().map(|&f| fps_label(f)).collect();
        let fps_width = auto_width(&fps_items, 160.);
        self.clip_fps
            .sync(fps_items, &fps_label(clips.fps), window, cx);

        let resolution_label = |r: &(&str, &str, Option<u32>)| {
            let s = doel_afmeting(screen, r.0);
            let b = clip_bitrate(s, clips.fps, &clips.quality);
            format!("{}{SEP}{}\u{d7}{}{SEP}{}", r.1, s.0, s.1, per_minute(b))
        };
        let resolution_items: Vec<String> = RESOLUTIES.iter().map(resolution_label).collect();
        let resolution_width = auto_width(&resolution_items, 160.);
        let resolution_now = RESOLUTIES
            .iter()
            .find(|r| r.0 == clips.resolution)
            .map_or_else(|| resolution_items[0].clone(), resolution_label);
        self.clip_resolution
            .sync(resolution_items, &resolution_now, window, cx);

        let quality_label = |k: &(&str, &str, u32)| {
            let b = clip_bitrate(size, clips.fps, k.0);
            format!(
                "{}{SEP}{:.0} Mbit/s{SEP}{}",
                k.1,
                f64::from(b) / 1e6,
                per_minute(b)
            )
        };
        let quality_items: Vec<String> = KWALITEITEN.iter().map(quality_label).collect();
        let quality_width = auto_width(&quality_items, 160.);
        let quality_now = KWALITEITEN
            .iter()
            .find(|k| k.0 == clips.quality)
            .map_or_else(|| quality_items[0].clone(), quality_label);
        self.clip_quality
            .sync(quality_items, &quality_now, window, cx);

        // A length from `config.toml` that is not on the list still shows, as its own item.
        let mut lengths: Vec<u32> = LENGTHS.to_vec();
        if !lengths.contains(&clips.window_sec) {
            lengths.push(clips.window_sec);
            lengths.sort_unstable();
        }
        let length_option = |sec: u32| {
            let mb = per_min * u64::from(sec) / 60;
            format!("{}{SEP}~{}", length_label(sec), size_label(mb))
        };
        let length_items: Vec<String> = lengths.iter().map(|&s| length_option(s)).collect();
        let length_width = auto_width(&length_items, 160.);
        self.clip_length
            .sync(length_items, &length_option(clips.window_sec), window, cx);

        let waiting = snap.v.hotkey_wait;
        // The debug view sets `hotkey_wait` without a click: hand the box the keyboard.
        #[cfg(debug_assertions)]
        if waiting && snap.v.dialog.is_none() && !self.hotkey_focus.is_focused(window) {
            window.focus(&self.hotkey_focus, cx);
        }
        let shown = match (waiting, self.pending_mods.is_empty()) {
            (false, _) => hotkey.clone(),
            (true, true) => "Press keys\u{2026} Esc cancels".to_string(),
            (true, false) => format!("{}+\u{2026}", self.pending_mods),
        };
        let capture = input_box(
            "clips-hotkey-capture",
            if waiting {
                theme::ACCENT
            } else {
                theme::BORDER_HOVER
            },
        )
        .min_w(px(200.))
        .track_focus(&self.hotkey_focus)
        .cursor_pointer()
        .when(waiting, |d| {
            d.border_color(theme::ACCENT).text_color(theme::TEXT_STRONG)
        })
        .on_click(cx.listener(|this, _, window, cx| {
            this.pending_mods.clear();
            window.focus(&this.hotkey_focus, cx);
            this.app.update(cx, |app, cx| {
                app.v_mut().hotkey_wait = true;
                cx.notify();
            });
        }))
        .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| this.hotkey_pressed(ev, cx)))
        .on_modifiers_changed(cx.listener(|this, ev: &ModifiersChangedEvent, _, cx| {
            this.pending_mods = modifier_names(&ev.modifiers).join("+");
            cx.notify();
        }))
        .child(shown);

        let (ui, enabled) = (snap.ui.clone(), clips.enabled);
        let toggle = Rc::new(move |_: &_, _: &mut _, _: &mut _| commands::set_clips(&ui, !enabled));
        let (ui_pick, ui_now, ui_open) = (snap.ui.clone(), snap.ui.clone(), snap.ui.clone());
        let (ui_share, ui_auto, auto_share) = (snap.ui.clone(), snap.ui.clone(), clips.auto_share);
        let auto_toggle = Rc::new(move |_: &_, _: &mut _, _: &mut _| {
            commands::set_clips_auto_share(&ui_auto, !auto_share)
        });

        let mut body = Body::new()
            .h2("Clips")
            .p(format!(
                concat!(
                    "While enabled, a screen is recorded into a rolling buffer that never leaves ",
                    "your machine. Press {} \u{2014} anywhere, also while the app sits in the tray ",
                    "\u{2014} to save the last {} as a playable clip, with game sound and ",
                    "your microphone mixed in."
                ),
                hotkey,
                length_label(clips.window_sec)
            ))
            .field(
                div()
                    .flex()
                    .items_start()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .child(label("Record for clips").mb(px(2.)))
                            .child(help(format!(
                                "Up to about {} of disk per minute at the settings below; older buffer parts are deleted automatically.",
                                size_label(per_min)
                            ))),
                    )
                    .child(switch("clips-toggle", clips.enabled, Some(toggle))),
            )
            .field(
                div()
                    .child(label("Screen"))
                    .child(help("Which screen gets recorded while the switch is on."))
                    .child(control_row().child(select("clips-monitor", &self.monitor, monitor_width, false))),
            )
            .field(
                div()
                    .child(label("Frame rate"))
                    .child(help(concat!(
                        "Above your screen's refresh rate you get the refresh rate. Sizes are for ",
                        "busy footage; a still desktop takes less."
                    )))
                    .child(control_row().child(select("clips-fps", &self.clip_fps, fps_width, false))),
            )
            .field(
                div()
                    .child(label("Resolution"))
                    .child(help("Scaled down on the graphics card; never above the screen's own size."))
                    .child(control_row().child(select(
                        "clips-resolution",
                        &self.clip_resolution,
                        resolution_width,
                        false,
                    ))),
            )
            .field(
                div()
                    .child(label("Quality"))
                    .child(help(concat!(
                        "The bitrate follows resolution and frame rate, so a level looks equally sharp ",
                        "at each. Changing frame rate, resolution or quality restarts the recording ",
                        "with an empty buffer."
                    )))
                    .child(control_row().child(select("clips-quality", &self.clip_quality, quality_width, false))),
            )
            .field(
                div()
                    .child(label("Clip length"))
                    .child(help(format!(
                        "How far back a saved clip goes. The rolling buffer keeps that much on disk: about {} at this length. Changing it restarts the recording with an empty buffer.",
                        size_label(per_min * u64::from(clips.window_sec) / 60)
                    )))
                    .child(control_row().child(select("clips-length", &self.clip_length, length_width, false))),
            )
            .field(
                div()
                    .child(label("Shortcut"))
                    .child(help(concat!(
                        "Click the box and press the key or combination you want. Esc cancels. ",
                        "Applies immediately, no restart."
                    )))
                    .child(control_row().child(capture)),
            )
            .field(
                div()
                    .flex()
                    .items_start()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .child(label("Post clips in the chat automatically").mb(px(2.)))
                            .child(help(concat!(
                                "Off by default. Only while you are in a voice call. A saved clip is ",
                                "offered in the general channel; nobody downloads it unless they click. ",
                                "Otherwise a card with a Share button appears in the general channel, ",
                                "for you only."
                            ))),
                    )
                    .child(switch("clips-auto-share", auto_share, Some(auto_toggle))),
            )
            .field(
                div()
                    .child(label("Clips folder"))
                    .child(help(concat!(
                        "Where saved clips land. Clips already saved stay where they are; the ",
                        "rolling buffer moves along and starts empty."
                    )))
                    .child(
                        control_row().child(code_inline(&clips.folder)).child(
                            Btn::new("btn-clips-dir", "Change\u{2026}")
                                .kind(BtnKind::Ghost)
                                .on_click(move |_, _, _| commands::pick_clips_dir(&ui_pick)),
                        ),
                    ),
            )
            .field(
                div()
                    .child(label("Save a clip now"))
                    .child(help(
                        clips.error.clone().unwrap_or_else(|| "Writes the last window to the clips folder.".into()),
                    ))
                    .child(
                        control_row()
                            .child(
                                Btn::new("clip-now", format!("Save last {}", length_short(clips.window_sec)))
                                    .kind(BtnKind::Primary)
                                    .icon(Icon::Play)
                                    .disabled(!clips.enabled)
                                    .on_click(move |_, _, _| commands::clip_now(&ui_now)),
                            )
                            .child(
                                Btn::new("clip-share", "Share last clip")
                                    .kind(BtnKind::Ghost)
                                    .icon(Icon::Clip)
                                    .disabled(!clips.can_share)
                                    .on_click(move |_, _, _| commands::share_last_clip(&ui_share)),
                            )
                            .child(
                                Btn::new("open-clips-folder", "Open folder")
                                    .kind(BtnKind::Ghost)
                                    .icon(Icon::Open)
                                    .on_click(move |_, _, _| commands::open_clips_folder(&ui_open)),
                            ),
                    ),
            );
        if let Some(last) = &clips.last_clip {
            body = body.field(
                div()
                    .child(label("Last saved"))
                    .child(control_row().child(code_inline(last))),
            );
        }
        body
    }

    /// While the box listens every key is its own (nothing reaches the rest of the window);
    /// a key the shortcut format knows ends the listening and goes to the engine, which
    /// validates and registers it.
    fn hotkey_pressed(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let Some(snap) = self.app.snapshot(cx) else {
            return;
        };
        if !snap.v.hotkey_wait {
            return;
        }
        cx.stop_propagation();
        let Some(name) = hotkey_name(&ev.keystroke.modifiers, &ev.keystroke.key) else {
            return;
        };
        self.pending_mods.clear();
        commands::set_clip_hotkey(&self.ui, &name);
        self.app.update(cx, |app, cx| {
            app.v_mut().hotkey_wait = false;
            cx.notify();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mods(control: bool, alt: bool, shift: bool, platform: bool) -> Modifiers {
        Modifiers {
            control,
            alt,
            shift,
            platform,
            ..Modifiers::default()
        }
    }

    #[test]
    fn a_shortcut_is_modifiers_in_wire_order_then_one_key() {
        assert_eq!(
            hotkey_name(&mods(true, true, false, false), "c").as_deref(),
            Some("ctrl+alt+c")
        );
        assert_eq!(
            hotkey_name(&mods(false, false, true, true), "7").as_deref(),
            Some("shift+win+7")
        );
        assert_eq!(
            hotkey_name(&mods(false, false, false, false), "f13").as_deref(),
            Some("f13")
        );
    }

    #[test]
    fn a_clip_length_label_reads_back_to_its_seconds() {
        for sec in LENGTHS.into_iter().chain([45, 90, 3600]) {
            assert_eq!(length_seconds(&length_label(sec)), Some(sec), "{sec}");
        }
        assert_eq!(length_label(1800), "30 minutes");
        assert_eq!(length_label(7200), "120 minutes");
        assert_eq!(length_label(60), "1 minute");
    }

    #[test]
    fn a_choice_reads_back_from_its_label_whatever_the_estimate_says() {
        assert_eq!(fps_of(&format!("144 fps{SEP}~1.1 GB/min")), Some(144));
        assert_eq!(
            resolution_of(&format!("Native{SEP}2560\u{d7}1440{SEP}~400 MB/min")),
            Some("native")
        );
        assert_eq!(
            quality_of(&format!("Very high{SEP}50 Mbit/s{SEP}~376 MB/min")),
            Some("very_high")
        );
        assert_eq!(length_seconds(&format!("5 minutes{SEP}~1.1 GB")), Some(300));
        assert_eq!(quality_of("Ultra"), None);
    }

    #[test]
    fn sizes_read_as_megabytes_then_gigabytes() {
        assert_eq!(size_label(226), "226 MB");
        assert_eq!(size_label(27_165), "27.2 GB");
    }

    #[test]
    fn keys_the_wire_format_does_not_know_keep_the_box_waiting() {
        let none = mods(false, false, false, false);
        for key in [
            "left", "enter", "space", "-", "f25", "f0", "shift", "f100", "\u{e9}",
        ] {
            assert_eq!(hotkey_name(&none, key), None, "{key}");
        }
    }
}
