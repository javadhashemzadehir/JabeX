use super::normalize::normalize_search_text;

/// Levenshtein distance (Unicode scalar values).
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

/// Similarity in `0.0..=1.0` (1 = identical).
pub fn similarity(a: &str, b: &str) -> f64 {
    let a = normalize_search_text(a);
    let b = normalize_search_text(b);
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let dist = levenshtein(&a, &b) as f64;
    let max_len = a.chars().count().max(b.chars().count()) as f64;
    (1.0 - dist / max_len).max(0.0)
}

/// Best fuzzy similarity of `needle` against any whitespace token / whole haystack.
pub fn best_token_similarity(needle: &str, haystack: &str) -> f64 {
    let needle = normalize_search_text(needle);
    let haystack = normalize_search_text(haystack);
    if needle.is_empty() {
        return 0.0;
    }
    let mut best = similarity(&needle, &haystack);
    for token in haystack.split_whitespace() {
        best = best.max(similarity(&needle, token));
        if token.starts_with(&needle) || needle.starts_with(token) {
            best = best.max(0.85);
        }
    }
    best
}

/// Generate short edit-distance variants for FTS OR expansion (typo tolerance).
pub fn fuzzy_variants(token: &str) -> Vec<String> {
    let token = normalize_search_text(token);
    if token.chars().count() < 3 {
        return vec![token];
    }
    let chars: Vec<char> = token.chars().collect();
    let mut out = vec![token.clone()];

    // Deletions
    for i in 0..chars.len() {
        let mut v = chars.clone();
        v.remove(i);
        out.push(v.into_iter().collect());
    }
    // Adjacent swaps
    for i in 0..chars.len().saturating_sub(1) {
        let mut v = chars.clone();
        v.swap(i, i + 1);
        out.push(v.into_iter().collect());
    }

    out.sort();
    out.dedup();
    // Cap expansion for performance
    out.truncate(24);
    out
}

/// Whether `candidate` is a fuzzy hit for `query` (typo / missing / swap).
pub fn is_fuzzy_match(query: &str, candidate: &str) -> bool {
    let q = normalize_search_text(query);
    let c = normalize_search_text(candidate);
    if q.is_empty() || c.is_empty() {
        return false;
    }
    if c.contains(&q) || q.contains(&c) {
        return true;
    }
    let max_dist = match q.chars().count() {
        0..=3 => 1,
        4..=6 => 2,
        _ => 3,
    };
    if levenshtein(&q, &c) <= max_dist {
        return true;
    }
    c.split_whitespace()
        .any(|token| levenshtein(&q, token) <= max_dist)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn javascript_typo() {
        assert!(is_fuzzy_match("javscript", "javascript"));
        assert!(best_token_similarity("javscript", "learn javascript today") > 0.7);
    }

    #[test]
    fn persian_prefix_typo() {
        assert!(is_fuzzy_match("فلسف", "فلسفه"));
        assert!(best_token_similarity("فلسف", "درس فلسفه غرب") > 0.7);
    }

    #[test]
    fn levenshtein_swap() {
        assert_eq!(levenshtein("ab", "ba"), 2.min(levenshtein("ab", "ba")));
        assert!(levenshtein("form", "from") <= 2);
    }
}
