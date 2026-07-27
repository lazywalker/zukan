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
fn parse(text: &str) -> Config {
    let mut c = Config::default();
    for line in text.lines() {
        // Strip inline comments: ` #` (space + hash) starts a comment, like
        // shell/dotenv. A hash inside a quoted value ("a#b") has no preceding
        // space so it survives.
        let line = strip_inline_comment(line).trim();
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
            "show_card_by_default" => c.show_card_by_default = parse_bool(val),
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

/// Drop an inline ` #...` comment from a line, respecting quotes. A `#` only
/// starts a comment when preceded by whitespace and outside a quoted region.
/// Bare leading `#` lines are handled by the caller.
fn strip_inline_comment(line: &str) -> &str {
    let mut in_quotes = false;
    let bytes = line.as_bytes();
    let mut cut = line.len();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => in_quotes = !in_quotes,
            b'#' if !in_quotes && i > 0 && bytes[i - 1].is_ascii_whitespace() => {
                cut = i;
                break;
            }
            _ => {}
        }
        i += 1;
    }
    &line[..cut]
}

/// Accept the common truthy spellings (true/yes/on/1) case-insensitively;
/// everything else is false. Matches dotenv/shell conventions so users aren't
/// silently dropped to the default for writing `yes` instead of `true`.
fn parse_bool(val: &str) -> bool {
    matches!(
        val.trim().to_ascii_lowercase().as_str(),
        "true" | "yes" | "on" | "1"
    )
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
        let c = parse("default_width = abc\nshow_card_by_default = maybe\nlanguage = zh");
        assert_eq!(c.default_width, 0);
        assert!(!c.show_card_by_default);
        assert_eq!(c.language, "zh");
    }

    #[test]
    fn inline_comments_are_stripped() {
        let text = "\
            language = zh              # en | ja | zh\n\
            default_width = 0          # built-in defaults\n\
            show_card_by_default = yes # show the info card";
        let c = parse(text);
        assert_eq!(c.language, "zh");
        assert_eq!(c.default_width, 0);
        assert!(c.show_card_by_default);
    }

    #[test]
    fn hash_inside_value_is_not_a_comment() {
        // A `#` with no preceding space stays part of the value.
        let c = parse(r#"default_game = "foo#bar""#);
        assert_eq!(c.default_game, "foo#bar");
    }

    #[test]
    fn bool_accepts_common_spellings() {
        assert!(parse("show_card_by_default = true").show_card_by_default);
        assert!(parse("show_card_by_default = yes").show_card_by_default);
        assert!(parse("show_card_by_default = on").show_card_by_default);
        assert!(parse("show_card_by_default = 1").show_card_by_default);
        assert!(parse("show_card_by_default = TRUE").show_card_by_default);
        assert!(!parse("show_card_by_default = false").show_card_by_default);
        assert!(!parse("show_card_by_default = no").show_card_by_default);
        assert!(!parse("show_card_by_default = 0").show_card_by_default);
        assert!(!parse("show_card_by_default = maybe").show_card_by_default);
    }
}
