use uuid::Uuid;

use crate::domain::card::{Card, CardListPage};
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::{CardRepository, DeckRepository};
use crate::domain::rich_text::RichTextDocument;
use crate::shared::time::now_utc;

/// Default page size for card list IPC (large decks).
pub const DEFAULT_CARD_LIST_LIMIT: u32 = 50;
const MAX_CARD_LIST_LIMIT: u32 = 200;

pub struct CardService<C: CardRepository, D: DeckRepository> {
    cards: C,
    decks: D,
}

impl<C: CardRepository, D: DeckRepository> CardService<C, D> {
    pub fn new(cards: C, decks: D) -> Self {
        Self { cards, decks }
    }

    pub fn create_card(
        &self,
        deck_id: Uuid,
        front_content: RichTextDocument,
        back_content: RichTextDocument,
    ) -> JabeXResult<Card> {
        validate_faces(&front_content, &back_content)?;
        self.ensure_deck(deck_id)?;
        let card = Card::new(deck_id, front_content, back_content);
        self.cards.create(&card)?;
        tracing::info!(card_id = %card.id, deck_id = %deck_id, "card created");
        Ok(card)
    }

    pub fn get_card(&self, id: Uuid) -> JabeXResult<Card> {
        self.cards
            .find(id)?
            .ok_or(JabeXError::NotFound { entity: "card", id })
    }

    pub fn list_cards(&self, deck_id: Uuid) -> JabeXResult<Vec<Card>> {
        self.ensure_deck(deck_id)?;
        self.cards.list_by_deck(deck_id)
    }

    pub fn count_cards(&self, deck_id: Uuid) -> JabeXResult<u32> {
        self.ensure_deck(deck_id)?;
        self.cards.count_by_deck(deck_id)
    }

    /// Paginated list projections without full TipTap documents.
    pub fn list_card_summaries(
        &self,
        deck_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> JabeXResult<CardListPage> {
        self.ensure_deck(deck_id)?;
        let limit = limit
            .unwrap_or(DEFAULT_CARD_LIST_LIMIT)
            .clamp(1, MAX_CARD_LIST_LIMIT);
        let offset = offset.unwrap_or(0);
        let total = self.cards.count_by_deck(deck_id)?;
        let cards = self.cards.list_by_deck_page(deck_id, limit, offset)?;
        let items = cards.iter().map(Card::to_summary).collect::<Vec<_>>();
        let loaded = offset.saturating_add(items.len() as u32);
        Ok(CardListPage {
            items,
            total,
            offset,
            limit,
            has_more: loaded < total,
        })
    }

    pub fn update_card_content(
        &self,
        id: Uuid,
        front_content: RichTextDocument,
        back_content: RichTextDocument,
    ) -> JabeXResult<Card> {
        validate_faces(&front_content, &back_content)?;
        let mut card = self.get_card(id)?;
        let content_changed =
            card.front_content != front_content || card.back_content != back_content;
        card.front_content = front_content;
        card.back_content = back_content;
        if content_changed {
            // Stale pronunciation/translation must not outlive edited faces.
            card.clear_language_assist();
        }
        card.updated_at = now_utc();
        card.bump_revision();
        self.cards.update(&card)?;
        Ok(card)
    }

    pub fn delete_card(&self, id: Uuid) -> JabeXResult<()> {
        self.cards.soft_delete(id, now_utc(), None)?;
        tracing::info!(card_id = %id, "card soft-deleted");
        Ok(())
    }

    fn ensure_deck(&self, deck_id: Uuid) -> JabeXResult<()> {
        self.decks.find(deck_id)?.ok_or(JabeXError::NotFound {
            entity: "deck",
            id: deck_id,
        })?;
        Ok(())
    }
}

fn validate_faces(front: &RichTextDocument, back: &RichTextDocument) -> JabeXResult<()> {
    front
        .validate()
        .map_err(|e| JabeXError::Validation(e))?;
    back.validate()
        .map_err(|e| JabeXError::Validation(e))?;
    if front.is_blank() || back.is_blank() {
        return Err(JabeXError::Validation(
            "front and back content are required".into(),
        ));
    }
    Ok(())
}
