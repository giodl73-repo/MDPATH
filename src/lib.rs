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

pub use error::MdPathError;
pub use uri::MdUri;
pub use resolver::ResolvedElement;
pub use uri::ElementType;

/// Parse an `md://` URI from a string.
pub fn parse(uri: &str) -> Result<MdUri, MdPathError> {
    MdUri::parse(uri)
}

/// Resolve an `md://` URI against a root directory.
///
/// `root` is the directory containing `proof.toml`. All paths in the URI
/// are resolved relative to this root.
pub fn resolve(uri: &MdUri, root: &std::path::Path) -> Result<ResolvedElement, MdPathError> {
    resolver::resolve(uri, root)
}
