# md:// URI Syntax

The `md://` URI grammar is designed to be composed incrementally. You start
with a file path and add components to narrow your target: first a section,
then a type, then a specific element by name, then content within that element.
You only include the components you need. Most real-world URIs use three or
four components — the full grammar exists for edge cases.

## Grammar

```
md://path[#heading-path][:[type[.kind]:]selector][sub-selector][?query]
```

Every component after `path` is optional. Components compose left to right,
narrowing from file → section → element → sub-element.

---

## Components reference

These are the six building blocks of a URI. Each one refines the target further.
Think of them as a sequence of filters applied in order: start with the whole
file, then narrow to a section, then to an element type, then to a specific
element, then to content within it.

<!-- proof:compiled from="proof:row" uri="md://src/data/uri-components.md" -->
```
scheme           │ yes        │ `md://`                      │ Always `md://` — identifies the URI sch…
path             │ yes        │ file path relative to root   │ Path to the markdown file from the proo…
heading-path     │ no         │ `#segment[/segment]*`        │ Section within the file — normalized he…
type             │ no         │ `:[type:]`                   │ Element type: figure, table, chart, tex…
kind             │ no         │ `type.kind`                  │ Sub-type of the element                 
selector         │ no         │ string or integer            │ Named label (preferred) or numeric inde…
sub-selector     │ no         │ `[key=value]`                │ Target within an element: row, col, box…
query            │ no         │ `?key=value`                 │ Post-resolution filtering or projection 
```
<!-- /proof:compiled -->

---

## Addressing levels

The same document element can be addressed at different granularities. Use the
coarsest level that uniquely identifies what you need. Heading-only URIs are
stable as long as heading text doesn't change. Element URIs are stable as long
as the element's label doesn't change. Cell URIs are stable as long as both
the row label and column header don't change.

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="" -->
```taxonomy
Addressing levels
├── File: md://computing/01-PACKAGE.md
├── Section: md://computing/01-PACKAGE.md#the-big-picture
└── Subsection: md://computing/01-PACKAGE.md#the-big-picture/layer-1
    ├── Element: md://computing/01-PACKAGE.md#the-big-picture:figure:0
    └── Named element: md://computing/01-PACKAGE.md#section:figure.flowchart:arch
        ├── Sub-element: md://computing/01-PACKAGE.md#section:figure:arch[box=CORE]
        └── Cell: md://computing/01-PACKAGE.md#section:table:0[row=X,col=Y]
```
<!-- /proof:compiled -->

---

## Path component

The `path` is the only required component — a file path relative to the proof
root (the directory containing `proof.toml`). It must end in `.md`. Never use
leading `./` or `/`; paths are always relative to the proof root, not the
file system root or the current directory.

| Valid | Invalid | Reason |
|-------|---------|--------|
| `languages/10-GO.md` | `./languages/10-GO.md` | No leading `./` |
| `computing/01-PACKAGE.md` | `/computing/01-PACKAGE.md` | No leading `/` |
| `data/metrics.md` | `data/metrics` | Must include `.md` extension |

---

## Heading path

The heading path narrows to a section within the file. Segments are the
GitHub-normalized form of heading text: lowercase, spaces replaced with dashes,
punctuation stripped. You don't write the `#` characters from the markdown heading.

The heading path is a slash-separated chain, parent to child. Use a longer
path whenever a heading text appears multiple times in the same file — the
chain disambiguates by requiring the parent to match too.

```
Heading text: "The Big Picture"
Normalized:   "the-big-picture"

URI:  md://computing/01-PACKAGE.md#the-big-picture
```

### Subsection navigation

```
# Root heading          → #root-heading
## Child heading        → #root-heading/child-heading
### Grandchild          → #root-heading/child-heading/grandchild
```

Parent/child paths disambiguate when the same heading text appears in multiple sections:

```
md://doc.md#chapter-1/introduction   ← unambiguous
md://doc.md#introduction             ← SectionAmbiguous if 2+ headings match
```

---

## Type and kind

The type component filters which elements within a section the resolver looks
at. Without a type, the URI addresses the section itself. With a type, the
resolver collects all elements of that type and applies the selector.

The optional `.kind` sub-type narrows further — `figure.flowchart` matches
only flowchart-style figures, not boxes or sequences. Use kind when there are
multiple figures of different types and you need a specific one.

```
:type:           → any element of that type
:type.kind:      → elements of that specific sub-type
```

| Type | Kinds | Notes |
|------|-------|-------|
| `figure` | `flowchart`, `box`, `sequence`, `timeline`, `graph` | ASCII art in fenced blocks |
| `table` | `key-value`, `schema`, `comparison`, `data` | Markdown pipe tables |
| `chart` | `bar`, `line`, `scatter`, `stacked`, `pie` | ASCII charts |
| `math` | — | LaTeX or rendered math expression |
| `tree` | `ascii`, `org`, `taxonomy` | Tree/hierarchy diagram |
| `slide` | — | Slide block |
| `dashboard` | — | Dashboard canvas block |
| `text` | `prose`, `list`, `code` | Paragraphs, lists, code blocks |
| `heading` | `h1`, `h2`, `h3` | Heading lines |

---

## Selector

The selector picks one element from the filtered type collection. Always prefer
a named selector — it's stable across edits because it identifies the element
by what it IS, not where it sits in the document. Numeric indexes are the last
resort: they break whenever any element is inserted or removed above the target.

**Named (preferred — stable):**
```
:figure:goroutine-scheduler
:table:metrics
:chart.bar:latency-comparison
```

**Numeric (fallback — breaks on reorder):**
```
:figure:0    ← first figure
:table:2     ← third table
```

Named selectors use a three-phase matching cascade: exact match, then
starts-with prefix, then substring. If two elements match at the same priority
level, the resolver returns `LabelAmbiguous` rather than picking arbitrarily.

---

## Sub-selectors

Sub-selectors target content WITHIN an already-resolved element. They're most
useful for extracting a specific row, column, or cell from a table, or a
specific labeled box from a figure. Sub-selectors use the same named/numeric
matching as main selectors — prefer names over indexes.

`[key=value]` targets content within an element:

| Sub-selector | Applies to | Example |
|-------------|-----------|---------|
| `[row=X]` | Tables | `[row=Goroutine]` — select row where first col = "Goroutine" |
| `[col=Y]` | Tables | `[col=Stack Size]` — select column with that header |
| `[row=X,col=Y]` | Tables | Cell at row X, column Y |
| `[box=Z]` | Figures | `[box=SCHEDULER]` — text block labeled Z inside the figure |
| `[bar=X]` | Charts | `[bar=Option A]` — bar with that label |

---

## Query parameters

Query parameters apply after all selectors and sub-selectors as post-processing
transformations. Use them to project columns, filter rows, or paginate large
tables. They don't change which element is resolved — they reshape the content
returned by resolution.

| Parameter | Description | Example |
|-----------|-------------|---------|
| `?select=a,b` | Return only listed columns from a table | `?select=name,value` |
| `?filter=expr` | Filter rows matching an expression | `?filter=status=passing` |
| `?count` | Return element count instead of content | `?count` |
| `?top=N` | Return first N results | `?top=5` |
| `?skip=N` | Skip first N results | `?skip=2` |

---

## Real URI examples

<!-- proof:compiled from="proof:tree kind=org" uri="" -->
```org
URI examples by specificity
├── Whole file: md://computing/01-PACKAGE.md
├── Section: md://computing/01-PACKAGE.md#the-big-picture
└── Subsection: md://computing/01-PACKAGE.md#the-big-picture/layer-1
    ├── First figure: md://computing/01-PACKAGE.md#the-big-picture:figure:0
    └── Named figure: md://computing/01-PACKAGE.md#section:figure.flowchart:arch
        ├── Table row: md://computing/01-PACKAGE.md#section:table:0[row=Binding]
        └── Table cell: md://computing/01-PACKAGE.md#section:table:0[row=Binding,col=Value]
            └── Figure box: md://computing/01-PACKAGE.md#section:figure:arch[box=CORE]
```
<!-- /proof:compiled -->

---

## Naming principle: strings over numbers

The most important rule in the URI scheme: use a name whenever one exists.
Named URIs are stable across document edits because they anchor to the content's
identity, not its position. Numeric URIs are a fallback for elements that have
no discoverable name.

When proof generates a URI (in error output, fix plans, or `proof pin`), it
always resolves to the named form first. Only when no name exists does it fall
back to a numeric index.

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="md://src/data/label-matching.md" -->
```taxonomy
1
└── Exact match
2
└── Starts-with prefix
3
└── Substring
4
└── Numeric index
5
└── Ambiguous — error
```
<!-- /proof:compiled -->
