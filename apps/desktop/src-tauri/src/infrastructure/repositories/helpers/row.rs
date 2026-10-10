use chrono::{DateTime, Utc};
use rusqlite::Row;
use uuid::Uuid;

use super::convert::{decode_optional_json, parse_optional_uuid, parse_time, parse_uuid};

pub fn get_uuid(row: &Row<'_>, column: &str) -> rusqlite::Result<Uuid> {
    let raw: String = row.get(column)?;
    parse_uuid(&raw)
}

pub fn get_optional_uuid(row: &Row<'_>, column: &str) -> rusqlite::Result<Option<Uuid>> {
    parse_optional_uuid(row.get(column)?)
}

pub fn get_rfc3339(row: &Row<'_>, column: &str) -> rusqlite::Result<DateTime<Utc>> {
    let raw: String = row.get(column)?;
    parse_time(&raw)
}

pub fn get_optional_rfc3339(
    row: &Row<'_>,
    column: &str,
) -> rusqlite::Result<Option<DateTime<Utc>>> {
    let raw: Option<String> = row.get(column)?;
    raw.map(|s| parse_time(&s)).transpose()
}

pub fn get_u32(row: &Row<'_>, column: &str) -> rusqlite::Result<u32> {
    Ok(row.get::<_, i64>(column)? as u32)
}

pub fn get_u64_from_i64(row: &Row<'_>, column: &str) -> rusqlite::Result<u64> {
    Ok(row.get::<_, i64>(column)? as u64)
}

pub fn get_optional_json<T: serde::de::DeserializeOwned>(
    row: &Row<'_>,
    column: &str,
) -> rusqlite::Result<Option<T>> {
    decode_optional_json(row.get(column)?)
}
