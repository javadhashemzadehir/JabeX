use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use super::attachment::{Attachment, AttachmentOwner};
use super::card::Card;
use super::deck::Deck;
use super::errors::JabeXResult;
use super::ink::{InkDocumentRef, InkOwner};
use super::review::Review;
use super::search::{
    PopularSearch, RecentSearch, SearchEntityType, SearchFilter, SearchHistoryEntry,
    SearchIndexDocument,
};
use super::study_activity::StudyActivity;

/// Port for deck persistence. Domain never depends on SQLite.
pub trait DeckRepository: Send + Sync {
    fn create(&self, deck: &Deck) -> JabeXResult<()>;
    fn find(&self, id: Uuid) -> JabeXResult<Option<Deck>>;
    /// Include soft-deleted rows when `include_deleted` is true.
    fn find_any(&self, id: Uuid) -> JabeXResult<Option<Deck>>;
    fn list(&self) -> JabeXResult<Vec<Deck>>;
    fn list_all(&self, include_deleted: bool) -> JabeXResult<Vec<Deck>>;
    fn update(&self, deck: &Deck) -> JabeXResult<()>;
    fn soft_delete(
        &self,
        id: Uuid,
        deleted_at: DateTime<Utc>,
        deleted_by: Option<Uuid>,
    ) -> JabeXResult<()>;
    fn hard_delete(&self, id: Uuid) -> JabeXResult<()>;
}

/// Port for card persistence.
pub trait CardRepository: Send + Sync {
    fn create(&self, card: &Card) -> JabeXResult<()>;
    fn find(&self, id: Uuid) -> JabeXResult<Option<Card>>;
    fn find_any(&self, id: Uuid) -> JabeXResult<Option<Card>>;
    fn list_by_deck(&self, deck_id: Uuid) -> JabeXResult<Vec<Card>>;
    /// Active cards in a deck with SQL pagination (ordered by `created_at` ASC).
    fn list_by_deck_page(&self, deck_id: Uuid, limit: u32, offset: u32) -> JabeXResult<Vec<Card>>;
    fn count_by_deck(&self, deck_id: Uuid) -> JabeXResult<u32>;
    /// Cards in a deck, optionally including soft-deleted.
    fn list_by_deck_all(&self, deck_id: Uuid, include_deleted: bool) -> JabeXResult<Vec<Card>>;
    fn list_all(&self, include_deleted: bool) -> JabeXResult<Vec<Card>>;
    fn list_due(&self, deck_id: Option<Uuid>, now: DateTime<Utc>) -> JabeXResult<Vec<Card>>;
    /// Due cards capped for review session windows.
    fn list_due_limited(
        &self,
        deck_id: Option<Uuid>,
        now: DateTime<Utc>,
        limit: u32,
    ) -> JabeXResult<Vec<Card>>;
    fn update(&self, card: &Card) -> JabeXResult<()>;
    fn soft_delete(
        &self,
        id: Uuid,
        deleted_at: DateTime<Utc>,
        deleted_by: Option<Uuid>,
    ) -> JabeXResult<()>;
    fn hard_delete(&self, id: Uuid) -> JabeXResult<()>;
}

/// Port for append-only review logs.
pub trait ReviewRepository: Send + Sync {
    fn create(&self, review: &Review) -> JabeXResult<()>;
    fn list_by_card(&self, card_id: Uuid) -> JabeXResult<Vec<Review>>;
    fn list_all(&self) -> JabeXResult<Vec<Review>>;
    fn hard_delete_by_card(&self, card_id: Uuid) -> JabeXResult<()>;
}

/// Port for attachment metadata (binaries via AttachmentStorage).
pub trait AttachmentRepository: Send + Sync {
    fn create(&self, attachment: &Attachment) -> JabeXResult<()>;
    fn find(&self, id: Uuid) -> JabeXResult<Option<Attachment>>;
    fn list_by_owner(
        &self,
        owner_type: AttachmentOwner,
        owner_id: Uuid,
        include_deleted: bool,
    ) -> JabeXResult<Vec<Attachment>>;
    fn list_all(&self, include_deleted: bool) -> JabeXResult<Vec<Attachment>>;
    fn update(&self, attachment: &Attachment) -> JabeXResult<()>;
    fn soft_delete(
        &self,
        id: Uuid,
        deleted_at: DateTime<Utc>,
        deleted_by: Option<Uuid>,
    ) -> JabeXResult<()>;
    /// Permanently remove the metadata row (files removed via storage separately).
    fn hard_delete(&self, id: Uuid) -> JabeXResult<()>;
}

/// Port for daily study-activity aggregates (projection of reviews).
pub trait StudyActivityRepository: Send + Sync {
    /// Insert or increment the row for `(profile_id, activity_date)`.
    ///
    /// `duration_seconds` is accumulated in storage; `minutes_spent` on the
    /// domain entity is whole minutes derived from that total.
    fn upsert_add(
        &self,
        profile_id: Uuid,
        activity_date: NaiveDate,
        cards_delta: u32,
        duration_seconds: u32,
    ) -> JabeXResult<StudyActivity>;

    fn find_by_date(
        &self,
        profile_id: Uuid,
        activity_date: NaiveDate,
    ) -> JabeXResult<Option<StudyActivity>>;

    fn list_by_profile(&self, profile_id: Uuid) -> JabeXResult<Vec<StudyActivity>>;

    fn list_range(
        &self,
        profile_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> JabeXResult<Vec<StudyActivity>>;

    fn count(&self, profile_id: Uuid) -> JabeXResult<u32>;

    /// Replace-all helper for `.jbx` import.
    fn create(&self, activity: &StudyActivity) -> JabeXResult<()>;
}

/// Port for the denormalized search index (FTS5-backed in infrastructure).
pub trait SearchIndexRepository: Send + Sync {
    fn upsert(&self, doc: &SearchIndexDocument) -> JabeXResult<()>;
    fn remove(&self, entity_type: SearchEntityType, entity_id: &str) -> JabeXResult<()>;
    fn remove_by_entity_types(&self, entity_types: &[SearchEntityType]) -> JabeXResult<()>;
    fn clear(&self) -> JabeXResult<()>;
    fn count(&self) -> JabeXResult<u64>;
    /// Raw candidate rows for ranking (FTS MATCH + fuzzy fallback).
    fn search_candidates(
        &self,
        fts_query: &str,
        entity_types: Option<&[SearchEntityType]>,
        limit: usize,
    ) -> JabeXResult<Vec<SearchIndexDocument>>;
    /// Broad fallback when FTS returns few rows (prefix / LIKE on normalized).
    fn search_normalized_fallback(
        &self,
        normalized_prefix: &str,
        entity_types: Option<&[SearchEntityType]>,
        limit: usize,
    ) -> JabeXResult<Vec<SearchIndexDocument>>;
}

/// Port for recent search queries (per profile).
pub trait RecentSearchRepository: Send + Sync {
    fn list(&self, profile_id: Uuid, limit: usize) -> JabeXResult<Vec<RecentSearch>>;
    fn push(
        &self,
        profile_id: Uuid,
        query: &str,
        filters: &[SearchFilter],
        max_size: usize,
    ) -> JabeXResult<RecentSearch>;
    fn remove(&self, profile_id: Uuid, id: Uuid) -> JabeXResult<()>;
    fn clear(&self, profile_id: Uuid) -> JabeXResult<()>;
}

/// Port for local search selection history (never leaves the device).
pub trait SearchHistoryRepository: Send + Sync {
    fn record(
        &self,
        profile_id: Uuid,
        query: &str,
        entity_type: Option<SearchEntityType>,
        entity_id: Option<&str>,
        max_size: usize,
    ) -> JabeXResult<SearchHistoryEntry>;
    fn list(&self, profile_id: Uuid, limit: usize) -> JabeXResult<Vec<SearchHistoryEntry>>;
    fn list_popular(&self, profile_id: Uuid, limit: usize) -> JabeXResult<Vec<PopularSearch>>;
    fn clear(&self, profile_id: Uuid) -> JabeXResult<()>;
}

/// Port for ink document references (JSON via InkStorage).
pub trait InkDocumentRepository: Send + Sync {
    fn create(&self, doc: &InkDocumentRef) -> JabeXResult<()>;
    fn find(&self, id: Uuid) -> JabeXResult<Option<InkDocumentRef>>;
    fn list_by_owner(
        &self,
        owner_type: InkOwner,
        owner_id: Uuid,
        include_deleted: bool,
    ) -> JabeXResult<Vec<InkDocumentRef>>;
    fn list_all(&self, include_deleted: bool) -> JabeXResult<Vec<InkDocumentRef>>;
    fn update(&self, doc: &InkDocumentRef) -> JabeXResult<()>;
    fn soft_delete(
        &self,
        id: Uuid,
        deleted_at: DateTime<Utc>,
        deleted_by: Option<Uuid>,
    ) -> JabeXResult<()>;
    fn hard_delete(&self, id: Uuid) -> JabeXResult<()>;
}
