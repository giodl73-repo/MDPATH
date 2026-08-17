# mdpath

**A stable addressing scheme for every figure, table, heading, and diagram in your markdown corpus.**

**Series:** [Standards & Protocols](https://github.com/giodl73-repo/giodl73-repo/blob/main/series/standards-protocols.md).

**Review roles:** This repo uses
[ROLES](https://github.com/giodl73-repo/ROLES), the `.roles` convention for
repository-local review panels.

## MD family

MDPATH is the addressing layer in the MD family:

```text
Markdown → MDPATH → MDCROP → MDLOOM → MDPORT
             address    select     build      transfer
```

| Repo | Responsibility |
|------|----------------|
| **MDPATH** | Stable `md://` addresses for Markdown elements. |
| [MDCROP](https://github.com/giodl73-repo/MDCROP) | Corpus indexing, graph selection, and bounded context. |
| [MDLOOM](https://github.com/giodl73-repo/MDLOOM) | Validation, compilation, rendering, and publication. |
| [MDPORT](https://github.com/giodl73-repo/MDPORT) | Compact portable `mdport.v1` records. |

Line numbers break. File paths break. `md://` URIs don't.

mdpath gives every element in every markdown file a permanent name — based on
what it *is*, not where it sits. Rename a heading, move a figure, grow a file
by 200 lines: every `md://` URI that pointed to it still resolves.

```
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
    └── file ──────────┘ └── section ────┘ └── type.kind ─┘ └── named label ──┘
```

It powers [MDLOOM](https://github.com/giodl73-repo/MDLOOM) — a full Markdown compilation toolchain
running across a 2,700-file corpus of technical guides, presentations, and
dashboards. Every `mdloom:include`, `mdloom:xref`, `mdloom:toc`, and DaVinci pin
resolves through mdpath at compile time.

```rust
use mdpath::{parse, resolve};

let uri = parse("md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler")?;
let element = resolve(&uri, Path::new("/repo"))?;

println!("{}", element.content);        // the figure text
println!("{}", element.label);          // "goroutine-scheduler"
println!("lines {}–{}", element.line_start, element.line_end);
```

---

## What it addresses

Ten element types — anything meaningful in a markdown file:

| Type | What it is | Example URI |
|------|-----------|-------------|
| `figure` | ASCII art diagram in a fenced block | `:figure.flowchart:arch` |
| `table` | Markdown pipe table | `:table:0[row=Goroutine,col=Stack Size]` |
| `chart` | ASCII bar/line chart | `:chart.bar:0[bar=Option A]` |
| `math` | LaTeX or rendered math expression | `:math:pythagorean` |
| `tree` | Tree/hierarchy diagram | `:tree:org` |
| `slide` | Slide block | `:slide:introduction` |
| `dashboard` | Dashboard canvas region | `:dashboard:header` |
| `text` | Prose paragraph or list | `:text:0` |
| `heading` | A heading line | `:heading:the-big-picture` |
| `section` | Heading + all content below it | `#concurrency-model` |

---

## The URI grammar

```
md://path[#heading-path][:[type[.kind]:]selector][sub-selector][?query]
```

Every component is optional after the path. Compose only what you need:

```
md://doc.md                                         whole file
md://doc.md#section                                 one section
md://doc.md#section:figure:arch                     named figure
md://doc.md#section:table:0[row=X,col=Y]            table cell
md://doc.md#section:figure:arch[box=SCHEDULER]      box inside figure
md://doc.md:table:metrics?select=name,value         projected columns
```

**Names over numbers.** Numeric indexes break when elements are reordered.
Named selectors use a three-phase cascade — exact → prefix → substring — so
`:figure:goroutine` matches `goroutine-scheduler` without specifying the full label.

---

## Label matching

```
:figure:goroutine-scheduler    exact match
:figure:goroutine              prefix match (matches goroutine-scheduler)
:figure:scheduler              substring match
:figure:0                      numeric fallback (breaks on reorder)
```

Ambiguous matches (two elements match at the same priority) return
`LabelAmbiguous` instead of guessing.

---

## BatchResolver — one parse, many resolves

```rust
use mdpath::resolver::BatchResolver;

let resolver = BatchResolver::new(root, "languages/10-GO.md")?;

// File is read and parsed exactly once
let fig = resolver.resolve_uri("md://languages/10-GO.md:figure:goroutine-scheduler")?;
let tbl = resolver.resolve_uri("md://languages/10-GO.md:table:0[row=Goroutine]")?;
let sec = resolver.resolve_uri("md://languages/10-GO.md#concurrency-model")?;
```

---

## Classifier extension

Tools that generate fenced blocks can teach mdpath how to classify them:

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
            "my:math"  => Some((ElementType::Math, None)),
            "my:tree"  => Some((ElementType::Tree, None)),
            "my:slide" => Some((ElementType::Slide, None)),
            _          => DefaultClassifier.classify(fence_info, content),
        }
    }
}

let element = resolve_with_classifier(&uri, root, &MyClassifier)?;
```

`DefaultClassifier` handles: `math`/`latex`/`tex` → Math, `mermaid` → Figure,
bar characters → Chart, box-drawing → Figure, `└──` branches → Tree.

`ChainClassifier` composes multiple classifiers in priority order.

---

## Resolved element

```rust
pub struct ResolvedElement {
    pub uri: String,               // canonical URI
    pub file: PathBuf,             // absolute path to source file
    pub line_start: usize,         // 1-based
    pub line_end: usize,           // 1-based, inclusive
    pub content: String,           // element content (fence delimiters stripped)
    pub label: Option<String>,     // detected label
    pub section_heading: Option<String>,
    pub element_type: ElementType, // Figure | Table | Math | Tree | ...
    pub kind: Option<String>,      // "flowchart" | "key-value" | "bar" | ...
}
```

---

## Sub-selectors

Target content within an element:

| Sub-selector | Applies to | Example |
|-------------|-----------|---------|
| `[row=X]` | Tables | Row where first column = X |
| `[col=Y]` | Tables | Column with header Y |
| `[row=X,col=Y]` | Tables | Single cell |
| `[box=Z]` | Figures | Labeled box inside the figure |
| `[bar=X]` | Charts | Bar with label X |

---

## Query parameters

Post-resolution transformations:

```
?select=name,value     return only listed columns
?filter=status=active  filter rows by expression
?count                 return element count instead of content
?top=10                first N results
?skip=5                skip first N
```

---

## Heading normalization

Heading paths normalize automatically — you write what the heading says:

```
Heading:    "The Big Picture"
In URI:     #the-big-picture

Heading:    "Concurrency Model / Goroutines"
In URI:     #concurrency-model/goroutines
```

Rules: lowercase, spaces → dashes, punctuation stripped, consecutive dashes collapsed.

---

## Error types

Every failure mode is a typed variant — match on what went wrong:

```rust
match resolve(&uri, root) {
    Ok(e) => use_element(e),
    Err(MdPathError::FileNotFound { path }) =>
        eprintln!("no file at {:?}", path),
    Err(MdPathError::SectionAmbiguous { segment, count }) =>
        eprintln!("{} headings match {:?} — use parent/child path", count, segment),
    Err(MdPathError::LabelAmbiguous { label, count }) =>
        eprintln!("{} elements match {:?} — use more specific label", count, label),
    Err(e) => eprintln!("{}", e),
}
```

---

## proof integration

[proof](../proof/README.md) uses mdpath for every `md://` reference it encounters —
compile directives, fix plans, DaVinci figure pinning, error reporting. proof
supplies `ProofClassifier` mapping `proof:math` → Math, `proof:tree` → Tree,
`proof:slide` → Slide, `proof:region` → Dashboard.

---

## Guides

The retained [proof surface](docs/proof-surface.md) records one accepted named
resolution and one structured ambiguity failure. Run it with
`cargo test --test proof_surface`.

```bash
bash scripts/build-guides.sh     # compile src/guides/ → docs/guides/
```

| Guide | |
|-------|-|
| [Overview](docs/guides/00-overview.md) | What mdpath is and how it works |
| [URI Syntax](docs/guides/01-uri-syntax.md) | Complete grammar reference |
| [Element Types](docs/guides/02-element-types.md) | All 10 types and detection |
| [Resolution](docs/guides/03-resolution.md) | Single-URI and BatchResolver |
| [Selectors](docs/guides/04-selectors.md) | Sub-selectors and query params |
| [Integration](docs/guides/05-integration.md) | Using mdpath with proof |
| [Errors](docs/guides/06-errors.md) | Error handling and pitfalls |
| [Classifier](docs/guides/07-classifier.md) | Extending type detection |

---

## License

MIT — see [LICENSE](LICENSE).
