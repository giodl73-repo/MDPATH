# md:// Error Handling

mdpath uses a typed error enum — every failure returns a specific `MdPathError`
variant that tells you exactly what went wrong. This matters because the fix for
`FileNotFound` ("check the path") is different from `SectionAmbiguous` ("use a
parent/child path") or `LabelAmbiguous` ("use a more specific label").

The error types follow a natural progression through the resolution phases.
URI errors happen before any file I/O. File errors happen when reading from disk.
Navigation errors happen when walking the heading path. Selection errors happen
when trying to find and match elements.

---

## Error variants

Each variant carries enough context to construct a meaningful error message.
The `resolution` column describes the standard fix — most errors have a
mechanical solution once you understand why they occurred.

<!-- proof:compiled from="proof:row" uri="md://src/data/error-variants.md" -->
```
FileNotFound           │ The path component does not point to an exis… │ Check that the path is relative to the proof root, …
SectionNotFound        │ The heading-path segment does not match any … │ Verify the heading text is normalized (lowercase, s…
SectionAmbiguous       │ Multiple headings normalize to the same segm… │ Use a longer heading path (parent/child) to disambi…
ElementNotFound        │ No element of the requested type exists at t… │ Check the element type, kind, and whether the file …
LabelNotFound          │ A named selector does not match any element … │ Verify the label — try substring matching or use a …
LabelAmbiguous         │ Multiple elements match the named selector    │ Use a more specific label or switch to numeric index
SubSelectorInvalid     │ A sub-selector key is not valid for this ele… │ Check supported sub-selectors for the element type …
InvalidUri             │ URI syntax is malformed                       │ Validate the URI against the grammar                
ParseError             │ The target file could not be parsed as markd… │ Check for encoding issues or file corruption        
```
<!-- /proof:compiled -->

---

## Error taxonomy

Grouping errors by phase helps you understand at what point in resolution
something went wrong — and whether the fix is in your URI or in your document.

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="" -->
```taxonomy
├── ```proof:tree kind=taxonomy
MdPathError variants
├── URI errors (parse-time)
├── InvalidUri: malformed URI syntax
├── File errors (I/O)
├── FileNotFound: path does not exist on disk
├── ParseError: file cannot be parsed as markdown
├── Navigation errors (heading-path)
├── SectionNotFound: heading segment does not match
├── SectionAmbiguous: multiple headings match the segment
├── Selection errors (type + selector)
├── ElementNotFound: no element matches type+selector
├── LabelNotFound: named selector has no matching label
├── LabelAmbiguous: multiple elements match the named selector
├── Sub-selection errors
└── SubSelectorInvalid: key not valid for element type
```
<!-- /proof:compiled -->

---

## Error handling patterns

Pattern-match on specific variants for actionable error messages. The wildcard
`Err(e)` catch-all is fine for logging, but matching specific variants lets you
provide targeted suggestions — "use a parent/child path" is far more useful than
"resolution failed".

### Match on variants

```rust
use mdpath::{parse, resolve, error::MdPathError};

match resolve(&uri, root) {
    Ok(element) => println!("{}", element.content),
    Err(MdPathError::FileNotFound { path }) => {
        eprintln!("File not found: {}", path.display());
    }
    Err(MdPathError::SectionAmbiguous { segment, count }) => {
        eprintln!("{} headings match '{}' — use a parent/child path", count, segment);
    }
    Err(MdPathError::LabelAmbiguous { label, count }) => {
        eprintln!("{} elements match '{}' — use a more specific label", count, label);
    }
    Err(e) => eprintln!("Resolution failed: {}", e),
}
```

### Fallback to numeric index

```rust
fn resolve_with_fallback(uri_str: &str, root: &Path) -> Result<ResolvedElement, MdPathError> {
    match resolve(&parse(uri_str)?, root) {
        Ok(e) => Ok(e),
        Err(MdPathError::LabelNotFound { .. }) => {
            // Try numeric index as fallback
            let fallback = uri_str.replace(":figure:name", ":figure:0");
            resolve(&parse(&fallback)?, root)
        }
        Err(e) => Err(e),
    }
}
```

---

## Common mistakes and fixes

Most errors have a mechanical fix once you know what went wrong. The table above
covers the causes; this section shows the before/after for the most common ones.

### SectionNotFound

Wrong:
```
md://doc.md#The Big Picture:figure:0
```

Fixed (normalized):
```
md://doc.md#the-big-picture:figure:0
```

### SectionAmbiguous

Wrong:
```
md://doc.md#introduction:figure:0
```
(Two sections named "Introduction")

Fixed (use parent path):
```
md://doc.md#chapter-1/introduction:figure:0
```

### LabelNotFound

Check that the figure's first line is text-only (no box-drawing characters):

````
```
my-figure-label      ← label line must be plain text
┌────────────────┐
│  box content   │
└────────────────┘
```
````

### FileNotFound

Ensure the path is relative to the proof root, not the current working directory:

```
# Wrong — relative to current directory
proof compile ./computing/01-PACKAGE.md

# Right — proof root is set by proof.toml location
proof compile computing/01-PACKAGE.md --root /path/to/repo
```

---

## PITFALLS reference

From `design/PITFALLS.md`:

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="" -->
```taxonomy
Known pitfalls
├── URI construction
├── Forgetting to normalize heading text (spaces not dashes)
├── Using absolute paths instead of root-relative paths
├── Including the .md extension twice
├── Label matching
├── Short labels that are substrings of many other labels cause LabelAmbiguous
├── Labels with special characters may not normalize as expected
├── Numeric indexes
├── Using :figure:0 in a file where figures are frequently reordered
├── Off-by-one errors (indexes are zero-based)
├── BatchResolver
├── Assuming BatchResolver state persists across processes — it is in-process only
└── Not invalidating BatchResolver cache after file writes
```
<!-- /proof:compiled -->
