use chrono::{DateTime, Utc};
use uuid::Uuid;

pub fn parse_uuid(value: &str) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(value).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

pub fn parse_optional_uuid(value: Option<String>) -> rusqlite::Result<Option<Uuid>> {
    match value {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => parse_uuid(&s).map(Some),
    }
}

pub fn parse_time(value: &str) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
}

pub fn encode_enum<T: serde::Serialize>(value: &T) -> rusqlite::Result<String> {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "failed to encode enum",
            )))
        })
}

pub fn decode_enum<T: serde::de::DeserializeOwned>(value: &str) -> rusqlite::Result<T> {
    // Values are stored as bare snake_case strings, so wrap as JSON string.
    let quoted = format!("\"{value}\"");
    serde_json::from_str(&quoted).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

pub fn encode_json<T: serde::Serialize>(
    value: &T,
) -> Result<String, crate::domain::errors::JabeXError> {
    serde_json::to_string(value)
        .map_err(|e| crate::domain::errors::JabeXError::Internal(e.to_string()))
}

pub fn encode_optional_json<T: serde::Serialize>(
    value: &Option<T>,
) -> Result<Option<String>, crate::domain::errors::JabeXError> {
    match value {
        None => Ok(None),
        Some(v) => encode_json(v).map(Some),
    }
}

pub fn decode_optional_json<T: serde::de::DeserializeOwned>(
    raw: Option<String>,
) -> rusqlite::Result<Option<T>> {
    match raw {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => serde_json::from_str(&s).map(Some).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        }),
    }
}
