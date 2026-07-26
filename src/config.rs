//! User config file at `~/.config/zukan/config` (or the platform equivalent
//! via `dirs::config_dir`).
//!
//! Format is a flat KEY=VAL text file: one `key = value` per line, `#` starts
//! a comment (line-leading only), values may be quoted or bare. Unknown keys
//! are ignored; a field that fails to parse falls back to its default rather
//! than failing the whole file.
//!
//! On first run, if the file is missing we don't write one. If present but
//! unreadable, we warn on stderr and fall back to defaults.
//!
//! A field applies only when the matching CLI flag was NOT passed; explicit
//! flags always win. That resolution lives in `main.rs`; this module only
//! loads and parses.

use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    /// Display language. Used when `--lang` is not explicitly passed.
    /// One of: en, ja, zh.
    pub language: String,

    /// Default icon width in terminal columns. 0 = built-in defaults
    /// (32 for monsters, 24 for items).
    pub default_width: u32,

    /// Default game code for icon selection (such as "mhwilds"). When set, a
    /// monster's icon from this game is preferred over the usual newest-first
    /// ordering. Empty string = use built-in ordering.
    pub default_game: String,

    /// Show the info card by default (without needing --detail).
    pub show_card_by_default: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            default_width: 0,
            default_game: String::new(),
            show_card_by_default: false,
        }
    }
}

impl Config {
    /// Load config from the user's config dir, applying defaults for any
    /// missing/invalid file. Never returns Err; bad config is non-fatal.
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Self::default();
        };
        match fs::read_to_string(&path) {
            Ok(text) => parse(&text),
            Err(_) => Self::default(),
        }
    }
}

/// Parse a KEY=VAL config body into a Config, starting from defaults.
/// Lines may be `key = value` (whitespace around `=` ignored), `#` for
/// comments, or blank. Quoted values (`"en"`) and bare (`en`) are both
/// accepted; the surrounding quotes are stripped. A per-field parse failure
/// leaves that field at its default (so one bad line doesn't poison the rest).
fn parse(text: &str) -> Config {
    let mut c = Config::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let val = val.trim().trim_matches('"');
        match key.trim() {
            "language" => c.language = val.to_string(),
            "default_width" => c.default_width = val.parse().unwrap_or(0),
            "default_game" => c.default_game = val.to_string(),
            "show_card_by_default" => c.show_card_by_default = val == "true",
            _ => {}
        }
    }
    c
}

/// Where the config file lives: `<config_dir>/zukan/config`.
/// Returns None only if the platform has no config dir (rare).
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("zukan").join("config"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let c = Config::default();
        assert_eq!(c.language, "en");
        assert_eq!(c.default_width, 0);
        assert!(c.default_game.is_empty());
        assert!(!c.show_card_by_default);
    }

    #[test]
    fn parse_partial_uses_defaults_for_missing() {
        // Only `language` set; rest should default.
        let c = parse("language = ja");
        assert_eq!(c.language, "ja");
        assert_eq!(c.default_width, 0);
        assert!(!c.show_card_by_default);
    }

    #[test]
    fn parse_full_config() {
        let text = "\
            language = zh\n\
            default_width = 32\n\
            default_game = mhwilds\n\
            show_card_by_default = true";
        let c = parse(text);
        assert_eq!(c.language, "zh");
        assert_eq!(c.default_width, 32);
        assert_eq!(c.default_game, "mhwilds");
        assert!(c.show_card_by_default);
    }

    #[test]
    fn unknown_keys_are_ignored() {
        // Forward-compatible: extra keys don't break parsing.
        let c = parse("language = en\nfuture_key = 42");
        assert_eq!(c.language, "en");
    }

    #[test]
    fn quoted_values_have_quotes_stripped() {
        let c = parse("language = \"en\"");
        assert_eq!(c.language, "en");
    }

    #[test]
    fn comments_and_blank_lines_skipped() {
        let text = "\
            # this is a comment\n\
            \n\
            language = ja\n\
            # another comment\n\
            default_width = 24";
        let c = parse(text);
        assert_eq!(c.language, "ja");
        assert_eq!(c.default_width, 24);
    }

    #[test]
    fn bad_field_falls_back_to_default() {
        // default_width is u32; "abc" can't parse. show_card_by_default only
        // accepts "true"; "yes" stays false. Other fields still apply.
        let c = parse("default_width = abc\nshow_card_by_default = yes\nlanguage = zh");
        assert_eq!(c.default_width, 0);
        assert!(!c.show_card_by_default);
        assert_eq!(c.language, "zh");
    }
}
