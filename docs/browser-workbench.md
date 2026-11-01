# Browser consumer verification scope

This first-party consumer uses the existing in-memory parser/resolver. Its wave
is [2026-11-01-address-workbench](../context/waves/2026-11-01-address-workbench/WAVE.md).
It adds no native URI grammar, matching or classifier changes and claims no new
external adoption. Root source is compared against integrated baseline 892b68c.
Root and companion lockfiles pin their separate dependency closures.

| Requirement | Evidence |
|---|---|
| Own Markdown and address perform real Rust resolution | Own document/path browser case; native fixtures and real WASM worker parity |
| Preserve named resolution, ambiguity and numeric boundary | Existing native consumer/numeric tests; seven examples; typed native report exports |
| Native/wasm32 integer behavior stays explicit | u32 fragment admission before core numeric fallback; max and over-limit selector, subselector and query fixtures |
| Paths refer only to the supplied document | Exact logical binding and invalid-path tests; companion calls no filesystem resolver |
| Original input survives failed/stale work | CRLF/Unicode, invalid UTF-8, oversize, delayed read, edit/reset and file deadline browser cases |
| Engine failures retain drafts and independent downloads | Startup/compute deadline and late-message cases; no-JS source/example access |
| Render supplied content inertly | Unicode/HTML-as-text browser case and native content fixture |
| Useful inspectable downloads and layouts | Report/content/request/original exports; source, WASM MIME/magic/notices; desktop/mobile screenshots |
| Reproducible tested delivery | Pinned clean builder, Git source archive, per-file hashes, hosted browser artifact before master deployment |

Producer review in declared role order: URI Contract Steward checks unchanged
grammar and preserved named-selector guard; Resolver Semantics Auditor checks
native outcomes, typed ambiguity, portable numeric admission and honest heading/
table output; Corpus Integration Reviewer checks default classifier, single
logical binding, public synthetic examples and absence of private corpus paths.
These checks do not constitute independent signoff or a broad corpus-identity
guarantee. Exact deployed-file and live-browser custody remain a release gate.
