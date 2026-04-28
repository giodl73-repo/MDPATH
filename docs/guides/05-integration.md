# mdpath Integration with proof

proof uses mdpath for every `md://` URI it encounters — in compile directives,
fix plans, DaVinci invariants, and error reporting.

---

## proof:tree with md:// source

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="md://src/data/element-types.md" -->
```taxonomy
math
slide
dashboard
section
```
<!-- /proof:compiled -->

The `source=md://` attribute in `proof:tree`, `proof:element`, and `proof:row`
directives all resolve through mdpath at compile time.

---

## proof:element from a data table

<!-- proof:compiled from="proof:row" uri="md://src/data/error-variants.md" -->
```
FileNotFound           │ The path component does not point to an existing file  
SectionNotFound        │ The heading-path segment does not match any heading in…
SectionAmbiguous       │ Multiple headings normalize to the same segment        
ElementNotFound        │ No element of the requested type exists at the selector
LabelNotFound          │ A named selector does not match any element label      
LabelAmbiguous         │ Multiple elements match the named selector             
SubSelectorInvalid     │ A sub-selector key is not valid for this element type  
InvalidUri             │ URI syntax is malformed                                
ParseError             │ The target file could not be parsed as markdown        
```
<!-- /proof:compiled -->

---

## proof:row iterating a data table

`proof:row` uses mdpath to resolve `source=md://...`, read the table,
and iterate one proof:element row per data row:

<!-- proof:compiled from="proof:row" uri="md://src/data/uri-components.md" -->
```
scheme           │ `md://`                      │ Always `md://` — identifies the URI sch…
path             │ file path relative to root   │ Path to the markdown file from the proo…
heading-path     │ `#segment[/segment]*`        │ Section within the file — normalized he…
type             │ `:[type:]`                   │ Element type: figure, table, chart, tex…
kind             │ `type.kind`                  │ Sub-type of the element                 
selector         │ string or integer            │ Named label (preferred) or numeric inde…
sub-selector     │ `[key=value]`                │ Target within an element: row, col, box…
query            │ `?key=value`                 │ Post-resolution filtering or projection 
```
<!-- /proof:compiled -->

---

## DaVinci invariants (proof pin)

`proof pin` stores a DaVinci invariant for a named figure using a `md://` URI:

```bash
proof pin md://computing/01-PACKAGE.md#the-big-picture:figure:arch-overview
```

The invariant records the figure's structural fingerprint. On every subsequent
`proof check`, mdpath resolves the URI and proof re-checks the fingerprint.
If the figure changes, proof reports a DaVinci violation.

---

## Error reporting

When proof reports a lint error, it includes the `md://` URI of the offending
element so you can navigate directly to it:

```
ERROR ascii_box_width at md://computing/01-PACKAGE.md:figure:arch-overview
  Expected width 60, got 58 on line 3
```

---

## Fix plans

`proof fix` generates fix plans using `md://` URIs as stable references:

```json
{
  "uri": "md://computing/01-PACKAGE.md:figure.box:arch-overview",
  "line": 42,
  "old_string": "─────────────────────────────────────────────────────────",
  "new_string": "────────────────────────────────────────────────────────────"
}
```

The fix applies to the correct element even if the file has changed since the
fix plan was generated — mdpath re-resolves the URI to find the current line.

---

## Proof root and URI resolution

All `md://` URIs are relative to the proof root — the directory containing `proof.toml`.

```
repo/
├── proof.toml        ← proof root
├── computing/
│   └── 01-PACKAGE.md
└── src/
    └── data/
        └── features.md

md://computing/01-PACKAGE.md    ← valid (relative to proof root)
md://src/data/features.md       ← valid
./computing/01-PACKAGE.md       ← invalid (not an md:// URI)
```

Set a custom root at compile time:

```bash
proof compile --root /path/to/repo src/guides/
```

---

## Using mdpath as a library

Add to `Cargo.toml`:

```toml
[dependencies]
mdpath = { path = "../mdpath" }
# or once published:
mdpath = "0.5"
```

### Minimal integration

```rust
use mdpath::{parse, resolve};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let uri_str = "md://src/data/features.md:table:0[row=proof:math]";
    let uri = parse(uri_str)?;
    let element = resolve(&uri, Path::new("."))?;
    println!("{}", element.content);
    Ok(())
}
```

### High-throughput integration

```rust
use mdpath::resolver::BatchResolver;

fn resolve_all(uris: &[&str], root: &Path) -> Vec<String> {
    let mut r = BatchResolver::new(root);
    uris.iter()
        .filter_map(|u| r.resolve(u).ok())
        .map(|e| e.content)
        .collect()
}
```
