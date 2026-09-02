# MDPATH Numeric URI Boundary

This boundary closes `MDPATH-PF-03`.

Numeric selectors are fallback addresses. They are allowed only when the target
element has no discoverable stable label. Once a fenced element has a label,
resolving it through a numeric selector returns `NumericUriStale` and the
consumer must update the reference to the named form.

## Promotion Rule

A consumer may keep or emit a numeric `md://` URI only when:

- the element has no inline or preceding label;
- the URI is not being used as a durable proof pin, publication reference, fix
  plan target, corpus compatibility canary, or generated cross-reference;
- the consumer records why no stable name is available;
- the consumer has a repair path for upgrading to a named selector when a label
  appears.

If a stable label exists, the numeric URI is stale. The repair is to replace the
numeric selector with the label reported by `NumericUriStale`.

## Scope

This guard applies to labelled fenced elements such as figures, charts, math,
trees, slides, and dashboards. Table indexes remain valid for table selection
because table-cell addressing commonly uses `:table:0[row=...,col=...]` and the
stable identity usually belongs to the row and column sub-selectors.

## Required Review Roles

Numeric fallback changes require URI Contract Steward, Resolver Semantics
Auditor, and Corpus Integration Reviewer review.
