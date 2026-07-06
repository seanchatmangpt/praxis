import Praxis.Corpus.prop_bottom
import Praxis.Corpus.prop_monoid

/-!
# thm:mono

Let $G\subseteq G'$ be obligation sets; then $d_G(o)\preceq d_{G'}(o)$ for every
observation $o$, hence $\Adm_{G'}\subseteq\Adm_G$: adding obligations can only enlarge
denial and shrink admission, never admit an observation a smaller obligation set
refused.

This is proved in the `Obligation`/`DenialPolarity` framework of `def:ob`/`prop:bottom`
(not the abstract `Deny n` framework of `prop:monoid`, which has no obligations of its
own to be a *subset* of) -- `G ⊆ G'` is realized as `List.Sublist G G'` (dropping zero or
more obligations from `G'` yields `G`), matching the set-inclusion reading of the
statement while working with the corpus's actual `List (Obligation Obs)` carrier.

`d_G(o) ≼ d_{G'}(o)` is stated, exactly as `prop:monoid` characterizes `≼` for its own
join operation (`d ≼ d' ↔ d ⊔ d' = d'`), as `compose (totalDenial G o) (totalDenial G' o)
= totalDenial G' o` -- `compose` (bitwise OR, `def:denialcode`) is `DenialPolarity`'s own
join, so this is the same lub-characterization of `≼`, transported to the `UInt64` carrier
`prop:monoid` does not cover. `compose`'s idempotence/commutativity/associativity (the
same three monoid facts `prop:monoid` proves generically for `Deny n`'s `Bool`-lattice)
are re-derived here for `UInt64` from Mathlib's `Nat.lor_assoc`/`Nat.lor_comm` plus
`Nat.testBit_lor`/`Bool.or_self` for idempotence, lifted through `UInt64.toNat_inj` -- no
new axioms, only the pre-built `Nat` bitwise lemma set already used by `prop:bottom`.

`Adm_{G'} ⊆ Adm_G` is then the direct corollary: `is_admitted` unfolds to `= Adml`
(`def:denialcode`), and `compose a b = b` together with `b = Adml` forces `a = Adml` via
`prop:bottom`'s own `compose_eq_Adml_iff` (`compose a b = Adml ↔ a = Adml ∧ b = Adml`,
since `b = Adml` here already, `compose a b = b = Adml`).
-/

open DenialPolarity Obligation Praxis.Corpus.PropBottom

namespace Praxis.Corpus.ThmMono

/-- `compose` is idempotent on `UInt64`-backed `DenialPolarity`: `a ||| a = a`, proved
from `Nat.testBit_lor`/`Bool.or_self` (no new axiom, direct bit-level fact) lifted through
`UInt64.toNat_inj`. -/
theorem compose_self (a : DenialPolarity) : DenialPolarity.compose a a = a := by
  have hnat : a.val.toNat ||| a.val.toNat = a.val.toNat := by
    apply Nat.eq_of_testBit_eq
    intro i
    rw [Nat.testBit_lor, Bool.or_self]
  have : a.val ||| a.val = a.val := by
    apply UInt64.toNat_inj.1
    simpa [UInt64.toNat_or] using hnat
  cases a
  simp [DenialPolarity.compose, this]

/-- `compose` is commutative, from `Nat.lor_comm` lifted through `UInt64.toNat_inj`
(reusing the same `toNat_or` bridge `prop:bottom` already established). -/
theorem compose_comm (a b : DenialPolarity) :
    DenialPolarity.compose a b = DenialPolarity.compose b a := by
  have hnat : a.val.toNat ||| b.val.toNat = b.val.toNat ||| a.val.toNat := Nat.lor_comm _ _
  have : a.val ||| b.val = b.val ||| a.val := by
    apply UInt64.toNat_inj.1
    simpa [UInt64.toNat_or] using hnat
  cases a; cases b
  simp [DenialPolarity.compose, this]

/-- `compose` is associative, from `Nat.lor_assoc` lifted through `UInt64.toNat_inj`. -/
theorem compose_assoc (a b c : DenialPolarity) :
    DenialPolarity.compose (DenialPolarity.compose a b) c
      = DenialPolarity.compose a (DenialPolarity.compose b c) := by
  have hnat : (a.val.toNat ||| b.val.toNat) ||| c.val.toNat
      = a.val.toNat ||| (b.val.toNat ||| c.val.toNat) := Nat.lor_assoc _ _ _
  have : (a.val ||| b.val) ||| c.val = a.val ||| (b.val ||| c.val) := by
    apply UInt64.toNat_inj.1
    simpa [UInt64.toNat_or] using hnat
  cases a; cases b; cases c
  simp [DenialPolarity.compose, this]

/-- The join-semilattice absorption step used by the induction below: if `compose a b = b`
(the `≼`-witness, per `prop:monoid`'s `d ≼ d' ↔ d ⊔ d' = d'`), then OR-ing in any extra
term `x` on both sides preserves it: `compose a (compose x b) = compose x b`. Pure algebra
from `compose_comm`/`compose_assoc`, no new axiom. -/
theorem compose_absorb {a b : DenialPolarity} (x : DenialPolarity) (h : compose a b = b) :
    compose a (compose x b) = compose x b := by
  calc compose a (compose x b) = compose a (compose b x) := by rw [compose_comm x b]
    _ = compose (compose a b) x := (compose_assoc a b x).symm
    _ = compose b x := by rw [h]
    _ = compose x b := compose_comm b x

/-- The two-sided version of `compose_absorb`, used for the `cons_cons` (shared-head)
step of the induction below: from `compose a b = b` and any `x`, `compose (compose x a)
(compose x b) = compose x b`. Pure algebra from `compose_assoc`/`compose_absorb`/
`compose_self`, stated with all three of `x a b` left to unify against the goal so the
induction step below does not need to name the underlying obligation lists. -/
theorem compose_absorb2 {a b : DenialPolarity} (x : DenialPolarity) (h : compose a b = b) :
    compose (compose x a) (compose x b) = compose x b := by
  calc compose (compose x a) (compose x b)
      = compose x (compose a (compose x b)) := compose_assoc x a (compose x b)
    _ = compose x (compose x b) := by rw [compose_absorb x h]
    _ = compose (compose x x) b := (compose_assoc x x b).symm
    _ = compose x b := by rw [compose_self]

/-- `≼` (the `prop:monoid` lub-characterization of `⊔`, transported to `DenialPolarity`)
holds between `totalDenial G o` and `totalDenial G' o` whenever `G` is a sublist of `G'`
(i.e. `G ⊆ G'` as obligation sets): `compose (totalDenial G o) (totalDenial G' o)
= totalDenial G' o`. Proved by induction on `List.Sublist`; the two inductive steps are
exactly `compose_absorb`/`compose_absorb2` applied to the one-step fold unfolding, with
Lean unifying the underlying obligation lists against the goal rather than us naming them. -/
theorem totalDenial_mono {Obs : Type} {G G' : List (Obligation Obs)}
    (hsub : G.Sublist G') (o : Obs) :
    DenialPolarity.compose (totalDenial G o) (totalDenial G' o) = totalDenial G' o := by
  induction hsub with
  | slnil => simpa [totalDenial] using compose_self (Adml)
  | cons _ _ ih =>
      simp only [totalDenial, List.foldr_cons] at ih ⊢
      exact compose_absorb _ ih
  | cons_cons _ _ ih =>
      simp only [totalDenial, List.foldr_cons] at ih ⊢
      exact compose_absorb2 _ ih

/-- `thm:mono`: `G ⊆ G'` (realized as `List.Sublist G G'`) implies `d_G(o) ≼ d_{G'}(o)`
for every `o` (stated via `compose`'s lub-characterization, matching `prop:monoid`), hence
`Adm_{G'} ⊆ Adm_G`: every observation admitted by the larger obligation set `G'` was
already admitted by the smaller set `G`. -/
theorem thm_mono {Obs : Type} {G G' : List (Obligation Obs)} (hsub : G.Sublist G') :
    (∀ o : Obs,
      DenialPolarity.compose (totalDenial G o) (totalDenial G' o) = totalDenial G' o) ∧
    (∀ o : Obs, DenialPolarity.is_admitted (totalDenial G' o) →
      DenialPolarity.is_admitted (totalDenial G o)) := by
  refine ⟨fun o => totalDenial_mono hsub o, ?_⟩
  intro o hadm
  have hjoin := totalDenial_mono hsub o
  unfold DenialPolarity.is_admitted at hadm ⊢
  have hval : (totalDenial G o).val ||| (totalDenial G' o).val = (totalDenial G' o).val :=
    congrArg DenialPolarity.val hjoin
  rw [hadm] at hval
  simpa using hval

end Praxis.Corpus.ThmMono
