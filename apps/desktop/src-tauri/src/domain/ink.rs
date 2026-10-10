use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

pub const INK_DOCUMENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InkOwner {
    Card,
    Document,
}

impl InkOwner {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Card => "card",
            Self::Document => "document",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "card" => Ok(Self::Card),
            "document" => Ok(Self::Document),
            other => Err(format!("unknown ink owner type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InkTool {
    Pen,
    Highlighter,
    Eraser,
}

/// A single sampled point on a stroke.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub x: f64,
    pub y: f64,
    /// Normalized pressure in `[0, 1]`; `1.0` when unavailable.
    pub pressure: f64,
    /// Milliseconds relative to stroke start (or absolute session clock).
    pub timestamp: u64,
}

/// Vector stroke — never a raster frame dump.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stroke {
    pub id: Uuid,
    pub tool: InkTool,
    pub color: String,
    pub width: f64,
    pub points: Vec<Point>,
}

impl Stroke {
    pub fn new(tool: InkTool, color: impl Into<String>, width: f64) -> Self {
        Self {
            id: create_id(),
            tool,
            color: color.into(),
            width,
            points: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InkLayer {
    pub id: Uuid,
    pub order: u32,
    pub visible: bool,
    pub opacity: f64,
    pub strokes: Vec<Stroke>,
}

impl InkLayer {
    pub fn new(order: u32) -> Self {
        Self {
            id: create_id(),
            order,
            visible: true,
            opacity: 1.0,
            strokes: Vec::new(),
        }
    }
}

/// Full ink payload stored as JSON on disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InkDocument {
    pub version: u32,
    pub id: Uuid,
    pub width: u32,
    pub height: u32,
    pub layers: Vec<InkLayer>,
}

impl InkDocument {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            version: INK_DOCUMENT_VERSION,
            id: create_id(),
            width,
            height,
            layers: vec![InkLayer::new(0)],
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version == 0 || self.version > INK_DOCUMENT_VERSION {
            return Err(format!(
                "unsupported ink document version: {}",
                self.version
            ));
        }
        if self.width == 0 || self.height == 0 {
            return Err("ink document width and height must be > 0".into());
        }
        if self.layers.is_empty() {
            return Err("ink document requires at least one layer".into());
        }
        for layer in &self.layers {
            if !(0.0..=1.0).contains(&layer.opacity) {
                return Err("layer opacity must be between 0 and 1".into());
            }
            for stroke in &layer.strokes {
                if stroke.width <= 0.0 {
                    return Err("stroke width must be > 0".into());
                }
                for point in &stroke.points {
                    if !(0.0..=1.0).contains(&point.pressure) {
                        return Err("point pressure must be between 0 and 1".into());
                    }
                }
            }
        }
        Ok(())
    }
}

/// SQLite reference row for an ink JSON file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InkDocumentRef {
    pub id: Uuid,
    pub owner_type: InkOwner,
    pub owner_id: Uuid,
    pub relative_path: String,
    pub width: u32,
    pub height: u32,
    pub content_hash: String,
    pub revision: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    /// Local profile UUID that soft-deleted this ink document.
    #[serde(default)]
    pub deleted_by: Option<Uuid>,
}

impl InkDocumentRef {
    pub fn new(
        id: Uuid,
        owner_type: InkOwner,
        owner_id: Uuid,
        relative_path: String,
        width: u32,
        height: u32,
        content_hash: String,
    ) -> Self {
        let now = now_utc();
        Self {
            id,
            owner_type,
            owner_id,
            relative_path,
            width,
            height,
            content_hash,
            revision: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            deleted_by: None,
        }
    }

    pub fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }

    pub fn soft_delete(&mut self, at: DateTime<Utc>, deleted_by: Option<Uuid>) {
        self.deleted_at = Some(at);
        self.deleted_by = deleted_by;
        self.updated_at = at;
        self.bump_revision();
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_new_document() {
        let doc = InkDocument::new(400, 300);
        assert!(doc.validate().is_ok());
        assert_eq!(doc.layers.len(), 1);
    }
}
