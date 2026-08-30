# MDPATH Principles

## MDPATH-P-01: Stable Names Beat Line Numbers

**Status:** ACTIVE

**Statement:** `md://` addresses identify Markdown elements by durable document
structure and names, not by line numbers or incidental file edits.

**Decision rule:** Grammar, canonicalization, and resolver changes must
preserve stable addresses across ordinary heading, figure, table, and file
movement.

**Evidence:** `README.md`, `.roles/parliament/uri-contract-steward.md`, and
`docs/guides/01-uri-syntax.md`.

## MDPATH-P-02: Named Selectors Beat Numeric Selectors

**Status:** ACTIVE

**Statement:** Named selectors are the preferred public contract; numeric
selectors are a fallback with explicit reorder risk.

**Decision rule:** Docs, examples, and consumer guidance must prefer named
selectors and flag numeric selectors when a named form is available.

**Evidence:** `README.md`, `design/PITFALLS.md`, and
`docs/consumer-compatibility.md`.

## MDPATH-P-03: Ambiguity Fails Loudly

**Status:** ACTIVE

**Statement:** If multiple headings, labels, rows, columns, boxes, or bars can
match at the same priority, MDPATH returns a typed ambiguity error instead of
guessing.

**Decision rule:** Exact, prefix, substring, and numeric fallback matching
order must remain deterministic, and ambiguity must stay actionable.

**Evidence:** `.roles/parliament/resolver-semantics-auditor.md`,
`docs/proof-surface.md`, and `tests/proof_surface.rs`.

## MDPATH-P-04: Core Protocol Stays Tool-Neutral

**Status:** ACTIVE

**Statement:** MDPATH is the addressing layer for the MD family, not a PROOF,
MDCROP, AMAZE, or MDPORT policy engine.

**Decision rule:** Integration examples may show consumers and classifiers, but
consumer admission, publication, selection, lint, and rendering policies stay
outside MDPATH.

**Evidence:** `README.md`, `.roles/parliament/corpus-integration-reviewer.md`,
and `docs/consumer-compatibility.md`.

## MDPATH-P-05: Retained Fixtures Are Contract Evidence

**Status:** ACTIVE

**Statement:** Accepted and rejected proof fixtures are part of the public
contract evidence, not disposable snapshots.

**Decision rule:** A contract change must update implementation, fixtures,
consumer guidance, and review notes together.

**Evidence:** `docs/proof-surface.md`, `fixtures/proof/`, and
`docs/consumer-compatibility.md`.

