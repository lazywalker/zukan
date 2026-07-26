//! CLI argument parsing (clap derive).

use clap::Parser;

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

    /// Terminal column width for the icon. 0 = native (48 for monsters, 24 for items).
    #[arg(long, default_value_t = 0)]
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
