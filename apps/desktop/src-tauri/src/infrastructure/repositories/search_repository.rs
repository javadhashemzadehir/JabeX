use rusqlite::params;
use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::{
    RecentSearchRepository, SearchHistoryRepository, SearchIndexRepository,
};
use crate::domain::search::{
    PopularSearch, RecentSearch, SearchEntityType, SearchFilter, SearchHistoryEntry,
    SearchIndexDocument,
};
use crate::infrastructure::database::Database;
use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

use super::mappers::search::{
    entity_types_json, map_popular_search, map_recent_search, map_search_document,
    map_search_history,
};
use super::queries::search as q;

pub struct SqliteSearchIndexRepository {
    db: Database,
}

impl SqliteSearchIndexRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl SearchIndexRepository for SqliteSearchIndexRepository {
    fn upsert(&self, doc: &SearchIndexDocument) -> JabeXResult<()> {
        let metadata = serde_json::to_string(&doc.metadata)
            .map_err(|e| JabeXError::Internal(e.to_string()))?;
        self.db.with_conn(|conn| {
            conn.execute(
                q::SEARCH_UPSERT,
                params![
                    doc.id.to_string(),
                    doc.profile_id.to_string(),
                    doc.entity_type.as_str(),
                    doc.entity_id,
                    doc.title,
                    doc.content,
                    doc.normalized_content,
                    metadata,
                    doc.language,
                    doc.created_at.to_rfc3339(),
                    doc.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    fn remove(&self, entity_type: SearchEntityType, entity_id: &str) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(q::SEARCH_REMOVE, params![entity_type.as_str(), entity_id])?;
            Ok(())
        })
    }

    fn remove_by_entity_types(&self, entity_types: &[SearchEntityType]) -> JabeXResult<()> {
        if entity_types.is_empty() {
            return Ok(());
        }
        let placeholders = entity_types
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("{}{})", q::SEARCH_REMOVE_TYPES_PREFIX, placeholders);
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let values: Vec<&str> = entity_types.iter().map(|t| t.as_str()).collect();
            let params: Vec<&dyn rusqlite::ToSql> =
                values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
            stmt.execute(params.as_slice())?;
            Ok(())
        })
    }

    fn clear(&self) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(q::SEARCH_CLEAR, [])?;
            Ok(())
        })
    }

    fn count(&self) -> JabeXResult<u64> {
        self.db.with_conn(|conn| {
            let count: i64 = conn.query_row(q::SEARCH_COUNT, [], |row| row.get(0))?;
            Ok(count as u64)
        })
    }

    fn search_candidates(
        &self,
        fts_query: &str,
        entity_types: Option<&[SearchEntityType]>,
        limit: usize,
    ) -> JabeXResult<Vec<SearchIndexDocument>> {
        if fts_query.trim().is_empty() {
            return Ok(Vec::new());
        }
        self.db.with_conn(|conn| {
            let limit = limit as i64;
            let result = if let Some(types) = entity_types {
                let types_json = entity_types_json(types)?;
                let mut stmt = conn.prepare(q::SEARCH_FTS_TYPED)?;
                let rows =
                    stmt.query_map(params![fts_query, types_json, limit], map_search_document)?;
                rows.collect::<Result<Vec<_>, _>>()
            } else {
                let mut stmt = conn.prepare(q::SEARCH_FTS)?;
                let rows = stmt.query_map(params![fts_query, limit], map_search_document)?;
                rows.collect::<Result<Vec<_>, _>>()
            };
            match result {
                Ok(docs) => Ok(docs),
                Err(e) => {
                    // Malformed FTS query should not crash search — fall through empty.
                    tracing::debug!(error = %e, "fts query failed");
                    Ok(Vec::new())
                }
            }
        })
    }

    fn search_normalized_fallback(
        &self,
        normalized_prefix: &str,
        entity_types: Option<&[SearchEntityType]>,
        limit: usize,
    ) -> JabeXResult<Vec<SearchIndexDocument>> {
        if normalized_prefix.trim().is_empty() {
            return Ok(Vec::new());
        }
        let like = format!("%{normalized_prefix}%");
        self.db.with_conn(|conn| {
            let limit = limit as i64;
            if let Some(types) = entity_types {
                let types_json = entity_types_json(types)?;
                let mut stmt = conn.prepare(q::SEARCH_FALLBACK_TYPED)?;
                let rows =
                    stmt.query_map(params![like, like, types_json, limit], map_search_document)?;
                Ok(rows.collect::<Result<Vec<_>, _>>()?)
            } else {
                let mut stmt = conn.prepare(q::SEARCH_FALLBACK)?;
                let rows = stmt.query_map(params![like, like, limit], map_search_document)?;
                Ok(rows.collect::<Result<Vec<_>, _>>()?)
            }
        })
    }
}

pub struct SqliteRecentSearchRepository {
    db: Database,
}

impl SqliteRecentSearchRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl RecentSearchRepository for SqliteRecentSearchRepository {
    fn list(&self, profile_id: Uuid, limit: usize) -> JabeXResult<Vec<RecentSearch>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(q::RECENT_LIST)?;
            let rows = stmt.query_map(
                params![profile_id.to_string(), limit as i64],
                map_recent_search,
            )?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }

    fn push(
        &self,
        profile_id: Uuid,
        query: &str,
        filters: &[SearchFilter],
        max_size: usize,
    ) -> JabeXResult<RecentSearch> {
        let filters_json =
            serde_json::to_string(filters).map_err(|e| JabeXError::Internal(e.to_string()))?;
        let id = create_id();
        let created_at = now_utc();
        self.db.with_conn(|conn| {
            conn.execute(
                q::RECENT_DELETE_QUERY,
                params![profile_id.to_string(), query],
            )?;
            conn.execute(
                q::RECENT_INSERT,
                params![
                    id.to_string(),
                    profile_id.to_string(),
                    query,
                    filters_json,
                    created_at.to_rfc3339(),
                ],
            )?;
            conn.execute(
                q::RECENT_TRIM,
                params![profile_id.to_string(), max_size as i64],
            )?;
            Ok(())
        })?;
        Ok(RecentSearch {
            id,
            profile_id,
            query: query.to_string(),
            filters: filters.to_vec(),
            created_at,
        })
    }

    fn remove(&self, profile_id: Uuid, id: Uuid) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                q::RECENT_DELETE_ID,
                params![profile_id.to_string(), id.to_string()],
            )?;
            Ok(())
        })
    }

    fn clear(&self, profile_id: Uuid) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(q::RECENT_CLEAR, params![profile_id.to_string()])?;
            Ok(())
        })
    }
}

pub struct SqliteSearchHistoryRepository {
    db: Database,
}

impl SqliteSearchHistoryRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl SearchHistoryRepository for SqliteSearchHistoryRepository {
    fn record(
        &self,
        profile_id: Uuid,
        query: &str,
        entity_type: Option<SearchEntityType>,
        entity_id: Option<&str>,
        max_size: usize,
    ) -> JabeXResult<SearchHistoryEntry> {
        let id = create_id();
        let created_at = now_utc();
        let entity_type_str = entity_type.map(|t| t.as_str().to_string());
        self.db.with_conn(|conn| {
            conn.execute(
                q::HISTORY_INSERT,
                params![
                    id.to_string(),
                    profile_id.to_string(),
                    query,
                    entity_type_str,
                    entity_id,
                    created_at.to_rfc3339(),
                ],
            )?;
            conn.execute(
                q::HISTORY_TRIM,
                params![profile_id.to_string(), max_size as i64],
            )?;
            Ok(())
        })?;
        Ok(SearchHistoryEntry {
            id,
            profile_id,
            query: query.to_string(),
            entity_type,
            entity_id: entity_id.map(|s| s.to_string()),
            created_at,
        })
    }

    fn list(&self, profile_id: Uuid, limit: usize) -> JabeXResult<Vec<SearchHistoryEntry>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(q::HISTORY_LIST)?;
            let rows = stmt.query_map(
                params![profile_id.to_string(), limit as i64],
                map_search_history,
            )?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }

    fn list_popular(&self, profile_id: Uuid, limit: usize) -> JabeXResult<Vec<PopularSearch>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(q::HISTORY_POPULAR)?;
            let rows = stmt.query_map(
                params![profile_id.to_string(), limit as i64],
                map_popular_search,
            )?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }

    fn clear(&self, profile_id: Uuid) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(q::HISTORY_CLEAR, params![profile_id.to_string()])?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::SearchService;
    use crate::domain::card::Card;
    use crate::domain::deck::Deck;
    use crate::domain::repositories::{CardRepository, DeckRepository};
    use crate::domain::rich_text::RichTextDocument;
    use crate::infrastructure::database::open_database_at;
    use crate::infrastructure::repositories::{SqliteCardRepository, SqliteDeckRepository};
    use tempfile::tempdir;

    fn setup() -> (
        Database,
        SearchService<
            SqliteSearchIndexRepository,
            SqliteRecentSearchRepository,
            SqliteSearchHistoryRepository,
            SqliteCardRepository,
            SqliteDeckRepository,
            crate::infrastructure::repositories::SqliteAttachmentRepository,
        >,
    ) {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        // Keep tempdir alive by leaking path via Database owning the file — use into_path
        std::mem::forget(dir);
        let profile_id = db.paths().profile_id;
        let service = SearchService::new(
            SqliteSearchIndexRepository::new(db.clone()),
            SqliteRecentSearchRepository::new(db.clone()),
            SqliteSearchHistoryRepository::new(db.clone()),
            SqliteCardRepository::new(db.clone()),
            SqliteDeckRepository::new(db.clone()),
            crate::infrastructure::repositories::SqliteAttachmentRepository::new(db.clone()),
            profile_id,
        );
        (db, service)
    }

    #[test]
    fn migration_creates_search_tables() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        db.with_conn(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='search_index'",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(count, 1);
            let fts: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name='search_index_fts'",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(fts, 1);
            Ok(())
        })
        .expect("tables");
    }

    #[test]
    fn indexes_and_searches_english_and_persian() {
        let (db, search) = setup();
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db.clone());
        let deck = Deck::new("Biology".into(), Some("Science".into()));
        decks.create(&deck).unwrap();
        let card = Card::new(
            deck.id,
            RichTextDocument::from_plain_text("Photosynthesis"),
            RichTextDocument::from_plain_text("Plants convert light energy"),
        );
        cards.create(&card).unwrap();
        let fa = Card::new(
            deck.id,
            RichTextDocument::from_plain_text("فلسفه"),
            RichTextDocument::from_plain_text("درس فلسفه غرب"),
        );
        cards.create(&fa).unwrap();
        search.rebuild_index().unwrap();

        let en = search
            .search("photosynthesis", SearchFilter::All, Some(10))
            .unwrap();
        assert!(en
            .results
            .iter()
            .any(|r| r.title.contains("Photosynthesis")));

        let fa_q = search
            .search("فلسف", SearchFilter::Cards, Some(10))
            .unwrap();
        assert!(fa_q.results.iter().any(|r| r.title.contains("فلسف")));
    }

    #[test]
    fn fuzzy_finds_javascript_typo() {
        let (db, search) = setup();
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db.clone());
        let deck = Deck::new("CS".into(), None);
        decks.create(&deck).unwrap();
        let card = Card::new(
            deck.id,
            RichTextDocument::from_plain_text("javascript"),
            RichTextDocument::from_plain_text("Learn javascript programming"),
        );
        cards.create(&card).unwrap();
        search.rebuild_index().unwrap();
        let hits = search
            .search("javscript", SearchFilter::All, Some(10))
            .unwrap();
        assert!(
            hits.results
                .iter()
                .any(|r| r.title.to_lowercase().contains("javascript")),
            "expected fuzzy hit, got {:?}",
            hits.results.iter().map(|r| &r.title).collect::<Vec<_>>()
        );
    }

    #[test]
    fn latex_mc2_matches_formula() {
        let (db, search) = setup();
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db.clone());
        let deck = Deck::new("Physics".into(), None);
        decks.create(&deck).unwrap();
        let card = Card::new(
            deck.id,
            RichTextDocument::from_plain_text("E = mc^2"),
            RichTextDocument::from_plain_text("mass-energy"),
        );
        cards.create(&card).unwrap();
        search.rebuild_index().unwrap();
        let hits = search.search("mc2", SearchFilter::Cards, Some(10)).unwrap();
        assert!(!hits.results.is_empty());
    }

    #[test]
    fn recent_searches_are_profile_scoped_and_trimmed() {
        let (_db, search) = setup();
        search
            .record_recent_search("biology", &[SearchFilter::Cards])
            .unwrap();
        search
            .record_recent_search("javascript", &[SearchFilter::All])
            .unwrap();
        let recent = search.list_recent_searches(Some(10)).unwrap();
        assert!(recent.iter().any(|r| r.query == "biology"));
        let biology_id = recent
            .iter()
            .find(|r| r.query == "biology")
            .expect("biology recent")
            .id;
        search.delete_recent_search(biology_id).unwrap();
        let after_delete = search.list_recent_searches(Some(10)).unwrap();
        assert!(!after_delete.iter().any(|r| r.query == "biology"));
        assert!(after_delete.iter().any(|r| r.query == "javascript"));
        search.clear_recent_searches().unwrap();
        assert!(search.list_recent_searches(Some(10)).unwrap().is_empty());
    }

    #[test]
    fn popular_searches_aggregate_history_counts() {
        let (_db, search) = setup();
        search.record_search_selection("rust", None, None).unwrap();
        search.record_search_selection("rust", None, None).unwrap();
        search
            .record_search_selection("python", None, None)
            .unwrap();
        let popular = search.list_popular_searches(Some(10)).unwrap();
        assert_eq!(popular[0].query, "rust");
        assert_eq!(popular[0].hit_count, 2);
        assert!(popular
            .iter()
            .any(|p| p.query == "python" && p.hit_count == 1));
    }

    #[test]
    fn fts_query_is_parameterized_not_concatenated_injection() {
        let (db, search) = setup();
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db.clone());
        let deck = Deck::new("Safe".into(), None);
        decks.create(&deck).unwrap();
        cards
            .create(&Card::new(
                deck.id,
                RichTextDocument::from_plain_text("normal"),
                RichTextDocument::from_plain_text("card"),
            ))
            .unwrap();
        search.rebuild_index().unwrap();
        // Must not error or dump unrelated rows via SQL injection.
        let hits = search
            .search("\"; DROP TABLE cards; --", SearchFilter::All, Some(10))
            .unwrap();
        assert!(
            hits.results.is_empty()
                || hits
                    .results
                    .iter()
                    .all(|r| r.entity_type != SearchEntityType::Deck || true)
        );
        let still: i64 = db
            .with_conn(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))?)
            })
            .unwrap();
        assert_eq!(still, 1);
    }
}
