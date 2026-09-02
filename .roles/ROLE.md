# MDPATH Role Index

MDPATH defines the `md://` addressing convention and the Rust resolver that
implements it. Use these roles when changing URI grammar, selector semantics,
resolution behavior, guides, or integration examples.

## Parliament

| File | Role | Primary tension |
|---|---|---|
| `parliament/uri-contract-steward.md` | URI Contract Steward | Stable identifiers vs. convenient syntax changes |
| `parliament/resolver-semantics-auditor.md` | Resolver Semantics Auditor | Deterministic resolution vs. permissive matching |
| `parliament/corpus-integration-reviewer.md` | Corpus Integration Reviewer | Portable protocol vs. one-tool assumptions |

## Review order

1. Use URI Contract Steward for grammar, canonicalization, or selector changes.
2. Use Resolver Semantics Auditor for parsing, matching, ambiguity, and error behavior.
3. Use Corpus Integration Reviewer for README, guides, and external-tool examples.

## PITFALL gates

| Pitfall | Gate | Required roles |
|---|---|---|
| `MDPATH-PF-03` | Numeric URI boundary. Blocks durable proof pins, publication references, fix-plan targets, compatibility canaries, and generated cross-references from keeping numeric fenced-element selectors after a stable label exists; labelled fenced elements return `NumericUriStale`, while table indexes remain available for row/column addressing. | URI Contract Steward; Resolver Semantics Auditor; Corpus Integration Reviewer |
