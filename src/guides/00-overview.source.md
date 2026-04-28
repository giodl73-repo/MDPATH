# mdpath — The md:// URI Scheme

mdpath implements a stable, named addressing system for individual elements
within markdown documents. Instead of fragile line numbers, `md://` URIs
identify content by what it *is* — its label, type, and position in the
document's heading hierarchy.

```
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
    └── path ──────────┘ └── section ────┘ └── type.kind ┘ └── label ─────────┘
```

The scheme is open — any tool (editor, CI, AI agent) can implement a resolver.
`proof` is the reference implementation.

---

## Why named addressing?

Line numbers break when files grow. `md://` URIs survive edits because they
anchor to content, not position.

```proof:tree kind=org
root: Stability hierarchy
- Named label: most stable (content must change)
  - figure.flowchart:goroutine-scheduler
  - table:0[row=Goroutine,col=Stack Size]
- Named prefix: stable enough for most edits
  - figure:goroutine (matches goroutine-scheduler)
- Numeric index: breaks on reorder
  - figure:2
- Line number: breaks on any edit above
  - line 347
```

---

## The resolution pipeline

```proof:tree kind=taxonomy source=md://src/data/uri-components.md name=component parent=required
```

---

## What can be addressed

```proof:tree kind=taxonomy source=md://src/data/element-types.md name=type parent=kinds
```

---

## Architecture

```proof:tree kind=org
root: mdpath modules
- parse(): Entry point — tokenize URI string into MdUri struct
  - uri.rs: MdUri struct, ElementType, Selector, SubSelector, QueryParams
- resolve(): Navigate document to find and return ResolvedElement
  - parser.rs: ParsedDocument, ParsedElement
  - heading.rs: Heading normalization, path traversal
  - label.rs: Exact, prefix, substring label matching
  - selector.rs: Named vs numeric selector parsing
  - kind.rs: Element type auto-detection
  - subselect.rs: row/col/box sub-selector application
- BatchResolver: Cache-friendly bulk resolution from one parsed document
  - resolver.rs: BatchResolver struct, per-file cache
```

---

## Quick start

```rust
use mdpath::{parse, resolve};
use std::path::Path;

// 1. Parse the URI
let uri = parse("md://languages/10-GO.md#concurrency-model:figure:goroutine-scheduler")?;

// 2. Resolve against a root directory
let root = Path::new("/path/to/repo");
let element = resolve(&uri, root)?;

println!("{}", element.content);   // the figure text
println!("{}", element.label);     // "goroutine-scheduler"
println!("{}", element.line_start); // line number in source file
```

---

## See also

- [URI Syntax](01-uri-syntax.md) — complete grammar reference
- [Element Types](02-element-types.md) — types, kinds, detection (including Math/Tree/Slide/Dashboard)
- [Resolution](03-resolution.md) — how the resolver works
- [Selectors](04-selectors.md) — sub-selectors and query params
- [Integration](05-integration.md) — using mdpath with proof
- [Errors](06-errors.md) — error handling and pitfalls
- [Classifier](07-classifier.md) — extending type detection for generated content
