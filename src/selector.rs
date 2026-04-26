/// Selector parsing for the type, kind, and index/label components.
///
/// Grammar fragment handled here:
///   `[type[.kind]:]selector`
///
/// Rules:
///   - If selector matches `^\d+$` → integer index (always 0-based)
///   - Otherwise → named label (strings preferred over numbers per spec)
///   - Labels must not be pure digit strings (see label.rs)
///
/// Sub-selector grammar: `[key=value]`
///   - key: alphanumeric + underscore
///   - value: if all digits → index; otherwise → named string

use crate::uri::{ElementType, Selector, SelectorValue, SubSelector};
use crate::error::MdPathError;

/// Parse a type string like "figure", "table.key-value", "chart.bar"
pub fn parse_type_kind(s: &str) -> Result<(ElementType, Option<String>), MdPathError> {
    let (type_str, kind) = if let Some(dot) = s.find('.') {
        (&s[..dot], Some(s[dot+1..].to_string()))
    } else {
        (s, None)
    };

    let element_type = match type_str {
        "figure" => ElementType::Figure,
        "table" => ElementType::Table,
        "chart" => ElementType::Chart,
        "text" => ElementType::Text,
        "heading" => ElementType::Heading,
        other => return Err(MdPathError::ParseError(
            format!("unknown element type {:?} — use figure, table, chart, text, or heading", other)
        )),
    };

    Ok((element_type, kind))
}

/// Parse a selector string: integer or named label.
pub fn parse_selector(s: &str) -> Selector {
    if s.is_empty() {
        Selector::None
    } else if s.chars().all(|c| c.is_ascii_digit()) {
        Selector::Index(s.parse().unwrap_or(0))
    } else {
        Selector::Named(s.to_string())
    }
}

/// Parse a sub-selector string like `row=Binding,col=Value`.
pub fn parse_sub_selectors(s: &str) -> Result<Vec<SubSelector>, MdPathError> {
    if s.is_empty() { return Ok(vec![]); }
    let mut result = Vec::new();
    for pair in s.split(',') {
        let pair = pair.trim();
        if pair.is_empty() { continue; }
        let (key, value) = pair.split_once('=').ok_or_else(|| {
            MdPathError::ParseError(format!("invalid sub-selector {:?} — expected key=value", pair))
        })?;
        let key = key.trim().to_string();
        let val_str = value.trim();
        let value = if val_str.chars().all(|c| c.is_ascii_digit()) {
            SelectorValue::Index(val_str.parse().unwrap_or(0))
        } else {
            SelectorValue::Named(val_str.to_string())
        };
        result.push(SubSelector { key, value });
    }
    Ok(result)
}

/// Validate that a sub-selector is legal for the given element type.
///
/// Invariant I-10: invalid combinations are rejected at parse time.
pub fn validate_sub_selectors(element_type: &ElementType, sub_selectors: &[SubSelector]) -> Result<(), MdPathError> {
    if sub_selectors.is_empty() { return Ok(()); }

    let allowed_keys: &[&str] = match element_type {
        ElementType::Figure => &["box", "row"],
        ElementType::Table => &["row", "col"],
        ElementType::Chart => &["bar"],
        ElementType::Text | ElementType::Heading => &[],
        ElementType::Section => &[],  // addressing a section directly
        _ => &[],
    };

    for sub in sub_selectors {
        if !allowed_keys.contains(&sub.key.as_str()) {
            return Err(MdPathError::InvalidSubSelector(
                format!("{:?}", element_type),
                format!("[{}=...]", sub.key),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_selector() {
        assert_eq!(parse_selector("0"), Selector::Index(0));
        assert_eq!(parse_selector("42"), Selector::Index(42));
    }

    #[test]
    fn named_selector() {
        assert_eq!(parse_selector("goroutine-scheduler"), Selector::Named("goroutine-scheduler".into()));
        assert_eq!(parse_selector(""), Selector::None);
    }

    #[test]
    fn type_kind_parsing() {
        let (t, k) = parse_type_kind("figure.flowchart").unwrap();
        assert_eq!(t, ElementType::Figure);
        assert_eq!(k, Some("flowchart".into()));

        let (t, k) = parse_type_kind("table").unwrap();
        assert_eq!(t, ElementType::Table);
        assert_eq!(k, None);
    }

    #[test]
    fn sub_selector_parsing() {
        let subs = parse_sub_selectors("row=Binding,col=Value").unwrap();
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].key, "row");
        assert!(matches!(subs[0].value, SelectorValue::Named(ref s) if s == "Binding"));
    }

    #[test]
    fn invalid_sub_selector_on_text() {
        let subs = vec![SubSelector { key: "row".into(), value: SelectorValue::Named("X".into()) }];
        assert!(validate_sub_selectors(&ElementType::Text, &subs).is_err());
    }
}
