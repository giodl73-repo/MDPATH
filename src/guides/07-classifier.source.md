# md:// Classifier Extension

The `Classifier` trait is the extension point for teaching mdpath how to
classify fenced code blocks your tool generates. Without it, mdpath falls
back to visual heuristics — which may misidentify your output.

---

## Why a classifier?

mdpath identifies element types from content structure. A `proof:math` block
renders fraction bars (`──`) that look like box figures. A compiled tree has
`└──` branches that look like figures too. Without context, mdpath gets it wrong.

The `Classifier` trait gives your tool a voice in the resolution process:

```proof:tree kind=taxonomy
root: Classification priority (first match wins)
- 1. Your Classifier (if provided to resolve_with_classifier)
  - fence_info exact/prefix match → return ElementType + kind
  - Return None to fall through
- 2. DefaultClassifier (always runs as fallback)
  - Well-known fence_info: math/latex → Math, mermaid → Figure.flowchart
  - Visual heuristics: bar chars → Chart.bar, box-drawing → Figure, └── → Tree
  - Return None for unrecognized content
- 3. Unclassified fallback
  - Unrecognized code blocks match Figure, Chart, and Text for backward compat
```

---

## The Classifier trait

```rust
pub trait Classifier: Send + Sync {
    /// Map a fenced code block to (ElementType, Option<kind_string>).
    /// Return None to defer to the next classifier in the chain.
    fn classify(&self, fence_info: &str, content: &[&str])
        -> Option<(ElementType, Option<String>)>;
}
```

- `fence_info` — the text after the opening ```` ``` ```` (e.g. `"proof:math"`, `"python"`, `""`)
- `content` — the lines inside the fence (excluding fence open/close lines)
- Return `Some(...)` to claim this block; `None` to defer

---

## DefaultClassifier

Handles common markdown patterns out of the box:

| fence_info | ElementType | kind |
|------------|-------------|------|
| `math`, `latex`, `tex` | Math | — |
| `mermaid` | Figure | flowchart |
| `plantuml` | Figure | sequence |
| `dot`, `graphviz` | Figure | graph |
| *(empty)* + bar chars (█▓) | Chart | bar |
| *(empty)* + box-drawing (┌└) | Figure | flowchart/box/etc. |
| *(empty)* + tree branches (├──) | Tree | ascii |

---

## Writing a custom classifier

```rust
use mdpath::classify::{Classifier, DefaultClassifier};
use mdpath::uri::ElementType;

struct MyToolClassifier;

impl Classifier for MyToolClassifier {
    fn classify(&self, fence_info: &str, content: &[&str])
        -> Option<(ElementType, Option<String>)>
    {
        match fence_info.trim() {
            // Claim your tool's fence_info strings
            "mytool:math"  => Some((ElementType::Math, None)),
            "mytool:tree"  => Some((ElementType::Tree, None)),
            "mytool:chart" => Some((ElementType::Chart, Some("bar".to_string()))),
            // Fall through to DefaultClassifier for everything else
            _ => DefaultClassifier.classify(fence_info, content),
        }
    }
}
```

---

## proof's classifier

proof ships `ProofClassifier` that maps all `proof:*` fence_info strings:

```proof:tree kind=taxonomy
root: ProofClassifier mappings
- proof:math → Math
- proof:tree (and variants) → Tree
- proof:slide → Slide
- proof:region → Dashboard
- proof:symbol → Figure (kind=symbol)
- proof:shape → Figure (kind=shape)
- org, dirtree, taxonomy, dependency, outline → Tree (compiled tree output)
- everything else → delegated to DefaultClassifier
```

This means compiled proof output is correctly addressed in generated docs:

```
md://docs/guides/05-trees.md:tree:0       ← compiled tree diagram
md://docs/guides/01-math.md:math:0        ← compiled math block
md://docs/guides/04-slides.md:slide:0     ← compiled slide
```

---

## Using resolve_with_classifier

```rust
use mdpath::{parse, resolve_with_classifier};
use mdpath::classify::{Classifier, DefaultClassifier};
use mdpath::uri::ElementType;
use std::path::Path;

// Build your classifier
struct MyClassifier;
impl Classifier for MyClassifier {
    fn classify(&self, fence_info: &str, content: &[&str])
        -> Option<(ElementType, Option<String>)>
    {
        match fence_info {
            "my-math" => Some((ElementType::Math, None)),
            _ => DefaultClassifier.classify(fence_info, content),
        }
    }
}

// Resolve with your classifier
let uri = parse("md://docs/output.md:math:0")?;
let element = resolve_with_classifier(&uri, Path::new("."), &MyClassifier)?;
println!("{}", element.content);
```

---

## ChainClassifier — compose multiple classifiers

Use `ChainClassifier` when you have multiple independent classifiers to combine:

```rust
use mdpath::classify::{ChainClassifier, DefaultClassifier};

let chain = ChainClassifier::new(vec![
    Box::new(MyPluginClassifier),   // checked first
    Box::new(AnotherClassifier),    // checked second
    Box::new(DefaultClassifier),    // fallback
]);

let element = resolve_with_classifier(&uri, root, &chain)?;
```

---

## BatchResolver with classifier

```rust
use mdpath::resolver::BatchResolver;

let resolver = BatchResolver::new(root, "docs/output.md")?;

// All three URIs share one file parse, all use ProofClassifier
let math = resolver.resolve_uri_with_classifier(
    "md://docs/output.md:math:0", &ProofClassifier)?;
let tree = resolver.resolve_uri_with_classifier(
    "md://docs/output.md:tree:0", &ProofClassifier)?;
let slide = resolver.resolve_uri_with_classifier(
    "md://docs/output.md:slide:0", &ProofClassifier)?;
```

---

## Element types added for generated content

```proof:row source=md://src/data/element-types.md foreach=row separator=" │ "
proof:element kind=badge field=type width=12
proof:element kind=label field=detected-by width=42
proof:element kind=label field=example-uri width=46
```

---

## Invariant: Classifier determines type, URI grammar provides the address

The `Classifier` determines what *type* a block is. The URI grammar provides the *address*. They are independent: you can write `:math:0` in a URI whether or not a classifier is provided — the classifier just ensures the block is correctly identified as `Math` during resolution.
