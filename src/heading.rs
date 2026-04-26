/// Heading normalization — converts markdown heading text to a URL-safe anchor.
///
/// Algorithm (GitHub-compatible + extended for md:// safety):
/// 1. Lowercase all characters
/// 2. Replace spaces and underscores with `-`
/// 3. Strip characters that are not alphanumeric or `-`
///    Note: `/` in heading text is stripped (not treated as path separator)
/// 4. Collapse consecutive `-` to a single `-`
/// 5. Strip leading and trailing `-`
///
/// Examples:
/// - "## The Big Picture"       → "the-big-picture"
/// - "## C++ Types"             → "c-types"  (++ stripped)
/// - "## Input/Output Handling" → "inputoutput-handling"  (/ stripped)
/// - "## Q&A"                   → "qa"
/// - "## Model: V2"             → "model-v2"
pub fn normalize_heading(text: &str) -> String {
    // Strip the leading `#` marks and whitespace
    let text = text.trim_start_matches('#').trim();

    let mut result = String::with_capacity(text.len());
    let mut last_was_dash = false;

    for c in text.chars() {
        let out = match c {
            'a'..='z' | '0'..='9' => Some(c),
            'A'..='Z' => Some(c.to_ascii_lowercase()),
            ' ' | '_' | '-' => Some('-'),
            // Strip everything else (punctuation, /, +, &, :, etc.)
            _ => None,
        };
        if let Some(ch) = out {
            if ch == '-' {
                if !last_was_dash && !result.is_empty() {
                    result.push('-');
                    last_was_dash = true;
                }
            } else {
                result.push(ch);
                last_was_dash = false;
            }
        }
    }

    // Strip trailing dash
    while result.ends_with('-') {
        result.pop();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_normalization() {
        assert_eq!(normalize_heading("## The Big Picture"), "the-big-picture");
        assert_eq!(normalize_heading("## Layer 1: OS Layer"), "layer-1-os-layer");
        assert_eq!(normalize_heading("Concurrency Model"), "concurrency-model");
    }

    #[test]
    fn punctuation_stripped() {
        assert_eq!(normalize_heading("## C++ Types"), "c-types");
        assert_eq!(normalize_heading("## Q&A"), "qa");
        assert_eq!(normalize_heading("## Model: V2"), "model-v2");
    }

    #[test]
    fn slash_stripped_not_separator() {
        // Slash in heading text is stripped — it does NOT create a path segment
        assert_eq!(normalize_heading("## Input/Output Handling"), "inputoutput-handling");
    }

    #[test]
    fn consecutive_dashes_collapsed() {
        assert_eq!(normalize_heading("## A -- B"), "a-b");
        assert_eq!(normalize_heading("## C++"), "c");
    }

    #[test]
    fn leading_trailing_stripped() {
        assert_eq!(normalize_heading("## -hello-"), "hello");
    }
}
