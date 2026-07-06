import Praxis.Corpus.def_staged

/-!
# `def:mut` — Mutation operators over a staged validator

A mutation operator `m` maps a valid subject `s⋆ ∈ ⋂ᵢ Iᵢ` to a mutant `m(s⋆)` violating a
non-empty set of invariants; its correct stage is `stg(m) = min {i : m(s⋆) ∉ Iᵢ}`; `m` is
killed by `V`, written `Kill(m) = 1`, iff `V` rejects `m(s⋆)`.

We reuse `Praxis.Corpus.DefStaged.StagedValidator` for the pipeline `V`, and represent the
mutation operator directly as a function `S → S` together with the hypothesis that it is
"mutating" relative to a chosen valid subject `s⋆` (i.e. `s⋆` is accepted by `V` but `m s⋆` is
not). The correct stage `stg(m)` is `V.firstRejection (m s⋆)`, already defined in `def:staged`
as the least rejecting stage index (`none` would mean no stage rejects it, which cannot happen
under the mutating hypothesis, but we keep the `Option` return type so no new partiality
machinery is introduced). `Kill(m)` is simply whether that rejection actually occurs, i.e.
whether `V.firstRejection (m s⋆)` is `some _`; we report it as a `Bool` via `Option.isSome`,
matching the `{0,1}`-valued `Kill(m) ∈ {0,1}` of the source statement. Everything here is
composed from the existing `StagedValidator` structure and core `Option`/`Bool` machinery —
no new axioms are introduced.
-/

namespace Praxis.Corpus.DefMut

open Praxis.Corpus.DefStaged

variable {S : Type*} {k : ℕ}

/-- A mutation operator `m : S → S` is *mutating* at the valid subject `s⋆` (relative to
pipeline `V`) if `s⋆` is accepted by every stage of `V` but `m s⋆` violates at least one
stage's invariant, i.e. `m s⋆ ∉ ⋂ᵢ Iᵢ`. -/
def Mutating (V : StagedValidator S k) (m : S → S) (sStar : S) : Prop :=
  V.accepts sStar ∧ ¬ V.accepts (m sStar)

/-- The correct stage of a mutant `m(s⋆)`: the least stage index `i` with `m(s⋆) ∉ Iᵢ`, i.e.
`stg(m) = min {i : m(s⋆) ∉ Iᵢ}`, computed via `firstRejection` from `def:staged`. Returns `none`
only if no stage rejects the mutant (which cannot happen when `Mutating V m sStar` holds, since
`k = 0` is excluded by that hypothesis forcing `V.accepts` to be nontrivial to violate). -/
def stg (V : StagedValidator S k) (m : S → S) (sStar : S) : Option (Fin k) :=
  V.firstRejection (m sStar)

/-- `m` is killed by `V` at `s⋆`, i.e. `Kill(m) = 1`, iff `V` rejects `m(s⋆)` at some stage,
i.e. `stg(m)` is defined (`some _` rather than `none`). We report this as a `Bool`, matching the
`{0,1}`-valued `Kill(m)` of the source statement (`true` ↦ `1`, `false` ↦ `0`). -/
def Kill (V : StagedValidator S k) (m : S → S) (sStar : S) : Bool :=
  (stg V m sStar).isSome

end Praxis.Corpus.DefMut
