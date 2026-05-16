/// Sub-selector application — [row=X], [col=Y], [box=Z] lookups
/// applied after the primary element is resolved.
use crate::{
    document::{CodeBlock, ParsedTable},
    error::MdPathError,
    label::label_matches,
    uri::{SelectorValue, SubSelector},
};

/// Apply sub-selectors to a resolved table, returning a narrowed result.
/// Returns (content_lines, row_idx, col_idx) where applicable.
pub struct TableSelection {
    pub content: String,
    pub row_idx: Option<usize>,
    pub col_idx: Option<usize>,
}

pub fn apply_table_subselectors(
    table: &ParsedTable,
    sub_selectors: &[SubSelector],
) -> Result<TableSelection, MdPathError> {
    let mut row_idx: Option<usize> = None;
    let mut col_idx: Option<usize> = None;

    for sub in sub_selectors {
        match sub.key.as_str() {
            "row" => {
                row_idx = Some(find_row(table, &sub.value)?);
            }
            "col" => {
                col_idx = Some(find_col(table, &sub.value)?);
            }
            other => {
                return Err(MdPathError::InvalidSubSelector(
                    "table".to_string(),
                    format!("[{}=...]", other),
                ));
            }
        }
    }

    let content = build_table_content(table, row_idx, col_idx);
    Ok(TableSelection {
        content,
        row_idx,
        col_idx,
    })
}

fn find_row(table: &ParsedTable, value: &SelectorValue) -> Result<usize, MdPathError> {
    match value {
        SelectorValue::Index(n) => {
            if *n < table.rows.len() {
                Ok(*n)
            } else {
                Err(MdPathError::SubKeyNotFound(format!(
                    "row index {} out of range",
                    n
                )))
            }
        }
        SelectorValue::Named(name) => {
            // Match against the first column value of each body row
            // Hierarchy: exact → starts-with → substring
            let mut starts_idx = None;
            let mut sub_idx = None;
            for (i, row) in table.rows.iter().enumerate() {
                let key = row.first().map(|s| s.trim()).unwrap_or("");
                let (matches, is_exact) = label_matches(name, key);
                if matches {
                    if is_exact {
                        return Ok(i);
                    }
                    let norm_name = crate::label::normalize_label(name);
                    let norm_key = crate::label::normalize_label(key);
                    if starts_idx.is_none() && norm_key.starts_with(&norm_name) {
                        starts_idx = Some(i);
                    } else if sub_idx.is_none() {
                        sub_idx = Some(i);
                    } else {
                        return Err(MdPathError::LabelAmbiguous(name.clone(), 2));
                    }
                }
            }
            starts_idx
                .or(sub_idx)
                .ok_or_else(|| MdPathError::SubKeyNotFound(format!("row {:?} not found", name)))
        }
    }
}

fn find_col(table: &ParsedTable, value: &SelectorValue) -> Result<usize, MdPathError> {
    match value {
        SelectorValue::Index(n) => {
            if *n < table.headers.len() {
                Ok(*n)
            } else {
                Err(MdPathError::SubKeyNotFound(format!(
                    "column index {} out of range",
                    n
                )))
            }
        }
        SelectorValue::Named(name) => {
            // Match against header cell values
            let norm_name = crate::label::normalize_label(name);
            let mut found = None;
            for (i, header) in table.headers.iter().enumerate() {
                let norm_h = crate::label::normalize_label(header.trim());
                if norm_h == norm_name {
                    return Ok(i);
                }
                if norm_h.starts_with(&norm_name) && found.is_none() {
                    found = Some(i);
                }
            }
            found.ok_or_else(|| MdPathError::SubKeyNotFound(format!("column {:?} not found", name)))
        }
    }
}

fn build_table_content(
    table: &ParsedTable,
    row_idx: Option<usize>,
    col_idx: Option<usize>,
) -> String {
    match (row_idx, col_idx) {
        (Some(r), Some(c)) => {
            // Single cell
            table
                .rows
                .get(r)
                .and_then(|row| row.get(c))
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        }
        (Some(r), None) => {
            // Whole row: headers + this body row
            let header = table
                .headers
                .iter()
                .map(|h| h.trim())
                .collect::<Vec<_>>()
                .join(" | ");
            let row = table
                .rows
                .get(r)
                .map(|r| r.iter().map(|c| c.trim()).collect::<Vec<_>>().join(" | "))
                .unwrap_or_default();
            format!("{}\n{}", header, row)
        }
        (None, Some(c)) => {
            // Whole column
            let mut lines = vec![table
                .headers
                .get(c)
                .map(|h| h.trim().to_string())
                .unwrap_or_default()];
            for row in &table.rows {
                lines.push(row.get(c).map(|v| v.trim().to_string()).unwrap_or_default());
            }
            lines.join("\n")
        }
        (None, None) => {
            // Full table (no sub-selector)
            let header = table
                .headers
                .iter()
                .map(|h| h.trim())
                .collect::<Vec<_>>()
                .join(" | ");
            let rows: Vec<String> = table
                .rows
                .iter()
                .map(|r| r.iter().map(|c| c.trim()).collect::<Vec<_>>().join(" | "))
                .collect();
            std::iter::once(header)
                .chain(rows)
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

/// Apply [box=Label] sub-selector to a figure's content lines.
/// Returns the content of the matched box.
pub fn apply_figure_subselectors(
    block: &CodeBlock,
    sub_selectors: &[SubSelector],
) -> Result<String, MdPathError> {
    for sub in sub_selectors {
        match sub.key.as_str() {
            "box" => {
                let content = find_box_in_figure(&block.content, &sub.value)?;
                return Ok(content);
            }
            "row" => {
                // Raw line access by index
                if let SelectorValue::Index(n) = &sub.value {
                    return block.content.get(*n).map(|s| s.clone()).ok_or_else(|| {
                        MdPathError::SubKeyNotFound(format!(
                            "line {} out of range (figure has {} lines)",
                            n,
                            block.content.len()
                        ))
                    });
                }
            }
            other => {
                return Err(MdPathError::InvalidSubSelector(
                    "figure".to_string(),
                    format!("[{}=...]", other),
                ));
            }
        }
    }
    // No sub-selectors or none matched
    Ok(block.content.join("\n"))
}

fn find_box_in_figure(lines: &[String], value: &SelectorValue) -> Result<String, MdPathError> {
    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

    match value {
        SelectorValue::Index(n) => {
            // Find the nth box in the figure
            let boxes = collect_boxes(&line_refs);
            boxes
                .into_iter()
                .nth(*n)
                .ok_or_else(|| MdPathError::SubKeyNotFound(format!("box index {} not found", n)))
        }
        SelectorValue::Named(name) => {
            // Find box whose first interior text line matches name
            let boxes = collect_boxes(&line_refs);
            let norm_name = crate::label::normalize_label(name);
            let mut found = None;
            let mut count = 0;
            for box_content in &boxes {
                let box_label = extract_box_label(box_content);
                if let Some(label) = box_label {
                    let norm_label = crate::label::normalize_label(&label);
                    if norm_label.contains(&norm_name) {
                        if found.is_some() {
                            count += 1;
                        }
                        found = Some(box_content.clone());
                    }
                }
            }
            if count > 0 {
                return Err(MdPathError::LabelAmbiguous(name.clone(), count + 1));
            }
            found.ok_or_else(|| MdPathError::SubKeyNotFound(format!("box {:?} not found", name)))
        }
    }
}

/// Collect all box contents from a figure (each box = lines from top border to bottom border).
fn collect_boxes(lines: &[&str]) -> Vec<String> {
    let mut boxes = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if is_top_border(lines[i].trim()) {
            let start = i;
            i += 1;
            while i < lines.len() && !is_any_border(lines[i].trim()) {
                i += 1;
            }
            if i < lines.len() {
                boxes.push(lines[start..=i].join("\n"));
            }
        }
        i += 1;
    }
    boxes
}

fn is_top_border(trimmed: &str) -> bool {
    matches!(
        trimmed.chars().next(),
        Some('┌') | Some('╔') | Some('╭') | Some('+')
    ) && trimmed
        .chars()
        .filter(|c| matches!(c, '┌' | '┐' | '+' | '╔' | '╗'))
        .count()
        >= 1
}

fn is_any_border(trimmed: &str) -> bool {
    matches!(
        trimmed.chars().next(),
        Some('┌')
            | Some('┐')
            | Some('└')
            | Some('┘')
            | Some('╔')
            | Some('╗')
            | Some('╚')
            | Some('╝')
            | Some('+')
    )
}

fn extract_box_label(box_content: &str) -> Option<String> {
    // First interior line (skip the top border)
    for line in box_content.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Strip leading │ or | and trailing │ or |
        let inner = trimmed
            .trim_start_matches(['│', '|', ' '])
            .trim_end_matches(['│', '|', ' ']);
        if !inner.is_empty() {
            return Some(
                inner
                    .split_whitespace()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
    }
    None
}

/// Apply OData-style query params to table rows.
pub fn apply_table_query(
    table: &ParsedTable,
    filter: Option<&str>,
    select_cols: Option<&[String]>,
    top: Option<usize>,
    skip: Option<usize>,
) -> String {
    let col_indices: Option<Vec<usize>> = select_cols.map(|cols| {
        cols.iter()
            .filter_map(|c| table.headers.iter().position(|h| h.trim() == c.as_str()))
            .collect()
    });

    let mut rows: Vec<&Vec<String>> = table.rows.iter().collect();

    // Apply filter (simple equality: "ColName eq Value")
    if let Some(filter_str) = filter {
        rows.retain(|row| row_matches_filter(row, &table.headers, filter_str));
    }

    // Apply skip/top
    let skip_n = skip.unwrap_or(0);
    let take_n = top.unwrap_or(usize::MAX);
    let rows: Vec<_> = rows.into_iter().skip(skip_n).take(take_n).collect();

    // Apply column selection
    let header = match &col_indices {
        None => table
            .headers
            .iter()
            .map(|h| h.trim())
            .collect::<Vec<_>>()
            .join(" | "),
        Some(idxs) => idxs
            .iter()
            .filter_map(|&i| table.headers.get(i))
            .map(|h| h.trim())
            .collect::<Vec<_>>()
            .join(" | "),
    };

    let body: Vec<String> = rows
        .iter()
        .map(|row| match &col_indices {
            None => row.iter().map(|c| c.trim()).collect::<Vec<_>>().join(" | "),
            Some(idxs) => idxs
                .iter()
                .filter_map(|&i| row.get(i))
                .map(|c| c.trim())
                .collect::<Vec<_>>()
                .join(" | "),
        })
        .collect();

    std::iter::once(header)
        .chain(body)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Simple filter: supports `ColName eq Value` and `ColName contains Text`.
fn row_matches_filter(row: &[String], headers: &[String], filter: &str) -> bool {
    // Parse: "ColName eq Value" or "ColName contains Value"
    let parts: Vec<&str> = filter.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return true;
    } // unparseable → pass all

    let col_name = parts[0];
    let op = parts[1];
    let value = parts[2].trim_matches('"').trim_matches('\'');

    let col_idx = headers.iter().position(|h| h.trim() == col_name);
    if let Some(idx) = col_idx {
        let cell = row.get(idx).map(|s| s.trim()).unwrap_or("");
        match op {
            "eq" => cell == value,
            "ne" => cell != value,
            "contains" => cell.contains(value),
            "startswith" => cell.starts_with(value),
            _ => true,
        }
    } else {
        true // column not found → pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::ParsedTable;

    fn make_table() -> ParsedTable {
        ParsedTable {
            headers: vec![" Axis ".into(), " Value ".into()],
            separator: vec!["---".into(), "---".into()],
            rows: vec![
                vec![" Binding ".into(), " Late ".into()],
                vec![" Typing ".into(), " Static ".into()],
                vec![" Strength ".into(), " Strong ".into()],
            ],
            line_start: 1,
            line_end: 5,
            heading_idx: 0,
        }
    }

    #[test]
    fn row_by_name() {
        let table = make_table();
        let sub = SubSelector {
            key: "row".into(),
            value: SelectorValue::Named("Binding".into()),
        };
        let result = apply_table_subselectors(&table, &[sub]).unwrap();
        assert!(result.content.contains("Binding"));
        assert!(result.content.contains("Late"));
    }

    #[test]
    fn col_by_name() {
        let table = make_table();
        let sub = SubSelector {
            key: "col".into(),
            value: SelectorValue::Named("Value".into()),
        };
        let result = apply_table_subselectors(&table, &[sub]).unwrap();
        assert!(result.content.contains("Late"));
        assert!(result.content.contains("Static"));
    }

    #[test]
    fn cell_row_and_col() {
        let table = make_table();
        let subs = vec![
            SubSelector {
                key: "row".into(),
                value: SelectorValue::Named("Typing".into()),
            },
            SubSelector {
                key: "col".into(),
                value: SelectorValue::Named("Value".into()),
            },
        ];
        let result = apply_table_subselectors(&table, &subs).unwrap();
        assert_eq!(result.content.trim(), "Static");
    }

    #[test]
    fn query_filter() {
        let table = make_table();
        let filtered = apply_table_query(&table, Some("Axis eq Binding"), None, None, None);
        assert!(filtered.contains("Binding"));
        assert!(!filtered.contains("Typing"));
    }

    #[test]
    fn query_select_cols() {
        let table = make_table();
        let cols = vec!["Value".to_string()];
        let selected = apply_table_query(&table, None, Some(&cols), None, None);
        assert!(selected.contains("Late"));
        assert!(!selected.contains("Axis"));
    }
}
