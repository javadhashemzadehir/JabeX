use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::speech::CardAudio;
use crate::infrastructure::database::Database;
use crate::infrastructure::repositories::helpers::{get_rfc3339, get_uuid};
use crate::infrastructure::repositories::queries::speech::{
    CARD_AUDIO_FIND_BY_CARD, CARD_AUDIO_FIND_BY_HASH, CARD_AUDIO_UPSERT,
};

#[derive(Clone)]
pub struct SqliteCardAudioRepository {
    db: Database,
}

impl SqliteCardAudioRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn find_by_hash(&self, hash: &str) -> JabeXResult<Option<CardAudio>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare_cached(CARD_AUDIO_FIND_BY_HASH)?;
            let mut rows = stmt.query(rusqlite::params![hash])?;
            match rows.next()? {
                Some(row) => Ok(Some(map_card_audio(row)?)),
                None => Ok(None),
            }
        })
    }

    pub fn list_for_card(&self, card_id: Uuid) -> JabeXResult<Vec<CardAudio>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare_cached(CARD_AUDIO_FIND_BY_CARD)?;
            let rows = stmt.query_map(rusqlite::params![card_id.to_string()], |row| {
                map_card_audio(row)
            })?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn upsert(&self, audio: &CardAudio) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                CARD_AUDIO_UPSERT,
                rusqlite::params![
                    audio.id.to_string(),
                    audio.card_id.to_string(),
                    audio.language,
                    audio.provider,
                    audio.voice,
                    audio.file_path,
                    audio.duration_ms.map(|v| v as i64),
                    audio.hash,
                    audio.created_at.to_rfc3339(),
                    audio.updated_at.to_rfc3339(),
                ],
            )
            .map_err(JabeXError::from)?;
            Ok(())
        })
    }
}

fn map_card_audio(row: &rusqlite::Row<'_>) -> rusqlite::Result<CardAudio> {
    let duration: Option<i64> = row.get("duration_ms")?;
    Ok(CardAudio {
        id: get_uuid(row, "id")?,
        card_id: get_uuid(row, "card_id")?,
        language: row.get("language")?,
        provider: row.get("provider")?,
        voice: row.get("voice")?,
        file_path: row.get("file_path")?,
        duration_ms: duration.map(|v| v as u32),
        hash: row.get("hash")?,
        created_at: get_rfc3339(row, "created_at")?,
        updated_at: get_rfc3339(row, "updated_at")?,
    })
}
