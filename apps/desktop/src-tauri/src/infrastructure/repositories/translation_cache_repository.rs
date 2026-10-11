use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::speech::TranslationCacheEntry;
use crate::infrastructure::database::Database;
use crate::infrastructure::repositories::helpers::{get_rfc3339, get_uuid};
use crate::infrastructure::repositories::queries::speech::{
    TRANSLATION_CACHE_FIND, TRANSLATION_CACHE_UPSERT,
};

#[derive(Clone)]
pub struct SqliteTranslationCacheRepository {
    db: Database,
}

impl SqliteTranslationCacheRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn find(
        &self,
        source_text: &str,
        source_language: &str,
        target_language: &str,
    ) -> JabeXResult<Option<TranslationCacheEntry>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare_cached(TRANSLATION_CACHE_FIND)?;
            let mut rows = stmt.query(rusqlite::params![
                source_language,
                target_language,
                source_text
            ])?;
            match rows.next()? {
                Some(row) => Ok(Some(map_entry(row)?)),
                None => Ok(None),
            }
        })
    }

    pub fn upsert(&self, entry: &TranslationCacheEntry) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                TRANSLATION_CACHE_UPSERT,
                rusqlite::params![
                    entry.id.to_string(),
                    entry.source_text,
                    entry.source_language,
                    entry.target_language,
                    entry.translated_text,
                    entry.created_at.to_rfc3339(),
                ],
            )
            .map_err(JabeXError::from)?;
            Ok(())
        })
    }
}

fn map_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<TranslationCacheEntry> {
    Ok(TranslationCacheEntry {
        id: get_uuid(row, "id")?,
        source_text: row.get("source_text")?,
        source_language: row.get("source_language")?,
        target_language: row.get("target_language")?,
        translated_text: row.get("translated_text")?,
        created_at: get_rfc3339(row, "created_at")?,
    })
}
