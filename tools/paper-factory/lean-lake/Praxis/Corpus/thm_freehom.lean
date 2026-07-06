import Praxis.Corpus.def_pipeline
import Praxis.Corpus.prop_monoid

/-!
# thm:freehom

`Φ_o : (Stage*, ·, ε) → (Deny, compose, Adml)` is the unique monoid homomorphism
extending `φ_o` along `Stage ↪ Stage*`; because the target is commutative and
idempotent, `Φ_o(w)` depends only on the set of distinct stage-denials occurring in
`w`, invariant under reordering and repetition.

`Φ_o` itself is `Pipeline.aggregateDenial φ`, already migrated in `def:pipeline` as
the fold of `DenialPolarity.compose` over a pipeline; `Stage*` is `Pipeline.Seq
Stage = List Stage`, Mathlib/core's free monoid on `Stage` (`[]` is `ε`, `++` is the
monoid product), reused rather than re-derived, matching `def:pipeline`'s own
justification. `prop:monoid` supplies the corpus's proof pattern for "commutative,
idempotent target" -- there it is proved for the `Fin n → Bool` encoding of `Deny`;
here the pipeline target is the `DenialPolarity`/`UInt64` encoding from
`def:denialcode`, so the same three algebraic facts (commutativity, associativity,
idempotence of `compose`, plus `Adml`'s two-sided identity law) are established
directly for `UInt64`'s `|||`, by unfolding to `BitVec`'s already-proved
`or_comm`/`or_assoc`/`or_self` (no new axioms, no bit-level re-derivation: cited
Mathlib/core lemmas only). No axiom is introduced anywhere in this file.

This file proves, in order: (1) `Φ_o` is a monoid homomorphism (`ε ↦ Adml`,
`++ ↦ compose`); (2) it is the *unique* such homomorphism agreeing with `φ_o` on
singletons; (3) it is invariant under reordering (`List.Perm`) and under repetition
(a repeated element may be contracted), which together witness "depends only on the
set of distinct stage-denials occurring in `w`".
-/

namespace Pipeline

open DenialPolarity

variable {Stage : Type} (φ : Stage → DenialPolarity)

/-! ## Algebraic facts about the target `(DenialPolarity, compose, Adml)` -/

theorem compose_comm (a b : DenialPolarity) : compose a b = compose b a := by
  cases a with | mk a =>
  cases b with | mk b =>
  cases a with | ofBitVec a =>
  cases b with | ofBitVec b =>
  show DenialPolarity.mk (UInt64.ofBitVec (a ||| b)) = DenialPolarity.mk (UInt64.ofBitVec (b ||| a))
  congr 2
  exact BitVec.or_comm a b

theorem compose_assoc (a b c : DenialPolarity) :
    compose (compose a b) c = compose a (compose b c) := by
  cases a with | mk a =>
  cases b with | mk b =>
  cases c with | mk c =>
  cases a with | ofBitVec a =>
  cases b with | ofBitVec b =>
  cases c with | ofBitVec c =>
  show DenialPolarity.mk (UInt64.ofBitVec ((a ||| b) ||| c))
      = DenialPolarity.mk (UInt64.ofBitVec (a ||| (b ||| c)))
  congr 2
  exact BitVec.or_assoc a b c

theorem compose_idem (a : DenialPolarity) : compose a a = a := by
  cases a with | mk a =>
  cases a with | ofBitVec a =>
  show DenialPolarity.mk (UInt64.ofBitVec (a ||| a)) = DenialPolarity.mk (UInt64.ofBitVec a)
  congr 2
  exact BitVec.or_self

theorem compose_adml_right (a : DenialPolarity) : compose a Adml = a := by
  cases a with | mk a => simp [compose, Adml]

theorem compose_adml_left (a : DenialPolarity) : compose Adml a = a := by
  rw [compose_comm]; exact compose_adml_right a

/-! ## `Φ_o = aggregateDenial φ` is a monoid homomorphism `(Seq Stage, ++, []) →
(DenialPolarity, compose, Adml)` -/

theorem aggregateDenial_nil : aggregateDenial φ [] = Adml := rfl

theorem aggregateDenial_cons (s : Stage) (w : Seq Stage) :
    aggregateDenial φ (s :: w) = compose (φ s) (aggregateDenial φ w) := rfl

/-- General fold-splitting fact underlying the homomorphism law: folding `compose`
started from an arbitrary accumulator `c` is the same as folding from `Adml` and
then `compose`-ing on `c`, by induction on the pipeline using associativity and
the left identity law. -/
theorem foldr_eq_compose (u : Seq Stage) (c : DenialPolarity) :
    u.foldr (fun s acc => compose (φ s) acc) c
      = compose (u.foldr (fun s acc => compose (φ s) acc) Adml) c := by
  induction u with
  | nil => simp [compose_adml_left]
  | cons s u ih =>
    show compose (φ s) (u.foldr _ c) = compose (compose (φ s) (u.foldr _ Adml)) c
    rw [ih, compose_assoc]

/-- `Φ_o` respects the free-monoid product: `Φ_o(u ++ v) = compose (Φ_o u) (Φ_o v)`. -/
theorem aggregateDenial_append (u v : Seq Stage) :
    aggregateDenial φ (u ++ v) = compose (aggregateDenial φ u) (aggregateDenial φ v) := by
  show (u ++ v).foldr (fun s acc => compose (φ s) acc) Adml
      = compose (u.foldr (fun s acc => compose (φ s) acc) Adml) (aggregateDenial φ v)
  rw [List.foldr_append]
  exact foldr_eq_compose φ u (aggregateDenial φ v)

/-- `Φ_o` extends `φ_o` along the singleton embedding `Stage ↪ Stage*`. -/
theorem aggregateDenial_singleton (s : Stage) : aggregateDenial φ [s] = φ s := by
  show compose (φ s) Adml = φ s
  exact compose_adml_right (φ s)

/-! ## Uniqueness -/

/-- `Φ_o` is the *unique* monoid homomorphism `(Seq Stage, ++, []) →
(DenialPolarity, compose, Adml)` extending `φ_o` along the singleton embedding:
any `Ψ` satisfying the same three laws agrees with `aggregateDenial φ` everywhere. -/
theorem aggregateDenial_unique (Ψ : Seq Stage → DenialPolarity)
    (h0 : Ψ [] = Adml)
    (hop : ∀ u v, Ψ (u ++ v) = compose (Ψ u) (Ψ v))
    (hext : ∀ s, Ψ [s] = φ s) :
    ∀ w, Ψ w = aggregateDenial φ w := by
  intro w
  induction w with
  | nil => rw [h0, aggregateDenial_nil]
  | cons s w ih =>
    have hsplit : (s :: w) = [s] ++ w := rfl
    rw [hsplit, hop, hext, ih]
    exact (aggregateDenial_cons φ s w).symm

/-! ## Invariance: `Φ_o(w)` depends only on the set of distinct stage-denials in
`w` -- invariant under reordering (permutation) and under repetition (a repeated
stage may be contracted), because the target `compose` is commutative, associative
and idempotent. -/

/-- Reordering invariance: `Φ_o` is invariant under permutation of the pipeline. -/
theorem aggregateDenial_perm {u v : Seq Stage} (h : u.Perm v) :
    aggregateDenial φ u = aggregateDenial φ v := by
  apply h.foldr_eq'
  intro x _ y _ z
  show compose (φ y) (compose (φ x) z) = compose (φ x) (compose (φ y) z)
  rw [← compose_assoc, ← compose_assoc, compose_comm (φ y) (φ x)]

/-- Repetition invariance: an immediately-repeated stage may be contracted without
changing `Φ_o`, since `compose` is idempotent. -/
theorem aggregateDenial_dup (s : Stage) (w : Seq Stage) :
    aggregateDenial φ (s :: s :: w) = aggregateDenial φ (s :: w) := by
  show compose (φ s) (compose (φ s) (aggregateDenial φ w)) = compose (φ s) (aggregateDenial φ w)
  rw [← compose_assoc, compose_idem]


end Pipeline
