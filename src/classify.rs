/// The Classifier trait — maps a fenced code block to an ElementType and optional kind string.
///
/// mdpath's `DefaultClassifier` covers common markdown patterns. Tools that generate
/// their own fenced block types (like proof) can provide a `Classifier` implementation
/// to teach mdpath how to recognize and address their output.
///
/// # Extension pattern
///
/// ```rust,ignore
/// use mdpath::classify::{Classifier, DefaultClassifier};
/// use mdpath::uri::ElementType;
///
/// struct MyClassifier;
///
/// impl Classifier for MyClassifier {
///     fn classify(&self, fence_info: &str, content: &[&str])
///         -> Option<(ElementType, Option<String>)>
///     {
///         match fence_info.trim() {
///             "my-math"   => Some((ElementType::Math, None)),
///             "my-tree"   => Some((ElementType::Tree, None)),
///             // Delegate everything else to the default
///             _ => DefaultClassifier.classify(fence_info, content),
///         }
///     }
/// }
/// ```

use crate::kind::detect_figure_kind;
use crate::uri::ElementType;

pub trait Classifier: Send + Sync {
    /// Given a fence's info string and its content lines, return the `ElementType`
    /// and an optional kind sub-string, or `None` to indicate "I don't know this type".
    ///
    /// Return `None` when you want to delegate to the next classifier in the chain.
    /// Returning `Some(...)` overrides all further classification.
    fn classify(&self, fence_info: &str, content: &[&str]) -> Option<(ElementType, Option<String>)>;
}

/// The default classifier — handles common markdown patterns and generic fence_info values.
///
/// **Priority order (first match wins):**
/// 1. Well-known fence_info strings (e.g. `"math"`, `"mermaid"`)
/// 2. Visual content heuristics (box-drawing chars → Figure, bar chars → Chart)
/// 3. Fall through to `None` (caller treats as Text)
pub struct DefaultClassifier;

impl Classifier for DefaultClassifier {
    fn classify(&self, fence_info: &str, content: &[&str]) -> Option<(ElementType, Option<String>)> {
        // 1. Well-known fence_info patterns (tool-agnostic)
        let info = fence_info.trim();
        match info {
            "math" | "latex" | "tex" => {
                return Some((ElementType::Math, None));
            }
            "mermaid" => {
                return Some((ElementType::Figure, Some("flowchart".to_string())));
            }
            "plantuml" => {
                return Some((ElementType::Figure, Some("sequence".to_string())));
            }
            "dot" | "graphviz" => {
                return Some((ElementType::Figure, Some("graph".to_string())));
            }
            _ => {}
        }

        // 2. Visual content heuristics for untagged or generic fences
        if content.is_empty() {
            return None;
        }

        // Bar chart detection (takes priority over box figures)
        let bar_lines = content.iter().filter(|l| {
            l.chars().filter(|c| matches!(c, '█' | '▓' | '▒' | '░')).count() >= 3
        }).count();
        if bar_lines >= 2 {
            return Some((ElementType::Chart, Some("bar".to_string())));
        }

        // Tree branch detection — check BEFORE box detection because └── shares prefix
        // with └──────┘ box borders. Tree branches don't end with ┘/╝/+.
        let has_tree = content.iter().any(|l| {
            let t = l.trim();
            (t.starts_with('├') || t.starts_with('└') || t.contains(" ├") || t.contains(" └"))
            && !t.ends_with('┘') && !t.ends_with('╝') && !t.ends_with('+')
        });

        if has_tree {
            return Some((ElementType::Tree, Some("ascii".to_string())));
        }

        // Box-drawing figure detection
        let has_box = content.iter().any(|l| {
            let t = l.trim();
            let first = t.chars().next();
            matches!(first, Some('+') | Some('┌') | Some('╔') | Some('╚') | Some('╭') | Some('╰'))
            || (matches!(first, Some('└')) && t.ends_with('┘'))  // box border only, not tree branch
        });

        if has_box {
            let kind = detect_figure_kind(content).map(|k| k.to_string());
            return Some((ElementType::Figure, kind));
        }

        None // Unknown — caller treats as Text
    }
}

/// A classifier chain: tries each classifier in order, returning the first non-None result.
pub struct ChainClassifier {
    classifiers: Vec<Box<dyn Classifier>>,
}

impl ChainClassifier {
    pub fn new(classifiers: Vec<Box<dyn Classifier>>) -> Self {
        Self { classifiers }
    }
}

impl Classifier for ChainClassifier {
    fn classify(&self, fence_info: &str, content: &[&str]) -> Option<(ElementType, Option<String>)> {
        for c in &self.classifiers {
            if let Some(result) = c.classify(fence_info, content) {
                return Some(result);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_classifies_math_fence() {
        let c = DefaultClassifier;
        let (t, k) = c.classify("math", &[]).unwrap();
        assert_eq!(t, ElementType::Math);
        assert_eq!(k, None);
    }

    #[test]
    fn default_classifies_latex_fence() {
        let c = DefaultClassifier;
        let (t, _) = c.classify("latex", &[]).unwrap();
        assert_eq!(t, ElementType::Math);
    }

    #[test]
    fn default_classifies_mermaid_as_figure() {
        let c = DefaultClassifier;
        let (t, k) = c.classify("mermaid", &[]).unwrap();
        assert_eq!(t, ElementType::Figure);
        assert_eq!(k.as_deref(), Some("flowchart"));
    }

    #[test]
    fn default_detects_bar_chart() {
        let c = DefaultClassifier;
        let content = &[
            "Option A  ████████████████ 80%",
            "Option B  █████████        52%",
        ];
        let (t, k) = c.classify("", content).unwrap();
        assert_eq!(t, ElementType::Chart);
        assert_eq!(k.as_deref(), Some("bar"));
    }

    #[test]
    fn default_detects_box_figure() {
        let c = DefaultClassifier;
        let content = &[
            "┌──────────┐",
            "│  Module  │",
            "└──────────┘",
        ];
        let (t, _) = c.classify("", content).unwrap();
        assert_eq!(t, ElementType::Figure);
    }

    #[test]
    fn default_detects_tree() {
        let c = DefaultClassifier;
        let content = &[
            "root",
            "├── child1",
            "└── child2",
        ];
        let (t, _) = c.classify("", content).unwrap();
        assert_eq!(t, ElementType::Tree);
    }

    #[test]
    fn empty_content_returns_none() {
        let c = DefaultClassifier;
        assert!(c.classify("", &[]).is_none());
    }

    #[test]
    fn chain_returns_first_match() {
        struct AlwaysMath;
        impl Classifier for AlwaysMath {
            fn classify(&self, _: &str, _: &[&str]) -> Option<(ElementType, Option<String>)> {
                Some((ElementType::Math, None))
            }
        }

        let chain = ChainClassifier::new(vec![
            Box::new(AlwaysMath),
            Box::new(DefaultClassifier),
        ]);

        let (t, _) = chain.classify("anything", &[]).unwrap();
        assert_eq!(t, ElementType::Math);
    }

    #[test]
    fn extension_classifier_overrides_default() {
        struct ProofLike;
        impl Classifier for ProofLike {
            fn classify(&self, fence_info: &str, content: &[&str]) -> Option<(ElementType, Option<String>)> {
                match fence_info.trim() {
                    "proof:math" => Some((ElementType::Math, None)),
                    "proof:tree" => Some((ElementType::Tree, None)),
                    "proof:slide" => Some((ElementType::Slide, None)),
                    "proof:region" => Some((ElementType::Dashboard, None)),
                    _ => DefaultClassifier.classify(fence_info, content),
                }
            }
        }

        let c = ProofLike;
        assert_eq!(c.classify("proof:math", &[]).unwrap().0, ElementType::Math);
        assert_eq!(c.classify("proof:tree", &[]).unwrap().0, ElementType::Tree);
        assert_eq!(c.classify("proof:slide", &[]).unwrap().0, ElementType::Slide);
        assert_eq!(c.classify("proof:region", &[]).unwrap().0, ElementType::Dashboard);
        // Falls back to default for unknown
        assert!(c.classify("unknown", &[]).is_none());
    }
}
