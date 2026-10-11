use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::speech::{LanguagePreference, TtsProviderKind};
use crate::infrastructure::database::Database;
use crate::infrastructure::repositories::helpers::{
    decode_enum, encode_enum, get_rfc3339, get_uuid,
};
use crate::infrastructure::repositories::queries::speech::{
    LANGUAGE_PREF_FIND, LANGUAGE_PREF_LIST, LANGUAGE_PREF_UPSERT,
};

#[derive(Clone)]
pub struct SqliteLanguagePreferencesRepository {
    db: Database,
}

impl SqliteLanguagePreferencesRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn find(&self, language: &str) -> JabeXResult<Option<LanguagePreference>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare_cached(LANGUAGE_PREF_FIND)?;
            let mut rows = stmt.query(rusqlite::params![language])?;
            match rows.next()? {
                Some(row) => Ok(Some(map_pref(row)?)),
                None => Ok(None),
            }
        })
    }

    pub fn list(&self) -> JabeXResult<Vec<LanguagePreference>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare_cached(LANGUAGE_PREF_LIST)?;
            let rows = stmt.query_map([], |row| map_pref(row))?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn upsert(&self, pref: &LanguagePreference) -> JabeXResult<()> {
        let provider = encode_enum(&pref.provider).map_err(JabeXError::from)?;
        self.db.with_conn(|conn| {
            conn.execute(
                LANGUAGE_PREF_UPSERT,
                rusqlite::params![
                    pref.id.to_string(),
                    pref.language,
                    pref.preferred_voice,
                    pref.speed as f64,
                    provider,
                    pref.created_at.to_rfc3339(),
                    pref.updated_at.to_rfc3339(),
                ],
            )
            .map_err(JabeXError::from)?;
            Ok(())
        })
    }
}

fn map_pref(row: &rusqlite::Row<'_>) -> rusqlite::Result<LanguagePreference> {
    let provider_raw: String = row.get("provider")?;
    let provider: TtsProviderKind =
        decode_enum(&provider_raw).unwrap_or(TtsProviderKind::Auto);
    Ok(LanguagePreference {
        id: get_uuid(row, "id")?,
        language: row.get("language")?,
        preferred_voice: row.get("preferred_voice")?,
        speed: row.get::<_, f64>("speed")? as f32,
        provider,
        created_at: get_rfc3339(row, "created_at")?,
        updated_at: get_rfc3339(row, "updated_at")?,
    })
}
