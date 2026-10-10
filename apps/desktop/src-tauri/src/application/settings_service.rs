use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use serde_json::Value;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::infrastructure::database::Database;
use crate::shared::time::now_utc;

#[derive(Debug, Clone)]
pub struct SettingEntry {
    pub key: String,
    pub value: Value,
    pub revision: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SettingsService {
    db: Database,
}

impl SettingsService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn get(&self, key: &str) -> JabeXResult<Option<SettingEntry>> {
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT key, value, revision, created_at, updated_at
                 FROM settings WHERE key = ?1 AND deleted_at IS NULL",
                params![key],
                |row| {
                    let value_raw: String = row.get(1)?;
                    let value: Value = serde_json::from_str(&value_raw).unwrap_or(Value::Null);
                    let created_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                3,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?;
                    let updated_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                4,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?;
                    Ok(SettingEntry {
                        key: row.get(0)?,
                        value,
                        revision: row.get::<_, i64>(2)? as u32,
                        created_at,
                        updated_at,
                    })
                },
            )
            .optional()
            .map_err(JabeXError::from)
        })
    }

    pub fn set(&self, key: String, value: Value) -> JabeXResult<SettingEntry> {
        if key.trim().is_empty() {
            return Err(JabeXError::Validation("settings key is required".into()));
        }
        let now = now_utc();
        let encoded =
            serde_json::to_string(&value).map_err(|e| JabeXError::Internal(e.to_string()))?;

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO settings (key, value, revision, created_at, updated_at, deleted_at)
                 VALUES (?1, ?2, 1, ?3, ?3, NULL)
                 ON CONFLICT(key) DO UPDATE SET
                    value = excluded.value,
                    revision = settings.revision + 1,
                    updated_at = excluded.updated_at,
                    deleted_at = NULL",
                params![key, encoded, now.to_rfc3339()],
            )?;
            Ok(())
        })?;

        let revision = self.get(&key)?.map(|e| e.revision).unwrap_or(1);

        Ok(SettingEntry {
            key,
            value,
            revision,
            created_at: now,
            updated_at: now,
        })
    }
}
