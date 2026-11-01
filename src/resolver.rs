use crate::{
    classify::{Classifier, DefaultClassifier},
    document::{ParsedDocument, ParsedElement, ParsedTable},
    error::MdPathError,
    label::label_matches,
    parser::parse_document,
    subselect::{apply_figure_subselectors, apply_table_query, apply_table_subselectors},
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

/// Resolve a single `md://` URI against a root directory using the default classifier.
///
/// For multiple URIs in the same file, prefer [`BatchResolver`].
/// To use a custom classifier, use [`resolve_with_classifier`].
pub fn resolve(uri: &MdUri, root: &Path) -> Result<ResolvedElement, MdPathError> {
    resolve_with_classifier(uri, root, &DefaultClassifier)
}

/// Resolve a single `md://` URI with a custom classifier.
///
/// The classifier determines how fenced code blocks are mapped to [`ElementType`].
/// Use this when you generate content that mdpath should recognize beyond its defaults.
pub fn resolve_with_classifier(
    uri: &MdUri,
    root: &Path,
    classifier: &dyn Classifier,
) -> Result<ResolvedElement, MdPathError> {
    let file_path = root.join(&uri.path);
    let content =
        std::fs::read_to_string(&file_path).map_err(|e| MdPathError::Io(uri.path.clone(), e))?;
    let doc = parse_document(&content);
    resolve_in_doc_with_classifier(uri, &doc, &file_path, classifier)
}

/// Resolve an `md://` URI against an already-parsed document using the default classifier.
pub fn resolve_in_doc(
    uri: &MdUri,
    doc: &ParsedDocument,
    file: &Path,
) -> Result<ResolvedElement, MdPathError> {
    resolve_in_doc_with_classifier(uri, doc, file, &DefaultClassifier)
}

/// Resolve an `md://` URI against an already-parsed document with a custom classifier.
pub fn resolve_in_doc_with_classifier(
    uri: &MdUri,
    doc: &ParsedDocument,
    file: &Path,
    classifier: &dyn Classifier,
) -> Result<ResolvedElement, MdPathError> {
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
    let typed: Vec<&ParsedElement> = candidates
        .iter()
        .filter(|e| {
            element_matches_type(e, target_type, classifier)
                && element_matches_kind(e, kind_filter, classifier)
        })
        .copied()
        .collect();

    // Step 4: Apply selector
    let selected = match &uri.selector {
        Selector::None | Selector::Index(0) if typed.len() == 1 => typed[0],
        Selector::Index(n) => typed
            .get(*n)
            .ok_or(MdPathError::ElementNotFound(*n, typed.len()))?,
        Selector::Named(label) => {
            // Exact → starts-with → substring, with ambiguity detection
            find_by_label(&typed, label)?
        }
        Selector::None => typed.first().ok_or(MdPathError::ElementNotFound(0, 0))?,
    };

    if matches!(uri.selector, Selector::Index(_)) && !matches!(target_type, ElementType::Table) {
        if let Some(label) = get_label(selected) {
            return Err(MdPathError::NumericUriStale(label));
        }
    }

    element_to_resolved(selected, uri, file, section_heading, classifier)
}

/// Returns true if a parsed element matches the requested type.
fn element_matches_type(
    elem: &ParsedElement,
    t: &ElementType,
    classifier: &dyn Classifier,
) -> bool {
    match elem {
        ParsedElement::Table(_) => matches!(t, ElementType::Table),
        ParsedElement::Paragraph(_) => matches!(t, ElementType::Text | ElementType::Heading),
        ParsedElement::CodeBlock(b) => {
            let refs: Vec<&str> = b.content.iter().map(|s| s.as_str()).collect();
            if let Some((detected, _)) = classifier.classify(&b.fence_info, &refs) {
                detected == *t
            } else {
                // Unclassified code blocks match Figure, Chart, and Text (backward compat)
                matches!(
                    t,
                    ElementType::Figure | ElementType::Chart | ElementType::Text
                )
            }
        }
    }
}

/// Find an element by label using exact → starts-with → substring hierarchy.
fn find_by_label<'a>(
    candidates: &[&'a ParsedElement],
    selector: &str,
) -> Result<&'a ParsedElement, MdPathError> {
    let labeled: Vec<_> = candidates
        .iter()
        .filter_map(|e| get_label(e).map(|l| (*e, l)))
        .collect();

    // Phase 1: exact matches
    let exact: Vec<_> = labeled
        .iter()
        .filter(|(_, l)| {
            let (matches, is_exact) = label_matches(selector, l);
            matches && is_exact
        })
        .collect();
    if exact.len() == 1 {
        return Ok(exact[0].0);
    }
    if exact.len() > 1 {
        return Err(MdPathError::LabelAmbiguous(
            selector.to_string(),
            exact.len(),
        ));
    }

    // Phase 2: starts-with
    let starts: Vec<_> = labeled
        .iter()
        .filter(|(_, l)| {
            let norm_sel = crate::label::normalize_label(selector);
            let norm_l = crate::label::normalize_label(l);
            !norm_l.is_empty() && norm_l.starts_with(&norm_sel) && norm_l != norm_sel
        })
        .collect();
    if starts.len() == 1 {
        return Ok(starts[0].0);
    }
    if starts.len() > 1 {
        return Err(MdPathError::LabelAmbiguous(
            selector.to_string(),
            starts.len(),
        ));
    }

    // Phase 3: substring
    let subs: Vec<_> = labeled
        .iter()
        .filter(|(_, l)| {
            let norm_sel = crate::label::normalize_label(selector);
            let norm_l = crate::label::normalize_label(l);
            norm_l.contains(&norm_sel)
        })
        .collect();
    if subs.len() == 1 {
        return Ok(subs[0].0);
    }
    if subs.len() > 1 {
        return Err(MdPathError::LabelAmbiguous(
            selector.to_string(),
            subs.len(),
        ));
    }

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
    classifier: &dyn Classifier,
) -> Result<ResolvedElement, MdPathError> {
    match elem {
        ParsedElement::CodeBlock(b) => {
            let refs: Vec<&str> = b.content.iter().map(|s| s.as_str()).collect();
            let (detected_type, detected_kind) = classifier
                .classify(&b.fence_info, &refs)
                .unwrap_or((ElementType::Figure, None));
            let kind = uri.kind.clone().or(detected_kind);

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
                element_type: uri.element_type.clone().unwrap_or(detected_type),
                kind,
            })
        }
        ParsedElement::Table(t) => {
            let content = if !uri.sub_selectors.is_empty() {
                apply_table_subselectors(t, &uri.sub_selectors)?.content
            } else if let Some(q) = &uri.query {
                apply_table_query(t, q.filter.as_deref(), q.select.as_deref(), q.top, q.skip)
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
                kind: uri.kind.clone(),
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
            kind: uri.kind.clone(),
        }),
    }
}

fn format_table(t: &ParsedTable) -> String {
    let mut rows = vec![t
        .headers
        .iter()
        .map(|h| h.trim())
        .collect::<Vec<_>>()
        .join(" | ")];
    rows.push(
        t.separator
            .iter()
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join(" | "),
    );
    for row in &t.rows {
        rows.push(row.iter().map(|c| c.trim()).collect::<Vec<_>>().join(" | "));
    }
    rows.join("\n")
}

/// Returns true if an element matches the requested kind (or no kind filter).
fn element_matches_kind(
    elem: &ParsedElement,
    kind: Option<&str>,
    classifier: &dyn Classifier,
) -> bool {
    let Some(k) = kind else {
        return true;
    };
    match elem {
        ParsedElement::Table(_) => true, // table kind filtering is advisory
        ParsedElement::Paragraph(_) => true,
        ParsedElement::CodeBlock(b) => {
            let refs: Vec<&str> = b.content.iter().map(|s| s.as_str()).collect();
            if let Some((_, detected_kind)) = classifier.classify(&b.fence_info, &refs) {
                detected_kind.as_deref() == Some(k)
            } else {
                true // no classification → don't filter out
            }
        }
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
    file_path: std::path::PathBuf,
    doc: ParsedDocument,
}

impl BatchResolver {
    pub fn new(root: &Path, relative_path: &str) -> Result<Self, MdPathError> {
        let file_path = root.join(relative_path);
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| MdPathError::Io(relative_path.to_string(), e))?;
        let doc = parse_document(&content);
        Ok(BatchResolver { file_path, doc })
    }

    pub fn resolve_uri(&self, uri: &str) -> Result<ResolvedElement, MdPathError> {
        self.resolve_uri_with_classifier(uri, &DefaultClassifier)
    }

    pub fn resolve_uri_with_classifier(
        &self,
        uri: &str,
        classifier: &dyn Classifier,
    ) -> Result<ResolvedElement, MdPathError> {
        let parsed = MdUri::parse(uri)?;
        resolve_in_doc_with_classifier(&parsed, &self.doc, &self.file_path, classifier)
    }

    pub fn resolve(&self, uri: &MdUri) -> Result<ResolvedElement, MdPathError> {
        resolve_in_doc_with_classifier(uri, &self.doc, &self.file_path, &DefaultClassifier)
    }

    pub fn resolve_with_classifier(
        &self,
        uri: &MdUri,
        classifier: &dyn Classifier,
    ) -> Result<ResolvedElement, MdPathError> {
        resolve_in_doc_with_classifier(uri, &self.doc, &self.file_path, classifier)
    }

    /// Number of headings detected in the document.
    pub fn heading_count(&self) -> usize {
        self.doc.headings.iter().filter(|h| h.level > 0).count()
    }
}
