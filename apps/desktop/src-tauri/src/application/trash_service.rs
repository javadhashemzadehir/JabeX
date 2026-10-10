use uuid::Uuid;

use crate::domain::attachment::AttachmentOwner;
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::ink::InkOwner;
use crate::domain::media_ports::{AttachmentStorage, InkStorage};
use crate::domain::repositories::{
    AttachmentRepository, CardRepository, DeckRepository, InkDocumentRepository,
};
use crate::domain::trash::{TrashEntityKind, TrashItem, TrashRepository};
use crate::shared::time::now_utc;

/// Application service for Trash: soft-delete cascade, restore, permanent delete.
pub struct TrashService<
    T: TrashRepository,
    D: DeckRepository,
    C: CardRepository,
    A: AttachmentRepository,
    I: InkDocumentRepository,
    S: AttachmentStorage + InkStorage,
> {
    trash: T,
    decks: D,
    cards: C,
    attachments: A,
    ink: I,
    storage: S,
}

impl<
        T: TrashRepository,
        D: DeckRepository,
        C: CardRepository,
        A: AttachmentRepository,
        I: InkDocumentRepository,
        S: AttachmentStorage + InkStorage,
    > TrashService<T, D, C, A, I, S>
{
    pub fn new(trash: T, decks: D, cards: C, attachments: A, ink: I, storage: S) -> Self {
        Self {
            trash,
            decks,
            cards,
            attachments,
            ink,
            storage,
        }
    }

    pub fn list_deleted_items(&self) -> JabeXResult<Vec<TrashItem>> {
        self.trash.list_deleted_items()
    }

    /// Soft-delete with parent→child cascade. Never hard-deletes.
    pub fn soft_delete_item(
        &self,
        kind: TrashEntityKind,
        id: Uuid,
        deleted_by: Option<Uuid>,
    ) -> JabeXResult<()> {
        let at = now_utc();
        match kind {
            TrashEntityKind::Deck => {
                self.decks
                    .find(id)?
                    .ok_or(JabeXError::NotFound { entity: "deck", id })?;
                let cards = self.cards.list_by_deck(id)?;
                for card in cards {
                    self.soft_delete_card_tree(card.id, at, deleted_by)?;
                }
                self.decks.soft_delete(id, at, deleted_by)?;
                tracing::info!(deck_id = %id, "deck moved to trash");
            }
            TrashEntityKind::Card => {
                self.cards
                    .find(id)?
                    .ok_or(JabeXError::NotFound { entity: "card", id })?;
                self.soft_delete_card_tree(id, at, deleted_by)?;
                tracing::info!(card_id = %id, "card moved to trash");
            }
            TrashEntityKind::Attachment => {
                self.attachments.soft_delete(id, at, deleted_by)?;
                tracing::info!(attachment_id = %id, "attachment moved to trash");
            }
            TrashEntityKind::InkDocument => {
                self.ink.soft_delete(id, at, deleted_by)?;
                tracing::info!(ink_id = %id, "ink document moved to trash");
            }
        }
        Ok(())
    }

    pub fn restore_item(&self, kind: TrashEntityKind, id: Uuid) -> JabeXResult<()> {
        self.trash.restore_item(kind, id, now_utc())?;
        tracing::info!(
            entity = kind.as_str(),
            id = %id,
            "trash item restored"
        );
        Ok(())
    }

    /// Explicit, irreversible permanent delete (DB rows + media files).
    pub fn permanently_delete_item(&self, kind: TrashEntityKind, id: Uuid) -> JabeXResult<()> {
        // Collect media paths before rows disappear.
        let media = self.collect_media_for_purge(kind, id)?;
        self.trash.permanently_delete_item(kind, id)?;
        for path in media.attachment_paths {
            if let Some(attachment) = media
                .attachment_snapshots
                .iter()
                .find(|a| a.relative_path == path)
            {
                let _ = self.storage.remove_attachment_files(attachment);
            }
        }
        for path in media.ink_paths {
            let _ = self.storage.remove_document(&path);
        }
        tracing::info!(
            entity = kind.as_str(),
            id = %id,
            "trash item permanently deleted"
        );
        Ok(())
    }

    fn soft_delete_card_tree(
        &self,
        card_id: Uuid,
        at: chrono::DateTime<chrono::Utc>,
        deleted_by: Option<Uuid>,
    ) -> JabeXResult<()> {
        let attachments = self
            .attachments
            .list_by_owner(AttachmentOwner::Card, card_id, false)?;
        for attachment in attachments {
            self.attachments
                .soft_delete(attachment.id, at, deleted_by)?;
        }
        // Also soft-delete document-owned attachments referencing this card id if any.
        let doc_attachments =
            self.attachments
                .list_by_owner(AttachmentOwner::Document, card_id, false)?;
        for attachment in doc_attachments {
            self.attachments
                .soft_delete(attachment.id, at, deleted_by)?;
        }
        for owner in [InkOwner::Card, InkOwner::Document] {
            let docs = self.ink.list_by_owner(owner, card_id, false)?;
            for doc in docs {
                self.ink.soft_delete(doc.id, at, deleted_by)?;
            }
        }
        self.cards.soft_delete(card_id, at, deleted_by)?;
        Ok(())
    }

    fn collect_media_for_purge(
        &self,
        kind: TrashEntityKind,
        id: Uuid,
    ) -> JabeXResult<MediaPurgePlan> {
        let mut plan = MediaPurgePlan::default();
        match kind {
            TrashEntityKind::Deck => {
                let cards = self.cards.list_by_deck_all(id, true)?;
                for card in cards {
                    self.append_card_media(&mut plan, card.id)?;
                }
            }
            TrashEntityKind::Card => {
                self.append_card_media(&mut plan, id)?;
            }
            TrashEntityKind::Attachment => {
                if let Some(attachment) = self.attachments.find(id)? {
                    plan.attachment_paths.push(attachment.relative_path.clone());
                    plan.attachment_snapshots.push(attachment);
                }
            }
            TrashEntityKind::InkDocument => {
                if let Some(doc) = self.ink.find(id)? {
                    plan.ink_paths.push(doc.relative_path);
                }
            }
        }
        Ok(plan)
    }

    fn append_card_media(&self, plan: &mut MediaPurgePlan, card_id: Uuid) -> JabeXResult<()> {
        for owner in [AttachmentOwner::Card, AttachmentOwner::Document] {
            for attachment in self.attachments.list_by_owner(owner, card_id, true)? {
                plan.attachment_paths.push(attachment.relative_path.clone());
                plan.attachment_snapshots.push(attachment);
            }
        }
        for owner in [InkOwner::Card, InkOwner::Document] {
            for doc in self.ink.list_by_owner(owner, card_id, true)? {
                plan.ink_paths.push(doc.relative_path);
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct MediaPurgePlan {
    attachment_paths: Vec<String>,
    ink_paths: Vec<String>,
    attachment_snapshots: Vec<crate::domain::attachment::Attachment>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::card::Card;
    use crate::domain::deck::Deck;
    use crate::domain::repositories::DeckRepository;
    use crate::infrastructure::database::open_database_at;
    use crate::infrastructure::database::DatabasePaths;
    use crate::infrastructure::filesystem::LocalMediaStorage;
    use crate::infrastructure::repositories::{
        SqliteAttachmentRepository, SqliteCardRepository, SqliteDeckRepository,
        SqliteInkDocumentRepository, SqliteTrashRepository,
    };
    use tempfile::tempdir;

    fn trash_service(
        dir: &tempfile::TempDir,
    ) -> TrashService<
        SqliteTrashRepository,
        SqliteDeckRepository,
        SqliteCardRepository,
        SqliteAttachmentRepository,
        SqliteInkDocumentRepository,
        LocalMediaStorage,
    > {
        let db = open_database_at(dir.path()).expect("db");
        let paths = DatabasePaths::from_root(dir.path().to_path_buf()).expect("paths");
        let storage = LocalMediaStorage::new(paths);
        TrashService::new(
            SqliteTrashRepository::new(db.clone()),
            SqliteDeckRepository::new(db.clone()),
            SqliteCardRepository::new(db.clone()),
            SqliteAttachmentRepository::new(db.clone()),
            SqliteInkDocumentRepository::new(db),
            storage,
        )
    }

    #[test]
    fn delete_restore_and_permanent_delete() {
        let dir = tempdir().expect("tempdir");
        let svc = trash_service(&dir);
        let db = open_database_at(dir.path()).expect("db");
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db);

        let deck = Deck::new("TrashMe".into(), None);
        decks.create(&deck).expect("deck");
        let card = Card::new(deck.id, "front", "back");
        cards.create(&card).expect("card");
        let profile = Uuid::now_v7();

        svc.soft_delete_item(TrashEntityKind::Deck, deck.id, Some(profile))
            .expect("soft delete");
        assert!(decks.find(deck.id).expect("find").is_none());
        assert!(cards.find(card.id).expect("find").is_none());

        let listed = svc.list_deleted_items().expect("list");
        assert!(listed.iter().any(|i| i.id == deck.id));
        assert!(listed.iter().any(|i| i.id == card.id));
        assert_eq!(
            listed.iter().find(|i| i.id == deck.id).unwrap().deleted_by,
            Some(profile)
        );

        svc.restore_item(TrashEntityKind::Deck, deck.id)
            .expect("restore");
        assert!(decks.find(deck.id).expect("find").is_some());
        assert!(cards.find(card.id).expect("find").is_some());
        assert!(svc.list_deleted_items().expect("list").is_empty());

        svc.soft_delete_item(TrashEntityKind::Deck, deck.id, Some(profile))
            .expect("soft delete again");
        svc.permanently_delete_item(TrashEntityKind::Deck, deck.id)
            .expect("purge");
        assert!(decks.find_any(deck.id).expect("any").is_none());
        assert!(cards.find_any(card.id).expect("any").is_none());
        assert!(svc.list_deleted_items().expect("list").is_empty());
    }

    #[test]
    fn parent_restore_restores_children() {
        let dir = tempdir().expect("tempdir");
        let svc = trash_service(&dir);
        let db = open_database_at(dir.path()).expect("db");
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db);

        let deck = Deck::new("Parent".into(), None);
        decks.create(&deck).expect("deck");
        let c1 = Card::new(deck.id, "a", "b");
        let c2 = Card::new(deck.id, "c", "d");
        cards.create(&c1).expect("c1");
        cards.create(&c2).expect("c2");

        svc.soft_delete_item(TrashEntityKind::Deck, deck.id, None)
            .expect("delete");
        svc.restore_item(TrashEntityKind::Deck, deck.id)
            .expect("restore");

        assert!(!cards.find(c1.id).expect("f1").expect("e1").is_deleted());
        assert!(!cards.find(c2.id).expect("f2").expect("e2").is_deleted());
    }
}
