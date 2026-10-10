use serde::{Deserialize, Serialize};

use super::card::Card;

/// Aggregate study counts for the active local profile. No telemetry — local IPC only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyStats {
    pub deck_count: u32,
    pub card_count: u32,
    pub due_count: u32,
    pub reviews_today: u32,
    /// Counts for Leitner boxes 1..=MAX_BOX (index 0 = box 1).
    pub cards_by_box: [u32; Card::MAX_BOX as usize],
}

impl StudyStats {
    pub fn empty() -> Self {
        Self {
            deck_count: 0,
            card_count: 0,
            due_count: 0,
            reviews_today: 0,
            cards_by_box: [0; Card::MAX_BOX as usize],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_has_five_box_slots() {
        let stats = StudyStats::empty();
        assert_eq!(stats.cards_by_box.len(), 5);
        assert_eq!(stats.card_count, 0);
    }
}
