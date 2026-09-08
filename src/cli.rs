//! CLI argument parsing (argh derive).

use argh::FromArgs;

/// Parse `--width`: 0 means "use the built-in default" (24 for all types),
/// otherwise the value must lie in [24, 48].
fn parse_width(s: &str) -> Result<u32, String> {
    let n: u32 = s
        .parse()
        .map_err(|_| format!("`{s}` is not a valid width"))?;
    if n == 0 || (24..=48).contains(&n) {
        Ok(n)
    } else {
        Err(format!("width must be 0 (default) or in 24..=48, got {n}"))
    }
}

/// Which icon set renders monsters: the game card icons (`game`, default) or
/// the hand-drawn pixel sprites (`pixel`, partial coverage, game icons as
/// fallback for uncovered monsters).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteSet {
    Game,
    Pixel,
}

fn parse_sprites(s: &str) -> Result<SpriteSet, String> {
    match s {
        "game" => Ok(SpriteSet::Game),
        "pixel" => Ok(SpriteSet::Pixel),
        _ => Err(format!("sprites must be game or pixel, got {s}")),
    }
}

/// Monster Hunter bestiary in your terminal.
#[derive(FromArgs, Debug)]
pub struct Args {
    /// monster or item name (fuzzy, typo-tolerant). multiple names render
    /// back-to-back.
    #[argh(positional)]
    pub query: Vec<String>,

    /// look up an item instead of a monster.
    #[argh(switch)]
    pub item: bool,

    /// look up an endemic-life creature instead of a monster.
    #[argh(switch)]
    pub endemic: bool,

    /// terminal column width for the icon. 0 = built-in default 24 for all
    /// types; otherwise must be in 24..=48.
    #[argh(option, from_str_fn(parse_width), default = "0")]
    pub width: u32,

    /// show an info card next to the icon.
    #[argh(switch)]
    pub detail: bool,

    /// pick a random monster (or item with --item, or endemic life with --endemic).
    #[argh(switch)]
    pub random: bool,

    /// list monsters belonging to a game (mhw, MHW, ...).
    #[argh(option)]
    pub list: Option<String>,

    /// filter by game code or abbreviation (mhw, MHW, ...).
    #[argh(option)]
    pub game: Option<String>,

    /// monster icon set: hand-drawn pixel sprites (pixel, default — rendered
    /// at native size, --width ignored) or game card icons (game, scaled).
    /// partial coverage: monsters without a pixel sprite fall back to game
    /// icons. items and endemic life are unaffected.
    #[argh(option, from_str_fn(parse_sprites), default = "SpriteSet::Pixel")]
    pub sprites: SpriteSet,

    /// display language: en / ja / zh. "auto" uses config or defaults to en.
    #[argh(option, default = "\"auto\".to_string()")]
    pub lang: String,

    /// suppress the name line on stderr.
    #[argh(switch)]
    pub hide_name: bool,

    /// icon only, no info card (overrides --detail).
    #[argh(switch)]
    pub no_card: bool,

    /// show every match instead of just the best one.
    #[argh(switch, short = 'a')]
    pub all: bool,

    /// print version and exit.
    #[argh(switch, short = 'V')]
    pub version: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_zero_means_default() {
        assert_eq!(parse_width("0").unwrap(), 0);
    }

    #[test]
    fn width_accepts_inclusive_bounds() {
        assert_eq!(parse_width("24").unwrap(), 24);
        assert_eq!(parse_width("48").unwrap(), 48);
        assert_eq!(parse_width("32").unwrap(), 32);
    }

    #[test]
    fn sprites_parses_both_sets() {
        assert_eq!(parse_sprites("game").unwrap(), SpriteSet::Game);
        assert_eq!(parse_sprites("pixel").unwrap(), SpriteSet::Pixel);
        assert!(parse_sprites("sprite").is_err());
    }

    #[test]
    fn width_rejects_out_of_range() {
        // 1..=23 too small, 49+ too large.
        assert!(parse_width("1").is_err());
        assert!(parse_width("23").is_err());
        assert!(parse_width("49").is_err());
        assert!(parse_width("1000").is_err());
    }

    #[test]
    fn width_rejects_non_numeric() {
        assert!(parse_width("wide").is_err());
        assert!(parse_width("-5").is_err());
    }
}
