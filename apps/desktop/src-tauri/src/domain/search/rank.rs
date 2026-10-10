use super::entity::SearchEntityType;
use super::fuzzy::{best_token_similarity, is_fuzzy_match};
use super::normalize::normalize_search_text;

/// Ranking priority (higher = better).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchKind {
    Content = 1,
    Fuzzy = 2,
    Prefix = 3,
    ExactTag = 4,
    ExactFront = 5,
    ExactTitle = 6,
}

#[derive(Debug, Clone)]
pub struct RankedScore {
    pub kind: MatchKind,
    pub score: f64,
}

/// Score a candidate row against the user query.
pub fn rank_match(
    query: &str,
    title: &str,
    content: &str,
    entity_type: SearchEntityType,
    metadata_front: Option<&str>,
) -> RankedScore {
    let q = normalize_search_text(query);
    if q.is_empty() {
        return RankedScore {
            kind: MatchKind::Content,
            score: 0.0,
        };
    }
    let title_n = normalize_search_text(title);
    let content_n = normalize_search_text(content);
    let front_n = metadata_front
        .map(normalize_search_text)
        .unwrap_or_default();

    if title_n == q {
        return RankedScore {
            kind: MatchKind::ExactTitle,
            score: 1000.0,
        };
    }
    if !front_n.is_empty() && front_n == q {
        return RankedScore {
            kind: MatchKind::ExactFront,
            score: 900.0,
        };
    }
    if entity_type == SearchEntityType::Tag && (title_n == q || title_n == format!("#{q}")) {
        return RankedScore {
            kind: MatchKind::ExactTag,
            score: 850.0,
        };
    }
    if title_n.starts_with(&q) || title_n.split_whitespace().any(|t| t.starts_with(&q)) {
        return RankedScore {
            kind: MatchKind::Prefix,
            score: 700.0 + best_token_similarity(&q, &title_n) * 50.0,
        };
    }
    if is_fuzzy_match(&q, &title_n) || title_n.split_whitespace().any(|t| is_fuzzy_match(&q, t)) {
        return RankedScore {
            kind: MatchKind::Fuzzy,
            score: 500.0 + best_token_similarity(&q, &title_n) * 100.0,
        };
    }
    if content_n.contains(&q) {
        return RankedScore {
            kind: MatchKind::Content,
            score: 300.0 + best_token_similarity(&q, &content_n) * 50.0,
        };
    }
    if is_fuzzy_match(&q, &content_n) || content_n.split_whitespace().any(|t| is_fuzzy_match(&q, t))
    {
        return RankedScore {
            kind: MatchKind::Fuzzy,
            score: 200.0 + best_token_similarity(&q, &content_n) * 80.0,
        };
    }

    RankedScore {
        kind: MatchKind::Content,
        score: best_token_similarity(&q, &title_n).max(best_token_similarity(&q, &content_n))
            * 100.0,
    }
}
