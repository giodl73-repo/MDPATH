use crate::error::MdPathError;
use crate::heading::normalize_heading;
use crate::selector::{parse_selector, parse_sub_selectors, parse_type_kind, validate_sub_selectors};

/// A parsed `md://` URI.
///
/// Grammar:
/// ```text
/// md://path[#heading-path][:[type[.kind]:]selector][sub-selector][?query]
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MdUri {
    /// File path relative to proof root, e.g. `computing/01-PACKAGE.md`
    pub path: String,
    /// Normalized heading path segments, e.g. `["the-big-picture", "layer-1"]`
    pub heading_path: Vec<String>,
    /// Element type (figure, table, chart, text, heading)
    pub element_type: Option<ElementType>,
    /// Kind qualifier (flowchart, key-value, etc.)
    pub kind: Option<String>,
    /// How to select within the type collection
    pub selector: Selector,
    /// Sub-selectors [row=X], [col=Y], [box=Z]
    pub sub_selectors: Vec<SubSelector>,
    /// Query parameters (?select, ?filter, ?count, ?top, ?skip)
    pub query: Option<QueryParams>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementType { Figure, Table, Chart, Text, Heading, Section }

/// Strings over numbers — named selectors always preferred.
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    Named(String),
    Index(usize),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubSelector {
    pub key: String,
    pub value: SelectorValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectorValue { Named(String), Index(usize) }

#[derive(Debug, Clone, PartialEq, Default)]
pub struct QueryParams {
    pub select: Option<Vec<String>>,
    pub filter: Option<String>,
    pub count: bool,
    pub top: Option<usize>,
    pub skip: Option<usize>,
}

impl MdUri {
    /// Parse a complete `md://` URI string.
    ///
    /// Extraction order (important — later extractions can't see earlier ones):
    /// 1. Strip `md://` prefix
    /// 2. Split on first `#` → path + fragment
    /// 3. From fragment: extract `?query` suffix
    /// 4. From remainder: extract `[sub-selectors]`
    /// 5. From remainder: split on first `:` → heading-path + type/selector block
    /// 6. From type/selector block: split on first `:` → type.kind + selector
    pub fn parse(s: &str) -> Result<Self, MdPathError> {
        let rest = s.strip_prefix("md://")
            .ok_or_else(|| MdPathError::ParseError(
                format!("URI must start with md://, got: {:?}", s)
            ))?;

        // Split path from fragment at first `#`
        let (path, fragment) = rest.split_once('#').unwrap_or((rest, ""));

        if path.is_empty() {
            return Err(MdPathError::ParseError("md:// path component is empty".into()));
        }
        if !path.ends_with(".md") {
            return Err(MdPathError::ParseError(
                format!("md:// path must end in .md, got: {:?}", path)
            ));
        }

        // Parse the fragment (everything after #)
        let (heading_path, element_type, kind, selector, sub_selectors, query) =
            parse_fragment(fragment)?;

        // Validate sub-selectors against the declared element type (Invariant I-10)
        if let Some(ref et) = element_type {
            validate_sub_selectors(et, &sub_selectors)?;
        }

        Ok(MdUri {
            path: path.to_string(),
            heading_path,
            element_type,
            kind,
            selector,
            sub_selectors,
            query,
        })
    }

    /// Return the canonical string form of this URI.
    pub fn to_uri_string(&self) -> String {
        let mut s = format!("md://{}", self.path);

        if !self.heading_path.is_empty() {
            s.push('#');
            s.push_str(&self.heading_path.join("/"));
        }

        let type_part = match &self.element_type {
            None => String::new(),
            Some(t) => {
                let base = match t {
                    ElementType::Figure => "figure",
                    ElementType::Table => "table",
                    ElementType::Chart => "chart",
                    ElementType::Text => "text",
                    ElementType::Heading => "heading",
                    ElementType::Section => "section",
                };
                match &self.kind {
                    None => base.to_string(),
                    Some(k) => format!("{}.{}", base, k),
                }
            }
        };

        let sel_part = match &self.selector {
            Selector::None => String::new(),
            Selector::Index(n) => n.to_string(),
            Selector::Named(name) => name.clone(),
        };

        if !type_part.is_empty() || !sel_part.is_empty() {
            s.push(':');
            if !type_part.is_empty() && !sel_part.is_empty() {
                s.push_str(&type_part);
                s.push(':');
                s.push_str(&sel_part);
            } else if !type_part.is_empty() {
                s.push_str(&type_part);
            } else {
                s.push_str(&sel_part);
            }
        }

        for sub in &self.sub_selectors {
            let val = match &sub.value {
                SelectorValue::Named(n) => n.clone(),
                SelectorValue::Index(i) => i.to_string(),
            };
            s.push_str(&format!("[{}={}]", sub.key, val));
        }

        if let Some(q) = &self.query {
            let mut parts = Vec::new();
            if let Some(sel) = &q.select {
                parts.push(format!("select={}", sel.join(",")));
            }
            if let Some(f) = &q.filter {
                parts.push(format!("filter={}", f));
            }
            if q.count { parts.push("count".to_string()); }
            if let Some(n) = q.top { parts.push(format!("top={}", n)); }
            if let Some(n) = q.skip { parts.push(format!("skip={}", n)); }
            if !parts.is_empty() {
                s.push('?');
                s.push_str(&parts.join("&"));
            }
        }

        s
    }

    /// Return this URI with the selector replaced by a named label.
    /// Used by proof to upgrade numeric URIs to named form.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.selector = Selector::Named(label.into());
        self
    }
}

/// Parse the fragment portion (everything after `#`).
///
/// Extraction order matters — later steps must not see content already extracted.
fn parse_fragment(fragment: &str) -> Result<(
    Vec<String>,        // heading_path
    Option<ElementType>, // element_type
    Option<String>,     // kind
    Selector,           // selector
    Vec<SubSelector>,   // sub_selectors
    Option<QueryParams>, // query
), MdPathError> {
    if fragment.is_empty() {
        return Ok((vec![], None, None, Selector::None, vec![], None));
    }

    // Step 1: Extract ?query from end
    let (main, query_str) = fragment.split_once('?').unwrap_or((fragment, ""));
    let query = if query_str.is_empty() { None } else { Some(parse_query(query_str)?) };

    // Step 2: Extract [sub-selectors] from end of main
    let (main, sub_str) = extract_sub_selectors(main);
    let sub_selectors = parse_sub_selectors(sub_str)?;

    // Step 3: Split on first `:` → heading-path + type/selector block
    let (heading_raw, type_sel_block) = main.split_once(':').unwrap_or((main, ""));

    // Parse heading path — normalize each segment
    let heading_path: Vec<String> = heading_raw
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| normalize_heading(s))
        .collect();

    if type_sel_block.is_empty() {
        return Ok((heading_path, None, None, Selector::None, sub_selectors, query));
    }

    // Step 4: Parse type[.kind][:selector]
    let (element_type, kind, selector) = parse_type_sel_block(type_sel_block)?;

    Ok((heading_path, element_type, kind, selector, sub_selectors, query))
}

/// Extract `[sub-selector-content]` from the end of a string.
/// Returns (remainder, sub-selector-content-or-empty).
fn extract_sub_selectors(s: &str) -> (&str, &str) {
    // Find the FIRST `[` (sub-selectors always come before `?`)
    if let Some(open) = s.find('[') {
        if s.ends_with(']') {
            return (&s[..open], &s[open+1..s.len()-1]);
        }
    }
    (s, "")
}

/// Parse `type[.kind][:selector]` block — the part after the first `:` in the fragment.
fn parse_type_sel_block(block: &str) -> Result<(Option<ElementType>, Option<String>, Selector), MdPathError> {
    // Could be:
    //   "0"                    → selector only (integer shorthand)
    //   "goroutine-scheduler"  → selector only (named shorthand)
    //   "figure"               → type only, no selector
    //   "figure.flowchart"     → type.kind, no selector
    //   "figure:0"             → type + integer selector
    //   "figure.flowchart:name"→ type.kind + named selector

    // First check if there's a second `:` → type:selector
    if let Some(colon) = block.find(':') {
        let type_part = &block[..colon];
        let sel_part = &block[colon+1..];
        let (etype, kind) = parse_type_kind(type_part)?;
        let selector = parse_selector(sel_part);
        return Ok((Some(etype), kind, selector));
    }

    // No second colon — could be type-only, or shorthand selector
    // Distinguish: if it looks like a type name or type.kind, treat as type
    // Otherwise treat as selector
    let is_type = matches!(block.split('.').next(), Some("figure" | "table" | "chart" | "text" | "heading" | "section"));

    if is_type {
        let (etype, kind) = parse_type_kind(block)?;
        Ok((Some(etype), kind, Selector::None))
    } else {
        // Pure selector (shorthand: number or label, no explicit type)
        Ok((None, None, parse_selector(block)))
    }
}

/// Parse OData-style query parameters.
fn parse_query(s: &str) -> Result<QueryParams, MdPathError> {
    let mut q = QueryParams::default();
    for pair in s.split('&') {
        let pair = pair.trim();
        if pair.is_empty() { continue; }
        if pair == "count" { q.count = true; continue; }

        if let Some((key, value)) = pair.split_once('=') {
            match key.trim() {
                "select" => {
                    q.select = Some(value.split(',').map(|s| s.trim().to_string()).collect());
                }
                "filter" => { q.filter = Some(value.to_string()); }
                "top" => {
                    q.top = Some(value.parse().map_err(|_| {
                        MdPathError::ParseError(format!("?top must be an integer, got {:?}", value))
                    })?);
                }
                "skip" => {
                    q.skip = Some(value.parse().map_err(|_| {
                        MdPathError::ParseError(format!("?skip must be an integer, got {:?}", value))
                    })?);
                }
                other => {
                    return Err(MdPathError::ParseError(format!("unknown query parameter {:?}", other)));
                }
            }
        }
    }
    Ok(q)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> MdUri { MdUri::parse(s).unwrap() }

    // ── Basic forms ────────────────────────────────────────────────────────

    #[test]
    fn whole_file() {
        let u = parse("md://computing/01-PACKAGE.md");
        assert_eq!(u.path, "computing/01-PACKAGE.md");
        assert!(u.heading_path.is_empty());
        assert!(u.element_type.is_none());
        assert_eq!(u.selector, Selector::None);
    }

    #[test]
    fn section_only() {
        let u = parse("md://computing/01-PACKAGE.md#the-big-picture");
        assert_eq!(u.heading_path, vec!["the-big-picture"]);
        assert!(u.element_type.is_none());
        assert_eq!(u.selector, Selector::None);
    }

    #[test]
    fn subsection_path() {
        let u = parse("md://file.md#parent/child/grandchild");
        assert_eq!(u.heading_path, vec!["parent", "child", "grandchild"]);
    }

    #[test]
    fn heading_normalized_in_path() {
        let u = parse("md://file.md#The Big Picture");
        assert_eq!(u.heading_path, vec!["the-big-picture"]);
    }

    // ── Shorthand selector (no explicit type) ──────────────────────────────

    #[test]
    fn shorthand_integer() {
        let u = parse("md://file.md#section:0");
        assert!(u.element_type.is_none()); // defaults to figure at resolution time
        assert_eq!(u.selector, Selector::Index(0));
    }

    #[test]
    fn shorthand_named() {
        let u = parse("md://file.md#section:goroutine-scheduler");
        assert!(u.element_type.is_none());
        assert_eq!(u.selector, Selector::Named("goroutine-scheduler".into()));
    }

    // ── Type + selector ────────────────────────────────────────────────────

    #[test]
    fn figure_with_index() {
        let u = parse("md://file.md#section:figure:0");
        assert_eq!(u.element_type, Some(ElementType::Figure));
        assert_eq!(u.kind, None);
        assert_eq!(u.selector, Selector::Index(0));
    }

    #[test]
    fn figure_flowchart_named() {
        let u = parse("md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler");
        assert_eq!(u.heading_path, vec!["concurrency-model"]);
        assert_eq!(u.element_type, Some(ElementType::Figure));
        assert_eq!(u.kind, Some("flowchart".into()));
        assert_eq!(u.selector, Selector::Named("goroutine-scheduler".into()));
    }

    #[test]
    fn table_key_value() {
        let u = parse("md://languages/05-CSHARP.md#type-system-snapshot:table.key-value:0");
        assert_eq!(u.element_type, Some(ElementType::Table));
        assert_eq!(u.kind, Some("key-value".into()));
        assert_eq!(u.selector, Selector::Index(0));
    }

    #[test]
    fn type_only_no_selector() {
        let u = parse("md://file.md#section:figure");
        assert_eq!(u.element_type, Some(ElementType::Figure));
        assert_eq!(u.selector, Selector::None);
    }

    // ── Sub-selectors ─────────────────────────────────────────────────────

    #[test]
    fn sub_selector_row() {
        let u = parse("md://file.md#section:table:0[row=Binding]");
        assert_eq!(u.sub_selectors.len(), 1);
        assert_eq!(u.sub_selectors[0].key, "row");
        assert!(matches!(u.sub_selectors[0].value, SelectorValue::Named(ref s) if s == "Binding"));
    }

    #[test]
    fn sub_selector_row_and_col() {
        let u = parse("md://file.md#section:table:0[row=Binding,col=Value]");
        assert_eq!(u.sub_selectors.len(), 2);
        assert_eq!(u.sub_selectors[0].key, "row");
        assert_eq!(u.sub_selectors[1].key, "col");
    }

    #[test]
    fn sub_selector_box() {
        let u = parse("md://computing/02-C.md#compilation-pipeline:figure.flowchart:0[box=PREPROCESSOR]");
        assert_eq!(u.element_type, Some(ElementType::Figure));
        assert_eq!(u.selector, Selector::Index(0));
        assert_eq!(u.sub_selectors[0].key, "box");
        assert!(matches!(&u.sub_selectors[0].value, SelectorValue::Named(s) if s == "PREPROCESSOR"));
    }

    // ── Query parameters ──────────────────────────────────────────────────

    #[test]
    fn query_select() {
        let u = parse("md://file.md#section:table:0?select=Axis,Value");
        let q = u.query.unwrap();
        assert_eq!(q.select, Some(vec!["Axis".into(), "Value".into()]));
    }

    #[test]
    fn query_filter() {
        let u = parse("md://file.md#section:table:0?filter=Axis eq Binding");
        let q = u.query.unwrap();
        assert_eq!(q.filter, Some("Axis eq Binding".into()));
    }

    #[test]
    fn query_count() {
        let u = parse("md://file.md#section:figure?count");
        assert!(u.query.unwrap().count);
    }

    #[test]
    fn query_top_skip() {
        let u = parse("md://file.md#section:table:0?top=10&skip=2");
        let q = u.query.unwrap();
        assert_eq!(q.top, Some(10));
        assert_eq!(q.skip, Some(2));
    }

    #[test]
    fn full_uri() {
        let u = parse("md://computing/01-PACKAGE.md#the-big-picture:figure.layer-stack:package-layers[box=SYSTEM]?select=content");
        assert_eq!(u.path, "computing/01-PACKAGE.md");
        assert_eq!(u.heading_path, vec!["the-big-picture"]);
        assert_eq!(u.element_type, Some(ElementType::Figure));
        assert_eq!(u.kind, Some("layer-stack".into()));
        assert_eq!(u.selector, Selector::Named("package-layers".into()));
        assert_eq!(u.sub_selectors[0].key, "box");
        assert!(u.query.unwrap().select.is_some());
    }

    // ── Error cases ────────────────────────────────────────────────────────

    #[test]
    fn wrong_scheme() {
        assert!(MdUri::parse("http://example.com").is_err());
    }

    #[test]
    fn missing_md_extension() {
        assert!(MdUri::parse("md://file.txt").is_err());
    }

    #[test]
    fn round_trip() {
        let uris = [
            "md://computing/01-PACKAGE.md",
            "md://computing/01-PACKAGE.md#the-big-picture",
            "md://file.md#section:0",
            "md://file.md#section:figure.flowchart:goroutine-scheduler",
            "md://file.md#section:table:0",
        ];
        for uri in &uris {
            let parsed = MdUri::parse(uri).unwrap();
            let emitted = parsed.to_uri_string();
            // Must re-parse without error (round-trip stability)
            MdUri::parse(&emitted).unwrap_or_else(|e| panic!("round-trip failed for {}: {} → {}", uri, emitted, e));
        }
    }
}
