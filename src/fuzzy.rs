//! Tiny Levenshtein edit-distance for typo-tolerant matching.
//!
//! No external crate; the full algorithm is ~30 lines. Used as the last-resort
//! fallback in `search` (after exact slug/name and substring matching) to catch
//! typos like "rathalas" -> "rathalos".

/// Edit distance between `a` and `b` (insertions / deletions / substitutions).
///
/// Iterative two-row DP. Returns 0 for identical strings; cost is symmetric.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    // Previous row: distances for prefix of `a` vs prefix of `b` of length j-1.
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr: Vec<usize> = vec![0; b.len() + 1];

    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1) // deletion
                .min(curr[j - 1] + 1) // insertion
                .min(prev[j - 1] + cost); // substitution
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_is_zero() {
        assert_eq!(levenshtein("rathalos", "rathalos"), 0);
    }

    #[test]
    fn empty_string_costs_length() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn single_substitution() {
        // rathalas vs rathalos: one substitution (a->o).
        assert_eq!(levenshtein("rathalas", "rathalos"), 1);
    }

    #[test]
    fn single_insertion_and_deletion() {
        assert_eq!(levenshtein("cat", "cats"), 1);
        assert_eq!(levenshtein("cats", "cat"), 1);
    }

    #[test]
    fn is_symmetric() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("sitting", "kitten"), 3);
    }

    #[test]
    fn case_sensitive() {
        // Matches preview.py behavior: queries are lowercased before compare,
        // so the distance itself is intentionally case-sensitive here.
        assert_eq!(levenshtein("Rathalos", "rathalos"), 1);
    }
}
