use uuid::Uuid;

use crate::domain::deck::{Deck, DeckSettings};
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::DeckRepository;
use crate::shared::time::now_utc;

pub struct DeckService<D: DeckRepository> {
    decks: D,
}

impl<D: DeckRepository> DeckService<D> {
    pub fn new(decks: D) -> Self {
        Self { decks }
    }

    pub fn create_deck(
        &self,
        name: String,
        description: Option<String>,
        settings: Option<DeckSettings>,
    ) -> JabeXResult<Deck> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(JabeXError::Validation("deck name is required".into()));
        }

        let mut deck = Deck::new(name, description);
        if let Some(settings) = settings {
            deck.settings = settings;
        }
        self.decks.create(&deck)?;
        tracing::info!(deck_id = %deck.id, "deck created");
        Ok(deck)
    }

    pub fn get_deck(&self, id: Uuid) -> JabeXResult<Deck> {
        self.decks.find(id)?.ok_or(JabeXError::NotFound {
            entity: "deck",
            id,
        })
    }

    pub fn list_decks(&self) -> JabeXResult<Vec<Deck>> {
        self.decks.list()
    }

    pub fn rename_deck(&self, id: Uuid, name: String) -> JabeXResult<Deck> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(JabeXError::Validation("deck name is required".into()));
        }
        let mut deck = self.get_deck(id)?;
        deck.name = name;
        deck.updated_at = now_utc();
        deck.bump_revision();
        self.decks.update(&deck)?;
        Ok(deck)
    }

    pub fn delete_deck(&self, id: Uuid) -> JabeXResult<()> {
        self.decks.soft_delete(id, now_utc())?;
        tracing::info!(deck_id = %id, "deck soft-deleted");
        Ok(())
    }
}
