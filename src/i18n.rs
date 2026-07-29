//! Localization glue: pick the right name/desc for a record, plus the
//! CJK-aware width helpers the card layout depends on.
//!
//! Monster/item records carry an embedded `i18n.{ja,zh}.{name,desc}` overlay.
//! With `lang == en` we use the English fields directly; otherwise we read the
//! overlay, falling back to English when a translation is missing.
//!
//! Field labels and value terms (Type -> 種類, Fire -> 火) come from the
//! hardcoded tables in `i18n_terms` (not in the Release artifact).

use crate::data::{EndemicLife, I18nMap, Monster};

/// Display language. `En` short-circuits localization entirely.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Ja,
    Zh,
}

impl Lang {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" => Some(Self::En),
            "ja" | "jp" | "japanese" => Some(Self::Ja),
            "zh" | "cn" | "chinese" => Some(Self::Zh),
            _ => None,
        }
    }
}

/// Localized (name, description) for a monster, with English fallback.
pub fn monster_localized(m: &Monster, lang: Lang) -> (String, Option<String>) {
    if let Some(entry) = pick(&m.i18n, lang) {
        let name = match (&entry.name, lang) {
            (Some(n), _) if !n.is_empty() => format!("{n} {}", m.name),
            _ => m.name.clone(),
        };
        let desc = entry.desc.clone().filter(|d| !d.is_empty());
        return (name, desc);
    }
    (m.name.clone(), monster_english_desc(m))
}

/// Best available English description: numeric description, else first game info.
pub fn monster_english_desc(m: &Monster) -> Option<String> {
    if let Some(n) = &m.numeric {
        if let Some(w) = &n.wilds
            && let Some(d) = &w.description
        {
            return Some(d.clone());
        }
        if let Some(w) = &n.mhw
            && let Some(d) = &w.description
        {
            return Some(d.clone());
        }
    }
    m.games.iter().find_map(|g| g.info.clone())
}

/// Localized (name, description) for an endemic-life record, with English fallback.
pub fn endemic_localized(e: &EndemicLife, lang: Lang) -> (String, Option<String>) {
    if let Some(entry) = pick(&e.i18n, lang) {
        let name = match (&entry.name, lang) {
            (Some(n), _) if !n.is_empty() => format!("{n} {}", e.name),
            _ => e.name.clone(),
        };
        let desc = entry.desc.clone().filter(|d| !d.is_empty());
        return (name, desc);
    }
    (e.name.clone(), endemic_english_desc(e))
}

/// Best available English description for endemic life: first game info.
pub fn endemic_english_desc(e: &EndemicLife) -> Option<String> {
    e.games.iter().find_map(|g| g.info.clone())
}

/// Localized (name, description) for an item, with English fallback.
pub fn item_localized(
    name: &str,
    desc: Option<&str>,
    i18n: &I18nMap,
    lang: Lang,
) -> (String, Option<String>) {
    if let Some(entry) = pick(i18n, lang) {
        let loc_name = match (&entry.name, lang) {
            (Some(n), _) if !n.is_empty() => format!("{n} {name}"),
            _ => name.to_string(),
        };
        let loc_desc = entry.desc.clone().filter(|d| !d.is_empty());
        return (loc_name, loc_desc);
    }
    (name.to_string(), desc.map(|s| s.to_string()))
}

fn pick(map: &I18nMap, lang: Lang) -> Option<&crate::data::I18nEntry> {
    match lang {
        Lang::Ja => map.ja.as_ref(),
        Lang::Zh => map.zh.as_ref(),
        Lang::En => None,
    }
}

/// Translate an English token by lookup. Falls back to the input unchanged.
pub fn term(text: &str, lang: Lang) -> String {
    crate::i18n_terms::term(text, lang)
}

/// Translate a field label by lookup.
pub fn label(text: &str, lang: Lang) -> String {
    crate::i18n_terms::label(text, lang)
}

// =================================================== CJK-aware width ===

/// Visible width of a string: SGR escapes stripped, CJK chars counted as 2.
///
/// Approximates Python's `unicodedata.east_asian_width`: W/F count as 2,
/// everything else (including Ambiguous A) as 1. Good enough for the small
/// vocabularies we display (hiragana/katakana, CJK ideographs, Latin).
pub fn visible_len(s: &str) -> usize {
    let stripped = strip_sgr(s);
    stripped.chars().map(|c| char_width(c) as usize).sum()
}

/// Left-justify `s` to a visible width, padding on the right with spaces.
/// Never truncates (matches preview.py's `_pad_to`).
pub fn pad_to(s: &str, width: usize) -> String {
    let pad = width.saturating_sub(visible_len(s));
    format!("{s}{}", " ".repeat(pad))
}

/// Greedy char-by-char wrap to a visible width, accounting for CJK.
/// Returns at least one line. Each line's visible width is ≤ `width`.
pub fn wrap_cjk(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut cur_w = 0usize;
    for ch in text.chars() {
        let cw = char_width(ch) as usize;
        if cur_w + cw > width && !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
            cur_w = 0;
        }
        cur.push(ch);
        cur_w += cw;
    }
    if !cur.is_empty() || lines.is_empty() {
        lines.push(cur);
    }
    lines
}

/// Strip SGR escape sequences (`\x1b[...m`) so width math isn't corrupted.
fn strip_sgr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            while let Some(&n) = chars.peek() {
                chars.next();
                if n.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Approximate East Asian Width: 2 for W/F (wide / fullwidth), 1 otherwise.
///
/// Ranges cover the common cases we hit (Hiragana, Katakana, CJK Unified
/// Ideographs, fullwidth forms). Ambiguous (A) counts as 1, matching
/// preview.py and avoiding double-counting box-drawing chars in the layout.
fn char_width(c: char) -> u8 {
    let cp = c as u32;
    // ASCII control + space are narrow.
    if cp < 0x1100 {
        return 1;
    }
    let wide = matches!(cp,
        0x1100..=0x115F   // Hangul Jamo
        | 0x2E80..=0x303E // CJK radicals, Kangxi
        | 0x3041..=0x33FF // Hiragana, Katakana, CJK symbols
        | 0x3400..=0x4DBF // CJK Unified Ideographs Ext A
        | 0x4E00..=0x9FFF // CJK Unified Ideographs
        | 0xA000..=0xA4CF // Yi
        | 0xAC00..=0xD7A3 // Hangul Syllables
        | 0xF900..=0xFAFF // CJK Compatibility Ideographs
        | 0xFE30..=0xFE4F // CJK Compatibility Forms
        | 0xFF00..=0xFF60 // Fullwidth Forms
        | 0xFFE0..=0xFFE6 // Fullwidth signs
        | 0x1F300..=0x1FAFF // Emoji & symbols (treat as wide)
    );
    if wide { 2 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lang_parse() {
        assert_eq!(Lang::parse("en"), Some(Lang::En));
        assert_eq!(Lang::parse("JA"), Some(Lang::Ja));
        assert_eq!(Lang::parse("chinese"), Some(Lang::Zh));
        assert_eq!(Lang::parse("xx"), None);
    }

    #[test]
    fn visible_len_ascii() {
        assert_eq!(visible_len("hello"), 5);
        assert_eq!(visible_len(""), 0);
    }

    #[test]
    fn visible_len_cjk_counts_double() {
        // リ (katakana) and 雄 (ideograph) both count as width 2.
        assert_eq!(visible_len("リ"), 2);
        assert_eq!(visible_len("雄"), 2);
        assert_eq!(visible_len("A雄"), 3);
    }

    #[test]
    fn visible_len_strips_ansi() {
        let s = "\x1b[38;2;1;2;3mAB\x1b[0m".to_string();
        assert_eq!(visible_len(&s), 2);
    }

    #[test]
    fn pad_to_left_justifies() {
        assert_eq!(pad_to("AB", 5), "AB   ");
        assert_eq!(pad_to("雄", 4), "雄  "); // CJK + 2 spaces = width 4
        // Never truncates.
        assert_eq!(pad_to("hello", 3), "hello");
    }

    #[test]
    fn wrap_cjk_breaks_on_overflow() {
        let lines = wrap_cjk("abcde", 3);
        assert_eq!(lines, vec!["abc", "de"]);

        // CJK char counts as 2, so 3-width line fits 1 CJK + 1 ASCII.
        let lines = wrap_cjk("雄bc", 3);
        assert_eq!(lines, vec!["雄b", "c"]);
    }

    #[test]
    fn wrap_cjk_empty_returns_one_empty_line() {
        assert_eq!(wrap_cjk("", 5), vec![""]);
    }

    #[test]
    fn term_translates_known() {
        assert_eq!(term("Fire", Lang::Ja), "火");
        assert_eq!(term("Fire", Lang::Zh), "火");
        assert_eq!(term("Flying Wyvern", Lang::Ja), "飛竜種");
    }

    #[test]
    fn term_falls_back_to_input() {
        assert_eq!(term("UnknownElement", Lang::Ja), "UnknownElement");
    }

    #[test]
    fn label_translates() {
        assert_eq!(label("Type", Lang::Ja), "種類");
        assert_eq!(label("Rarity", Lang::Zh), "稀有度");
        assert_eq!(label("Unknown", Lang::En), "Unknown");
    }
}
