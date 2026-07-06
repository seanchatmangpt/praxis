import Mathlib.Order.FixedPoints
import Mathlib.Order.OmegaCompletePartialOrder
import Mathlib.Data.Set.Lattice
import Praxis.Corpus.def_net
import Praxis.Corpus.def_saturation

/-!
# prop:satcorrect — Saturation correctness

`p ∈ sat(Π)` iff there exists a sequence of ground actions whose add-effects include
`p`, i.e. `p` is reachable from `m₀` in the Petri-net sense.

`sat(Π)` is `Praxis.Corpus.DefSaturation.Saturation T`, i.e. `OrderHom.lfp T` for the
program's (monotone) immediate-consequence operator `T` on `Set (GroundAtom D Ob)`
(`Praxis.Corpus.DefSaturation.ConsequenceOp`, reusing Mathlib's `Set`
`CompleteLattice`). We reuse Mathlib's own **Kleene fixed-point theorem**
(`fixedPoints.lfp_eq_sSup_iterate`, `Mathlib/Order/FixedPoints.lean`) rather than
reconstructing an iterate/fixpoint argument by hand: for any `ωScottContinuous`
monotone self-map `T` on a complete lattice, `lfp T = ⨆ n, T^[n] ⊥`. Specializing to
`Set (GroundAtom D Ob)` (`⊥ = ∅`, `⨆ = ⋃`, via Mathlib's `Set.iSup_eq_iUnion` /
`Set.mem_iUnion`) gives exactly the paper's correctness statement: `p ∈ sat(Π)` iff
`p ∈ T^[n] ∅` for some `n : ℕ`, i.e. iff `p` is produced after finitely many *rounds*
of firing every currently-enabled ground action (`T` applied once = "fire every
enabled ground action whose preconditions already hold, add its add-effects"). This
round-based sequence is the standard Datalog/Petri-net presentation of "there exists
a sequence of ground actions reaching `p`" (Van Emden–Kowalski 1976 immediate-
consequence semantics): since accumulation is monotone (facts, once derived, are
never retracted — `Praxis.Corpus.DefSaturation.ConsequenceOp` is already required to
be a monotone `OrderHom`), any interleaving/order of firing the same finite multiset
of enabled ground actions across `n` rounds reaches the same accumulated atom set, so
"reachable by *some* sequence of ground action firings" and "reachable within some
finite number of consequence-operator rounds" coincide.

The one hypothesis genuinely needed beyond `def:saturation`'s existing `Monotone`
requirement is `ωScottContinuous T`: Kleene's theorem needs continuity (distributing
over `ω`-chains' suprema), not mere monotonicity, to identify `lfp T` with the
*iterate* union rather than a possibly-larger transfinite closure ordinal. For the
concrete immediate-consequence operator built from a **finite** set of ground action
rules (`Praxis.Corpus.DefNet.Net`/`Praxis.Corpus.DefGround.GroundAction` both being
`Fintype`-indexed) this continuity is automatic — a fact each new atom depends on only
finitely many already-derived atoms (the rule's finite precondition list), so no atom
can require an infinite/limit stage to appear — but formalizing that finitary-rule ⇒
ωScottContinuous derivation in general (for an arbitrary abstract `ConsequenceOp`,
which is exactly what `def:saturation` deliberately keeps abstract, only recording
"monotone self-map on `Set (GroundAtom D Ob)`" without a rule-body/arity structure to
induct on) is a separate, structural piece of Datalog metatheory outside the scope of
`def:net`/`def:saturation`'s existing corpus vocabulary. We therefore take it as an
explicit hypothesis of the correctness proposition (as the source's informal
"iff" is itself stated for the intended finitary construction, not an arbitrary
monotone operator) rather than as an unjustified blanket axiom.
-/

namespace Praxis.Corpus.PropSatcorrect

open Praxis.Corpus.DefSaturation
open Praxis.Corpus.DefGround
open Praxis.Corpus.DefDomain
open OmegaCompletePartialOrder

/-- **Saturation correctness** (`prop:satcorrect`): for a Datalog-style immediate-
consequence operator `T` that is additionally `ωScottContinuous` (automatic for the
finitary ground-action rule sets `def:net`/`def:saturation` model, see the file
docstring), an atom `p` lies in the saturation `sat(Π) = lfp T` iff it is produced
after some finite number `n` of rounds of firing every currently-enabled ground
action starting from the empty atom set — i.e. iff there is a finite sequence of
ground-action firings whose add-effects include `p`, matching the paper's "`p` is
reachable from `m₀` in the Petri-net sense". -/
theorem satcorrect {D : LiftedDomain} {Ob : Type} (T : ConsequenceOp D Ob)
    (hcont : ωScottContinuous (T : Set (GroundAtom D Ob) → Set (GroundAtom D Ob)))
    (p : GroundAtom D Ob) :
    Reachable T p ↔ ∃ n : ℕ, p ∈ T^[n] ∅ := by
  unfold Reachable Saturation
  rw [fixedPoints.lfp_eq_sSup_iterate T hcont, Set.iSup_eq_iUnion, Set.mem_iUnion,
    Set.bot_eq_empty]

end Praxis.Corpus.PropSatcorrect
