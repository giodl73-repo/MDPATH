# md:// Selectors and Sub-selectors

A selector picks one element from the collection of elements matching a given
type. A sub-selector picks content within that element. Together they let you
address any piece of a document — from a whole figure down to a single table cell.

The key principle is the same at every level: prefer names over numbers.
Named selectors survive document edits; numeric indexes break the moment
anything is inserted or removed above the target.

---

## Selector types

The main selector appears after the type declaration: `:figure:my-selector`.
There are two kinds — named (preferred) and numeric (fallback). Named selectors
use a three-phase matching cascade so you don't have to write the full label
when a prefix or substring uniquely identifies the element.

### Named selector (preferred)

Matches by element label. Uses a priority cascade:

```proof:tree kind=taxonomy
root: Named selector cascade
- 1. Exact match: selector == label
- 2. Starts-with: label.starts_with(selector)
- 3. Substring: label.contains(selector)
- 4. Ambiguous: multiple matches at same priority → LabelAmbiguous error
```

Examples:
```
:figure:goroutine-scheduler          ← exact
:figure:goroutine                    ← prefix (matches "goroutine-scheduler")
:figure:scheduler                    ← substring
```

### Numeric selector (fallback)

Zero-indexed position within the type collection in the section:
```
:figure:0      ← first figure
:table:1       ← second table
:chart:0       ← first chart
```

Numeric selectors are fragile — they break when elements are reordered.
Use named selectors whenever possible. For labelled fenced elements, a numeric
selector returns `NumericUriStale`; table indexes remain valid for table
row/column sub-selector addressing.

---

## Sub-selectors

Sub-selectors go deeper than the main selector. Where `:table:metrics` gives you
the whole table, `:table:metrics[row=Goroutine]` gives you just that one row.
Sub-selectors use the same named/numeric cascade as main selectors — `[row=Goroutine]`
will match `Goroutine Stack`, `Goroutine Pool`, etc. via prefix matching if there's
no exact match. Chain multiple sub-selectors to narrow to a single cell.

Applied after the element is selected. Format: `[key=value]`

### Table sub-selectors

| Sub-selector | Meaning | Example |
|-------------|---------|---------|
| `[row=X]` | Row where first column = X | `[row=Goroutine]` |
| `[col=Y]` | Column with header = Y | `[col=Stack Size]` |
| `[row=X,col=Y]` | Cell at row X, column Y | `[row=Goroutine,col=Stack Size]` |

Row and column values also use exact → prefix → substring matching.

### Figure sub-selectors

| Sub-selector | Meaning | Example |
|-------------|---------|---------|
| `[box=Z]` | Text block labeled Z inside the figure | `[box=SCHEDULER]` |

### Chart sub-selectors

| Sub-selector | Meaning | Example |
|-------------|---------|---------|
| `[bar=X]` | Bar with label X | `[bar=Option A]` |

---

## Combining sub-selectors

Multiple sub-selectors chain left to right, each further narrowing the result:

```
md://doc.md#section:table:0[row=Row1,col=Value]

Step 1: Select the table at index 0
Step 2: Filter to row where first col = "Row1"
Step 3: Extract cell in column "Value"
```

---

## Query parameters

Query parameters apply after all selectors and sub-selectors.

### `?select=col1,col2`

Return only listed columns from a table result:

```
md://data.md:table:metrics?select=name,p99
→ returns table with only "name" and "p99" columns
```

### `?filter=expression`

Filter rows matching a condition:

```
md://data.md:table:metrics?filter=status=passing
→ returns only rows where status column = "passing"
```

### `?count`

Return the count of matching elements instead of their content:

```
md://doc.md#section:figure?count
→ returns "3" (three figures in the section)
```

### `?top=N` and `?skip=N`

Pagination:

```
md://data.md:table:metrics?top=10
→ first 10 rows

md://data.md:table:metrics?skip=10&top=10
→ rows 11-20
```

---

## Sub-selector examples: self-referential

These URIs address content within this very guide file:

```
md://src/guides/04-selectors.source.md#table-sub-selectors:table:0
→ the "Table sub-selectors" table above

md://src/guides/04-selectors.source.md#query-parameters:table:0[row=?select=col1,col2]
→ the ?select row in the query parameters table
```

---

## Sub-selector validation

| Error | Cause |
|-------|-------|
| `SubSelectorInvalid` | Key not valid for element type (e.g. `[box=X]` on a table) |
| `ElementNotFound` | Sub-selector value not found in element |
| `LabelAmbiguous` | Multiple rows/cols match the sub-selector value |
