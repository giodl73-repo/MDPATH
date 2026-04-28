# md:// Resolution

Resolution converts a parsed `MdUri` into a `ResolvedElement` — the actual
content from the file, with metadata about where it came from.

---

## Single-URI resolution

```rust
use mdpath::{parse, resolve};
use std::path::Path;

let uri = parse("md://languages/10-GO.md#concurrency-model:figure:goroutine-scheduler")?;
let root = Path::new("/path/to/repo");
let element = resolve(&uri, root)?;

// ResolvedElement fields:
println!("{}", element.content);         // the figure text
println!("{}", element.label);           // "goroutine-scheduler"
println!("{}", element.line_start);      // 42
println!("{}", element.line_end);        // 58
println!("{:?}", element.element_type);  // ElementType::Figure
println!("{}", element.section_heading); // "Concurrency Model"
```

### With a custom classifier

When resolving generated/compiled output files, pass your tool's `Classifier`
so content is identified correctly:

```rust
use mdpath::{parse, resolve_with_classifier};
use mdpath::classify::{Classifier, DefaultClassifier};
use mdpath::uri::ElementType;

struct MyClassifier;
impl Classifier for MyClassifier {
    fn classify(&self, fence_info: &str, content: &[&str])
        -> Option<(ElementType, Option<String>)>
    {
        match fence_info {
            "my:math" => Some((ElementType::Math, None)),
            "my:tree" => Some((ElementType::Tree, None)),
            _ => DefaultClassifier.classify(fence_info, content),
        }
    }
}

let uri = parse("md://docs/output.md:math:0")?;
let element = resolve_with_classifier(&uri, root, &MyClassifier)?;
```

See [Classifier guide](07-classifier.md) for the full extension pattern.

---

## The three resolution phases

Resolution always runs the same three phases in sequence. If any phase fails,
it returns an error immediately — there's no partial resolution. Understanding
the phases helps you debug errors: `SectionNotFound` means Phase 2 failed
(the heading path didn't match), `ElementNotFound` means Phase 3 failed (the
type + selector combination found nothing).

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="" -->
```taxonomy
Resolution phases
├── Phase 1: Parse URI
├── Validate URI syntax
├── Split into: path, heading_path, type, kind, selector, sub_selectors, query
├── Phase 2: Navigate document
├── Read file at path (from proof root)
├── Parse markdown into ParsedDocument
├── Walk heading_path → locate section
├── Collect elements of requested type within section
├── Phase 3: Select element
├── Apply selector: named label (exact → prefix → substring) or numeric index
├── Apply sub-selectors: [row=X], [col=Y], [box=Z]
├── Apply query params: ?select, ?filter, ?count, ?top, ?skip
└── Return ResolvedElement
```
<!-- /proof:compiled -->

---

## Label matching

Named selectors don't need to be exact. mdpath tries three matching strategies
in priority order, stopping at the first unambiguous match. This means you can
use a short prefix like `:figure:goroutine` to match a figure labeled
`goroutine-scheduler` — as long as only one figure starts with that prefix.
Ambiguous matches (two elements match at the same priority level) return
`LabelAmbiguous` rather than silently picking one.

Named selectors use a priority cascade:

<!-- proof:compiled from="proof:row" uri="md://src/data/label-matching.md" -->
```
1    │ Exact match        │ goroutine-scheduler        │ goroutine-scheduler  │ yes             
2    │ Starts-with prefix │ goroutine-scheduler        │ goroutine            │ yes             
3    │ Substring          │ goroutine-scheduler        │ scheduler            │ yes             
4    │ Numeric index      │ (any label)                │ 0                    │ first element   
5    │ Ambiguous — error  │ goroutine-scheduler, goro… │ goroutine            │ LabelAmbiguous  
```
<!-- /proof:compiled -->

When multiple elements match at the same priority level → `LabelAmbiguous` error.

---

## BatchResolver — one parse, many resolves

`BatchResolver` is the performance-optimized path for resolving multiple URIs
from the same file. The default `resolve()` function reads and parses the file
on every call. When you have N URIs from the same file, that's N file reads and
N parses — wasteful for large corpora. `BatchResolver` reads and parses once,
then resolves all subsequent URIs from the cached in-memory document.

Use `BatchResolver` whenever you're resolving more than one URI from the same
file in the same process — proof's compile pipeline uses it internally for
exactly this reason.

When you need to resolve multiple URIs from the same file, use `BatchResolver`
to avoid re-reading and re-parsing the file for each URI:

```rust
use mdpath::resolver::BatchResolver;
use std::path::Path;

let root = Path::new("/path/to/repo");
let resolver = BatchResolver::new(root, "languages/10-GO.md")?;

// All three resolve from one file read + one parse
let fig = resolver.resolve_uri("md://languages/10-GO.md:figure:goroutine-scheduler")?;
let tbl = resolver.resolve_uri("md://languages/10-GO.md:table:0[row=Goroutine]")?;
let sec = resolver.resolve_uri("md://languages/10-GO.md#concurrency-model")?;
```

With a custom classifier:

```rust
let math = resolver.resolve_uri_with_classifier(
    "md://docs/output.md:math:0", &MyClassifier)?;
```

BatchResolver caches the `ParsedDocument` per file path. Subsequent URIs
hitting the same file skip the I/O and parse step.

---

## ResolvedElement fields

`ResolvedElement` carries everything you need about the resolved content: the
text itself, its location in the file, and the metadata that tells you what
kind of element it is. The `content` field is the primary output — the actual
text inside the fenced block or table, stripped of fence delimiters. Use
`line_start` and `line_end` for displaying source locations in error messages.

| Field | Type | Description |
|-------|------|-------------|
| `uri` | `String` | The original URI string |
| `file` | `PathBuf` | Absolute path to the resolved file |
| `line_start` | `usize` | First line of the element (1-indexed) |
| `line_end` | `usize` | Last line of the element (inclusive) |
| `content` | `String` | Full text content of the element |
| `label` | `String` | Detected label (empty if none) |
| `section_heading` | `String` | Heading of the enclosing section |
| `element_type` | `ElementType` | Figure, Table, Chart, Text, Heading |
| `kind` | `Option<String>` | Sub-type (flowchart, key-value, etc.) |

---

## Heading normalization

Heading paths normalize heading text for stable matching:

| Rule | Input | Normalized |
|------|-------|-----------|
| Lowercase | `The Big Picture` | `the-big-picture` |
| Spaces → dashes | `Layer 1 Runtime` | `layer-1-runtime` |
| Strip punctuation | `What's New?` | `whats-new` |
| Collapse dashes | `Multi--word` | `multi-word` |

The same normalization applies to both the URI and the document headings,
so the match is always case-insensitive and punctuation-tolerant.

---

## Stability guarantees

<!-- proof:compiled from="proof:tree kind=org" uri="" -->
```org
What breaks URIs
├── File deleted or renamed → FileNotFound
├── Fix: update the path component
├── Heading renamed → SectionNotFound
├── Fix: update heading-path or use parent/child path
├── Figure label changed → LabelNotFound
├── Fix: update selector or use proof pin to protect it
├── Element moved to different section → ElementNotFound
├── Fix: update heading-path or remove it
├── Element reordered (numeric URIs only) → Wrong element returned
└── Fix: use named selector instead
```
<!-- /proof:compiled -->
