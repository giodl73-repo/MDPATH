# mdpath

The `md://` URI scheme — stable, named addressing for elements in markdown documents.

## What it does

`mdpath` lets you address specific figures, tables, charts, text, and headings
within markdown files using a stable URI that survives line number changes:

```
md://computing/01-PACKAGE.md#the-big-picture:figure.flowchart:package-layers
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
md://languages/05-CSHARP.md#type-system-snapshot:table.key-value:0[row=Binding,col=Value]
md://sections/computing-software.md#directories:table:0
```

## Why

Line numbers break. Labels don't. `md://` gives every important element a stable
identity — one that survives edits, can be stored in configuration files, and can
be resolved by any tool that implements the spec.

## URI Grammar

```
md://path[#heading-path][:[type[.kind]:]selector][sub-selector][?query]

Types:   figure | table | chart | text | heading
Kinds:   figure.flowchart | figure.layer-stack | table.key-value | chart.bar | ...
```

**Strings over numbers** — named selectors are always preferred. Numbers are
the fallback when no label exists.

See [`design/SPEC.md`](design/SPEC.md) for the complete specification.

## Quick start

```rust
use mdpath::{parse, resolve};
use std::path::Path;

let uri = parse("md://computing/01-PACKAGE.md#the-big-picture:0").unwrap();
let element = resolve(&uri, Path::new("/my/repo")).unwrap();
println!("{}", element.content);
```

## Batch resolution (N URIs in one file)

For proof's use case — validating many elements in the same file — use
`BatchResolver` to read the file once and resolve all URIs from a cached parse tree:

```rust
use mdpath::resolver::BatchResolver;
use std::path::Path;

let mut batch = BatchResolver::new(Path::new("/repo"), "computing/01-PACKAGE.md").unwrap();
let fig = batch.resolve("md://computing/01-PACKAGE.md#the-big-picture:0").unwrap();
let tbl = batch.resolve("md://computing/01-PACKAGE.md#layer-comparison:table:0").unwrap();
// File read exactly once.
```

## Status

**Scaffolded** — spec locked, core types defined, label detection implemented.
Full resolver (section parsing, element detection, sub-selectors, query params)
in progress.

See [`design/SPEC.md`](design/SPEC.md) for the full specification including
the DaVinci protection system and template invariants.

## License

MIT
