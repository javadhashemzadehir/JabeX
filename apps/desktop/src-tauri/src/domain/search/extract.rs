use serde_json::Value;

use super::normalize::normalize_search_text;

/// Extracted searchable text: display/content plus normalized / LaTeX forms.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtractedSearchText {
    pub text: String,
    pub normalized: String,
    pub latex_raw: String,
    pub latex_normalized: String,
}

impl ExtractedSearchText {
    pub fn combined_content(&self) -> String {
        let mut parts = Vec::new();
        if !self.text.is_empty() {
            parts.push(self.text.as_str());
        }
        if !self.latex_raw.is_empty() {
            parts.push(self.latex_raw.as_str());
        }
        if !self.latex_normalized.is_empty() {
            parts.push(self.latex_normalized.as_str());
        }
        parts.join("\n")
    }

    pub fn combined_normalized(&self) -> String {
        normalize_search_text(&self.combined_content())
    }
}

/// Reusable extraction pipeline for plain text, Markdown, HTML, rich JSON, LaTeX.
pub fn extract_search_text(input: &str) -> ExtractedSearchText {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return ExtractedSearchText::default();
    }

    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            return extract_from_json(&value);
        }
    }

    if looks_like_html(trimmed) {
        return finalize(&strip_html(trimmed));
    }

    if looks_like_markdown(trimmed) {
        return finalize(&strip_markdown(trimmed));
    }

    finalize(trimmed)
}

fn finalize(text: &str) -> ExtractedSearchText {
    let (plain, latex_raw, latex_norm) = split_latex(text);
    let text = collapse_ws(&plain);
    ExtractedSearchText {
        normalized: normalize_search_text(&text),
        text,
        latex_raw,
        latex_normalized: latex_norm,
    }
}

fn extract_from_json(value: &Value) -> ExtractedSearchText {
    let mut plain = String::new();
    let mut latex_raw = String::new();
    let mut latex_norm = String::new();
    walk_json(value, &mut plain, &mut latex_raw, &mut latex_norm);
    ExtractedSearchText {
        text: collapse_ws(&plain),
        normalized: normalize_search_text(&plain),
        latex_raw: collapse_ws(&latex_raw),
        latex_normalized: collapse_ws(&latex_norm),
    }
}

fn walk_json(value: &Value, plain: &mut String, latex_raw: &mut String, latex_norm: &mut String) {
    match value {
        Value::String(s) => {
            push_spaced(plain, s);
        }
        Value::Array(items) => {
            for item in items {
                walk_json(item, plain, latex_raw, latex_norm);
            }
        }
        Value::Object(map) => {
            let node_type = map.get("type").and_then(|v| v.as_str()).unwrap_or("");

            if matches!(node_type, "mathInline" | "mathDisplay") {
                if let Some(latex) = map
                    .get("attrs")
                    .and_then(|a| a.get("latex"))
                    .and_then(|v| v.as_str())
                {
                    push_spaced(latex_raw, latex);
                    push_spaced(latex_norm, &normalize_latex(latex));
                    push_spaced(plain, latex);
                    push_spaced(plain, &normalize_latex(latex));
                }
                return;
            }

            if let Some(text) = map.get("text").and_then(|v| v.as_str()) {
                push_spaced(plain, text);
            }

            // Slate-style children
            if let Some(children) = map.get("children") {
                walk_json(children, plain, latex_raw, latex_norm);
            }
            // TipTap content
            if let Some(content) = map.get("content") {
                walk_json(content, plain, latex_raw, latex_norm);
            }

            // Common caption / alt / title metadata
            for key in ["title", "alt", "caption", "fileName", "name", "label"] {
                if let Some(v) = map.get(key).and_then(|v| v.as_str()) {
                    push_spaced(plain, v);
                }
            }
        }
        _ => {}
    }
}

fn looks_like_html(input: &str) -> bool {
    input.contains('<') && input.contains('>') && (input.contains("</") || input.contains("/>"))
}

fn looks_like_markdown(input: &str) -> bool {
    input.contains('#')
        || input.contains("**")
        || input.contains("__")
        || input.contains("```")
        || input.contains("[") && input.contains("](")
        || input
            .lines()
            .any(|l| l.trim_start().starts_with(['-', '*', '>']))
}

fn strip_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    let mut in_entity = false;
    let mut entity = String::new();
    for ch in input.chars() {
        if in_tag {
            if ch == '>' {
                in_tag = false;
                out.push(' ');
            }
            continue;
        }
        if in_entity {
            if ch == ';' || ch.is_whitespace() {
                out.push_str(&decode_entity(&entity));
                entity.clear();
                in_entity = false;
                if ch != ';' {
                    out.push(ch);
                }
            } else {
                entity.push(ch);
            }
            continue;
        }
        match ch {
            '<' => in_tag = true,
            '&' => in_entity = true,
            _ => out.push(ch),
        }
    }
    if in_entity {
        out.push('&');
        out.push_str(&entity);
    }
    out
}

fn decode_entity(name: &str) -> String {
    match name {
        "amp" => "&".into(),
        "lt" => "<".into(),
        "gt" => ">".into(),
        "quot" => "\"".into(),
        "apos" | "#39" => "'".into(),
        "nbsp" => " ".into(),
        other if other.starts_with('#') => {
            let digits = other.trim_start_matches('#');
            let code = if let Some(hex) = digits
                .strip_prefix('x')
                .or_else(|| digits.strip_prefix('X'))
            {
                u32::from_str_radix(hex, 16).ok()
            } else {
                digits.parse().ok()
            };
            code.and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn strip_markdown(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_code_fence = false;
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_fence = !in_code_fence;
            out.push(' ');
            continue;
        }
        if in_code_fence {
            out.push_str(line);
            out.push(' ');
            continue;
        }
        let mut s = trimmed.to_string();
        // Headings
        while s.starts_with('#') {
            s = s.trim_start_matches('#').trim_start().to_string();
        }
        // Blockquote
        if let Some(rest) = s.strip_prefix('>') {
            s = rest.trim_start().to_string();
        }
        // List markers
        if let Some(rest) = s.strip_prefix("- ").or_else(|| s.strip_prefix("* ")) {
            s = rest.to_string();
        }
        // Bold / italic markers
        s = s
            .replace("**", " ")
            .replace("__", " ")
            .replace(['*', '_'], " ");
        // Images / links: ![alt](url) / [text](url)
        s = strip_md_links(&s);
        // Inline code
        s = s.replace('`', " ");
        out.push_str(&s);
        out.push(' ');
    }
    out
}

fn strip_md_links(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '!' && i + 1 < chars.len() && chars[i + 1] == '[' {
            i += 2;
            let start = i;
            while i < chars.len() && chars[i] != ']' {
                i += 1;
            }
            out.push_str(&chars[start..i].iter().collect::<String>());
            if i < chars.len() {
                i += 1;
            }
            if i < chars.len() && chars[i] == '(' {
                while i < chars.len() && chars[i] != ')' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
            }
            continue;
        }
        if chars[i] == '[' {
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != ']' {
                i += 1;
            }
            out.push_str(&chars[start..i].iter().collect::<String>());
            if i < chars.len() {
                i += 1;
            }
            if i < chars.len() && chars[i] == '(' {
                while i < chars.len() && chars[i] != ')' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
            }
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Split `$...$` / `$$...$$` style LaTeX; also normalize bare formulas.
fn split_latex(input: &str) -> (String, String, String) {
    let mut plain = String::new();
    let mut latex_raw = String::new();
    let mut latex_norm = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' {
            let display = i + 1 < chars.len() && chars[i + 1] == '$';
            i += if display { 2 } else { 1 };
            let start = i;
            while i < chars.len() {
                if chars[i] == '$' {
                    if display && i + 1 < chars.len() && chars[i + 1] == '$' {
                        break;
                    }
                    if !display {
                        break;
                    }
                }
                i += 1;
            }
            let formula: String = chars[start..i].iter().collect();
            push_spaced(&mut latex_raw, &formula);
            push_spaced(&mut latex_norm, &normalize_latex(&formula));
            push_spaced(&mut plain, &formula);
            push_spaced(&mut plain, &normalize_latex(&formula));
            if i < chars.len() {
                i += if display { 2 } else { 1 };
            }
            continue;
        }
        plain.push(chars[i]);
        i += 1;
    }

    // Also index normalized form of the whole string for bare formulas like E = mc^2
    if latex_raw.is_empty() && input.chars().any(|c| matches!(c, '^' | '_' | '\\')) {
        let norm = normalize_latex(input);
        if norm != normalize_search_text(input) {
            push_spaced(&mut latex_raw, input);
            push_spaced(&mut latex_norm, &norm);
            push_spaced(&mut plain, &norm);
        }
    }

    (plain, latex_raw, latex_norm)
}

/// `E = mc^2` → `e mc2` style searchable token stream.
pub fn normalize_latex(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            // Keep alphanumerics adjacent across ^ _ so `mc^2` → `mc2`.
            '^' | '_' | '\\' => {}
            '{' | '}' | '[' | ']' | '(' | ')' | '=' | '+' | '-' | '*' | '/' | ',' | '.' | ';'
            | ':' | '!' | '?' | '&' | '%' | '#' | '$' | '~' => out.push(' '),
            c if c.is_whitespace() => out.push(' '),
            c => {
                for lower in c.to_lowercase() {
                    out.push(lower);
                }
            }
        }
    }
    collapse_ws(&out)
}

fn push_spaced(buf: &mut String, text: &str) {
    let t = text.trim();
    if t.is_empty() {
        return;
    }
    if !buf.is_empty() && !buf.ends_with(char::is_whitespace) {
        buf.push(' ');
    }
    buf.push_str(t);
}

fn collapse_ws(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_space = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            if !prev_space && !out.is_empty() {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_plain_text() {
        let e = extract_search_text("hello world");
        assert_eq!(e.text, "hello world");
    }

    #[test]
    fn extracts_markdown() {
        let e = extract_search_text("# Biology\n\n**Cell structure**");
        assert!(e.text.contains("Biology"));
        assert!(e.text.contains("Cell structure"));
        assert!(!e.text.contains('#'));
        assert!(!e.text.contains("**"));
    }

    #[test]
    fn extracts_html() {
        let e = extract_search_text("<p>Hello <b>world</b></p>");
        assert!(e.text.to_lowercase().contains("hello"));
        assert!(e.text.to_lowercase().contains("world"));
        assert!(!e.text.contains('<'));
    }

    #[test]
    fn extracts_tiptap_json() {
        let doc = json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{ "type": "text", "text": "Photosynthesis" }]
            }]
        })
        .to_string();
        let e = extract_search_text(&doc);
        assert_eq!(e.text, "Photosynthesis");
    }

    #[test]
    fn extracts_slate_style_children() {
        let doc = json!({
            "type": "paragraph",
            "children": [{ "text": "Photosynthesis" }]
        })
        .to_string();
        let e = extract_search_text(&doc);
        assert!(e.text.contains("Photosynthesis"));
    }

    #[test]
    fn latex_normalized_finds_mc2() {
        let e = extract_search_text("E = mc^2");
        assert!(e.combined_normalized().contains("mc2") || e.latex_normalized.contains("mc2"));
        let math = extract_search_text(
            &json!({
                "version": 2,
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "mathInline",
                        "attrs": { "latex": "E = mc^2" }
                    }]
                }]
            })
            .to_string(),
        );
        assert!(math.combined_content().contains("mc2") || math.latex_normalized.contains("mc2"));
    }
}
