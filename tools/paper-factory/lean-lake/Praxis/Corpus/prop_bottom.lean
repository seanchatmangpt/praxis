import Praxis.Corpus.def_ob
import Praxis.Corpus.prop_monoid

/-!
# prop:bottom

$\adm_G(o)\ne\Rfsl\iff d_G(o)=\Adml\iff$ every $g_i\in G$ is satisfied by $o$.

`adm_G(o) ≠ Rfsl` is, by `def:ob`'s own admission predicate `is_admitted`, exactly
`d_G(o) = Adml` (a "refused" outcome is precisely a nonzero total denial word) -- so the
first `iff` is definitional and is stated directly via `DenialPolarity.is_admitted`
applied to `Obligation.totalDenial`. The real proof obligation is the second `iff`:
`d_G(o) = Adml ↔ every g_i ∈ G is satisfied by o`.

`d_G` is built (in `def:ob`) as a left fold of `DenialPolarity.compose` (bitwise OR,
`def:denialcode`) over each obligation's lane map `δ_g`, starting from `Adml`. `compose`
is OR on the underlying `UInt64`, so `compose a b = Adml ↔ a = Adml ∧ b = Adml`: this is
proved here from core's `UInt64.toNat_or` plus Mathlib's bit-testing lemma
`Nat.zero_of_testBit_eq_false` and `Nat.testBit_lor`/`Bool.or_eq_false_iff` -- no new
axiom, a direct instance of "OR of machine words is zero iff both words are zero".
Given that, the fold telescopes by list induction: `d_G(o) = Adml` iff every
`δ_{g_i}(o) = Adml`, and `δ_g(o) = Adml ↔ g.satisfies o` follows from `deltaG`'s own
`if`-definition together with the side condition that each obligation's failure lane
`g.lane` is itself nonzero (`g.lane ≠ Adml`) -- i.e. lanes are genuinely denial codes,
never the clean word, matching `def:denialcode`'s reading that `Adml` is reserved for
the empty/clean word and the seven lane constants are each nonzero. This nonzero-lane
side condition is not re-derivable from `def:ob` alone (it is a well-formedness
assumption on `Obligation`, not a theorem), so it appears as an explicit hypothesis
rather than an axiom.
-/

open DenialPolarity

namespace Praxis.Corpus.PropBottom

/-- Bitwise OR of `UInt64` words is the clean (zero) word iff both words are zero:
composing two `UInt64`s never manufactures a zero out of two nonzero words. Proved from
core's `UInt64.toNat_or` and Mathlib's `Nat.zero_of_testBit_eq_false`, not axiomatized. -/
theorem uint64_or_eq_zero_iff (a b : UInt64) :
    a ||| b = 0 ↔ a = 0 ∧ b = 0 := by
  constructor
  · intro h
    have hnat : a.toNat ||| b.toNat = 0 := by
      have := congrArg UInt64.toNat h
      simpa [UInt64.toNat_or] using this
    have ha : a.toNat = 0 := by
      apply Nat.zero_of_testBit_eq_false
      intro i
      have hbit : Nat.testBit (a.toNat ||| b.toNat) i = false := by
        rw [hnat]; simp
      rw [Nat.testBit_lor] at hbit
      exact Bool.or_eq_false_iff.mp hbit |>.1
    have hb : b.toNat = 0 := by
      apply Nat.zero_of_testBit_eq_false
      intro i
      have hbit : Nat.testBit (a.toNat ||| b.toNat) i = false := by
        rw [hnat]; simp
      rw [Nat.testBit_lor] at hbit
      exact Bool.or_eq_false_iff.mp hbit |>.2
    exact ⟨UInt64.toNat_inj.1 (by simpa using ha),
           UInt64.toNat_inj.1 (by simpa using hb)⟩
  · rintro ⟨rfl, rfl⟩
    rfl

/-- `compose a b = Adml ↔ a = Adml ∧ b = Adml`, the `DenialPolarity`-level restatement
of `uint64_or_eq_zero_iff` (`compose` is defined as OR on the underlying `val` field,
`Adml` as the zero word). -/
theorem compose_eq_Adml_iff (a b : DenialPolarity) :
    DenialPolarity.compose a b = Adml ↔ a = Adml ∧ b = Adml := by
  constructor
  · intro h
    have hval : a.val ||| b.val = 0 := congrArg DenialPolarity.val h
    obtain ⟨ha, hb⟩ := (uint64_or_eq_zero_iff a.val b.val).mp hval
    exact ⟨by cases a; simp_all [Adml], by cases b; simp_all [Adml]⟩
  · rintro ⟨rfl, rfl⟩
    simp [DenialPolarity.compose, Adml]

end Praxis.Corpus.PropBottom

open Praxis.Corpus.PropBottom Obligation

/-- `prop:bottom`: `adm_G(o) ≠ Rfsl ↔ d_G(o) = Adml ↔` every `g_i ∈ G` is satisfied by
`o`, given each obligation's failure lane is genuinely a denial code (`g.lane ≠ Adml`).
The first `iff` is `def:ob`'s own `is_admitted` predicate applied to `totalDenial`
(definitional); the second is proved here by induction on `G`, unfolding one `compose`
step at a time via `compose_eq_Adml_iff`. -/
theorem prop_bottom {Obs : Type} (G : List (Obligation Obs))
    (hlane : ∀ g ∈ G, g.lane ≠ Adml) (o : Obs) :
    (DenialPolarity.is_admitted (totalDenial G o) ↔ totalDenial G o = Adml) ∧
    (totalDenial G o = Adml ↔ ∀ g ∈ G, g.satisfies o) := by
  refine ⟨?_, ?_⟩
  · constructor
    · intro h
      cases hd : totalDenial G o with
      | mk v =>
        have : v = 0 := by
          have := h; unfold DenialPolarity.is_admitted at this; rw [hd] at this; exact this
        simp [this, Adml]
    · intro h
      unfold DenialPolarity.is_admitted
      rw [h]
      rfl
  · induction G with
    | nil => simp [totalDenial]
    | cons g0 gs ih =>
      have hlane' : ∀ g' ∈ gs, g'.lane ≠ Adml := fun g' hg' => hlane g' (List.mem_cons_of_mem g0 hg')
      have hg_lane : g0.lane ≠ Adml := hlane g0 List.mem_cons_self
      constructor
      · intro h
        have hstep : totalDenial (g0 :: gs) o
            = DenialPolarity.compose (@deltaG Obs g0 g0.dec o) (totalDenial gs o) := by
          simp [totalDenial]
        rw [hstep] at h
        obtain ⟨hδ, hrest⟩ := (compose_eq_Adml_iff _ _).mp h
        intro g' hg'
        rcases List.mem_cons.mp hg' with heq | hg'mem
        · subst heq
          by_contra hns
          have : (@deltaG Obs g' g'.dec o) = g'.lane := by
            simp [deltaG, hns]
          rw [this] at hδ
          exact hg_lane hδ
        · exact (ih hlane').mp hrest g' hg'mem
      · intro hall
        have hstep : totalDenial (g0 :: gs) o
            = DenialPolarity.compose (@deltaG Obs g0 g0.dec o) (totalDenial gs o) := by
          simp [totalDenial]
        rw [hstep]
        have hgsat : g0.satisfies o := hall g0 List.mem_cons_self
        have hδ : (@deltaG Obs g0 g0.dec o) = Adml := by simp [deltaG, hgsat]
        have hrest : totalDenial gs o = Adml :=
          (ih hlane').mpr (fun g' hg' => hall g' (List.mem_cons_of_mem g0 hg'))
        rw [hδ, hrest]
        simp [DenialPolarity.compose, Adml]
