use thiserror::Error;

/// All errors produced by the mdpath resolver.
#[derive(Debug, Error)]
pub enum MdPathError {
    #[error("invalid md:// URI: {0}")]
    ParseError(String),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("section not found: #{0}")]
    SectionNotFound(String),

    #[error("section ambiguous: #{0} matches multiple headings — use a longer heading path")]
    SectionAmbiguous(String),

    #[error("element not found at index {0} (section has {1} elements of this type)")]
    ElementNotFound(usize, usize),

    #[error("label not found: no element in section matches {0:?}")]
    LabelNotFound(String),

    #[error("label ambiguous: {0:?} matches {1} elements — use a more specific selector")]
    LabelAmbiguous(String, usize),

    #[error("sub-selector key not found: {0}")]
    SubKeyNotFound(String),

    #[error("invalid sub-selector [{1}] on type '{0}' — not supported for this element type")]
    InvalidSubSelector(String, String),

    #[error("invalid query parameter '{1}' on type '{0}' — only supported on table and chart")]
    InvalidQueryOnType(String, String),

    #[error("invariant violated on {0}: {1}")]
    InvariantViolated(String, String),

    #[error("template not found: {0}")]
    TemplateNotFound(String),

    #[error("numeric URI is stale: element now has label {0:?} — update to named form")]
    NumericUriStale(String),

    #[error("IO error reading {0}: {1}")]
    Io(String, #[source] std::io::Error),
}
