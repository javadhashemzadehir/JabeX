use uuid::Uuid;

/// Create a new UUIDv7 identifier (time-ordered).
pub fn create_id() -> Uuid {
    Uuid::now_v7()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_id_returns_v7() {
        let id = create_id();
        assert_eq!(id.get_version(), Some(uuid::Version::SortRand));
    }
}
