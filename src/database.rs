//! Embedded assets and in-memory database.
//!
//! At build time rust-embed bakes `data/` + `icons/` into the binary (path set
//! by build.rs). At runtime `Database::load()` deserializes the two JSON files
//! once and builds slug-to-index maps for O(1) lookup.

// `monster_by_slug` / `item_by_slug` and the index fields form the public
// lookup API; they're used by tests and intended for future subcommands, even
// when the current main path goes through `search::search`.
#![allow(dead_code)]

use std::collections::HashMap;

use rust_embed::{EmbeddedFile, RustEmbed};

use crate::data::{Item, Monster};

#[derive(RustEmbed)]
#[folder = "$ASSETS_DIR/"]
pub struct Assets;

#[derive(Debug)]
pub struct Database {
    pub monsters: Vec<Monster>,
    pub items: Vec<Item>,
    /// slug to index into `monsters`
    pub monster_index: HashMap<String, usize>,
    /// slug to index into `items`
    pub item_index: HashMap<String, usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("embedded asset not found: {0}")]
    Missing(String),
    #[error("failed to parse {file}: {error}")]
    Parse { file: &'static str, error: String },
}

impl Database {
    /// Deserialize the embedded JSON and build lookup indices.
    pub fn load() -> Result<Self, LoadError> {
        let monsters: Vec<Monster> = parse_embedded("data/monsters.json")?;
        let items: Vec<Item> = parse_embedded("data/items.json")?;

        let monster_index = build_index(&monsters, |m| &m.slug);
        let item_index = build_index(&items, |i| &i.slug);

        Ok(Self {
            monsters,
            items,
            monster_index,
            item_index,
        })
    }

    pub fn monster_by_slug(&self, slug: &str) -> Option<&Monster> {
        self.monster_index.get(slug).map(|&i| &self.monsters[i])
    }

    pub fn item_by_slug(&self, slug: &str) -> Option<&Item> {
        self.item_index.get(slug).map(|&i| &self.items[i])
    }

    /// Fetch an embedded file by relative path, such as "icons/mhw/rathalos.png".
    ///
    /// `rust_embed::get` returns an owned `EmbeddedFile`; its `.data` is a
    /// `Cow<'static, [u8]>` borrowing the binary's read-only data segment, so
    /// callers can keep the slice for `'static`.
    pub fn asset(path: &str) -> Result<EmbeddedFile, LoadError> {
        Assets::get(path).ok_or_else(|| LoadError::Missing(path.to_string()))
    }
}

fn parse_embedded<T: serde::de::DeserializeOwned>(path: &'static str) -> Result<T, LoadError> {
    let file = Database::asset(path)?;
    serde_json::from_slice(&file.data).map_err(|e| LoadError::Parse {
        file: path,
        error: e.to_string(),
    })
}

fn build_index<T, S: AsRef<str> + ?Sized>(
    records: &[T],
    slug: impl Fn(&T) -> &S,
) -> HashMap<String, usize> {
    // First occurrence wins; later duplicates are ignored, matching the asset
    // build (which dedupes upstream).
    records
        .iter()
        .enumerate()
        .fold(HashMap::with_capacity(records.len()), |mut acc, (i, r)| {
            acc.entry(slug(r).as_ref().to_string()).or_insert(i);
            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_records() {
        // Lower bounds, not exact counts: the payload size depends on which
        // zukan-assets Release build.rs pulled, and changes over time.
        let db = Database::load().expect("embedded assets should parse");
        assert!(
            db.monsters.len() >= 300,
            "monster count: {}",
            db.monsters.len()
        );
        assert!(db.items.len() >= 1500, "item count: {}", db.items.len());
        assert!(
            db.monster_by_slug("rathalos").is_some(),
            "rathalos should be indexed"
        );
        assert!(
            db.item_by_slug("mega-potion").is_some(),
            "mega-potion should be indexed"
        );
    }

    #[test]
    fn monster_index_dedupes_duplicate_slugs() {
        // zukan-assets has shipped duplicate slugs before (Versa Pietru).
        // Newer builds dedup upstream, older Release artifacts don't. Either
        // way the index keeps the first occurrence and drops the rest.
        let db = Database::load().expect("parse");
        assert!(
            db.monster_index.len() <= db.monsters.len(),
            "index ({}) should never exceed records ({})",
            db.monster_index.len(),
            db.monsters.len()
        );
        // Known slugs must always resolve regardless of dedup state.
        assert!(db.monster_by_slug("rathalos").is_some());
        assert!(db.monster_by_slug("versa-pietru").is_some());
    }

    #[test]
    fn every_monster_has_ja_and_zh() {
        let db = Database::load().expect("parse");
        let offenders: Vec<&str> = db
            .monsters
            .iter()
            .filter(|m| m.i18n.ja.is_none() || m.i18n.zh.is_none())
            .map(|m| m.slug.as_str())
            .collect();
        assert!(offenders.is_empty(), "monsters missing i18n: {offenders:?}");
    }

    #[test]
    fn rathalos_has_all_three_numeric_subkeys() {
        let db = Database::load().expect("parse");
        let m = db.monster_by_slug("rathalos").expect("rathalos");
        let n = m.numeric.as_ref().expect("rathalos has numeric");
        assert!(n.mhw.is_some(), "missing mhw");
        assert!(n.wilds.is_some(), "missing wilds");
        assert!(n.mhgu.is_some(), "missing mhgu");
    }
}
