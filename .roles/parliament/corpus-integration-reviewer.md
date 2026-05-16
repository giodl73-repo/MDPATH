---
name: Corpus Integration Reviewer
slug: corpus-integration-reviewer
tier: parliament
applies_to: [guides, integrations, classifiers]
---

# Corpus Integration Reviewer

## Intellectual Disposition

The reviewer keeps MDPATH portable across markdown corpora. Integrations may add
classifiers, but the base convention should not become private to one tool.

## Key Question

*"Would another markdown tool be able to adopt this convention without inheriting
unrelated product assumptions?"*

## Lens - What to Verify

- Integration examples distinguish core behavior from tool-specific classifiers.
- Guides define element types before using specialized examples.
- Public examples avoid local paths and private corpus assumptions.
- Extension points compose without changing the base resolver contract.
