# MDPATH Pitfalls

## MDPATH-PF-01: Normalization Diverges Across Tools

**Status:** MITIGATED

**Pattern:** Different Markdown tools interpret heading normalization
differently, so the same `md://` URI resolves to different headings.

**Domain:** URI grammar, heading paths, guides, consumer integrations, and
canonical URI output.

**Detection difficulty:** Divergence usually appears only after a second tool
or corpus consumes the same address.

**Structural solution:** Keep the normalization algorithm specified and tested
against retained examples.

**Evidence:** `design/PITFALLS.md`, `docs/guides/01-uri-syntax.md`, and
URI Contract Steward review.

## MDPATH-PF-02: Ambiguous Label Chooses First Match

**Status:** MITIGATED

**Pattern:** Prefix or substring matching returns the first document-order
element when two candidates match at the same priority.

**Domain:** Resolver semantics, label matching, subselectors, proof pins, and
consumer canaries.

**Detection difficulty:** The selected element can be plausible, so the bug may
hide until a proof or rendered output references the wrong block.

**Structural solution:** Return `LabelAmbiguous` or the corresponding typed
ambiguity error instead of guessing.

**Evidence:** `design/PITFALLS.md`, `docs/proof-surface.md`, and
`cargo test --test proof_surface`.

## MDPATH-PF-03: Numeric URI Survives After A Name Exists

**Status:** OPEN

**Pattern:** A numeric URI such as `:figure:0` continues to resolve after a
stable label is added, leaving a reorder-sensitive stale reference.

**Domain:** PROOF pinning, resolver output, consumer compatibility, and
authoring workflows.

**Detection difficulty:** The URI still works until another same-type element
is inserted before it.

**Structural solution:** Preserve this as an open consumer-boundary issue:
resolver/proof surfaces should warn or refuse numeric pins when a named form is
available.

**Evidence:** `design/PITFALLS.md` `MP-06` and prior PROOF stale numeric URI
fixes tracked in the portfolio PITFALL wave.

## MDPATH-PF-04: Consumer Policy Leaks Into Core Protocol

**Status:** MITIGATED

**Pattern:** PROOF compilation, MDCROP selection, AMAZE linting, or MDPORT
transfer rules are treated as MDPATH semantics.

**Domain:** Guides, classifiers, integration examples, consumer canaries, and
MD family docs.

**Detection difficulty:** Integration examples are useful and can quietly
become normative if core and consumer-owned behavior are not separated.

**Structural solution:** Keep consumer behavior in consumer repos and use
canaries only for shared parse, resolve, classification, and error contracts.

**Evidence:** `docs/consumer-compatibility.md`, README MD family table, and
Corpus Integration Reviewer review.

## MDPATH-PF-05: Retained Proof Fixtures Are Treated As Golden Noise

**Status:** MITIGATED

**Pattern:** Accepted or rejected proof fixtures are updated to satisfy a test
without reviewing the public addressing contract they represent.

**Domain:** `fixtures/proof/`, proof-surface tests, consumer canaries, and
contract-changing PRs.

**Detection difficulty:** Fixture churn looks like ordinary test maintenance
unless the semantic contract is reviewed.

**Structural solution:** Require implementation, fixture, docs, and role-review
changes to move together for contract changes.

**Evidence:** `docs/proof-surface.md`, `fixtures/proof/`, and
`.roles/ROLE.md`.

