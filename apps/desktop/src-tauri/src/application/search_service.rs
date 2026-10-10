use std::collections::HashSet;

use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::domain::attachment::Attachment;
use crate::domain::card::Card;
use crate::domain::deck::Deck;
use crate::domain::errors::JabeXResult;
use crate::domain::repositories::{
    AttachmentRepository, CardRepository, DeckRepository, RecentSearchRepository,
    SearchHistoryRepository, SearchIndexRepository,
};
use crate::domain::search::{
    best_token_similarity, detect_text_direction, extract_search_text, fuzzy_variants,
    highlight_ranges, make_snippet, normalize_latex, normalize_search_text, rank_match,
    PopularSearch, RecentSearch, SearchEntityType, SearchFilter, SearchHistoryEntry,
    SearchIndexDocument, SearchResponse, SearchResult, DEFAULT_SEARCH_LIMIT, POPULAR_SEARCHES_MAX,
    RECENT_SEARCHES_MAX, SEARCH_HISTORY_MAX,
};
use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

/// Global search use-cases: index lifecycle, query, recent + history (local only).
pub struct SearchService<
    I: SearchIndexRepository,
    R: RecentSearchRepository,
    H: SearchHistoryRepository,
    C: CardRepository,
    D: DeckRepository,
    A: AttachmentRepository,
> {
    index: I,
    recent: R,
    history: H,
    cards: C,
    decks: D,
    attachments: A,
    profile_id: Uuid,
}

impl<
        I: SearchIndexRepository,
        R: RecentSearchRepository,
        H: SearchHistoryRepository,
        C: CardRepository,
        D: DeckRepository,
        A: AttachmentRepository,
    > SearchService<I, R, H, C, D, A>
{
    pub fn new(
        index: I,
        recent: R,
        history: H,
        cards: C,
        decks: D,
        attachments: A,
        profile_id: Uuid,
    ) -> Self {
        Self {
            index,
            recent,
            history,
            cards,
            decks,
            attachments,
            profile_id,
        }
    }

    pub fn search(
        &self,
        query: &str,
        filter: SearchFilter,
        limit: Option<usize>,
    ) -> JabeXResult<SearchResponse> {
        self.ensure_index()?;
        let limit = limit.unwrap_or(DEFAULT_SEARCH_LIMIT).clamp(1, 100);
        let query = query.trim();
        if query.is_empty() {
            return Ok(SearchResponse {
                results: Vec::new(),
                query: String::new(),
                filter,
            });
        }

        let entity_types = filter.entity_types();
        let normalized = normalize_search_text(query);
        let fts_query = build_fts_query(&normalized);
        let mut candidates =
            self.index
                .search_candidates(&fts_query, entity_types, limit.saturating_mul(3))?;

        if candidates.len() < limit {
            for token in normalized.split_whitespace() {
                let char_len = token.chars().count();
                if char_len < 3 {
                    continue;
                }
                // Short prefix catches typos like javscript → javascript.
                // Keep LIKE fallback windows small so large indexes stay responsive.
                for prefix_len in [3usize, 4] {
                    if char_len < prefix_len {
                        continue;
                    }
                    let prefix: String = token.chars().take(prefix_len).collect();
                    let extra = self.index.search_normalized_fallback(
                        &prefix,
                        entity_types,
                        limit.saturating_mul(2).min(40),
                    )?;
                    merge_docs(&mut candidates, extra);
                }
                for variant in fuzzy_variants(token) {
                    let extra = self.index.search_normalized_fallback(
                        &variant,
                        entity_types,
                        limit.min(20),
                    )?;
                    merge_docs(&mut candidates, extra);
                }
            }
        }

        // Reviews filter: prefer due cards (metadata.due == true) after ranking.
        let mut ranked: Vec<SearchResult> = candidates
            .into_iter()
            .filter_map(|doc| self.to_result(query, &normalized, doc, filter))
            .collect();

        ranked.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.title.cmp(&b.title))
        });
        ranked.truncate(limit);

        Ok(SearchResponse {
            results: ranked,
            query: query.to_string(),
            filter,
        })
    }

    pub fn ensure_index(&self) -> JabeXResult<()> {
        if self.index.count()? == 0 {
            self.rebuild_index()?;
        }
        Ok(())
    }

    pub fn rebuild_index(&self) -> JabeXResult<u64> {
        self.index.clear()?;
        let decks = self.decks.list()?;
        let mut deck_names = std::collections::HashMap::new();
        for deck in &decks {
            deck_names.insert(deck.id, deck.name.clone());
            let cards = self.cards.list_by_deck(deck.id)?;
            self.index_deck(deck, cards.len() as u32)?;
        }

        let cards = self.cards.list_all(false)?;
        for card in &cards {
            let deck_name = deck_names.get(&card.deck_id).cloned().unwrap_or_default();
            let attachments = self.attachments.list_by_owner(
                crate::domain::attachment::AttachmentOwner::Card,
                card.id,
                false,
            )?;
            self.index_card(card, &deck_name, &attachments)?;
        }

        for attachment in self.attachments.list_all(false)? {
            self.index_attachment(&attachment)?;
        }

        self.index_static_commands_and_settings()?;
        self.index.count()
    }

    pub fn index_card(
        &self,
        card: &Card,
        deck_name: &str,
        attachments: &[Attachment],
    ) -> JabeXResult<()> {
        let front = extract_search_text(&card.front_content.to_storage_string());
        let back = extract_search_text(&card.back_content.to_storage_string());
        let mut content_parts = vec![front.combined_content(), back.combined_content()];
        if let Some(translation) = &card.translation {
            content_parts.push(translation.clone());
        }
        let attachment_names: Vec<String> =
            attachments.iter().map(|a| a.file_name.clone()).collect();
        if !attachment_names.is_empty() {
            content_parts.push(attachment_names.join(" "));
        }

        let title = if front.text.is_empty() {
            "Untitled card".into()
        } else {
            truncate(&front.text, 120)
        };
        let content = content_parts.join("\n");
        let normalized = normalize_search_text(&content);
        let due = card.due_at <= Utc::now() && card.deleted_at.is_none();
        let language = card.detected_language.clone();
        let metadata = json!({
            "deckId": card.deck_id,
            "deckName": deck_name,
            "front": front.text,
            "back": back.text,
            "due": due,
            "attachmentNames": attachment_names,
            "latex": [front.latex_raw, back.latex_raw],
        });

        let doc = SearchIndexDocument {
            id: create_id(),
            profile_id: self.profile_id,
            entity_type: SearchEntityType::Card,
            entity_id: card.id.to_string(),
            title: title.clone(),
            content: content.clone(),
            normalized_content: normalized,
            metadata,
            language,
            created_at: card.created_at,
            updated_at: now_utc(),
        };
        self.index.upsert(&doc)?;

        // Hashtag â†’ Tag entities (extensible until a first-class Tag table exists).
        for tag in extract_hashtags(&format!("{} {}", front.text, back.text)) {
            let tag_doc = SearchIndexDocument {
                id: create_id(),
                profile_id: self.profile_id,
                entity_type: SearchEntityType::Tag,
                entity_id: format!("tag:{}", normalize_search_text(&tag)),
                title: format!("#{tag}"),
                content: format!("{tag} {deck_name} {title}"),
                normalized_content: normalize_search_text(&format!("{tag} {deck_name} {title}")),
                metadata: json!({ "deckName": deck_name, "sourceCardId": card.id }),
                language: None,
                created_at: card.created_at,
                updated_at: now_utc(),
            };
            self.index.upsert(&tag_doc)?;
        }

        // Due cards also appear under Reviews filter.
        if due {
            let review_doc = SearchIndexDocument {
                id: create_id(),
                profile_id: self.profile_id,
                entity_type: SearchEntityType::Review,
                entity_id: format!("review:{}", card.id),
                title: title.clone(),
                content: content.clone(),
                normalized_content: normalize_search_text(&content),
                metadata: json!({
                    "deckId": card.deck_id,
                    "deckName": deck_name,
                    "cardId": card.id,
                    "due": true,
                }),
                language: card.detected_language.clone(),
                created_at: card.created_at,
                updated_at: now_utc(),
            };
            self.index.upsert(&review_doc)?;
        } else {
            let _ = self
                .index
                .remove(SearchEntityType::Review, &format!("review:{}", card.id));
        }

        Ok(())
    }

    pub fn index_deck(&self, deck: &Deck, card_count: u32) -> JabeXResult<()> {
        let desc = deck.description.clone().unwrap_or_default();
        let content = format!("{} {}", deck.name, desc);
        let doc = SearchIndexDocument {
            id: create_id(),
            profile_id: self.profile_id,
            entity_type: SearchEntityType::Deck,
            entity_id: deck.id.to_string(),
            title: deck.name.clone(),
            content: content.clone(),
            normalized_content: normalize_search_text(&content),
            metadata: json!({ "cardCount": card_count, "description": desc }),
            language: None,
            created_at: deck.created_at,
            updated_at: now_utc(),
        };
        self.index.upsert(&doc)
    }

    pub fn index_attachment(&self, attachment: &Attachment) -> JabeXResult<()> {
        let content = format!(
            "{} {} {}",
            attachment.file_name, attachment.mime_type, attachment.relative_path
        );
        let doc = SearchIndexDocument {
            id: create_id(),
            profile_id: self.profile_id,
            entity_type: SearchEntityType::Attachment,
            entity_id: attachment.id.to_string(),
            title: attachment.file_name.clone(),
            content: content.clone(),
            normalized_content: normalize_search_text(&content),
            metadata: json!({
                "ownerType": attachment.owner_type.as_str(),
                "ownerId": attachment.owner_id,
                "mimeType": attachment.mime_type,
            }),
            language: None,
            created_at: attachment.created_at,
            updated_at: now_utc(),
        };
        self.index.upsert(&doc)
    }

    pub fn remove_entity(&self, entity_type: SearchEntityType, entity_id: &str) -> JabeXResult<()> {
        self.index.remove(entity_type, entity_id)?;
        if entity_type == SearchEntityType::Card {
            let _ = self
                .index
                .remove(SearchEntityType::Review, &format!("review:{entity_id}"));
        }
        Ok(())
    }

    pub fn list_recent_searches(&self, limit: Option<usize>) -> JabeXResult<Vec<RecentSearch>> {
        self.recent
            .list(self.profile_id, limit.unwrap_or(RECENT_SEARCHES_MAX))
    }

    pub fn record_recent_search(
        &self,
        query: &str,
        filters: &[SearchFilter],
    ) -> JabeXResult<RecentSearch> {
        let query = query.trim();
        if query.is_empty() {
            return Err(crate::domain::errors::JabeXError::Validation(
                "search query is required".into(),
            ));
        }
        let recent = self
            .recent
            .push(self.profile_id, query, filters, RECENT_SEARCHES_MAX)?;
        // Feed local popularity aggregation (never uploaded).
        let _ = self
            .history
            .record(self.profile_id, query, None, None, SEARCH_HISTORY_MAX);
        Ok(recent)
    }

    pub fn clear_recent_searches(&self) -> JabeXResult<()> {
        self.recent.clear(self.profile_id)
    }

    pub fn delete_recent_search(&self, id: Uuid) -> JabeXResult<()> {
        self.recent.remove(self.profile_id, id)
    }

    pub fn record_search_selection(
        &self,
        query: &str,
        entity_type: Option<SearchEntityType>,
        entity_id: Option<&str>,
    ) -> JabeXResult<SearchHistoryEntry> {
        self.history.record(
            self.profile_id,
            query.trim(),
            entity_type,
            entity_id,
            SEARCH_HISTORY_MAX,
        )
    }

    pub fn list_search_history(
        &self,
        limit: Option<usize>,
    ) -> JabeXResult<Vec<SearchHistoryEntry>> {
        self.history.list(self.profile_id, limit.unwrap_or(50))
    }

    pub fn list_popular_searches(&self, limit: Option<usize>) -> JabeXResult<Vec<PopularSearch>> {
        self.history
            .list_popular(self.profile_id, limit.unwrap_or(POPULAR_SEARCHES_MAX))
    }

    pub fn clear_search_history(&self) -> JabeXResult<()> {
        self.history.clear(self.profile_id)
    }

    fn index_static_commands_and_settings(&self) -> JabeXResult<()> {
        let commands = [
            ("cmd:home", "Home", "Go to home dashboard", "/"),
            ("cmd:decks", "Decks", "Browse and manage decks", "/decks"),
            ("cmd:review", "Start review", "Review due cards", "/review"),
            ("cmd:media", "Media library", "Browse attachments", "/media"),
            (
                "cmd:trash",
                "Trash",
                "Restore or permanently delete items",
                "/trash",
            ),
            (
                "cmd:settings",
                "Settings",
                "Application and study settings",
                "/settings",
            ),
        ];
        for (id, title, content, href) in commands {
            let text = format!("{title} {content}");
            self.index.upsert(&SearchIndexDocument {
                id: create_id(),
                profile_id: self.profile_id,
                entity_type: SearchEntityType::Command,
                entity_id: id.into(),
                title: title.into(),
                content: text.clone(),
                normalized_content: normalize_search_text(&text),
                metadata: json!({ "href": href }),
                language: Some("en".into()),
                created_at: now_utc(),
                updated_at: now_utc(),
            })?;
        }

        let settings = [
            (
                "setting:language-assist",
                "Language assist",
                "Translation and pronunciation settings",
                "/settings",
            ),
            (
                "setting:leitner",
                "Leitner algorithm",
                "Interval presets and custom schedules",
                "/settings",
            ),
            (
                "setting:tray",
                "Tray and startup",
                "System tray preferences",
                "/settings",
            ),
            (
                "setting:backup",
                "Backup and restore",
                "Export or import .jbx backups",
                "/settings#backup",
            ),
        ];
        for (id, title, content, href) in settings {
            let text = format!("{title} {content}");
            self.index.upsert(&SearchIndexDocument {
                id: create_id(),
                profile_id: self.profile_id,
                entity_type: SearchEntityType::Setting,
                entity_id: id.into(),
                title: title.into(),
                content: text.clone(),
                normalized_content: normalize_search_text(&text),
                metadata: json!({ "href": href }),
                language: Some("en".into()),
                created_at: now_utc(),
                updated_at: now_utc(),
            })?;
        }
        Ok(())
    }

    fn to_result(
        &self,
        query: &str,
        normalized_query: &str,
        doc: SearchIndexDocument,
        filter: SearchFilter,
    ) -> Option<SearchResult> {
        if filter == SearchFilter::Reviews {
            let due = doc
                .metadata
                .get("due")
                .and_then(|v| v.as_bool())
                .unwrap_or(doc.entity_type == SearchEntityType::Review);
            if doc.entity_type == SearchEntityType::Card && !due {
                return None;
            }
        }

        let front = doc.metadata.get("front").and_then(|v| v.as_str());
        let ranked = rank_match(query, &doc.title, &doc.content, doc.entity_type, front);
        if ranked.score < 40.0
            && !doc.normalized_content.contains(normalized_query)
            && best_token_similarity(normalized_query, &doc.normalized_content) < 0.55
        {
            // Still allow latex normalized hits (mc2 â†” mc^2)
            let latex_hit = doc
                .metadata
                .get("latex")
                .map(|v| v.to_string())
                .map(|s| normalize_latex(&s).contains(&normalize_latex(query)))
                .unwrap_or(false);
            if !latex_hit {
                return None;
            }
        }

        let breadcrumb = match doc.entity_type {
            SearchEntityType::Card | SearchEntityType::Review => doc
                .metadata
                .get("deckName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            SearchEntityType::Deck => doc
                .metadata
                .get("cardCount")
                .and_then(|v| v.as_u64())
                .map(|n| format!("{n} cards")),
            SearchEntityType::Tag => Some("Tag".into()),
            SearchEntityType::Attachment => doc
                .metadata
                .get("mimeType")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            SearchEntityType::Command | SearchEntityType::Setting => None,
            SearchEntityType::Note => None,
        };

        let snippet_source = front
            .filter(|s| !s.is_empty())
            .unwrap_or(doc.content.as_str());
        let snippet = make_snippet(snippet_source, query, 140);
        let highlights = highlight_ranges(&snippet, query);
        let href = resolve_href(&doc);
        let text_direction = detect_text_direction(&doc.title).to_string();

        Some(SearchResult {
            entity_type: doc.entity_type,
            entity_id: doc.entity_id,
            title: doc.title,
            snippet,
            breadcrumb,
            highlights,
            score: ranked.score,
            href,
            metadata: doc.metadata,
            text_direction,
        })
    }
}

fn resolve_href(doc: &SearchIndexDocument) -> Option<String> {
    if let Some(href) = doc.metadata.get("href").and_then(|v| v.as_str()) {
        return Some(href.to_string());
    }
    match doc.entity_type {
        SearchEntityType::Deck => Some(format!("/decks/view?id={}", doc.entity_id)),
        SearchEntityType::Card => {
            let deck_id = doc.metadata.get("deckId").and_then(|v| v.as_str())?;
            Some(format!("/decks/view?id={deck_id}&cardId={}", doc.entity_id))
        }
        SearchEntityType::Review => {
            let deck_id = doc.metadata.get("deckId").and_then(|v| v.as_str());
            match deck_id {
                Some(id) => Some(format!("/review?deckId={id}")),
                None => Some("/review".into()),
            }
        }
        SearchEntityType::Attachment => {
            let owner = doc.metadata.get("ownerId").and_then(|v| v.as_str())?;
            let owner_type = doc
                .metadata
                .get("ownerType")
                .and_then(|v| v.as_str())
                .unwrap_or("card");
            if owner_type == "card" {
                Some(format!("/media?focus={owner}"))
            } else {
                Some("/media".into())
            }
        }
        SearchEntityType::Tag => Some("/decks".into()),
        SearchEntityType::Command | SearchEntityType::Setting => None,
        SearchEntityType::Note => None,
    }
}

fn build_fts_query(normalized: &str) -> String {
    let tokens: Vec<&str> = normalized
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.is_empty() {
        return String::new();
    }
    tokens
        .into_iter()
        .map(|t| {
            let escaped = escape_fts_token(t);
            // Prefix match for progressive search
            format!("\"{escaped}\"*")
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn escape_fts_token(token: &str) -> String {
    token.replace('"', "\"\"")
}

fn merge_docs(into: &mut Vec<SearchIndexDocument>, extra: Vec<SearchIndexDocument>) {
    let mut seen: HashSet<(SearchEntityType, String)> = into
        .iter()
        .map(|d| (d.entity_type, d.entity_id.clone()))
        .collect();
    for doc in extra {
        let key = (doc.entity_type, doc.entity_id.clone());
        if seen.insert(key) {
            into.push(doc);
        }
    }
}

fn extract_hashtags(text: &str) -> Vec<String> {
    let mut tags = Vec::new();
    for raw in text.split_whitespace() {
        let t =
            raw.trim_matches(|c: char| matches!(c, ',' | '.' | ';' | ':' | '!' | '?' | ')' | '('));
        if let Some(tag) = t.strip_prefix('#') {
            if !tag.is_empty()
                && tag
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c > '\u{007F}')
            {
                tags.push(tag.to_string());
            }
        }
    }
    tags.sort();
    tags.dedup();
    tags
}

fn truncate(text: &str, max_chars: usize) -> String {
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
    fn fts_query_uses_prefix() {
        let q = build_fts_query("hello world");
        assert!(q.contains("hello\"*") || q.contains("\"hello\"*"));
        assert!(q.contains("AND"));
    }

    #[test]
    fn extracts_hashtags() {
        let tags = extract_hashtags("Learn #biology and #Cell_structure today");
        assert!(tags.iter().any(|t| t == "biology"));
        assert!(tags.iter().any(|t| t == "Cell_structure"));
    }

    #[test]
    fn latex_normalize_helper() {
        assert!(normalize_latex("E = mc^2").contains("mc2"));
    }
}
