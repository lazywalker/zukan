//! Query engine: free-text queries to ranked record matches.
//!
//! Strategy (first non-empty tier wins):
//!
//! 1. Exact slug (case-insensitive).
//! 2. Exact display name (case-insensitive).
//! 3. Substring of slug or name.
//! 4. Levenshtein <= 2 (typo tolerance, such as "rathalas" -> "rathalos").
//!
//! Each tier runs only if the previous produced no matches, so a precise query
//! never degrades into fuzzy results.

use crate::data::{Item, Monster};
use crate::fuzzy::levenshtein;

/// Anything searchable has a slug and a display name.
pub trait Searchable {
    fn slug(&self) -> &str;
    fn name(&self) -> &str;
}

impl Searchable for Monster {
    fn slug(&self) -> &str {
        &self.slug
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl Searchable for Item {
    fn slug(&self) -> &str {
        &self.slug
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// Find records matching `query`, ranked best-first. Returns slice indices so
/// the caller can deref into the source `Vec`.
///
/// `limit` caps the candidate list for broad queries (empty string, one
/// letter); `0` means no cap.
pub fn search<T: Searchable>(records: &[T], query: &str, limit: usize) -> Vec<usize> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();

    // Tier 1: exact slug.
    for (i, r) in records.iter().enumerate() {
        if r.slug().eq_ignore_ascii_case(&q) {
            matches.push(i);
        }
    }
    if !matches.is_empty() {
        return cap(matches, limit);
    }

    // Tier 2: exact name.
    for (i, r) in records.iter().enumerate() {
        if r.name().eq_ignore_ascii_case(&q) {
            matches.push(i);
        }
    }
    if !matches.is_empty() {
        return cap(matches, limit);
    }

    // Tier 3: substring of slug or name.
    for (i, r) in records.iter().enumerate() {
        if r.slug().to_lowercase().contains(&q) || r.name().to_lowercase().contains(&q) {
            matches.push(i);
        }
    }
    if !matches.is_empty() {
        return cap(matches, limit);
    }

    // Tier 4: typo tolerance via Levenshtein <= 2 against slug or name.
    for (i, r) in records.iter().enumerate() {
        let ds = levenshtein(&r.slug().to_lowercase(), &q);
        let dn = levenshtein(&r.name().to_lowercase(), &q);
        if ds.min(dn) <= 2 {
            matches.push(i);
        }
    }
    // Sort fuzzy hits by ascending edit distance (min of slug/name distance).
    matches.sort_by_key(|&i| {
        let r = &records[i];
        levenshtein(&r.slug().to_lowercase(), &q).min(levenshtein(&r.name().to_lowercase(), &q))
    });
    cap(matches, limit)
}

/// Pick a uniformly random record index, optionally filtered by `predicate`.
/// Returns `None` if the filtered slice is empty.
pub fn random<T>(records: &[T], predicate: impl Fn(&T) -> bool) -> Option<usize> {
    let candidates: Vec<usize> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| predicate(r))
        .map(|(i, _)| i)
        .collect();
    if candidates.is_empty() {
        None
    } else {
        Some(candidates[fastrand::usize(..candidates.len())])
    }
}

fn cap(mut v: Vec<usize>, limit: usize) -> Vec<usize> {
    if limit != 0 && v.len() > limit {
        v.truncate(limit);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{GameEntry, Monster};

    fn monster(slug: &str, name: &str) -> Monster {
        Monster {
            id: String::new(),
            name: name.to_string(),
            slug: slug.to_string(),
            kind: "Test".to_string(),
            species: None,
            is_large: true,
            sub_species: Vec::new(),
            elements: Vec::new(),
            ailments: Vec::new(),
            weakness: Vec::new(),
            games: Vec::<GameEntry>::new(),
            numeric: None,
            i18n: Default::default(),
        }
    }

    fn sample_db() -> Vec<Monster> {
        vec![
            monster("rathalos", "Rathalos"),
            monster("rathian", "Rathian"),
            monster("azure-rathalos", "Azure Rathalos"),
            monster("diablos", "Diablos"),
            monster("deviljho", "Deviljho"),
        ]
    }

    #[test]
    fn exact_slug_returns_single_match() {
        let db = sample_db();
        let m = search(&db, "rathalos", 0);
        assert_eq!(m, vec![0]);
    }

    #[test]
    fn exact_slug_case_insensitive() {
        let db = sample_db();
        let m = search(&db, "RATHALOS", 0);
        assert_eq!(m, vec![0]);
    }

    #[test]
    fn exact_name_match_tier_2() {
        let db = sample_db();
        let m = search(&db, "Diablos", 0);
        assert_eq!(m, vec![3]);
    }

    #[test]
    fn substring_returns_all_matches() {
        let db = sample_db();
        let m = search(&db, "rath", 0);
        // rathalos, rathian, azure-rathalos all contain "rath".
        assert_eq!(m.len(), 3);
        assert!(m.contains(&0));
        assert!(m.contains(&1));
        assert!(m.contains(&2));
    }

    #[test]
    fn typo_falls_through_to_levenshtein() {
        let db = sample_db();
        // "rathalas" is one substitution from "rathalos".
        let m = search(&db, "rathalas", 0);
        assert_eq!(m, vec![0]);
    }

    #[test]
    fn no_match_returns_empty() {
        let db = sample_db();
        let m = search(&db, "zzzzzzz", 0);
        assert!(m.is_empty());
    }

    #[test]
    fn empty_query_returns_empty() {
        let db = sample_db();
        assert!(search(&db, "", 0).is_empty());
        assert!(search(&db, "   ", 0).is_empty());
    }

    #[test]
    fn limit_truncates_results() {
        let db = sample_db();
        let m = search(&db, "rath", 2);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn fuzzy_results_sorted_by_distance() {
        let db = sample_db();
        // "rathalas" is distance 1 from rathalos, distance ~4 from rathian.
        let m = search(&db, "rathalas", 0);
        assert_eq!(m.first(), Some(&0));
    }

    #[test]
    fn random_returns_valid_index() {
        let db = sample_db();
        let i = random(&db, |_| true).expect("non-empty db");
        assert!(i < db.len());
    }

    #[test]
    fn random_respects_predicate() {
        let db = sample_db();
        // All entries are large, so this behaves like no filter.
        let i = random(&db, |m: &Monster| m.is_large).expect("some large");
        assert!(i < db.len());
    }

    #[test]
    fn random_empty_predicate_returns_none() {
        let db = sample_db();
        assert!(random(&db, |_| false).is_none());
    }
}
