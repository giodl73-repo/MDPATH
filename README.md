# mdpath

The Rust library implementing the `md://` URI scheme — stable, named addressing for figures, tables, charts, text, and headings within markdown documents.

---

## What it is

`mdpath` gives every important element in a markdown file a stable address that survives line number changes. Instead of "the box on line 47," you write `md://computing/01-PACKAGE.md#the-big-picture:figure.flowchart:package-layers`. That URI resolves to the same element even after the file grows or shrinks around it.

`mdpath` is a standalone library crate. It has no dependency on `proof` — any tool (editor, CI pipeline, agent) can implement a resolver against the same spec. `proof` is the reference implementation.

---

## Status

Fully implemented. 56+ passing tests covering URI parsing, section navigation, element detection, label matching, sub-selectors, query parameters, and round-trip stability.

---

## URI Grammar

```
md://path[#heading-path][:[type[.kind]:]selector][sub-selector][?query]
```

### Components

| Component | Description | Example |
|-----------|-------------|---------|
| `path` | File path relative to proof root, must end in `.md` | `languages/10-GO.md` |
| `#heading-path` | Slash-separated normalized heading segments | `#concurrency-model/goroutines` |
| `type` | Element type | `figure`, `table`, `chart`, `text`, `heading` |
| `kind` | Type qualifier | `figure.flowchart`, `table.key-value`, `chart.bar` |
| `selector` | Which element within the type collection | `:goroutine-scheduler` (named) or `:0` (index) |
| `[sub-selector]` | Row, column, or box within the element | `[row=Binding,col=Value]`, `[box=PREPROCESSOR]` |
| `?query` | OData-style filter and projection | `?select=Axis,Value&filter=Axis eq Binding` |

**Strings over numbers** — named selectors are always preferred. Numeric indices are the fallback when no label exists.

### Examples

```
md://computing/01-PACKAGE.md
md://computing/01-PACKAGE.md#the-big-picture
md://computing/01-PACKAGE.md#the-big-picture:0
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
md://languages/05-CSHARP.md#type-system-snapshot:table.key-value:0
md://languages/05-CSHARP.md#type-system-snapshot:table.key-value:0[row=Binding,col=Value]
md://sections/computing-software.md#directories:table:0?select=Directory,Description
```

Heading slugs are normalized: spaces become hyphens, ASCII-lowercased. `#The Big Picture` and `#the-big-picture` address the same heading.

---

## Quick Start

```rust
use mdpath::{parse, resolve};
use std::path::Path;

// Parse a URI
let uri = parse("md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler")
    .unwrap();

// Resolve against a repo root
let element = resolve(&uri, Path::new("/path/to/repo")).unwrap();

println!("label:   {:?}", element.label);
println!("lines:   {}–{}", element.line_start, element.line_end);
println!("content:\n{}", element.content);
```

---

## BatchResolver

For resolving multiple URIs in the same file — `proof`'s primary use case — use `BatchResolver` to read the file once and resolve all URIs from the cached parse tree:

```rust
use mdpath::resolver::BatchResolver;
use std::path::Path;

let root = Path::new("/path/to/repo");
let mut batch = BatchResolver::new(root, "languages/10-GO.md").unwrap();

// File is read and parsed exactly once.
let fig = batch.resolve_uri("md://languages/10-GO.md#concurrency-model:0").unwrap();
let tbl = batch.resolve_uri("md://languages/10-GO.md#type-system-snapshot:table:0").unwrap();

println!("figure at lines {}–{}", fig.line_start, fig.line_end);
println!("table at lines {}–{}", tbl.line_start, tbl.line_end);
```

---

## Resolved Element Fields

```rust
pub struct ResolvedElement {
    pub uri: String,                    // canonical URI string
    pub file: std::path::PathBuf,       // absolute path to source file
    pub line_start: usize,              // 1-based, inclusive
    pub line_end: usize,                // 1-based, inclusive
    pub content: String,                // element content (fence delimiters stripped for figures)
    pub label: Option<String>,          // detected label text, if any
    pub section_heading: Option<String>,// heading text of the enclosing section
    pub element_type: ElementType,      // Figure | Table | Chart | Text | Heading | Section
    pub kind: Option<String>,           // detected or declared kind (e.g. "flowchart", "key-value")
}
```

For figures, `content` is the text inside the code fence — fence delimiter lines are never included.

---

## The `proof:figure` Marker Format

Figure files are standalone `.md` files whose code blocks are marked with HTML comments immediately before each fence. The comment is hidden in rendered output but gives the following code block a stable named identity that `mdpath` can address:

```markdown
<!-- proof:figure id="goroutine-scheduler" kind="figure.flowchart" -->
```
GOROUTINE SCHEDULER — M:N multiplexing
┌─────────────────────────────────────┐
│  OS Thread (M)                      │
│  ┌──────┐ ┌──────┐ ┌──────┐        │
│  │  G   │ │  G   │ │  G   │  ...   │
│  └──────┘ └──────┘ └──────┘        │
└─────────────────────────────────────┘
```
```

The `id` attribute maps directly to a named selector: `md://figures/goroutine-scheduler.md#:goroutine-scheduler` addresses this figure regardless of which line it appears on.

---

## Label Matching

When a named selector is used, `mdpath` resolves it through a three-phase hierarchy:

1. **Exact match** — label equals selector (case-insensitive, normalized)
2. **Starts-with** — label begins with selector
3. **Substring** — label contains selector

Ambiguity (more than one match at any phase) returns an error rather than silently picking the wrong element.

---

## Integration with proof

`mdpath` is used by `proof` for:

- **`proof resolve`** — resolve a URI and print element content and metadata
- **`proof pin`** — register a figure's URI for DaVinci invariant tracking
- **`proof compile`** — resolve every `proof:include` and `proof:layout` directive in source documents before embedding

`proof` depends on `mdpath` via Cargo path dependency (`{ path = "../mdpath" }`). When published, this will become a crates.io version dependency.

---

## Design

The full URI specification, addressing rules, label detection algorithm, sub-selector semantics, and failure mode catalog are in `proof/design/`:

- `proof/design/FIG-SPEC.md` — complete `md://` specification
- `proof/design/md-path/INVARIANTS.md` — resolver properties
- `proof/design/md-path/PITFALLS.md` — failure mode catalog

---

## GitHub

[https://github.com/giodl73-repo/MDPATH](https://github.com/giodl73-repo/MDPATH)

---

## License

MIT — see [LICENSE](LICENSE).
