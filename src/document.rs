/// The in-memory document model built by parsing a markdown file once.
/// Used by BatchResolver for efficient N-URI resolution.

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub headings: Vec<ParsedHeading>,
    pub elements: Vec<ParsedElement>,
}

#[derive(Debug, Clone)]
pub struct ParsedHeading {
    pub level: usize,       // 1 = H1, 2 = H2, etc.
    pub text: String,       // raw heading text (without # marks)
    pub anchor: String,     // GitHub-normalized anchor
    pub line: usize,        // 1-based line number of the heading
}

/// An addressable element within a section.
#[derive(Debug, Clone)]
pub enum ParsedElement {
    CodeBlock(CodeBlock),
    Table(ParsedTable),
    Paragraph(Paragraph),
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub fence_info: String,     // text after ``` e.g. "python", "" for unlabeled
    pub content: Vec<String>,   // lines inside the fence (excluding fence lines)
    pub line_start: usize,      // 1-based line of opening fence
    pub line_end: usize,        // 1-based line of closing fence
    pub label: Option<String>,  // detected label (rule 1 or rule 2)
    pub heading_idx: usize,     // index of nearest containing heading
}

#[derive(Debug, Clone)]
pub struct ParsedTable {
    pub headers: Vec<String>,           // header row cells (trimmed)
    pub separator: Vec<String>,         // separator row cells
    pub rows: Vec<Vec<String>>,         // body row cells
    pub line_start: usize,
    pub line_end: usize,
    pub heading_idx: usize,
}

#[derive(Debug, Clone)]
pub struct Paragraph {
    pub lines: Vec<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub heading_idx: usize,
}

impl ParsedDocument {
    /// Find the index of the heading matching a normalized anchor, optionally
    /// within the content of a parent heading (for subsection paths).
    pub fn find_heading(&self, anchor: &str, parent_idx: Option<usize>) -> Vec<usize> {
        self.headings.iter().enumerate()
            .filter(|(i, h)| {
                h.anchor == anchor && match parent_idx {
                    None => true,
                    Some(parent) => {
                        let parent_level = self.headings[parent].level;
                        let parent_line = self.headings[parent].line;
                        // Heading must come after the parent AND be at a deeper level
                        h.line > parent_line && h.level > parent_level &&
                        // Heading must be before the next heading at the same level as parent
                        {
                            let next_sibling = self.headings[parent+1..].iter()
                                .find(|s| s.level <= parent_level)
                                .map(|s| s.line)
                                .unwrap_or(usize::MAX);
                            h.line < next_sibling
                        }
                    }
                }
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Collect all elements belonging to a section identified by heading_idx.
    /// A section spans from its heading to the next heading at the same or higher level.
    pub fn elements_in_section(&self, heading_idx: usize) -> Vec<&ParsedElement> {
        let heading = &self.headings[heading_idx];
        let section_end_line = self.headings[heading_idx+1..].iter()
            .find(|h| h.level <= heading.level)
            .map(|h| h.line)
            .unwrap_or(usize::MAX);

        self.elements.iter().filter(|e| {
            let (line_start, h_idx) = match e {
                ParsedElement::CodeBlock(b) => (b.line_start, b.heading_idx),
                ParsedElement::Table(t) => (t.line_start, t.heading_idx),
                ParsedElement::Paragraph(p) => (p.line_start, p.heading_idx),
            };
            // Element must be within this section's line range
            // and attributed to this heading or a child heading
            line_start > heading.line && line_start < section_end_line &&
            self.headings[h_idx].line >= heading.line
        }).collect()
    }
}
