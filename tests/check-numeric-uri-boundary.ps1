$ErrorActionPreference = 'Stop'

function Assert-Contains {
  param(
    [string]$Path,
    [string]$Needle
  )

  $text = Get-Content -Raw -LiteralPath $Path
  if ($text.IndexOf($Needle, [StringComparison]::Ordinal) -lt 0) {
    throw "Missing expected text in ${Path}: ${Needle}"
  }
}

Assert-Contains 'README.md' 'numeric URI boundary'
Assert-Contains 'README.md' 'MDPATH-PF-03'
Assert-Contains 'README.md' 'NumericUriStale'

Assert-Contains 'docs/numeric-uri-boundary.md' '`MDPATH-PF-03`'
Assert-Contains 'docs/numeric-uri-boundary.md' 'If a stable label exists'
Assert-Contains 'docs/numeric-uri-boundary.md' 'Table indexes remain valid'

Assert-Contains 'docs/consumer-compatibility.md' 'numeric URI boundary'
Assert-Contains 'docs/consumer-compatibility.md' 'MDPATH-PF-03'

Assert-Contains 'docs/guides/01-uri-syntax.md' 'NumericUriStale'
Assert-Contains 'docs/guides/04-selectors.md' 'NumericUriStale'

Assert-Contains '.roles/ROLE.md' '## PITFALL gates'
Assert-Contains '.roles/ROLE.md' '`MDPATH-PF-03`'
Assert-Contains '.roles/ROLE.md' 'URI Contract Steward; Resolver Semantics Auditor; Corpus Integration Reviewer'

Assert-Contains '.pitfall/mdpath-pitfalls.md' '**Status:** MITIGATED'
Assert-Contains '.pitfall/mdpath-pitfalls.md' 'tests/numeric_uri_boundary.rs'
Assert-Contains '.pitfall/mdpath-invariants.md' 'MDPATH-I-06'
Assert-Contains '.pitfall/mdpath-invariants.md' 'Numeric Fallback Refuses Named Elements'
Assert-Contains '.pitfall/mdpath-invariants.md' 'MDPATH-PF-03'

Write-Host 'MDPATH numeric URI boundary check passed.'
