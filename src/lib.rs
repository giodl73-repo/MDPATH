//! # mdpath
//!
//! The `md://` URI scheme — stable, named addressing for elements in
//! markdown documents (figures, tables, charts, text, headings).
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use mdpath::{parse, resolve};
//! use std::path::Path;
//!
//! let uri = parse("md://computing/01-PACKAGE.md#the-big-picture:figure.flowchart:package-layers").unwrap();
//! let element = resolve(&uri, Path::new("/path/to/repo")).unwrap();
//! println!("Found at lines {}–{}", element.line_start, element.line_end);
//! ```
//!
//! ## URI grammar
//!
//! ```text
//! md://path[#heading-path][:[type[.kind]:]selector][sub-selector][?query]
//! ```
//!
//! See `design/SPEC.md` for the full specification.

pub mod classify;
pub mod document;
pub mod error;
pub mod heading;
pub mod kind;
pub mod label;
pub mod parser;
pub mod resolver;
pub mod selector;
pub mod subselect;
pub mod uri;

pub use classify::{Classifier, DefaultClassifier, ChainClassifier};
pub use error::MdPathError;
pub use uri::{MdUri, ElementType};
pub use resolver::ResolvedElement;

/// Parse an `md://` URI from a string.
pub fn parse(uri: &str) -> Result<MdUri, MdPathError> {
    MdUri::parse(uri)
}

/// Resolve an `md://` URI against a root directory using the default classifier.
pub fn resolve(uri: &MdUri, root: &std::path::Path) -> Result<ResolvedElement, MdPathError> {
    resolver::resolve(uri, root)
}

/// Resolve an `md://` URI with a custom classifier.
///
/// Use this when your tool generates fenced blocks that mdpath should recognize:
///
/// ```rust,no_run
/// use mdpath::{parse, resolve_with_classifier, classify::{Classifier, DefaultClassifier}, uri::ElementType};
/// use std::path::Path;
///
/// struct MyClassifier;
/// impl Classifier for MyClassifier {
///     fn classify(&self, fence_info: &str, content: &[&str]) -> Option<(ElementType, Option<String>)> {
///         match fence_info {
///             "my-math" => Some((ElementType::Math, None)),
///             _ => DefaultClassifier.classify(fence_info, content),
///         }
///     }
/// }
///
/// let uri = parse("md://doc.md:math:0").unwrap();
/// let element = resolve_with_classifier(&uri, Path::new("."), &MyClassifier).unwrap();
/// ```
pub fn resolve_with_classifier(
    uri: &MdUri,
    root: &std::path::Path,
    classifier: &dyn Classifier,
) -> Result<ResolvedElement, MdPathError> {
    resolver::resolve_with_classifier(uri, root, classifier)
}
