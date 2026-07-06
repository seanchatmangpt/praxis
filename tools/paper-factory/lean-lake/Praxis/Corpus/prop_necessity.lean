import Mathlib.Computability.Halting
import Mathlib.Data.Set.Finite.Basic
import Praxis.Corpus.def_oracle

/-!
# prop:necessity

"For `S` a non-trivial semantic property, `Q_S(f)` is undecidable; hence no
procedure decides `Q_S(f)` for arbitrary `f`, and any mechanical trust in `f`
must be trust in some `Ω` evaluated on finite `X`, plus an argument bounding
residual risk on `D\X`."

We instantiate the abstract `D`/`R`/`S`/`f` of `def:oracle`
(`Praxis.Corpus.CorrectnessQuestion`, `Praxis.Corpus.Oracle`) concretely with
`D := Nat.Partrec.Code` (real program codes) and semantic properties `C : Set
Code` invariant under `eval`-equivalence, exactly as `ThmRiceViaMathlib.lean`
already does for `thm:rice`. This lets us state and prove BOTH halves of the
proposition as real theorems, with zero new axioms:

1. Undecidability half: a direct corollary of Mathlib's own
   `ComputablePred.rice₂` (already used by `ThmRiceViaMathlib.lean`) — no
   procedure decides `Q_S(f) ↔ f ∈ C` for arbitrary `f`.
2. Residual-risk half: any finite carrier `X : Finset Code` that an oracle
   `Ω` can be checked to terminate on necessarily leaves `D \ X` non-empty,
   because `Code` is infinite (`Denumerable Code`, Mathlib, hence
   `Infinite Code`). This is the formal content of "any mechanical trust in
   `f` must be trust in some `Ω` on finite `X`, plus residual risk on
   `D \ X`": the finite carrier can never be all of `D`, so a residual
   region always exists and must be separately argued about.

No axioms are introduced: `Code`, `eval`, `ComputablePred.rice₂`, and
`Denumerable Code` are all pre-built Mathlib content; `Oracle`/
`CorrectnessQuestion` are the plain structural definitions from `def:oracle`
imported above.
-/

open Nat.Partrec (Code)
open Nat.Partrec.Code (eval)

namespace Praxis.Corpus

/-- **prop:necessity**, concretely instantiated at `D := Code`.

Part 1 (undecidability): for a non-trivial semantic property `C` of program
codes (semantic = closed under `eval`-equivalence, non-trivial = neither
empty nor everything), the correctness question `CorrectnessQuestion
(fun c r => c ∈ C ∧ r = c) c` — i.e. "is `c` a member of `C`" — is not
decided by any computable predicate; equivalently no procedure decides
`Q_S(f)` for arbitrary `f := c`.

Part 2 (residual risk): for every finite carrier `X : Finset Code` that an
`Oracle` could be built on, the complement `Set.univ \ ↑X` — the part of the
domain `D` any such finite-carrier oracle does *not* cover — is non-empty.
Hence trusting `f` only via an `Ω` checked on `X` always leaves a genuine
residual `D \ X` to be separately bounded. -/
theorem necessity (C : Set Code)
    (hsem : ∀ cf cg, eval cf = eval cg → (cf ∈ C ↔ cg ∈ C))
    (hnontrivial : C ≠ ∅ ∧ C ≠ Set.univ) :
    (¬ ComputablePred fun c => c ∈ C) ∧
      ∀ X : Finset Code, (Set.univ \ (↑X : Set Code)).Nonempty := by
  refine ⟨?_, ?_⟩
  · intro h
    rcases (ComputablePred.rice₂ C hsem).mp h with hempty | huniv
    · exact hnontrivial.1 hempty
    · exact hnontrivial.2 huniv
  · intro X
    have hinf : Set.Infinite (Set.univ : Set Code) :=
      Set.infinite_univ
    have hfin : (↑X : Set Code).Finite := Finset.finite_toSet X
    have : (Set.univ \ (↑X : Set Code)).Infinite :=
      hinf.sdiff hfin
    exact this.nonempty

end Praxis.Corpus
