# MDPATH PITFALL index

MDPATH uses PITFALL as a compact index over durable `md://` addressing
doctrine. The index cites existing design notes, guides, roles, retained proof
fixtures, consumer canaries, and tests without moving those local sources of
truth.

| Namespace | Kind | Path | Owner |
|---|---|---|---|
| `mdpath` | `principles` | [mdpath-principles.md](mdpath-principles.md) | MDPATH maintainers |
| `mdpath` | `invariants` | [mdpath-invariants.md](mdpath-invariants.md) | MDPATH maintainers |
| `mdpath` | `pitfalls` | [mdpath-pitfalls.md](mdpath-pitfalls.md) | MDPATH maintainers |

## Integration

- ROLES: `.roles/ROLE.md` routes URI grammar, resolver semantics, and corpus
  integration changes to the matching reviewers.
- VTRACE: MDPATH does not yet carry repo-local VTRACE docs; PITFALL cites
  design doctrine, retained proofs, consumer compatibility docs, and role
  review findings until a trace layer exists.
- Tests: `proof_surface`, `consumer_contracts`, workspace tests, strict clippy,
  and PITFALL validators are the executable evidence hooks.

