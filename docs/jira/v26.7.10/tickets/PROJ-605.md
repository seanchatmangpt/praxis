# PROJ-605 — New `CNG_R11 AuditMismatch` refusal variant

Status: CLOSED

Closed by commit `40f6020` (`CngRefusal::AuditMismatch`, code `CNG_R11`). Verification evidence
recorded in `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 7 (ladder items 1 and 10:
`audit_mismatch_refusal_has_stable_code` passes; tampering one `obs/*.ttl` byte → replay exits 1
with `CNG_R11: obs digest mismatch`) — this ticket cites that record rather than re-asserting
it.

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
