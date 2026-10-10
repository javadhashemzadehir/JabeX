use chrono::{DateTime, Utc};
use rusqlite::Row;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::search::{
    PopularSearch, RecentSearch, SearchEntityType, SearchFilter, SearchHistoryEntry,
    SearchIndexDocument,
};

pub fn map_search_document(row: &Row<'_>) -> rusqlite::Result<SearchIndexDocument> {
    let entity_type_raw: String = row.get("entity_type")?;
    let entity_type = SearchEntityType::parse(&entity_type_raw).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown search entity type: {entity_type_raw}"),
            )),
        )
    })?;
    let metadata_json: String = row.get("metadata_json")?;
    let metadata: Value = serde_json::from_str(&metadata_json).unwrap_or_else(|_| Value::Object(Default::default()));
    let created_at = parse_time(row.get::<_, String>("created_at")?)?;
    let updated_at = parse_time(row.get::<_, String>("updated_at")?)?;
    Ok(SearchIndexDocument {
        id: parse_uuid(row.get("id")?)?,
        profile_id: parse_uuid(row.get("profile_id")?)?,
        entity_type,
        entity_id: row.get("entity_id")?,
        title: row.get("title")?,
        content: row.get("content")?,
        normalized_content: row.get("normalized_content")?,
        metadata,
        language: row.get("language")?,
        created_at,
        updated_at,
    })
}

pub fn map_recent_search(row: &Row<'_>) -> rusqlite::Result<RecentSearch> {
    let filters_json: String = row.get("filters")?;
    let filters: Vec<SearchFilter> = serde_json::from_str(&filters_json).unwrap_or_default();
    Ok(RecentSearch {
        id: parse_uuid(row.get("id")?)?,
        profile_id: parse_uuid(row.get("profile_id")?)?,
        query: row.get("query")?,
        filters,
        created_at: parse_time(row.get::<_, String>("created_at")?)?,
    })
}

pub fn map_search_history(row: &Row<'_>) -> rusqlite::Result<SearchHistoryEntry> {
    let entity_type_raw: Option<String> = row.get("entity_type")?;
    let entity_type = entity_type_raw
        .as_deref()
        .and_then(SearchEntityType::parse);
    Ok(SearchHistoryEntry {
        id: parse_uuid(row.get("id")?)?,
        profile_id: parse_uuid(row.get("profile_id")?)?,
        query: row.get("query")?,
        entity_type,
        entity_id: row.get("entity_id")?,
        created_at: parse_time(row.get::<_, String>("created_at")?)?,
    })
}

pub fn map_popular_search(row: &Row<'_>) -> rusqlite::Result<PopularSearch> {
    Ok(PopularSearch {
        query: row.get("query")?,
        hit_count: row.get::<_, i64>("hit_count")? as u64,
        last_searched_at: parse_time(row.get::<_, String>("last_searched_at")?)?,
    })
}

fn parse_uuid(value: String) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(&value).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        )
    })
}

fn parse_time(value: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        })
}

pub fn entity_types_json(types: &[SearchEntityType]) -> JabeXResult<String> {
    let values: Vec<&str> = types.iter().map(|t| t.as_str()).collect();
    serde_json::to_string(&values).map_err(|e| JabeXError::Internal(e.to_string()))
}
