use serde::{Deserialize, Serialize};

/// Unified searchable entity kinds. New kinds register into the index without
/// changing the search UI contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchEntityType {
    Card,
    Deck,
    Tag,
    Review,
    Note,
    Attachment,
    Setting,
    Command,
}

impl SearchEntityType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Card => "card",
            Self::Deck => "deck",
            Self::Tag => "tag",
            Self::Review => "review",
            Self::Note => "note",
            Self::Attachment => "attachment",
            Self::Setting => "setting",
            Self::Command => "command",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "card" => Some(Self::Card),
            "deck" => Some(Self::Deck),
            "tag" => Some(Self::Tag),
            "review" => Some(Self::Review),
            "note" => Some(Self::Note),
            "attachment" => Some(Self::Attachment),
            "setting" => Some(Self::Setting),
            "command" => Some(Self::Command),
            _ => None,
        }
    }
}

/// Filter chips shown in the command palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SearchFilter {
    #[default]
    All,
    Cards,
    Decks,
    Tags,
    Reviews,
    Commands,
}

impl SearchFilter {
    pub fn entity_types(self) -> Option<&'static [SearchEntityType]> {
        match self {
            Self::All => None,
            Self::Cards => Some(&[SearchEntityType::Card]),
            Self::Decks => Some(&[SearchEntityType::Deck]),
            Self::Tags => Some(&[SearchEntityType::Tag]),
            Self::Reviews => Some(&[SearchEntityType::Review, SearchEntityType::Card]),
            Self::Commands => Some(&[SearchEntityType::Command, SearchEntityType::Setting]),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Cards => "cards",
            Self::Decks => "decks",
            Self::Tags => "tags",
            Self::Reviews => "reviews",
            Self::Commands => "commands",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "all" => Some(Self::All),
            "cards" => Some(Self::Cards),
            "decks" => Some(Self::Decks),
            "tags" => Some(Self::Tags),
            "reviews" => Some(Self::Reviews),
            "commands" => Some(Self::Commands),
            _ => None,
        }
    }
}
