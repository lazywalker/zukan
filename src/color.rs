//! ANSI color helpers and element/status to color tables.
//!
//! 256-color SGR codes (38;5;N) for elements and ailments, plus box-drawing
//! characters and game abbreviations used by the card layout.

use std::collections::HashMap;

// ============================================================ SGR codes ===

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const CYAN: &str = "\x1b[36m";
pub const YELLOW: &str = "\x1b[33m";

/// English element/status token to its 256-color escape.
///
/// Keys come straight from the data files. Color is applied regardless of
/// display language: the localized text is colored by the English key, then
/// shown translated (same trick preview.py uses).
pub fn element_color(en: &str) -> Option<&'static str> {
    let map: &[(&str, &str)] = &[
        ("Fire", "\x1b[38;5;196m"),
        ("Water", "\x1b[38;5;39m"),
        ("Thunder", "\x1b[38;5;220m"),
        ("Ice", "\x1b[38;5;231m"),
        ("Dragon", "\x1b[38;5;129m"),
        ("Poison", "\x1b[38;5;91m"),
        ("Sleep", "\x1b[38;5;83m"),
        ("Paralysis", "\x1b[38;5;226m"),
        ("Blast", "\x1b[38;5;208m"),
        ("Stun", "\x1b[38;5;215m"),
    ];
    map.iter().find(|(k, _)| *k == en).map(|(_, v)| *v)
}

/// Wrap `name` in the color of its first element (used by `--list`).
///
/// - No elements: returned unchanged (keeps pure-physical monsters distinct
///   from elementals).
/// - Multiple elements: the first wins, respecting data order.
/// - Unknown token: no color.
pub fn name_colored_by_element(name: &str, elements: &[String]) -> String {
    match elements.first().and_then(|e| element_color(e)) {
        Some(c) => format!("{c}{name}{RESET}"),
        None => name.to_string(),
    }
}

/// Color an ailment by sniffing its English name for an element/status keyword.
///
/// Substring match (case-insensitive), in priority order. Falls back to a dark
/// red for bleed/blood/drain, then no color. Matches preview.py exactly.
pub fn ailment_color(ailment: &str) -> &'static str {
    let lower = ailment.to_lowercase();
    let rules: &[(&str, &str)] = &[
        ("fire", "\x1b[38;5;196m"),
        ("water", "\x1b[38;5;39m"),
        ("thunder", "\x1b[38;5;220m"),
        ("hellfire", "\x1b[38;5;220m"),
        ("ice", "\x1b[38;5;231m"),
        ("snowman", "\x1b[38;5;231m"),
        ("dragon", "\x1b[38;5;129m"),
        ("poison", "\x1b[38;5;91m"),
        ("sleep", "\x1b[38;5;83m"),
        ("paralysis", "\x1b[38;5;226m"),
        ("blast", "\x1b[38;5;208m"),
        ("stun", "\x1b[38;5;215m"),
        ("fatigue", "\x1b[38;5;215m"),
        ("soiled", "\x1b[38;5;215m"),
        ("bleed", "\x1b[38;5;52m"),
        ("blood", "\x1b[38;5;52m"),
        ("drain", "\x1b[38;5;52m"),
    ];
    for (needle, color) in rules {
        if lower.contains(needle) {
            return color;
        }
    }
    RESET
}

/// Star rating string: N filled stars + (3-N) empty, for 0 < N ≤ 3.
pub fn stars(n: u32) -> String {
    if n == 0 || n > 3 {
        return String::new();
    }
    let filled: String = "★".repeat(n as usize);
    let empty: String = "☆".repeat((3 - n) as usize);
    format!("{filled}{empty}")
}

/// Full game title to short display code: "Monster Hunter World" -> "MHW".
pub fn game_abbr(full: &str) -> &'static str {
    let map: &[(&str, &str)] = &[
        ("Monster Hunter Freedom Unite", "MHFU"),
        ("Monster Hunter 3 Ultimate", "MH3U"),
        ("Monster Hunter 4 Ultimate", "MH4U"),
        ("Monster Hunter Generations Ultimate", "MHGU"),
        ("Monster Hunter World", "MHW"),
        ("Monster Hunter Rise", "MHRise"),
        ("Monster Hunter Wilds", "MHWilds"),
        ("Monster Hunter Stories", "MHST"),
        ("Monster Hunter Stories 2", "MHST2"),
    ];
    map.iter()
        .find(|(k, _)| *k == full)
        .map(|(_, v)| *v)
        .unwrap_or("")
}

/// Game code (`mhw`, `wilds`) to short display code (`MHW`, `MHWilds`).
///
/// Item `sources[].game` uses API vocabulary (`wilds`, not `mhwilds`),
/// differing from monster `games[].game`. This maps either form to the
/// canonical abbreviation, falling back to uppercasing the input.
pub fn game_code_abbr(code: &str) -> String {
    let canonical = normalize_game(code).unwrap_or(code);
    let map: &[(&str, &str)] = &[
        ("mhfu", "MHFU"),
        ("mh3u", "MH3U"),
        ("mh4u", "MH4U"),
        ("mhgu", "MHGU"),
        ("mhw", "MHW"),
        ("mhwi", "MHWI"),
        ("mhrise", "MHRise"),
        ("mhrs", "MHRS"),
        ("mhwilds", "MHWilds"),
        ("mhst", "MHST"),
        ("mhst2", "MHST2"),
    ];
    map.iter()
        .find(|(k, _)| *k == canonical)
        .map(|(_, v)| (*v).to_string())
        .unwrap_or_else(|| code.to_uppercase())
}

/// Normalize a user-facing game token (`mhw`, `MHW`, `Monster Hunter World`)
/// to the internal code (`mhw`). Returns None for unrecognized input.
pub fn normalize_game(token: &str) -> Option<&'static str> {
    // Built per call; cheap enough for a CLI.
    let mut m = HashMap::new();
    let pairs: &[(&str, &str)] = &[
        ("mhfu", "mhfu"),
        ("Monster Hunter Freedom Unite", "mhfu"),
        ("mh3u", "mh3u"),
        ("Monster Hunter 3 Ultimate", "mh3u"),
        ("mh4u", "mh4u"),
        ("Monster Hunter 4 Ultimate", "mh4u"),
        ("mhgu", "mhgu"),
        ("Monster Hunter Generations Ultimate", "mhgu"),
        ("mhw", "mhw"),
        ("Monster Hunter World", "mhw"),
        ("mhwi", "mhwi"),
        ("Monster Hunter World: Iceborne", "mhwi"),
        ("mhrise", "mhrise"),
        ("Monster Hunter Rise", "mhrise"),
        ("mhrs", "mhrs"),
        ("Monster Hunter Rise: Sunbreak", "mhrs"),
        ("mhwilds", "mhwilds"),
        // Item sources use the API vocabulary "wilds" rather than "mhwilds".
        ("wilds", "mhwilds"),
        ("Monster Hunter Wilds", "mhwilds"),
        ("mhst", "mhst"),
        ("Monster Hunter Stories", "mhst"),
        ("mhst2", "mhst2"),
        ("Monster Hunter Stories 2", "mhst2"),
    ];
    for (k, v) in pairs {
        m.insert(*k, *v);
    }
    // Try exact, then uppercase abbreviation.
    m.get(token).copied().or_else(|| {
        let up = token.to_uppercase();
        // MHW → mhw, MHRISE stays MHRISE (no abbrev). Handle the abbrev set.
        let abbrev: &[(&str, &str)] = &[
            ("MHFU", "mhfu"),
            ("MH3U", "mh3u"),
            ("MH4U", "mh4u"),
            ("MHGU", "mhgu"),
            ("MHW", "mhw"),
            ("MHWI", "mhwi"),
            ("MHRISE", "mhrise"),
            ("MHRS", "mhrs"),
            ("MHWILDS", "mhwilds"),
            ("MHST", "mhst"),
            ("MHST2", "mhst2"),
        ];
        abbrev.iter().find(|(k, _)| *k == up).map(|(_, v)| *v)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_colors_known() {
        assert_eq!(element_color("Fire"), Some("\x1b[38;5;196m"));
        assert_eq!(element_color("Dragon"), Some("\x1b[38;5;129m"));
        assert_eq!(element_color("Unknown"), None);
    }

    #[test]
    fn name_colored_uses_first_element() {
        let out = name_colored_by_element("Rathalos", &["Fire".to_string()]);
        assert!(out.contains("\x1b[38;5;196m"), "Fire color: {out:?}");
        assert!(out.contains("Rathalos"));
        assert!(out.ends_with("\x1b[0m"), "must reset at end: {out:?}");
    }

    #[test]
    fn name_colored_multi_element_uses_first() {
        // [Dragon, Ice] picks Dragon, not Ice.
        let out = name_colored_by_element("Kushala Daora", &["Dragon".into(), "Ice".into()]);
        assert!(
            out.contains("\x1b[38;5;129m"),
            "should pick first element (Dragon=129): {out:?}"
        );
        assert!(
            !out.contains("\x1b[38;5;231m"),
            "must NOT use second element (Ice=231): {out:?}"
        );
    }

    #[test]
    fn name_colored_no_element_is_plain() {
        let out = name_colored_by_element("Deviljho", &[]);
        assert_eq!(out, "Deviljho", "no-element name must be uncolored");
    }

    #[test]
    fn name_colored_unknown_element_is_plain() {
        let out = name_colored_by_element("Mystery", &["Quantum".to_string()]);
        assert_eq!(out, "Mystery", "unknown token must fall back to plain");
    }

    #[test]
    fn ailment_color_sniffs_keyword() {
        assert_eq!(ailment_color("Fireblight"), "\x1b[38;5;196m");
        assert_eq!(ailment_color("Thunderblight"), "\x1b[38;5;220m");
        // "Hellfireblight" matches the Fire rule first (substring scan in
        // declaration order, same quirk as preview.py).
        assert_eq!(ailment_color("Hellfireblight"), "\x1b[38;5;196m");
        assert_eq!(ailment_color("Blastblight"), "\x1b[38;5;208m");
        assert_eq!(ailment_color("Effluvium"), RESET);
        assert_eq!(ailment_color("Bleeding"), "\x1b[38;5;52m");
    }

    #[test]
    fn stars_format() {
        assert_eq!(stars(0), "");
        assert_eq!(stars(1), "★☆☆");
        assert_eq!(stars(2), "★★☆");
        assert_eq!(stars(3), "★★★");
        assert_eq!(stars(4), "");
    }

    #[test]
    fn game_abbr_known() {
        assert_eq!(game_abbr("Monster Hunter World"), "MHW");
        assert_eq!(game_abbr("Monster Hunter Wilds"), "MHWilds");
        assert_eq!(game_abbr("Unknown Title"), "");
    }

    #[test]
    fn normalize_game_handles_codes_abbrs_and_titles() {
        assert_eq!(normalize_game("mhw"), Some("mhw"));
        assert_eq!(normalize_game("MHW"), Some("mhw"));
        assert_eq!(normalize_game("Monster Hunter World"), Some("mhw"));
        assert_eq!(normalize_game("mhwilds"), Some("mhwilds"));
        assert_eq!(normalize_game("MHWILDS"), Some("mhwilds"));
        // Item-source API vocabulary maps to mhwilds.
        assert_eq!(normalize_game("wilds"), Some("mhwilds"));
        assert_eq!(normalize_game("garbage"), None);
    }

    #[test]
    fn game_code_abbr_maps_api_vocabulary() {
        assert_eq!(game_code_abbr("mhw"), "MHW");
        assert_eq!(game_code_abbr("wilds"), "MHWilds");
        assert_eq!(game_code_abbr("mhwilds"), "MHWilds");
        // Unknown codes fall back to uppercasing.
        assert_eq!(game_code_abbr("future"), "FUTURE");
    }
}
