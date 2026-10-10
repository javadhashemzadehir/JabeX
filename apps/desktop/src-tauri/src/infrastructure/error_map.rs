use crate::domain::errors::JabeXError;

impl From<rusqlite::Error> for JabeXError {
    fn from(value: rusqlite::Error) -> Self {
        JabeXError::Database(value.to_string())
    }
}
