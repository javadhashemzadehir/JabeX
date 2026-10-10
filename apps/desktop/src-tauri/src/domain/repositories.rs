use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use super::attachment::{Attachment, AttachmentOwner};
use super::card::Card;
use super::deck::Deck;
use super::errors::JabeXResult;
use super::ink::{InkDocumentRef, InkOwner};
use super::review::Review;
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
    /// Cards in a deck, optionally including soft-deleted.
    fn list_by_deck_all(&self, deck_id: Uuid, include_deleted: bool) -> JabeXResult<Vec<Card>>;
    fn list_all(&self, include_deleted: bool) -> JabeXResult<Vec<Card>>;
    fn list_due(&self, deck_id: Option<Uuid>, now: DateTime<Utc>) -> JabeXResult<Vec<Card>>;
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
