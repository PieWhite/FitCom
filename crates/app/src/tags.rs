//! Herkennen van `@naam`-tags in chatberichten. Puur tekstwerk, geen state — gedeeld
//! tussen de meldingslaag (`engine.rs`) en het highlighten in de UI (`ui.rs`), zodat
//! "geldige tag" overal precies hetzelfde betekent.

/// Of `body` een geldige `@naam`-tag naar `naam` bevat. Hoofdletterongevoelig, met een
/// woordgrens aan beide kanten — zonder die grens zou `@Alice` ook `@Aliceie` raken.
pub fn bevat_tag(body: &str, naam: &str) -> bool {
    if naam.is_empty() {
        return false;
    }
    let doel = format!("@{naam}").to_lowercase();
    let lower = body.to_lowercase();

    let mut vanaf = 0;
    while let Some(rel) = lower[vanaf..].find(&doel) {
        let start = vanaf + rel;
        let eind = start + doel.len();
        let voor_ok = lower[..start]
            .chars()
            .next_back()
            .map(|c| !is_tag_teken(c))
            .unwrap_or(true);
        let na_ok = lower[eind..]
            .chars()
            .next()
            .map(|c| !is_tag_teken(c))
            .unwrap_or(true);
        if voor_ok && na_ok {
            return true;
        }
        vanaf = start + 1;
    }
    false
}

/// De cursor-parsing en het filteren van suggesties stonden hier ook. Die zijn met de
/// Tauri-migratie vervallen: de autocomplete in de invoerbalk is presentatie en zit nu
/// in `ui::view::composer::update_autocomplete`. Wat *wel* hier moet blijven is `bevat_tag`: dat is
/// de regel waarop de meldingslaag beslist, en die mag maar op één plek staan.
fn is_tag_teken(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn herkent_een_gewone_tag() {
        assert!(bevat_tag("hoi @Alice, kijk eens", "Alice"));
    }

    #[test]
    fn is_hoofdletterongevoelig() {
        assert!(bevat_tag("hoi @alice", "Alice"));
        assert!(bevat_tag("hoi @ALICE", "alice"));
    }

    #[test]
    fn respecteert_de_woordgrens() {
        assert!(!bevat_tag("hoi @Aliceie", "Alice"));
        assert!(!bevat_tag("een adres zoals bla@Alice.nl", "Alice"));
    }

    #[test]
    fn tag_aan_het_begin_of_einde_van_het_bericht_telt() {
        assert!(bevat_tag("@Alice", "Alice"));
        assert!(bevat_tag("kijk @Alice", "Alice"));
        assert!(bevat_tag("@Alice kijk", "Alice"));
    }

    #[test]
    fn lege_naam_matcht_nooit() {
        assert!(!bevat_tag("@ per ongeluk", ""));
    }
}
