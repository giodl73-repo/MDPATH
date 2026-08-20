# Consumer compatibility

MDPATH has three recorded source-level consumers. Its reusable contract is
stable `md://` parsing, Markdown document parsing, heading normalization,
resolution, typed element classification, and structured errors. Each consumer
retains its own admission and product policy.

| Consumer | Retained MDPATH canary | Consumer-owned behavior |
|---|---|---|
| AMAZE | Document parsing and heading normalization | Room Markdown lint rules and operator findings |
| MDCROP | Canonical nested-section resolution | Corpus selection, graph traversal, and view policy |
| PROOF | Typed fenced-block resolution | Compilation, validation, rendering, and publication |

Run:

```powershell
cargo test --test consumer_contracts
```

The test resolves all three projections against
`fixtures/consumers/corpus.md` and compares complete reports with retained
fixtures. It also proves that a non-`md://` input remains `ParseError` and a
missing heading remains `SectionNotFound`; consumers must not convert either
case into a guessed target.

These canaries are representative, not exhaustive inventories of every imported
symbol.

## Compatibility and lifecycle

Existing URI grammar, canonical string form, heading normalization, selector
precedence, element classification, source ranges, and `MdPathError` meanings
must not change silently. Consumers should pin a tested MDPATH revision.

A breaking change requires an explicit version boundary, affected-consumer
list, updated accepted and failure fixtures, consumer rehearsal, migration
instructions, and rollback instructions. A surface may be deprecated only with
a replacement or removal reason and an explicit removal condition.

Consumers may remain on their last passing revision while migrating. Rollback
restores that revision and its retained fixtures; it must not percent-encode
paths, choose an arbitrary ambiguous label, guess a missing section, or move
consumer policy into MDPATH.

## Review findings

- **URI contract steward:** accepted; canonical URI and structured-error
  semantics remain unchanged.
- **Corpus integration reviewer:** accepted; the three real integration seams
  are exercised without importing consumer policy.
- **Resolver semantics auditor:** accepted; exact nested-section and typed-fence
  resolution remain deterministic, while missing targets fail explicitly.
