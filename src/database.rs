//! Embedded assets and in-memory database.
//!
//! At build time build.rs walks `data/` + `icons/` and emits a static
//! `(path, &[u8])` table to `OUT_DIR/assets_gen.rs` (included below). At
//! runtime `Database::load()` deserializes the two JSON files once and builds
//! slug-to-index maps

// `monster_by_slug` / `item_by_slug` and the index fields form the public
// lookup API; they're used by tests and intended for future subcommands, even
// when the current main path goes through `search::search`.
#![allow(dead_code)]

use std::collections::HashMap;

use crate::data::{EndemicLife, Item, Monster};

include!(concat!(env!("OUT_DIR"), "/assets_gen.rs"));

#[derive(Debug)]
pub struct Database {
    pub monsters: Vec<Monster>,
    pub items: Vec<Item>,
    pub endemics: Vec<EndemicLife>,
    /// slug to index into `monsters`
    pub monster_index: HashMap<String, usize>,
    /// slug to index into `items`
    pub item_index: HashMap<String, usize>,
    /// slug to index into `endemics`
    pub endemic_index: HashMap<String, usize>,
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
        let endemics: Vec<EndemicLife> = parse_embedded("data/endemic_life.json")?;

        let monster_index = build_index(&monsters, |m| &m.slug);
        let item_index = build_index(&items, |i| &i.slug);
        let endemic_index = build_index(&endemics, |e| &e.slug);

        Ok(Self {
            monsters,
            items,
            endemics,
            monster_index,
            item_index,
            endemic_index,
        })
    }

    pub fn monster_by_slug(&self, slug: &str) -> Option<&Monster> {
        self.monster_index.get(slug).map(|&i| &self.monsters[i])
    }

    pub fn item_by_slug(&self, slug: &str) -> Option<&Item> {
        self.item_index.get(slug).map(|&i| &self.items[i])
    }

    pub fn endemic_by_slug(&self, slug: &str) -> Option<&EndemicLife> {
        self.endemic_index.get(slug).map(|&i| &self.endemics[i])
    }

    /// Fetch an embedded asset by relative path, like "icons/mhw/rathalos.png".
    /// The returned slice is `'static` (lives in the binary's read-only data).
    pub fn asset(path: &str) -> Result<&'static [u8], LoadError> {
        ASSETS
            .iter()
            .find(|(p, _)| *p == path)
            .map(|(_, data)| *data)
            .ok_or_else(|| LoadError::Missing(path.to_string()))
    }
}

fn parse_embedded<T: serde::de::DeserializeOwned>(path: &'static str) -> Result<T, LoadError> {
    let data = Database::asset(path)?;
    serde_json::from_slice(data).map_err(|e| LoadError::Parse {
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
    fn mhdb_monsters_have_ja_and_zh() {
        let db = Database::load().expect("parse");
        // Supplement monsters (MHO/MH4U exclusives) have no i18n source; skip
        // them by the id==None marker MHDB-native monsters don't share.
        let offenders: Vec<&str> = db
            .monsters
            .iter()
            .filter(|m| m.id.is_some() && (m.i18n.ja.is_none() || m.i18n.zh.is_none()))
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

    #[test]
    fn loads_endemic_records() {
        let db = Database::load().expect("parse");
        assert!(
            db.endemics.len() >= 100,
            "endemic count: {}",
            db.endemics.len()
        );
        assert!(
            db.endemic_by_slug("andangler").is_some(),
            "andangler should be indexed"
        );
    }

    #[test]
    fn every_endemic_has_ja_and_zh() {
        let db = Database::load().expect("parse");
        let offenders: Vec<&str> = db
            .endemics
            .iter()
            .filter(|e| e.i18n.ja.is_none() || e.i18n.zh.is_none())
            .map(|e| e.slug.as_str())
            .collect();
        assert!(offenders.is_empty(), "endemics missing i18n: {offenders:?}");
    }
}
