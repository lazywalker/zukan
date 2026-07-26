//! Info card: icon on the left, labeled key/value rows on the right.
//!
//! Layout constants:
//!
//! - `total_cols = 80`  -- overall width budget
//! - `pad = 3`          -- blank columns between icon and card
//! - `key_w = 13`       -- label column width (such as "Weakness:   ")
//! - `value_w = total_cols - icon_w - pad - key_w`
//!
//! Card text is built into `card_lines`, then zipped row-by-row with the icon
//! block (whichever is taller wins; the shorter side is padded with blanks).

use image::DynamicImage;

use crate::color::{
    BOLD, CYAN, DIM, RESET, YELLOW, ailment_color, element_color, game_abbr, stars,
};
use crate::data::{Item, Monster};
use crate::i18n::{
    Lang, item_localized, label, monster_localized, pad_to, term, visible_len, wrap_cjk,
};
use crate::render::render_halfblock;

const TOTAL_COLS: usize = 80;
const PAD: usize = 3;
const KEY_W: usize = 13;

/// Render a monster card: icon + info, name on stderr is the caller's job.
pub fn render_monster(img: &DynamicImage, m: &Monster, width: u32, lang: Lang) -> String {
    let icon_w = width as usize;
    let value_w = TOTAL_COLS.saturating_sub(icon_w + PAD + KEY_W);
    let rule = "─".repeat(KEY_W - 1 + value_w.max(1));

    let rendered = render_halfblock(img, width, true);
    let icon_block: Vec<&str> = rendered.split('\n').collect();
    let mut card = CardLines::new();

    let (loc_name, loc_desc) = monster_localized(m, lang);
    card.push_plain(format!("{BOLD}{loc_name}{RESET}"));
    card.push_plain(format!("{DIM}{rule}{RESET}"));

    // Type
    card.add_kv(label("Type", lang), term(&m.kind, lang), value_w);
    if let Some(species) = &m.species
        && !species.is_empty()
    {
        card.add_kv(label("Species", lang), term(species, lang), value_w);
    }
    card.add_kv(
        label("Size", lang),
        term(if m.is_large { "Large" } else { "Small" }, lang),
        value_w,
    );

    if !m.elements.is_empty() {
        card.add_kv_rich(
            label("Element", lang),
            m.elements.iter().map(|e| colored_term(e, lang)).collect(),
            value_w,
        );
    }
    if !m.weakness.is_empty() {
        card.add_kv_rich(
            label("Weakness", lang),
            m.weakness.iter().map(|w| colored_term(w, lang)).collect(),
            value_w,
        );
    }
    if !m.ailments.is_empty() {
        card.add_kv_rich(
            label("Ailments", lang),
            m.ailments
                .iter()
                .map(|a| format!("{}{}{RESET}", ailment_color(a), term(a, lang)))
                .collect(),
            value_w,
        );
    }

    // Locations (prefer mhw, then wilds).
    let locations: &[crate::data::NamedLocation] = m
        .numeric
        .as_ref()
        .and_then(|n| {
            n.mhw
                .as_ref()
                .map(|m| &m.locations[..])
                .or_else(|| n.wilds.as_ref().map(|w| &w.locations[..]))
        })
        .unwrap_or(&[]);
    if !locations.is_empty() {
        let locs: Vec<String> = locations.iter().map(|l| term(&l.name, lang)).collect();
        card.add_kv(label("Location", lang), locs.join(", "), value_w);
    }

    // Star-rated weaknesses (mhw only; wilds uses level, not stars).
    if let Some(mhw) = m.numeric.as_ref().and_then(|n| n.mhw.as_ref())
        && !mhw.weaknesses.is_empty()
    {
        let mut top: Vec<&crate::data::ElementStars> =
            mhw.weaknesses.iter().filter(|w| w.stars > 0).collect();
        top.sort_by_key(|w| std::cmp::Reverse(w.stars));
        top.truncate(3);
        if !top.is_empty() {
            let parts: Vec<String> = top
                .iter()
                .map(|w| {
                    // mhw.weaknesses[].element is lowercase ("dragon"); title-case
                    // before display + color lookup (matches preview.py).
                    let titled = titlecase_word(&w.element);
                    format!("{} {}", colored_term(&titled, lang), stars(w.stars))
                })
                .collect();
            card.add_kv_rich(label("Weak(stars)", lang), parts, value_w);
        }
    }

    // Sub-species: stored as English display names; localize if possible.
    if !m.sub_species.is_empty() {
        let names: Vec<String> = m.sub_species.iter().map(|s| term(s, lang)).collect();
        card.add_kv(label("Sub-species", lang), names.join(", "), value_w);
    }

    // Games: dedup by full title, abbreviate.
    if !m.games.is_empty() {
        let mut seen: Vec<&str> = Vec::new();
        for g in &m.games {
            if !seen.contains(&g.game_full.as_str()) {
                seen.push(g.game_full.as_str());
            }
        }
        let abbrs: Vec<String> = seen
            .iter()
            .map(|t| {
                let a = game_abbr(t);
                if a.is_empty() {
                    (*t).to_string()
                } else {
                    a.to_string()
                }
            })
            .collect();
        card.add_kv(label("Games", lang), abbrs.join(", "), value_w);
    }

    // Description: prefer localized, fall back to numeric desc or game info.
    let desc = loc_desc.or_else(|| crate::i18n::monster_english_desc(m));
    if let Some(desc) = desc {
        card.push_plain(format!("{DIM}{rule}{RESET}"));
        for line in wrap_cjk(&desc, KEY_W - 1 + value_w) {
            card.push_plain(format!("{DIM}{line}{RESET}"));
        }
    }

    assemble(&icon_block, &card.lines, icon_w)
}

/// Render an item card.
pub fn render_item(img: Option<&DynamicImage>, it: &Item, width: u32, lang: Lang) -> String {
    let icon_w = width as usize;
    let value_w = TOTAL_COLS.saturating_sub(icon_w + PAD + KEY_W);
    let rule = "─".repeat(KEY_W - 1 + value_w.max(1));

    let icon_block: Vec<String> = match img {
        Some(im) => render_halfblock(im, width, true)
            .split('\n')
            .map(str::to_string)
            .collect(),
        None => placeholder_block(icon_w, 6),
    };

    let mut card = CardLines::new();
    let (loc_name, loc_desc) = item_localized(&it.name, it.description.as_deref(), &it.i18n, lang);
    card.push_plain(format!("{BOLD}{loc_name}{RESET}"));
    card.push_plain(format!("{DIM}{rule}{RESET}"));

    if let Some(r) = it.rarity {
        let s = "★".repeat(r as usize);
        card.add_kv(
            label("Rarity", lang),
            format!("{YELLOW}{s}{RESET} ({r}/10)"),
            value_w,
        );
    }
    if let Some(v) = it.value {
        card.add_kv(label("Value", lang), format!("{v}z"), value_w);
    }
    if let Some(c) = it.carry_limit {
        card.add_kv(label("Carry", lang), format!("{c}"), value_w);
    }
    if !it.sources.is_empty() {
        // Item sources use API game codes (mhw, wilds); normalize to display
        // abbreviations (MHW, MHWilds).
        let mut games: Vec<String> = it
            .sources
            .iter()
            .map(|s| crate::color::game_code_abbr(&s.game))
            .collect();
        games.sort();
        games.dedup();
        card.add_kv(label("Games", lang), games.join(", "), value_w);
    }
    if let Some(is) = &it.icon_source {
        let cleaned = is.strip_prefix("item-type:").unwrap_or(is);
        card.add_kv(label("Icon type", lang), cleaned.to_string(), value_w);
    }

    let desc = loc_desc.or_else(|| it.description.clone());
    if let Some(desc) = desc {
        card.push_plain(format!("{DIM}{rule}{RESET}"));
        for line in wrap_cjk(&desc, KEY_W - 1 + value_w) {
            card.push_plain(format!("{DIM}{line}{RESET}"));
        }
    }

    // assemble takes &str icon rows; we have owned Strings, borrow them.
    let icon_refs: Vec<&str> = icon_block.iter().map(String::as_str).collect();
    assemble(&icon_refs, &card.lines, icon_w)
}

/// Apply the element/status color (by English key) but display the translated text.
fn colored_term(en: &str, lang: Lang) -> String {
    let text = term(en, lang);
    match element_color(en) {
        Some(c) => format!("{c}{text}{RESET}"),
        None => text,
    }
}

/// Capitalize the first ASCII letter of each word, lowercase the rest.
/// Normalizes lowercase data values like "dragon" to "Dragon" before display
/// and color lookup (matches preview.py's `.title()`).
fn titlecase_word(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut up_next = true;
    for c in s.chars() {
        if c.is_ascii_whitespace() {
            up_next = true;
            out.push(c);
        } else if up_next {
            out.push(c.to_ascii_uppercase());
            up_next = false;
        } else {
            out.push(c.to_ascii_lowercase());
        }
    }
    out
}

// ---------------------------------------------------------------- builder ---

struct CardLines {
    lines: Vec<String>,
}

impl CardLines {
    fn new() -> Self {
        Self { lines: Vec::new() }
    }

    fn push_plain(&mut self, s: String) {
        self.lines.push(s);
    }

    /// Add a key: value row, wrapping the value to `value_w` visible columns.
    fn add_kv(&mut self, key: impl AsRef<str>, value: String, value_w: usize) {
        let key = key.as_ref();
        let wrapped = wrap_cjk(&value, value_w);
        for (i, chunk) in wrapped.iter().enumerate() {
            if i == 0 {
                let lbl = format!("{CYAN}{}{RESET}", pad_to(&format!("{key}:"), KEY_W));
                self.lines.push(format!("{lbl}{chunk}"));
            } else {
                self.lines.push(format!("{}{chunk}", " ".repeat(KEY_W)));
            }
        }
    }

    /// Add a key: value row where the value is a list of pre-colored tokens,
    /// joined by ", " and greedily packed to fit `value_w` visible columns.
    fn add_kv_rich(&mut self, key: impl AsRef<str>, tokens: Vec<String>, value_w: usize) {
        let key = key.as_ref();
        let mut packed: Vec<String> = Vec::new();
        let mut cur = String::new();
        for tok in &tokens {
            let sep = if cur.is_empty() { "" } else { ", " };
            if visible_len(&cur) + visible_len(sep) + visible_len(tok) <= value_w {
                cur.push_str(sep);
                cur.push_str(tok);
            } else {
                if !cur.is_empty() {
                    packed.push(std::mem::take(&mut cur));
                }
                cur.push_str(tok);
            }
        }
        if !cur.is_empty() {
            packed.push(cur);
        }
        if packed.is_empty() {
            packed.push(String::new());
        }
        for (i, chunk) in packed.iter().enumerate() {
            if i == 0 {
                let lbl = format!("{CYAN}{}{RESET}", pad_to(&format!("{key}:"), KEY_W));
                self.lines.push(format!("{lbl}{chunk}"));
            } else {
                self.lines.push(format!("{}{chunk}", " ".repeat(KEY_W)));
            }
        }
    }
}

/// Zip the icon block (left) with the card text (right) into final output.
fn assemble(icon: &[&str], card: &[String], icon_w: usize) -> String {
    let rows = icon.len().max(card.len());
    let pad_str = " ".repeat(PAD);
    let mut out = String::with_capacity(rows * (icon_w + PAD + 60));
    for r in 0..rows {
        let left = icon.get(r).copied().unwrap_or("");
        // Pad/truncate the icon row to exactly icon_w visible columns so the
        // card text starts at a fixed column. The icon row carries ANSI escapes
        // but its visible width equals the pixel width by construction.
        let left = pad_icon_row(left, icon_w);
        let right = card.get(r).map(String::as_str).unwrap_or("");
        out.push_str(&left);
        out.push_str(&pad_str);
        out.push_str(right);
        out.push_str(RESET);
        out.push('\n');
    }
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

/// Ensure an icon row occupies exactly `width` visible columns.
///
/// Half-block rows may end with a RESET but no trailing spaces, so shorter rows
/// need right-padding to keep the card column aligned. We never crop; rows are
/// exactly `width` cells wide by construction.
fn pad_icon_row(row: &str, width: usize) -> String {
    let w = visible_len(row);
    if w >= width {
        row.to_string()
    } else {
        format!("{row}{}", " ".repeat(width - w))
    }
}

/// A blank icon-sized block for items without an embedded PNG.
fn placeholder_block(width: usize, height: usize) -> Vec<String> {
    let row = format!("{}{}", "\x1b[38;5;238m", "·".repeat(width));
    (0..height).map(|_| format!("{row}{RESET}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{GameEntry, I18nEntry, I18nMap};

    fn sample_monster() -> Monster {
        Monster {
            id: String::new(),
            name: "Rathalos".into(),
            slug: "rathalos".into(),
            kind: "Flying Wyvern".into(),
            species: Some("flying wyvern".into()),
            is_large: true,
            sub_species: vec!["Azure Rathalos".into()],
            elements: vec!["Fire".into()],
            ailments: vec!["Fireblight".into(), "Poison".into()],
            weakness: vec!["Dragon".into(), "Thunder".into()],
            games: vec![
                GameEntry {
                    game: "mhw".into(),
                    game_full: "Monster Hunter World".into(),
                    info: Some("King of the Skies.".into()),
                    danger: None,
                    icon: Some("mhw/rathalos.png".into()),
                    icon_source: None,
                },
                GameEntry {
                    game: "mhwilds".into(),
                    game_full: "Monster Hunter Wilds".into(),
                    info: None,
                    danger: None,
                    icon: Some("mhwilds/rathalos.png".into()),
                    icon_source: None,
                },
            ],
            numeric: None,
            i18n: I18nMap {
                ja: Some(I18nEntry {
                    name: Some("リオレウス".into()),
                    desc: Some("天空の王者".into()),
                    source: Some("official".into()),
                }),
                zh: Some(I18nEntry {
                    name: Some("火龙".into()),
                    desc: Some("天空王者".into()),
                    source: Some("ai".into()),
                }),
            },
        }
    }

    #[test]
    fn card_renders_without_panic() {
        let m = sample_monster();
        let img = DynamicImage::new_rgba8(48, 48);
        let out = render_monster(&img, &m, 32, Lang::En);
        assert!(out.contains("Rathalos"));
        assert!(out.contains("Flying Wyvern"));
        assert!(out.contains("Fire"));
    }

    #[test]
    fn card_localizes_to_japanese() {
        let m = sample_monster();
        let img = DynamicImage::new_rgba8(48, 48);
        let out = render_monster(&img, &m, 32, Lang::Ja);
        // Name line: localized ja name prepended.
        assert!(out.contains("リオレウス"));
        // Type label localized.
        assert!(out.contains("種類"));
        assert!(out.contains("飛竜種"));
    }

    #[test]
    fn colored_term_uses_english_color_localized_text() {
        let s = colored_term("Fire", Lang::Ja);
        // Japanese text "火" wrapped in the Fire color escape.
        assert!(s.contains("火"));
        assert!(s.contains("\x1b[38;5;196m"));
    }

    #[test]
    fn placeholder_block_has_dimensions() {
        let b = placeholder_block(4, 3);
        assert_eq!(b.len(), 3);
        for row in &b {
            assert!(row.contains("·"));
        }
    }
}
