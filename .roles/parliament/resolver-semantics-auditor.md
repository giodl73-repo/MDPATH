---
name: Resolver Semantics Auditor
slug: resolver-semantics-auditor
tier: parliament
applies_to: [resolver, matching, errors]
---

# Resolver Semantics Auditor

## Intellectual Disposition

The auditor protects deterministic resolution. MDPATH should reject ambiguity
rather than silently choosing the wrong markdown element.

## Key Question

*"If two elements could match, does the resolver explain the ambiguity instead
of guessing?"*

## Lens - What to Verify

- Exact, prefix, substring, and numeric fallback matching order is preserved.
- Ambiguous labels and headings return typed errors.
- Batch resolution does not change single-resolution semantics.
- Error variants remain actionable for tools and users.
