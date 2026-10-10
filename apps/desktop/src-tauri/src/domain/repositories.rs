use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::card::Card;
use super::deck::Deck;
use super::errors::JabeXResult;
use super::review::Review;

/// Port for deck persistence. Domain never depends on SQLite.
pub trait DeckRepository: Send + Sync {
    fn create(&self, deck: &Deck) -> JabeXResult<()>;
    fn find(&self, id: Uuid) -> JabeXResult<Option<Deck>>;
    fn list(&self) -> JabeXResult<Vec<Deck>>;
    fn update(&self, deck: &Deck) -> JabeXResult<()>;
    fn soft_delete(&self, id: Uuid, deleted_at: DateTime<Utc>) -> JabeXResult<()>;
}

/// Port for card persistence.
pub trait CardRepository: Send + Sync {
    fn create(&self, card: &Card) -> JabeXResult<()>;
    fn find(&self, id: Uuid) -> JabeXResult<Option<Card>>;
    fn list_by_deck(&self, deck_id: Uuid) -> JabeXResult<Vec<Card>>;
    fn list_due(&self, deck_id: Option<Uuid>, now: DateTime<Utc>) -> JabeXResult<Vec<Card>>;
    fn update(&self, card: &Card) -> JabeXResult<()>;
    fn soft_delete(&self, id: Uuid, deleted_at: DateTime<Utc>) -> JabeXResult<()>;
}

/// Port for append-only review logs.
pub trait ReviewRepository: Send + Sync {
    fn create(&self, review: &Review) -> JabeXResult<()>;
    fn list_by_card(&self, card_id: Uuid) -> JabeXResult<Vec<Review>>;
    fn list_all(&self) -> JabeXResult<Vec<Review>>;
}
