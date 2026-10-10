use super::normalize::normalize_search_text;

/// Inclusive UTF-8 byte ranges for highlighting in the original display string.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HighlightRange {
    pub start: usize,
    pub end: usize,
}

/// Find highlight ranges for query terms inside `text` (case/script tolerant).
pub fn highlight_ranges(text: &str, query: &str) -> Vec<HighlightRange> {
    let terms: Vec<Vec<char>> = normalize_search_text(query)
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| t.chars().collect())
        .collect();
    if terms.is_empty() || text.is_empty() {
        return Vec::new();
    }

    // Normalized char stream with mapping back to original byte offsets.
    let mut norm_chars: Vec<char> = Vec::new();
    let mut norm_to_byte: Vec<usize> = Vec::new();
    for (byte_i, ch) in text.char_indices() {
        let norm = normalize_search_text(&ch.to_string());
        let norm_ch = norm.chars().next().unwrap_or(ch);
        if norm_ch.is_whitespace() {
            if norm_chars.last().is_some_and(|c| !c.is_whitespace()) {
                norm_chars.push(' ');
                norm_to_byte.push(byte_i);
            }
        } else {
            norm_chars.push(norm_ch);
            norm_to_byte.push(byte_i);
        }
    }

    let mut ranges = Vec::new();
    for term in &terms {
        if term.is_empty() || term.len() > norm_chars.len() {
            continue;
        }
        let last_start = norm_chars.len() - term.len();
        for i in 0..=last_start {
            if &norm_chars[i..i + term.len()] == term.as_slice() {
                let byte_start = norm_to_byte[i];
                let last_char_index = i + term.len() - 1;
                let last_byte = norm_to_byte[last_char_index];
                let byte_end = text[last_byte..]
                    .chars()
                    .next()
                    .map(|c| last_byte + c.len_utf8())
                    .unwrap_or(text.len());
                ranges.push(HighlightRange {
                    start: byte_start,
                    end: byte_end,
                });
            }
        }
    }

    ranges.sort_by_key(|r| r.start);
    merge_ranges(ranges)
}

fn merge_ranges(ranges: Vec<HighlightRange>) -> Vec<HighlightRange> {
    let mut out: Vec<HighlightRange> = Vec::new();
    for r in ranges {
        if let Some(last) = out.last_mut() {
            if r.start <= last.end {
                last.end = last.end.max(r.end);
                continue;
            }
        }
        out.push(r);
    }
    out
}

/// Build a short snippet around the first match (preserves original characters).
pub fn make_snippet(text: &str, query: &str, max_chars: usize) -> String {
    let ranges = highlight_ranges(text, query);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if ranges.is_empty() {
        return truncate_chars(trimmed, max_chars);
    }
    let match_start = ranges[0].start.min(trimmed.len());
    // Walk back to a char boundary.
    let match_start = floor_char_boundary(trimmed, match_start);
    let prefix_budget = max_chars / 3;
    let start_char = trimmed[..match_start]
        .chars()
        .count()
        .saturating_sub(prefix_budget);
    let sliced: String = trimmed.chars().skip(start_char).take(max_chars).collect();
    let mut snippet = sliced;
    if start_char > 0 {
        snippet = format!("\u{2026}{snippet}");
    }
    if trimmed.chars().count() > start_char + max_chars {
        snippet.push('\u{2026}');
    }
    snippet
}

fn floor_char_boundary(text: &str, index: usize) -> usize {
    if index >= text.len() {
        return text.len();
    }
    let mut i = index;
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    let mut s: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    s.push('\u{2026}');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_english_case_insensitive() {
        let ranges = highlight_ranges("Learn Java programming", "java");
        assert!(!ranges.is_empty());
        let r = &ranges[0];
        assert_eq!(&"Learn Java programming"[r.start..r.end], "Java");
    }

    #[test]
    fn highlights_persian() {
        let text = "درس فلسفه غرب";
        let ranges = highlight_ranges(text, "فلسفه");
        assert!(!ranges.is_empty());
        let r = &ranges[0];
        assert!(text.is_char_boundary(r.start));
        assert!(text.is_char_boundary(r.end));
        assert!(text[r.start..r.end].contains("فلسف"));
    }
}
