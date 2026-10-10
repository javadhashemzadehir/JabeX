/// Unicode + Arabic/Persian character normalization for indexing and queries.
pub fn normalize_search_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            // Arabic Yeh / Alef Maksura → Persian Yeh
            '\u{064A}' | '\u{0649}' => out.push('\u{06CC}'),
            // Arabic Kaf → Persian Keheh
            '\u{0643}' => out.push('\u{06A9}'),
            // Arabic / Persian digits → ASCII (helps mixed queries)
            '\u{06F0}'..='\u{06F9}' => out.push(char::from(b'0' + (ch as u32 - 0x06F0) as u8)),
            '\u{0660}'..='\u{0669}' => out.push(char::from(b'0' + (ch as u32 - 0x0660) as u8)),
            // Collapse ZWNJ / ZWJ to space for matching
            '\u{200C}' | '\u{200D}' => out.push(' '),
            c if c.is_control() => {}
            c => {
                for lower in c.to_lowercase() {
                    out.push(lower);
                }
            }
        }
    }
    collapse_whitespace(&out)
}

/// Detect primary text direction for UI (mixed → rtl if any strong RTL char).
pub fn detect_text_direction(input: &str) -> &'static str {
    let mut has_rtl = false;
    let mut has_ltr = false;
    for ch in input.chars() {
        if is_rtl_char(ch) {
            has_rtl = true;
        } else if ch.is_ascii_alphabetic() {
            has_ltr = true;
        }
    }
    if has_rtl && !has_ltr {
        "rtl"
    } else if has_rtl {
        "rtl"
    } else {
        "ltr"
    }
}

fn is_rtl_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{0590}'..='\u{05FF}' // Hebrew
            | '\u{0600}'..='\u{06FF}' // Arabic
            | '\u{0750}'..='\u{077F}'
            | '\u{08A0}'..='\u{08FF}'
            | '\u{FB50}'..='\u{FDFF}'
            | '\u{FE70}'..='\u{FEFF}'
    )
}

fn collapse_whitespace(input: &str) -> String {
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

    #[test]
    fn normalizes_arabic_yeh_and_kaf_to_persian() {
        assert_eq!(normalize_search_text("كتاب"), "کتاب");
        assert_eq!(normalize_search_text("علي"), "علی");
    }

    #[test]
    fn detects_rtl_and_ltr() {
        assert_eq!(detect_text_direction("فلسفه"), "rtl");
        assert_eq!(detect_text_direction("javascript"), "ltr");
        assert_eq!(detect_text_direction("learn فلسفه"), "rtl");
    }

    #[test]
    fn lowercases_and_collapses_space() {
        assert_eq!(normalize_search_text("  Hello   World  "), "hello world");
    }
}
