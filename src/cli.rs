//! CLI argument parsing (clap derive).

use clap::Parser;

/// Parse `--width`: 0 means "use the built-in default" (32 monsters / 24 items),
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

/// Monster Hunter bestiary in your terminal.
#[derive(Parser, Debug)]
#[command(
    name = "zukan",
    version,
    about = "Monster Hunter bestiary in your terminal"
)]
pub struct Args {
    /// Monster or item name (fuzzy, typo-tolerant). Multiple names render
    /// back-to-back.
    #[arg(num_args = 0..)]
    pub query: Vec<String>,

    /// Look up an item instead of a monster.
    #[arg(long)]
    pub item: bool,

    /// Terminal column width for the icon. 0 = built-in default (32 monsters /
    /// 24 items); otherwise must be in 24..=48.
    #[arg(
        long,
        default_value = "0",
        value_parser = clap::builder::ValueParser::new(parse_width)
    )]
    pub width: u32,

    /// Show an info card next to the icon.
    #[arg(long)]
    pub detail: bool,

    /// Pick a random monster (or item with --item).
    #[arg(long)]
    pub random: bool,

    /// List monsters belonging to a game (mhw, MHW, ...).
    #[arg(long, value_name = "GAME")]
    pub list: Option<String>,

    /// Filter by game code or abbreviation (mhw, MHW, ...).
    #[arg(long, value_name = "GAME")]
    pub game: Option<String>,

    /// Display language: en / ja / zh. "auto" uses config or defaults to en.
    #[arg(long, default_value = "auto")]
    pub lang: String,

    /// Suppress the name line on stderr.
    #[arg(long)]
    pub hide_name: bool,

    /// Icon only, no info card (overrides --detail).
    #[arg(long)]
    pub no_card: bool,

    /// Show every match instead of just the best one.
    #[arg(long, short = 'a')]
    pub all: bool,
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
