/// Label detection for figures within code blocks.
///
/// Three priority rules (applied in order, first match wins):
///
/// **Rule 1 — Inline label:** First non-empty line INSIDE the fence, if:
///   - The opening fence has NO language info string (` ``` ` not ` ```python `)
///   - The line contains ONLY text characters (no box-drawing chars, no `|`, `+`, `┌`, etc.)
///   - The label is not a pure digit string (reserved for numeric selectors)
///
/// **Rule 2 — Preceding label:** Last non-empty markdown line BEFORE the fence
/// (with no blank lines between that line and the fence), if:
///   - The line is bold (`**text**`) OR
///   - The line is standalone text of ≤ 60 characters
///   - The line is not a heading, list item, link, or code
///
/// **Rule 3 — No label:** Numeric fallback (`:0`, `:1`, etc.)
/// Returns true if a line contains box-drawing characters that disqualify it
/// from being an inline label.
pub fn has_box_chars(line: &str) -> bool {
    line.chars().any(|c| {
        matches!(
            c,
            '┌' | '┐'
                | '└'
                | '┘'
                | '├'
                | '┤'
                | '┬'
                | '┴'
                | '┼'
                | '─'
                | '│'
                | '╔'
                | '╗'
                | '╚'
                | '╝'
                | '═'
                | '║'
                | '+'
                | '|' // ASCII box chars
        )
    })
}

/// Returns true if a string is a pure digit sequence (reserved for numeric selectors).
/// Labels must not be pure digits.
pub fn is_pure_digit(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// Attempt to detect Rule 1: inline label from inside a code fence.
///
/// `fence_info` is the text after the backticks on the opening fence line.
/// `lines` is the content of the code block (excluding fences).
pub fn detect_inline_label(fence_info: &str, lines: &[&str]) -> Option<String> {
    // Rule 1 requires no language info string
    if !fence_info.trim().is_empty() {
        return None;
    }
    // Find the first non-empty line
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Must not contain box-drawing chars
        if has_box_chars(trimmed) {
            return None;
        }
        // Must not be pure digits
        if is_pure_digit(trimmed) {
            return None;
        }
        return Some(trimmed.to_string());
    }
    None
}

/// Attempt to detect Rule 2: preceding label from before a code fence.
///
/// `lines_before` is a slice of lines immediately before the fence (reverse order:
/// index 0 is the line just before the fence, index 1 is two lines before, etc.).
pub fn detect_preceding_label(lines_before: &[&str]) -> Option<String> {
    let trimmed = lines_before.first()?.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Accept bold FIRST: **text** — must check before list-item rejection
    // since "**text**" starts with '*' but is not a list item
    if trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() > 4 {
        let inner = &trimmed[2..trimmed.len() - 2];
        if !inner.is_empty() && !is_pure_digit(inner) {
            return Some(inner.to_string());
        }
    }

    // Reject headings, list items, links, code spans (non-bold)
    if trimmed.starts_with('#') {
        return None;
    }
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("1.") {
        return None;
    }
    if trimmed.starts_with('[') || trimmed.starts_with('`') {
        return None;
    }

    // Accept standalone text ≤ 60 chars
    if trimmed.len() <= 60 && !is_pure_digit(trimmed) {
        return Some(trimmed.to_string());
    }

    // Line is too long and not bold — no label
    None
}

/// Normalize a label for matching (lowercase, collapse whitespace, strip punctuation).
pub fn normalize_label(label: &str) -> String {
    let lower = label.to_lowercase();
    let mut result = String::new();
    let mut last_space = false;
    for c in lower.chars() {
        if c.is_alphanumeric() {
            result.push(c);
            last_space = false;
        } else if (c.is_whitespace() || c == '-' || c == '_') && !last_space && !result.is_empty() {
            result.push(' ');
            last_space = true;
        }
        // Strip other punctuation
    }
    result.trim().to_string()
}

/// Match a selector against a label using the priority hierarchy:
/// 1. Exact match (normalized)
/// 2. Starts-with match (normalized)
/// 3. Substring match (normalized)
///
/// Returns (matches, is_exact) — callers check for ambiguity.
pub fn label_matches(selector: &str, label: &str) -> (bool, bool) {
    let norm_sel = normalize_label(selector);
    let norm_label = normalize_label(label);
    let exact = norm_label == norm_sel;
    let starts = !exact && norm_label.starts_with(&norm_sel);
    let contains = !exact && !starts && norm_label.contains(&norm_sel);
    (exact || starts || contains, exact)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_label_detected() {
        let lines = &[
            "GOROUTINE SCHEDULER — M:N multiplexing",
            "┌──────────────────┐",
        ];
        assert_eq!(
            detect_inline_label("", lines),
            Some("GOROUTINE SCHEDULER — M:N multiplexing".into())
        );
    }

    #[test]
    fn inline_label_blocked_by_language_string() {
        let lines = &["def foo(): pass", "# comment"];
        assert_eq!(detect_inline_label("python", lines), None);
    }

    #[test]
    fn inline_label_blocked_by_box_chars() {
        let lines = &["┌──────────────────┐", "│ content │"];
        assert_eq!(detect_inline_label("", lines), None);
    }

    #[test]
    fn inline_label_blocked_if_pure_digit() {
        let lines = &["0", "┌──────────────────┐"];
        assert_eq!(detect_inline_label("", lines), None);
    }

    #[test]
    fn preceding_bold_label() {
        let before = ["**Architecture Overview**", "some prior text"];
        assert_eq!(
            detect_preceding_label(&before),
            Some("Architecture Overview".into())
        );
    }

    #[test]
    fn preceding_short_text_label() {
        let before = ["Go Scheduler", "other content"];
        assert_eq!(detect_preceding_label(&before), Some("Go Scheduler".into()));
    }

    #[test]
    fn preceding_label_blocked_by_blank_line() {
        let before = ["", "Go Scheduler"];
        assert_eq!(detect_preceding_label(&before), None);
    }

    #[test]
    fn label_exact_match() {
        let (matches, exact) = label_matches(
            "goroutine scheduler",
            "GOROUTINE SCHEDULER — M:N multiplexing",
        );
        assert!(matches);
        assert!(!exact); // not exact, but starts-with or contains
    }

    #[test]
    fn label_exact_wins_over_substring() {
        let (matches, exact) = label_matches("Binding", "Binding");
        assert!(matches);
        assert!(exact);
    }
}
