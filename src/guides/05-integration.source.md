# mdpath Integration with proof

proof uses mdpath for every `md://` URI it encounters — in compile directives,
fix plans, DaVinci invariants, and error reporting.

---

## proof:tree with md:// source

```proof:tree kind=taxonomy source=md://src/data/element-types.md name=type parent=kinds
```

The `source=md://` attribute in `proof:tree`, `proof:element`, and `proof:row`
directives all resolve through mdpath at compile time.

---

## proof:element from a data table

```proof:row source=md://src/data/error-variants.md foreach=row separator=" │ "
proof:element kind=badge field=variant width=22
proof:element kind=label field=cause width=55
```

---

## proof:row iterating a data table

`proof:row` uses mdpath to resolve `source=md://...`, read the table,
and iterate one proof:element row per data row:

```proof:row source=md://src/data/uri-components.md foreach=row separator=" │ "
proof:element kind=badge field=component width=16
proof:element kind=label field=syntax width=28
proof:element kind=label field=description width=40
```

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
