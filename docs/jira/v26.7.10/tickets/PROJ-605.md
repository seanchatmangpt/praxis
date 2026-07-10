# PROJ-605 — New `CNG_R11 AuditMismatch` refusal variant

Status: PLANNED

`CngRefusal` (`crates/cng/src/powl.rs:37-111`, `CNG_R01`-`CNG_R10`) has no exhaustiveness
registry and is open for extension. `CNG_R08 Nondeterminism` ("repeated manufacture produced
different bytes") is the wrong reuse target for a third-party auditor digest mismatch — that
variant means same-producer re-manufacture drift, semantically distinct from an independent
integrity check against a previously recorded digest (the PROJ-502 `cng evidence replay` use
case). Add a genuine new `CNG_R11 AuditMismatch` variant with an end-to-end negative test: tamper
one observation triple, run replay, confirm it refuses with `CNG_R11`. Links back to
`docs/releases/v26.7.10/PRD.md` (Claims Reconciliation row 7 discussion in `ARD.md` Sec. 14) and
`RELEASE_CONTROL.md` Sec. 5.

Implementation detail: `docs/releases/v26.7.10/IMPLEMENTATION_SPEC.md` (exact edits,
anchors, tests, and acceptance commands for this ticket).
