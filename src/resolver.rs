use crate::{
    document::{ParsedDocument, ParsedElement, CodeBlock, ParsedTable, Paragraph},
    error::MdPathError,
    kind::{detect_figure_kind, figure_matches_kind},
    label::label_matches,
    parser::parse_document,
    subselect::{apply_table_subselectors, apply_figure_subselectors, apply_table_query},
    uri::{ElementType, MdUri, Selector},
};
use std::path::Path;

/// A resolved markdown element.
#[derive(Debug, Clone)]
pub struct ResolvedElement {
    pub uri: String,
    pub file: std::path::PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub content: String,
    pub label: Option<String>,
    pub section_heading: Option<String>,
    pub element_type: ElementType,
    pub kind: Option<String>,
}

/// Resolve a single `md://` URI against a root directory.
///
/// For multiple URIs in the same file, prefer [`BatchResolver`].
pub fn resolve(uri: &MdUri, root: &Path) -> Result<ResolvedElement, MdPathError> {
    let file_path = root.join(&uri.path);
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| MdPathError::Io(uri.path.clone(), e))?;
    let doc = parse_document(&content);
    resolve_in_doc(uri, &doc, &file_path)
}

/// Resolve an `md://` URI against an already-parsed document.
pub fn resolve_in_doc(uri: &MdUri, doc: &ParsedDocument, file: &Path) -> Result<ResolvedElement, MdPathError> {
    // Step 1: Walk the heading path to find the target section
    let section_idx = if uri.heading_path.is_empty() {
        None // whole-file scope
    } else {
        let mut parent: Option<usize> = None;
        for segment in &uri.heading_path {
            let matches = doc.find_heading(segment, parent);
            match matches.len() {
                0 => return Err(MdPathError::SectionNotFound(segment.clone())),
                1 => parent = Some(matches[0]),
                _ => return Err(MdPathError::SectionAmbiguous(segment.clone())),
            }
        }
        parent
    };

    let section_heading = section_idx.map(|i| doc.headings[i].text.clone());

    // Step 2: If no type/selector, return the section itself
    if uri.element_type.is_none() && matches!(uri.selector, Selector::None) {
        if let Some(idx) = section_idx {
            let h = &doc.headings[idx];
            return Ok(ResolvedElement {
                uri: uri.path.clone(),
                file: file.to_path_buf(),
                line_start: h.line,
                line_end: h.line,
                content: format!("{} {}", "#".repeat(h.level), h.text),
                label: Some(h.text.clone()),
                section_heading,
                element_type: ElementType::Section,
                kind: None,
            });
        }
    }

    // Step 3: Collect elements of the target type within the section
    let target_type = uri.element_type.as_ref().unwrap_or(&ElementType::Figure);

    let candidates: Vec<&ParsedElement> = if let Some(idx) = section_idx {
        doc.elements_in_section(idx)
    } else {
        doc.elements.iter().collect()
    };

    let kind_filter = uri.kind.as_deref();
    let typed: Vec<&ParsedElement> = candidates.iter()
        .filter(|e| element_matches_type(e, target_type) && element_matches_kind(e, kind_filter))
        .copied()
        .collect();

    // Step 4: Apply selector
    let selected = match &uri.selector {
        Selector::None | Selector::Index(0) if typed.len() == 1 => typed[0],
        Selector::Index(n) => {
            typed.get(*n)
                .ok_or(MdPathError::ElementNotFound(*n, typed.len()))?
        }
        Selector::Named(label) => {
            // Exact → starts-with → substring, with ambiguity detection
            find_by_label(&typed, label)?
        }
        Selector::None => {
            typed.first().ok_or(MdPathError::ElementNotFound(0, 0))?
        }
    };

    element_to_resolved(selected, uri, file, section_heading)
}

/// Returns true if a parsed element matches the requested type.
fn element_matches_type(elem: &ParsedElement, t: &ElementType) -> bool {
    match (elem, t) {
        (ParsedElement::CodeBlock(_), ElementType::Figure) => true,
        (ParsedElement::Table(_), ElementType::Table) => true,
        (ParsedElement::Paragraph(_), ElementType::Text) => true,
        _ => false,
    }
}

/// Find an element by label using exact → starts-with → substring hierarchy.
fn find_by_label<'a>(candidates: &[&'a ParsedElement], selector: &str) -> Result<&'a ParsedElement, MdPathError> {
    let labeled: Vec<_> = candidates.iter()
        .filter_map(|e| get_label(e).map(|l| (*e, l)))
        .collect();

    // Phase 1: exact matches
    let exact: Vec<_> = labeled.iter()
        .filter(|(_, l)| {
            let (matches, is_exact) = label_matches(selector, l);
            matches && is_exact
        })
        .collect();
    if exact.len() == 1 { return Ok(exact[0].0); }
    if exact.len() > 1 { return Err(MdPathError::LabelAmbiguous(selector.to_string(), exact.len())); }

    // Phase 2: starts-with
    let starts: Vec<_> = labeled.iter()
        .filter(|(_, l)| {
            let norm_sel = crate::label::normalize_label(selector);
            let norm_l = crate::label::normalize_label(l);
            !norm_l.is_empty() && norm_l.starts_with(&norm_sel) && norm_l != norm_sel
        })
        .collect();
    if starts.len() == 1 { return Ok(starts[0].0); }
    if starts.len() > 1 { return Err(MdPathError::LabelAmbiguous(selector.to_string(), starts.len())); }

    // Phase 3: substring
    let subs: Vec<_> = labeled.iter()
        .filter(|(_, l)| {
            let norm_sel = crate::label::normalize_label(selector);
            let norm_l = crate::label::normalize_label(l);
            norm_l.contains(&norm_sel)
        })
        .collect();
    if subs.len() == 1 { return Ok(subs[0].0); }
    if subs.len() > 1 { return Err(MdPathError::LabelAmbiguous(selector.to_string(), subs.len())); }

    Err(MdPathError::LabelNotFound(selector.to_string()))
}

fn get_label(elem: &ParsedElement) -> Option<String> {
    match elem {
        ParsedElement::CodeBlock(b) => b.label.clone(),
        ParsedElement::Table(t) => t.headers.first().map(|h| h.trim().to_string()),
        ParsedElement::Paragraph(_) => None,
    }
}

fn element_to_resolved(
    elem: &ParsedElement,
    uri: &MdUri,
    file: &Path,
    section_heading: Option<String>,
) -> Result<ResolvedElement, MdPathError> {
    let detected_kind = match elem {
        ParsedElement::CodeBlock(b) => {
            let refs: Vec<&str> = b.content.iter().map(|s| s.as_str()).collect();
            uri.kind.clone().or_else(|| detect_figure_kind(&refs).map(|k| k.to_string()))
        }
        _ => uri.kind.clone(),
    };

    match elem {
        ParsedElement::CodeBlock(b) => {
            let content = if uri.sub_selectors.is_empty() {
                b.content.join("\n")
            } else {
                apply_figure_subselectors(b, &uri.sub_selectors)?
            };
            Ok(ResolvedElement {
                uri: uri.to_uri_string(),
                file: file.to_path_buf(),
                line_start: b.line_start,
                line_end: b.line_end,
                content,
                label: b.label.clone(),
                section_heading,
                element_type: ElementType::Figure,
                kind: detected_kind,
            })
        }
        ParsedElement::Table(t) => {
            let content = if !uri.sub_selectors.is_empty() {
                apply_table_subselectors(t, &uri.sub_selectors)?.content
            } else if let Some(q) = &uri.query {
                apply_table_query(
                    t,
                    q.filter.as_deref(),
                    q.select.as_deref(),
                    q.top,
                    q.skip,
                )
            } else {
                format_table(t)
            };
            Ok(ResolvedElement {
                uri: uri.to_uri_string(),
                file: file.to_path_buf(),
                line_start: t.line_start,
                line_end: t.line_end,
                content,
                label: t.headers.first().map(|h| h.trim().to_string()),
                section_heading,
                element_type: ElementType::Table,
                kind: detected_kind,
            })
        }
        ParsedElement::Paragraph(p) => Ok(ResolvedElement {
            uri: uri.to_uri_string(),
            file: file.to_path_buf(),
            line_start: p.line_start,
            line_end: p.line_end,
            content: p.lines.join("\n"),
            label: None,
            section_heading,
            element_type: ElementType::Text,
            kind: detected_kind,
        }),
    }
}

fn format_table(t: &ParsedTable) -> String {
    let mut rows = vec![t.headers.iter().map(|h| h.trim()).collect::<Vec<_>>().join(" | ")];
    rows.push(t.separator.iter().map(|s| s.trim()).collect::<Vec<_>>().join(" | "));
    for row in &t.rows { rows.push(row.iter().map(|c| c.trim()).collect::<Vec<_>>().join(" | ")); }
    rows.join("\n")
}

/// Returns true if an element matches the requested kind (or no kind filter).
fn element_matches_kind(elem: &ParsedElement, kind: Option<&str>) -> bool {
    match (elem, kind) {
        (_, None) => true,
        (ParsedElement::CodeBlock(b), Some(k)) => {
            let refs: Vec<&str> = b.content.iter().map(|s| s.as_str()).collect();
            figure_matches_kind(&refs, Some(k))
        }
        (ParsedElement::Table(_), Some(k)) => {
            // Table kinds: key-value, comparison, reference, decision
            // For now: any table matches any table kind (kind filtering is advisory)
            let _ = k;
            true
        }
        _ => false,
    }
}

/// Efficiently resolve multiple URIs in the same file without re-reading it.
///
/// proof checks N elements per file in one pass. BatchResolver reads once,
/// builds the document model, then resolves each URI from the cached tree.
///
/// # Example
/// ```rust,no_run
/// use mdpath::resolver::BatchResolver;
/// use std::path::Path;
///
/// let root = Path::new("/repo");
/// let mut batch = BatchResolver::new(root, "computing/01-PACKAGE.md").unwrap();
/// let fig = batch.resolve_uri("md://computing/01-PACKAGE.md#the-big-picture:0").unwrap();
/// ```
pub struct BatchResolver {
    root: std::path::PathBuf,
    file_path: std::path::PathBuf,
    doc: ParsedDocument,
}

impl BatchResolver {
    pub fn new(root: &Path, relative_path: &str) -> Result<Self, MdPathError> {
        let file_path = root.join(relative_path);
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| MdPathError::Io(relative_path.to_string(), e))?;
        let doc = parse_document(&content);
        Ok(BatchResolver { root: root.to_path_buf(), file_path, doc })
    }

    pub fn resolve_uri(&self, uri: &str) -> Result<ResolvedElement, MdPathError> {
        let parsed = MdUri::parse(uri)?;
        resolve_in_doc(&parsed, &self.doc, &self.file_path)
    }

    pub fn resolve(&self, uri: &MdUri) -> Result<ResolvedElement, MdPathError> {
        resolve_in_doc(uri, &self.doc, &self.file_path)
    }

    /// Number of headings detected in the document.
    pub fn heading_count(&self) -> usize {
        self.doc.headings.iter().filter(|h| h.level > 0).count()
    }
}
