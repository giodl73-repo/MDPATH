# MDPATH Invariants

## MDPATH-I-01: Heading Normalization Is Canonical

**Status:** VERIFIED

**Invariant:** Heading paths normalize through one specified algorithm, not a
vague "GitHub-compatible" reference.

**Why it matters:** Different tools resolving the same `md://` URI must not
reach different headings.

**Test:** Workspace tests and retained guides cover canonical heading paths and
consumer canaries.

**Evidence:** `design/PITFALLS.md`, `docs/guides/01-uri-syntax.md`, and
`cargo test --workspace`.

## MDPATH-I-02: Label Matching Order Is Stable

**Status:** VERIFIED

**Invariant:** Label resolution proceeds exact, prefix, substring, then numeric
fallback, with ambiguity errors at each matching level.

**Why it matters:** Permissive matching silently pins the wrong Markdown
element in generated proof and consumer workflows.

**Test:** `cargo test --test proof_surface` and workspace resolver tests.

**Evidence:** `README.md`, `docs/proof-surface.md`, and
`.roles/parliament/resolver-semantics-auditor.md`.

## MDPATH-I-03: Batch Resolution Does Not Change Semantics

**Status:** VERIFIED

**Invariant:** `BatchResolver` may read and parse once for efficiency, but it
must not change single-resolution matching, errors, or canonical output.

**Why it matters:** Large-corpus consumers need performance without getting a
different addressing contract.

**Test:** Workspace tests and consumer canaries exercise batch and ordinary
resolution seams.

**Evidence:** README `BatchResolver` section, `src/resolver.rs`, and
`.roles/parliament/resolver-semantics-auditor.md`.

## MDPATH-I-04: Consumer Canaries Keep Policy Out Of MDPATH

**Status:** VERIFIED

**Invariant:** AMAZE, MDCROP, and PROOF canaries exercise shared address
contracts without importing consumer-owned policy into MDPATH.

**Why it matters:** MDPATH must remain a portable protocol for Markdown corpora,
not a hidden dependency on one downstream product.

**Test:** `cargo test --test consumer_contracts`.

**Evidence:** `docs/consumer-compatibility.md` and
`.roles/parliament/corpus-integration-reviewer.md`.

## MDPATH-I-05: Structured Errors Stay Actionable

**Status:** VERIFIED

**Invariant:** Parse, file, section, element, label, and subselector failures
remain distinct typed errors with mechanical recovery guidance.

**Why it matters:** Consumers cannot safely recover if missing targets,
ambiguous targets, and malformed URIs collapse into generic failure.

**Test:** `cargo test --workspace` plus consumer canaries for `ParseError` and
`SectionNotFound`.

**Evidence:** `docs/guides/06-errors.md`, `src/error.rs`, and
`docs/consumer-compatibility.md`.

## MDPATH-I-06: Numeric Fallback Refuses Named Elements

**Status:** VERIFIED

**Invariant:** A numeric selector for a labelled fenced element fails with
`NumericUriStale` instead of resolving a reorder-sensitive stale URI; numeric
fallback remains available for unlabeled fenced elements and table row/column
addressing.

**Why it matters:** Durable proof pins and generated cross-references should
move to stable names as soon as a label exists.

**Test:** `cargo test --test numeric_uri_boundary` and
`pwsh -NoProfile -File tests/check-numeric-uri-boundary.ps1`.

**Evidence:** `MDPATH-PF-03`, `docs/numeric-uri-boundary.md`,
`src/resolver.rs`, `tests/numeric_uri_boundary.rs`, `.roles/ROLE.md`, and
`docs/consumer-compatibility.md`.
