/// Markdown document parser — builds ParsedDocument from file content.
/// Reads a file exactly once; the result is cached in BatchResolver.
use crate::document::*;
use crate::heading::normalize_heading;
use crate::label::{detect_inline_label, detect_preceding_label};

/// Parse markdown content into a document model.
/// All line numbers in the result are 1-based.
pub fn parse_document(content: &str) -> ParsedDocument {
    let lines: Vec<&str> = content.lines().collect();
    let n = lines.len();
    let mut headings: Vec<ParsedHeading> = Vec::new();
    let mut elements: Vec<ParsedElement> = Vec::new();

    // State machine
    let mut i = 0;
    let mut in_fence = false;
    let mut fence_char = '`';
    let mut fence_len = 0usize;
    let mut fence_info = String::new();
    let mut fence_start = 0usize;
    let mut fence_content: Vec<String> = Vec::new();
    let mut fence_preceding_label: Option<String> = None;

    // Track table state
    let mut table_start = 0usize;
    let mut table_rows: Vec<&str> = Vec::new();
    let mut in_table = false;

    // Track paragraph state
    let mut para_start = 0usize;
    let mut para_lines: Vec<String> = Vec::new();
    let mut in_para = false;

    let flush_para = |elements: &mut Vec<ParsedElement>,
                      para_lines: &mut Vec<String>,
                      para_start: &mut usize,
                      in_para: &mut bool,
                      heading_idx: usize,
                      end_line: usize| {
        if *in_para && !para_lines.is_empty() {
            elements.push(ParsedElement::Paragraph(Paragraph {
                lines: std::mem::take(para_lines),
                line_start: *para_start,
                line_end: end_line,
                heading_idx,
            }));
            *in_para = false;
        }
    };

    let flush_table = |elements: &mut Vec<ParsedElement>,
                       table_rows: &mut Vec<&str>,
                       table_start: usize,
                       heading_idx: usize,
                       end_line: usize| {
        if table_rows.len() >= 2 {
            if let Some(parsed) = try_parse_table(table_rows, table_start, end_line, heading_idx) {
                elements.push(ParsedElement::Table(parsed));
            }
        }
    };

    while i < n {
        let line = lines[i];
        let line_no = i + 1; // 1-based
        let trimmed = line.trim_start();

        if !in_fence {
            // Check for fence opening
            if let Some((fc, fl, fi)) = detect_fence_open(trimmed) {
                let preceding: Vec<&str> = para_lines.iter().rev().map(|s| s.as_str()).collect();
                fence_preceding_label = detect_preceding_label(&preceding);

                // Flush pending paragraph and table
                let h_idx = headings.len().saturating_sub(1);
                flush_para(
                    &mut elements,
                    &mut para_lines,
                    &mut para_start,
                    &mut in_para,
                    h_idx,
                    line_no.saturating_sub(1),
                );
                if in_table {
                    flush_table(
                        &mut elements,
                        &mut table_rows,
                        table_start,
                        h_idx,
                        line_no.saturating_sub(1),
                    );
                    in_table = false;
                }

                in_fence = true;
                fence_char = fc;
                fence_len = fl;
                fence_info = fi;
                fence_start = line_no;
                fence_content.clear();
                i += 1;
                continue;
            }

            // Check for heading
            if trimmed.starts_with('#') {
                let h_idx = headings.len().saturating_sub(1);
                flush_para(
                    &mut elements,
                    &mut para_lines,
                    &mut para_start,
                    &mut in_para,
                    h_idx,
                    line_no.saturating_sub(1),
                );
                if in_table {
                    flush_table(
                        &mut elements,
                        &mut table_rows,
                        table_start,
                        h_idx,
                        line_no.saturating_sub(1),
                    );
                    in_table = false;
                    table_rows.clear();
                }

                let level = trimmed.chars().take_while(|&c| c == '#').count();
                let text = trimmed[level..].trim().to_string();
                let anchor = normalize_heading(&text);
                headings.push(ParsedHeading {
                    level,
                    text,
                    anchor,
                    line: line_no,
                });
                i += 1;
                continue;
            }

            // Check for table row
            let is_table_row = trimmed.starts_with('|')
                && trimmed.ends_with('|')
                && trimmed.chars().filter(|&c| c == '|').count() >= 2;

            if is_table_row {
                let h_idx = headings.len().saturating_sub(1);
                if in_para {
                    flush_para(
                        &mut elements,
                        &mut para_lines,
                        &mut para_start,
                        &mut in_para,
                        h_idx,
                        line_no.saturating_sub(1),
                    );
                }
                if !in_table {
                    table_start = line_no;
                    in_table = true;
                }
                table_rows.push(line);
                i += 1;
                continue;
            }

            // End of table
            if in_table && !is_table_row {
                let h_idx = headings.len().saturating_sub(1);
                if table_rows.len() >= 2 {
                    let rows: Vec<&str> = table_rows.drain(..).collect();
                    if let Some(parsed) =
                        try_parse_table(&rows, table_start, line_no.saturating_sub(1), h_idx)
                    {
                        elements.push(ParsedElement::Table(parsed));
                    }
                } else {
                    table_rows.clear();
                }
                in_table = false;
            }

            // Blank line ends paragraph
            if trimmed.is_empty() {
                let h_idx = headings.len().saturating_sub(1);
                flush_para(
                    &mut elements,
                    &mut para_lines,
                    &mut para_start,
                    &mut in_para,
                    h_idx,
                    line_no.saturating_sub(1),
                );
                i += 1;
                continue;
            }

            // Prose line
            if !in_para {
                para_start = line_no;
                in_para = true;
            }
            para_lines.push(line.to_string());
            i += 1;
            continue;
        }

        // Inside fence — check for closing fence
        if detect_fence_close(trimmed, fence_char, fence_len).is_some() {
            let h_idx = headings.len().saturating_sub(1);

            // Detect label from inline content (Rule 1)
            let content_refs: Vec<&str> = fence_content.iter().map(|s| s.as_str()).collect();
            let preceding_label = fence_preceding_label.take();
            let label = detect_inline_label(&fence_info, &content_refs).or(preceding_label);

            elements.push(ParsedElement::CodeBlock(CodeBlock {
                fence_info: std::mem::take(&mut fence_info),
                content: std::mem::take(&mut fence_content),
                line_start: fence_start,
                line_end: line_no,
                label,
                heading_idx: h_idx,
            }));

            in_fence = false;
            i += 1;
            continue;
        }

        // Collect fence content
        fence_content.push(line.to_string());
        i += 1;
    }

    // Flush trailing state
    let h_idx = headings.len().saturating_sub(1);

    // Flush unclosed fence at end-of-file as a code block
    if in_fence && !fence_content.is_empty() {
        let content_refs: Vec<&str> = fence_content.iter().map(|s| s.as_str()).collect();
        let label = detect_inline_label(&fence_info, &content_refs);
        elements.push(ParsedElement::CodeBlock(CodeBlock {
            fence_info,
            content: fence_content,
            line_start: fence_start,
            line_end: n,
            label,
            heading_idx: h_idx,
        }));
    }

    if in_para && !para_lines.is_empty() {
        elements.push(ParsedElement::Paragraph(Paragraph {
            lines: para_lines,
            line_start: para_start,
            line_end: n,
            heading_idx: h_idx,
        }));
    }
    if in_table && !table_rows.is_empty() {
        if let Some(parsed) = try_parse_table(&table_rows, table_start, n, h_idx) {
            elements.push(ParsedElement::Table(parsed));
        }
        table_rows.clear();
    }

    // Sentinel heading at end (simplifies range calculation)
    headings.push(ParsedHeading {
        level: 0,
        text: String::new(),
        anchor: String::new(),
        line: n + 1,
    });

    ParsedDocument { headings, elements }
}

/// Detect an opening fence. Returns (fence_char, fence_len, info_string) or None.
fn detect_fence_open(trimmed: &str) -> Option<(char, usize, String)> {
    let first = trimmed.chars().next()?;
    if !matches!(first, '`' | '~') {
        return None;
    }
    let len = trimmed.chars().take_while(|&c| c == first).count();
    if len < 3 {
        return None;
    }
    let info = trimmed[len..].trim().to_string();
    Some((first, len, info))
}

/// Detect a closing fence matching the given char and minimum length.
fn detect_fence_close(trimmed: &str, fence_char: char, fence_len: usize) -> Option<()> {
    let first = trimmed.chars().next()?;
    if first != fence_char {
        return None;
    }
    let len = trimmed.chars().take_while(|&c| c == fence_char).count();
    if len >= fence_len && trimmed[len..].trim().is_empty() {
        Some(())
    } else {
        None
    }
}

/// Try to parse a slice of table lines into a ParsedTable.
fn try_parse_table(
    rows: &[&str],
    line_start: usize,
    line_end: usize,
    heading_idx: usize,
) -> Option<ParsedTable> {
    if rows.len() < 2 {
        return None;
    }
    let headers = parse_table_row(rows[0]);
    if headers.is_empty() {
        return None;
    }

    // Row 1 must be a separator
    let sep = parse_table_row(rows[1]);
    let is_sep = sep.iter().all(|cell| {
        let c = cell.trim().trim_start_matches(':').trim_end_matches(':');
        c.chars().all(|ch| ch == '-') && !c.is_empty()
    });
    if !is_sep {
        return None;
    }

    let body_rows: Vec<Vec<String>> = rows[2..].iter().map(|r| parse_table_row(r)).collect();

    Some(ParsedTable {
        headers,
        separator: sep,
        rows: body_rows,
        line_start,
        line_end,
        heading_idx,
    })
}

/// Split a GFM table row into cells using escaped-pipe-aware parsing.
fn parse_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return vec![];
    }
    let inner =
        &trimmed[1..trimmed
            .len()
            .saturating_sub(if trimmed.ends_with('|') { 1 } else { 0 })];

    let mut cells = Vec::new();
    let mut current = String::new();
    let mut in_code = false;
    let mut chars = inner.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                current.push('\\');
                current.push('|');
                chars.next();
            }
            '`' => {
                in_code = !in_code;
                current.push(c);
            }
            '|' if !in_code => {
                cells.push(current.clone());
                current.clear();
            }
            other => {
                current.push(other);
            }
        }
    }
    cells.push(current);
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"# Document

## The Big Picture

Some intro text.

```
LAYER DIAGRAM
┌──────────────┐
│  Top Layer   │
└──────────────┘
```

### Sub-section

| Axis | Value |
|------|-------|
| Binding | Late |
| Typing | Static |

## Second Section

More prose here.
"#;

    #[test]
    fn headings_parsed() {
        let doc = parse_document(SAMPLE);
        // Exclude sentinel
        let real: Vec<_> = doc.headings.iter().filter(|h| h.level > 0).collect();
        assert_eq!(real.len(), 4); // H1 document, H2 big-picture, H3 sub-section, H2 second-section
        assert_eq!(real[0].anchor, "document");
        assert_eq!(real[1].anchor, "the-big-picture");
        assert_eq!(real[2].anchor, "sub-section");
        assert_eq!(real[3].anchor, "second-section");
    }

    #[test]
    fn code_block_with_label() {
        let doc = parse_document(SAMPLE);
        let blocks: Vec<_> = doc
            .elements
            .iter()
            .filter_map(|e| match e {
                ParsedElement::CodeBlock(b) => Some(b),
                _ => None,
            })
            .collect();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].label.as_deref(), Some("LAYER DIAGRAM"));
    }

    #[test]
    fn table_parsed() {
        let doc = parse_document(SAMPLE);
        let tables: Vec<_> = doc
            .elements
            .iter()
            .filter_map(|e| match e {
                ParsedElement::Table(t) => Some(t),
                _ => None,
            })
            .collect();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].headers[0].trim(), "Axis");
        assert_eq!(tables[0].rows.len(), 2);
    }

    #[test]
    fn elements_in_section() {
        let doc = parse_document(SAMPLE);
        let big_pic_idx = doc
            .headings
            .iter()
            .position(|h| h.anchor == "the-big-picture")
            .unwrap();
        let elems = doc.elements_in_section(big_pic_idx);
        // Should include the code block and sub-section's table (both are in "the-big-picture" scope)
        assert!(!elems.is_empty());
    }

    #[test]
    fn heading_path_resolution() {
        let doc = parse_document(SAMPLE);
        // Find "sub-section" as child of "the-big-picture"
        let parent_idx = doc
            .headings
            .iter()
            .position(|h| h.anchor == "the-big-picture")
            .unwrap();
        let matches = doc.find_heading("sub-section", Some(parent_idx));
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn no_false_positive_on_adjacent_section() {
        let doc = parse_document(SAMPLE);
        // "sub-section" should NOT be found as child of "second-section"
        let parent_idx = doc
            .headings
            .iter()
            .position(|h| h.anchor == "second-section")
            .unwrap();
        let matches = doc.find_heading("sub-section", Some(parent_idx));
        assert_eq!(matches.len(), 0);
    }
}
