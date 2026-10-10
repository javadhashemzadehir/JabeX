use thiserror::Error;
use uuid::Uuid;

/// Application-wide error type for JabeX backend.
#[derive(Debug, Error)]
pub enum JabeXError {
    #[error("not found: {entity} ({id})")]
    NotFound { entity: &'static str, id: Uuid },

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("filesystem error: {0}")]
    Filesystem(String),

    /// Optional network providers (translation / TTS). Never blocks card CRUD.
    #[error("network error: {0}")]
    Network(String),

    #[error("backup error: {0}")]
    Backup(String),

    #[error("migration error: {0}")]
    Migration(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type JabeXResult<T> = Result<T, JabeXError>;

impl From<std::io::Error> for JabeXError {
    fn from(value: std::io::Error) -> Self {
        JabeXError::Filesystem(value.to_string())
    }
}

impl serde::Serialize for JabeXError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
