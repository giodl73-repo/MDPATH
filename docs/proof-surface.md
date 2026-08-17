# Proof surface

MDPATH retains a small end-to-end proof under `fixtures/proof`.

- `accepted.json` records a named figure resolving to its canonical URI, label,
  type, and source range.
- `rejected.json` records two equally ranked labels producing the typed
  `LabelAmbiguous` failure instead of an arbitrary match.
- `corpus.md` is the stable source used by both cases.

Run the proof from the repository root:

```powershell
cargo test --test proof_surface
```

The test exercises the public `parse` and `resolve` APIs and compares their
complete reports with the retained fixtures. A contract change must update the
implementation, fixtures, and reader guidance together; do not weaken the
ambiguity failure to preserve an old report.
