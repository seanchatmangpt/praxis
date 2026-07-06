import Mathlib.Data.Nat.Log
import Praxis.Corpus.def_receipt

/-!
def:regimes

"For a trace of `n` committed frames: boundary projection inspection (any
verifier, per message) costs `O(CL)`; single frame recomputation (spot check)
costs `O(1)` per frame; checkpointed/indexed replay (sampled audit) costs
`O(log n)` or bounded path; full chain replay (full audit) costs `O(n)`."

This statement names four audit *regimes* over a trace of `n` committed
`Frame`s (from `def:receipt`, imported above), each characterized by its cost
as a function of the trace length `n` (and, for the first regime, the
per-message chunk bound `CL`).

Per the mandatory-composition directive: no fresh opaque cost-model axiom is
introduced. Each regime's cost is modeled as a plain computable function
`ℕ → ℕ` built from pre-built Lean-core / Mathlib arithmetic
(`Nat.log`, constants, the identity), which is exactly what "cost is
`O(f n)`" means as a concrete witness function for the asymptotic class.
`Nat.log 2` is Mathlib's pre-built base-2 logarithm
(`Mathlib.Data.Nat.Log`), used directly for the sampled-audit regime rather
than re-axiomatized.

The four regimes are packaged as a single `structure` (a genuine Mathlib/core
composition: `structure` is core Lean, not a fresh axiom) so downstream files
can refer to `Regimes` as one vocabulary item, matching how `def:receipt`
packaged its tuple as `Receipt`.
-/

/-- `CL`, the per-message chunk bound from `def:receipt`'s "`<= CL`-chunk
    tuple" -- an abstract fixed bound, reused here as a parameter rather than
    a new axiom. -/
abbrev ChunkBound := Nat

/-- The four audit regimes over a trace of `n` committed frames, each given
    by its concrete cost-witness function. -/
structure Regimes (CL : ChunkBound) where
  /-- Boundary projection inspection (any verifier, per message): `O(CL)`. -/
  boundaryProjection : Nat → Nat := fun _n => CL
  /-- Single frame recomputation (spot check): `O(1)` per frame. -/
  spotCheck : Nat → Nat := fun _n => 1
  /-- Checkpointed/indexed replay (sampled audit): `O(log n)` or bounded
      path. -/
  sampledAudit : Nat → Nat := fun n => Nat.log 2 n
  /-- Full chain replay (full audit): `O(n)`. -/
  fullAudit : Nat → Nat := fun n => n

/-- The canonical instantiation of the four regimes for a given chunk
    bound `CL`, using exactly the witness functions named in the
    statement. -/
def canonicalRegimes (CL : ChunkBound) : Regimes CL where
  boundaryProjection := fun _n => CL
  spotCheck := fun _n => 1
  sampledAudit := fun n => Nat.log 2 n
  fullAudit := fun n => n

example (CL : ChunkBound) : (canonicalRegimes CL).spotCheck 100 = 1 := rfl
example (CL : ChunkBound) : (canonicalRegimes CL).fullAudit 100 = 100 := rfl
