use crate::{error::MdPathError, uri::MdUri};
use std::path::Path;

/// A resolved markdown element.
#[derive(Debug, Clone)]
pub struct ResolvedElement {
    pub uri: String,
    pub file: std::path::PathBuf,
    pub line_start: usize,  // 1-based
    pub line_end: usize,    // 1-based
    pub content: String,
    pub label: Option<String>,
    pub section_heading: Option<String>,
    pub element_type: ElementType,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementType { Figure, Table, Chart, Text, Heading, Section }

/// Resolve a single `md://` URI.
///
/// For multiple URIs in the same file, use [`BatchResolver`] instead —
/// it reads the file once and resolves all URIs from the cached parse tree.
pub fn resolve(uri: &MdUri, root: &Path) -> Result<ResolvedElement, MdPathError> {
    let file_path = root.join(&uri.path);
    if !file_path.exists() {
        return Err(MdPathError::FileNotFound(uri.path.clone()));
    }
    // Full implementation pending — see design/SPEC.md
    Err(MdPathError::ParseError("resolver not yet implemented".into()))
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
/// let fig = batch.resolve("md://computing/01-PACKAGE.md#the-big-picture:0").unwrap();
/// let tbl = batch.resolve("md://computing/01-PACKAGE.md#layer-1:table:0").unwrap();
/// // File read exactly once — both resolved from the same parse tree.
/// ```
pub struct BatchResolver {
    root: std::path::PathBuf,
    #[allow(dead_code)]
    file_path: std::path::PathBuf,
    // document: ParsedDocument  — added during implementation
}

impl BatchResolver {
    pub fn new(root: &Path, relative_path: &str) -> Result<Self, MdPathError> {
        let file_path = root.join(relative_path);
        if !file_path.exists() {
            return Err(MdPathError::FileNotFound(relative_path.to_string()));
        }
        Ok(BatchResolver { root: root.to_path_buf(), file_path })
    }

    pub fn resolve(&mut self, uri: &str) -> Result<ResolvedElement, MdPathError> {
        let parsed = MdUri::parse(uri)?;
        resolve(&parsed, &self.root)
    }
}
