use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};

/// Current write schema version for card face documents (TipTap-compatible JSON).
///
/// v1: StarterKit nodes. v2: spoilers, math, tables, highlighted code (ADR 014).
pub const RICH_TEXT_DOCUMENT_VERSION: u32 = 2;

/// Versioned rich-text document stored on card faces.
///
/// Wire / storage shape:
/// `{ "version": 1|2, type: "doc", "content": [ ...TipTap nodes ] }`
///
/// Legacy plain-text strings (pre-rich-text) are accepted on deserialize and
/// normalized via [`RichTextDocument::parse_stored`]. Readers accept v1–v2;
/// new writes emit [`RICH_TEXT_DOCUMENT_VERSION`].
#[derive(Debug, Clone, PartialEq)]
pub struct RichTextDocument {
    pub version: u32,
    pub content: Vec<Value>,
}

impl RichTextDocument {
    pub fn empty() -> Self {
        Self {
            version: RICH_TEXT_DOCUMENT_VERSION,
            content: vec![json!({ "type": "paragraph" })],
        }
    }

    pub fn from_plain_text(text: &str) -> Self {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Self::empty();
        }
        Self {
            version: RICH_TEXT_DOCUMENT_VERSION,
            content: vec![json!({
                "type": "paragraph",
                "content": [{ "type": "text", "text": trimmed }]
            })],
        }
    }

    /// Parse a SQLite TEXT cell: versioned JSON, bare TipTap doc, or plain text.
    pub fn parse_stored(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Self::empty();
        }
        if !trimmed.starts_with('{') {
            return Self::from_plain_text(trimmed);
        }
        match serde_json::from_str::<Value>(trimmed) {
            Ok(value) => {
                Self::from_json_value(value).unwrap_or_else(|_| Self::from_plain_text(trimmed))
            }
            Err(_) => Self::from_plain_text(trimmed),
        }
    }

    pub fn from_json_value(value: Value) -> Result<Self, String> {
        let obj = value
            .as_object()
            .ok_or_else(|| "rich text document must be an object".to_string())?;

        let doc_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("doc");
        if doc_type != "doc" {
            return Err(format!("unsupported document type '{doc_type}'"));
        }

        let version = obj
            .get("version")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(RICH_TEXT_DOCUMENT_VERSION);

        if version == 0 || version > RICH_TEXT_DOCUMENT_VERSION {
            return Err(format!(
                "unsupported rich text version {version} (max {RICH_TEXT_DOCUMENT_VERSION})"
            ));
        }

        let content = match obj.get("content") {
            None => Vec::new(),
            Some(Value::Array(items)) => items.clone(),
            Some(_) => return Err("document content must be an array".into()),
        };

        Ok(Self { version, content })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version == 0 || self.version > RICH_TEXT_DOCUMENT_VERSION {
            return Err(format!(
                "unsupported rich text version {} (max {RICH_TEXT_DOCUMENT_VERSION})",
                self.version
            ));
        }
        Ok(())
    }

    pub fn to_storage_string(&self) -> String {
        serde_json::to_string(&self.to_json_value())
            .expect("rich text document serialization cannot fail")
    }

    pub fn to_json_value(&self) -> Value {
        json!({
            "version": self.version,
            "type": "doc",
            "content": self.content,
        })
    }

    pub fn plain_text(&self) -> String {
        let mut out = String::new();
        for (index, node) in self.content.iter().enumerate() {
            if index > 0 {
                out.push('\n');
            }
            append_plain_text(node, &mut out);
        }
        out
    }

    pub fn is_blank(&self) -> bool {
        self.plain_text().trim().is_empty()
    }
}

impl From<&str> for RichTextDocument {
    fn from(value: &str) -> Self {
        Self::from_plain_text(value)
    }
}

impl From<String> for RichTextDocument {
    fn from(value: String) -> Self {
        Self::from_plain_text(&value)
    }
}

impl Serialize for RichTextDocument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_json_value().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RichTextDocument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(s) => Ok(Self::parse_stored(&s)),
            other => Self::from_json_value(other).map_err(serde::de::Error::custom),
        }
    }
}

fn append_plain_text(node: &Value, out: &mut String) {
    if let Some(text) = node.get("text").and_then(|v| v.as_str()) {
        out.push_str(text);
    }

    // Math leaf nodes store LaTeX in attrs, not text children.
    if matches!(
        node.get("type").and_then(|v| v.as_str()),
        Some("mathInline" | "mathDisplay")
    ) {
        if let Some(latex) = node
            .get("attrs")
            .and_then(|a| a.get("latex"))
            .and_then(|v| v.as_str())
        {
            out.push_str(latex);
        }
        return;
    }

    if let Some(children) = node.get("content").and_then(|v| v.as_array()) {
        for (index, child) in children.iter().enumerate() {
            let block_break = matches!(
                child.get("type").and_then(|v| v.as_str()),
                Some(
                    "paragraph"
                        | "heading"
                        | "bulletList"
                        | "orderedList"
                        | "listItem"
                        | "codeBlock"
                        | "spoilerBlock"
                        | "mathDisplay"
                        | "table"
                        | "tableRow"
                )
            );
            if index > 0 && block_break {
                out.push('\n');
            }
            append_plain_text(child, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_roundtrip() {
        let doc = RichTextDocument::from_plain_text("  hello  ");
        assert_eq!(doc.plain_text(), "hello");
        assert!(!doc.is_blank());
        let stored = doc.to_storage_string();
        let parsed = RichTextDocument::parse_stored(&stored);
        assert_eq!(parsed.plain_text(), "hello");
        assert_eq!(parsed.version, RICH_TEXT_DOCUMENT_VERSION);
    }

    #[test]
    fn legacy_plain_string_parses() {
        let doc = RichTextDocument::parse_stored("سیب");
        assert_eq!(doc.plain_text(), "سیب");
        assert!(doc.to_storage_string().contains("\"version\":2"));
    }

    #[test]
    fn deserialize_accepts_string_or_object() {
        let from_string: RichTextDocument = serde_json::from_str(r#""legacy""#).expect("string");
        assert_eq!(from_string.plain_text(), "legacy");

        let from_object: RichTextDocument = serde_json::from_str(
            r#"{"version":1,"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"x"}]}]}"#,
        )
        .expect("object");
        assert_eq!(from_object.plain_text(), "x");
        assert_eq!(from_object.version, 1);
    }

    #[test]
    fn accepts_v2_rejects_v3() {
        let v2 = RichTextDocument::from_json_value(json!({
            "version": 2,
            "type": "doc",
            "content": [{"type": "paragraph"}]
        }))
        .expect("v2");
        assert_eq!(v2.version, 2);
        assert!(v2.validate().is_ok());

        let err = RichTextDocument::from_json_value(json!({
            "version": 3,
            "type": "doc",
            "content": []
        }));
        assert!(err.is_err());
    }

    #[test]
    fn math_nodes_contribute_plain_text() {
        let doc = RichTextDocument::from_json_value(json!({
            "version": 2,
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{
                    "type": "mathInline",
                    "attrs": { "latex": "E=mc^2" }
                }]
            }]
        }))
        .expect("math doc");
        assert_eq!(doc.plain_text(), "E=mc^2");
        assert!(!doc.is_blank());
    }

    #[test]
    fn blank_document() {
        assert!(RichTextDocument::empty().is_blank());
        assert!(RichTextDocument::from_plain_text("   ").is_blank());
    }
}
