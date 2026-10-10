use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};

/// Treat zero affected rows as a typed NotFound error.
pub fn require_changed(changed: usize, entity: &'static str, id: Uuid) -> JabeXResult<()> {
    if changed == 0 {
        Err(JabeXError::NotFound { entity, id })
    } else {
        Ok(())
    }
}
