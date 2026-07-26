//! User config file at `~/.config/zukan/config.toml` (or the platform
//! equivalent via `dirs::config_dir`).
//!
//! On first run, if the file is missing we don't write one. If present but
//! malformed, we warn on stderr and fall back to defaults rather than failing
//! the whole invocation.
//!
//! A field applies only when the matching CLI flag was NOT passed; explicit
//! flags always win. That resolution lives in `main.rs`; this module only
//! loads and parses.

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    /// Display language. Used when `--lang` is not explicitly passed.
    /// One of: en, ja, zh.
    pub language: String,

    /// Default icon width in terminal columns. 0 = built-in defaults
    /// (40 for monsters, 24 for items).
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
            Ok(text) => match toml::from_str::<Config>(&text) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(
                        "zukan: warning: failed to parse {} (using defaults): {e}",
                        path.display()
                    );
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }
}

/// Where the config file lives: `<config_dir>/zukan/config.toml`.
/// Returns None only if the platform has no config dir (rare).
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("zukan").join("config.toml"))
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
        let c: Config = toml::from_str("language = \"ja\"").unwrap();
        assert_eq!(c.language, "ja");
        assert_eq!(c.default_width, 0);
        assert!(!c.show_card_by_default);
    }

    #[test]
    fn parse_full_config() {
        let text = r#"
            language = "zh"
            default_width = 32
            default_game = "mhwilds"
            show_card_by_default = true
        "#;
        let c: Config = toml::from_str(text).unwrap();
        assert_eq!(c.language, "zh");
        assert_eq!(c.default_width, 32);
        assert_eq!(c.default_game, "mhwilds");
        assert!(c.show_card_by_default);
    }

    #[test]
    fn unknown_keys_are_ignored() {
        // Forward-compatible: extra keys don't break parsing.
        let c: Config = toml::from_str("language = \"en\"\nfuture_key = 42").unwrap();
        assert_eq!(c.language, "en");
    }

    #[test]
    fn roundtrip_serialization() {
        let c = Config {
            language: "ja".into(),
            default_width: 24,
            default_game: "mhw".into(),
            show_card_by_default: true,
        };
        let text = toml::to_string(&c).unwrap();
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(back.language, c.language);
        assert_eq!(back.default_width, c.default_width);
        assert_eq!(back.default_game, c.default_game);
        assert_eq!(back.show_card_by_default, c.show_card_by_default);
    }
}
