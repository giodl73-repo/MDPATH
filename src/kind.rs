//! Figure kind detection — auto-classify ASCII art diagrams.
//!
//! Given the content of a code block, determines which `figure.*` kind it is.
//! This is used when a URI specifies a kind filter (e.g. `figure.flowchart`)
//! or when `proof spec generate` needs to auto-classify for template selection.

/// Detect the kind of a figure from its content lines.
/// Returns the kind string (e.g. "flowchart", "layer-stack") or None if uncategorized.
pub fn detect_figure_kind(content: &[&str]) -> Option<&'static str> {
    // Count structural signals
    let box_count = count_boxes(content);
    let has_connector_arrows = has_connectors(content);
    let has_forward_arrows = has_forward_arrows(content);
    let has_tree_branches = has_tree_branches(content);
    let side_by_side = has_side_by_side_boxes(content);
    let is_bar_chart = is_bar_chart(content);

    if is_bar_chart {
        return Some("bar");
    }

    if box_count == 0 {
        return None;
    } // No boxes — uncategorized

    if box_count == 1 {
        return Some("box");
    }

    // Multiple boxes:
    if side_by_side && !has_connector_arrows {
        return Some("side-by-side");
    }

    if has_tree_branches {
        return Some("tree");
    }

    if has_connector_arrows && has_forward_arrows {
        // Has both │/▼ connectors AND → arrows — likely a flowchart
        return Some("flowchart");
    }

    if has_connector_arrows && !has_forward_arrows {
        // Vertical connectors only — could be layer-stack or flowchart
        if is_layer_stack(content) {
            return Some("layer-stack");
        }
        return Some("flowchart");
    }

    Some("side-by-side")
}

/// Returns true if the content matches the requested kind (or kind is None).
pub fn figure_matches_kind(content: &[&str], kind: Option<&str>) -> bool {
    match kind {
        None => true,
        Some(requested) => match detect_figure_kind(content) {
            None => false,
            Some(detected) => detected == requested,
        },
    }
}

// ─────────────────────────────────────────────────────────
// Detection heuristics
// ─────────────────────────────────────────────────────────

fn count_boxes(lines: &[&str]) -> usize {
    lines
        .iter()
        .filter(|l| {
            let t = l.trim();
            // A border line: starts with + or ┌/└ and has fill chars
            is_box_border(t)
        })
        .count()
        / 2 // Each box has top + bottom border → divide by 2 (approximate)
}

fn is_box_border(trimmed: &str) -> bool {
    let first = trimmed.chars().next();
    matches!(
        first,
        Some('+') | Some('┌') | Some('└') | Some('╔') | Some('╚') | Some('╭') | Some('╰')
    ) && trimmed
        .chars()
        .filter(|c| {
            matches!(
                c,
                '+' | '┌'
                    | '┐'
                    | '└'
                    | '┘'
                    | '├'
                    | '┤'
                    | '┬'
                    | '┴'
                    | '┼'
                    | '╔'
                    | '╗'
                    | '╚'
                    | '╝'
            )
        })
        .count()
        >= 2
}

fn has_connectors(lines: &[&str]) -> bool {
    lines.iter().any(|l| {
        let t = l.trim();
        // Standalone vertical connector lines (│ alone or with ▼/▲/↓/↑)
        matches!(t, "│" | "▼" | "▲" | "↓" | "↑")
            || t.chars().filter(|c| matches!(c, '│')).count() == 1 && t.len() < 5
    })
}

fn has_forward_arrows(lines: &[&str]) -> bool {
    lines.iter().any(|l| {
        l.contains('→')
            || l.contains('►')
            || l.contains("-->")
            || l.contains("──►")
            || l.contains('▶')
    })
}

fn has_tree_branches(lines: &[&str]) -> bool {
    lines.iter().any(|l| {
        let t = l.trim();
        // ├── or └── followed by text (NOT followed only by box chars)
        // Must NOT be a box border (which would end with ┘ or +)
        let starts_branch =
            t.starts_with('├') || t.starts_with('└') || t.contains(" ├") || t.contains(" └");
        let is_box_border = t.ends_with('┘') || t.ends_with('╝') || t.ends_with('+');
        starts_branch && !is_box_border
    })
}

fn has_side_by_side_boxes(lines: &[&str]) -> bool {
    // A border line with multiple box starts on the same line
    lines.iter().any(|l| {
        let corners: usize = l.chars().filter(|c| matches!(c, '┌' | '╔' | '+')).count();
        corners >= 2
    })
}

fn is_layer_stack(lines: &[&str]) -> bool {
    // Layer stacks: equal-width boxes stacked vertically with NO directional arrows between them.
    // Flowcharts have ▼/↓/▲ arrows between boxes — layer stacks do not.
    let has_directional_arrows = lines.iter().any(|l| {
        let t = l.trim();
        matches!(t, "▼" | "↓" | "▲" | "↑")
            || t.contains('▼')
            || t.contains('↓')
            || t.contains('▲')
            || t.contains('↑')
    });
    if has_directional_arrows {
        return false;
    }

    let border_widths: Vec<usize> = lines
        .iter()
        .filter(|l| is_box_border(l.trim()))
        .map(|l| l.chars().count())
        .collect();
    if border_widths.len() < 4 {
        return false;
    }
    let first = border_widths[0];
    let all_same = border_widths.iter().all(|&w| w.abs_diff(first) <= 2);
    all_same
}

fn is_bar_chart(lines: &[&str]) -> bool {
    let bar_lines = lines
        .iter()
        .filter(|l| {
            l.chars()
                .filter(|c| matches!(c, '█' | '▓' | '▒' | '░'))
                .count()
                >= 3
        })
        .count();
    bar_lines >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_bar_chart() {
        let content = &[
            "Option A  ████████████████████ 80%",
            "Option B  █████████████        52%",
            "Option C  ██████               19%",
        ];
        assert_eq!(detect_figure_kind(content), Some("bar"));
    }

    #[test]
    fn detects_flowchart() {
        let content = &[
            "┌──────────┐",
            "│  Step 1  │",
            "└──────────┘",
            "     │",
            "     ▼",
            "┌──────────┐",
            "│  Step 2  │",
            "└──────────┘",
        ];
        assert_eq!(detect_figure_kind(content), Some("flowchart"));
    }

    #[test]
    fn detects_single_box() {
        let content = &[
            "┌──────────────────┐",
            "│  JVM Runtime     │",
            "└──────────────────┘",
        ];
        assert_eq!(detect_figure_kind(content), Some("box"));
    }
}
