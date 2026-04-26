use crate::error::MdPathError;

/// A parsed `md://` URI.
///
/// Grammar: `md://path[#heading-path][:[type[.kind]:]selector][sub-selector][?query]`
#[derive(Debug, Clone, PartialEq)]
pub struct MdUri {
    pub path: String,
    pub heading_path: Vec<String>,
    pub element_type: Option<ElementType>,
    pub kind: Option<String>,
    pub selector: Selector,
    pub sub_selectors: Vec<SubSelector>,
    pub query: Option<QueryParams>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementType { Figure, Table, Chart, Text, Heading, Section }

/// Strings over numbers — named selectors are always preferred.
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
    pub fn parse(s: &str) -> Result<Self, MdPathError> {
        if !s.starts_with("md://") {
            return Err(MdPathError::ParseError(
                format!("URI must start with md://, got: {:?}", s)
            ));
        }
        // Minimal stub — full parser in implementation phase
        let rest = &s[5..];
        let (path, _fragment) = rest.split_once('#').unwrap_or((rest, ""));
        Ok(MdUri {
            path: path.to_string(),
            heading_path: vec![],
            element_type: None,
            kind: None,
            selector: Selector::None,
            sub_selectors: vec![],
            query: None,
        })
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.selector = Selector::Named(label.into());
        self
    }
}
