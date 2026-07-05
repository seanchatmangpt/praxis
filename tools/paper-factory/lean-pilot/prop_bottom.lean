/-
prop:bottom (depends on def:ob, prop:monoid)

LaTeX: $\adm_G(o)\ne\Rfsl\iff d_G(o)=\Adml\iff$ every $g_i\in G$ is satisfied by $o$.

`def:ob` (already verified in `def_ob.lean`) does not itself define `adm_G` or
`Rfsl`, only the denial fold `d_G(o) = Obligation.dG`. The admission-succeeds
statement `adm_G(o) ≠ Rfsl` is definitionally the same fact as `d_G(o) = Adml`
in this development (admission fails exactly when some lane is set), so the
real mathematical content still open to formalize here is the second
equivalence: `d_G(o) = Adml` iff every obligation in the battery is satisfied
at `o`, where "satisfied" is read through the lane map `δ` already fixed in
`def:ob` (`δ g o = Adml` exactly when `g` contributes no denial at `o`).

This is proved here as a genuine theorem by induction on the obligation list,
using that `DenialPolarity.compose` (bitwise OR on the underlying `UInt64`)
is zero iff both arguments are zero — the same OR-lattice fact `prop:monoid`
establishes abstractly for `Deny_n` bitvectors, specialized to the concrete
`DenialPolarity` word `def:ob` uses.
-/

/-- `DenialPolarity` is a transparent newtype over a 64-bit word, carved into
eight byte-lanes (one bit-group per named denial reason). Reused verbatim from
`def_ob.lean` so this file type-checks standalone. -/
structure DenialPolarity where
  val : UInt64
deriving DecidableEq, Repr

namespace DenialPolarity

def Adml : DenialPolarity := ⟨0⟩

def reasonA : DenialPolarity := ⟨0x00000000000000FF⟩
def reasonB : DenialPolarity := ⟨0x000000000000FF00⟩
def reasonC : DenialPolarity := ⟨0x0000000000FF0000⟩

def compose (a b : DenialPolarity) : DenialPolarity := ⟨a.val ||| b.val⟩

end DenialPolarity

inductive Obligation where
  | schema
  | policy
  | signature
deriving DecidableEq, Repr

namespace Obligation

abbrev Deny := DenialPolarity
abbrev Obs := DenialPolarity

def ℓ : Obligation → Deny
  | schema => DenialPolarity.reasonA
  | policy => DenialPolarity.reasonB
  | signature => DenialPolarity.reasonC

def δ (g : Obligation) (o : Obs) : Deny :=
  if o = DenialPolarity.Adml then DenialPolarity.Adml else ℓ g

def dG (G : List Obligation) (o : Obs) : Deny :=
  G.foldl (fun acc g => DenialPolarity.compose acc (δ g o)) DenialPolarity.Adml

/-- Bitwise OR of two `UInt64`s is `0` iff both are `0`, via the underlying
`BitVec` (`BitVec.or_eq_zero_iff` from Lean core). -/
theorem uint64_or_eq_zero_iff (a b : UInt64) :
    (a ||| b) = 0 ↔ a = 0 ∧ b = 0 := by
  constructor
  · intro h
    have hval : a.toBitVec ||| b.toBitVec = 0#64 := by
      have := congrArg UInt64.toBitVec h
      simpa using this
    obtain ⟨ha, hb⟩ := BitVec.or_eq_zero_iff.mp hval
    exact ⟨UInt64.eq_of_toBitVec_eq (by simpa using ha),
           UInt64.eq_of_toBitVec_eq (by simpa using hb)⟩
  · rintro ⟨rfl, rfl⟩
    simp

/-- `d = Adml` iff its underlying word is `0`. -/
theorem eq_Adml_iff_val_eq_zero (d : DenialPolarity) :
    d = DenialPolarity.Adml ↔ d.val = 0 := by
  constructor
  · intro h; subst h; rfl
  · intro h
    have heta : d = (⟨d.val⟩ : DenialPolarity) := rfl
    rw [heta, h]; rfl

/-- `compose a b = Adml` iff both `a` and `b` are `Adml`. -/
theorem compose_eq_Adml_iff (a b : DenialPolarity) :
    DenialPolarity.compose a b = DenialPolarity.Adml ↔
      a = DenialPolarity.Adml ∧ b = DenialPolarity.Adml := by
  rw [eq_Adml_iff_val_eq_zero, eq_Adml_iff_val_eq_zero, eq_Adml_iff_val_eq_zero]
  show a.val ||| b.val = 0 ↔ a.val = 0 ∧ b.val = 0
  exact uint64_or_eq_zero_iff a.val b.val

/-- "Every obligation in `G` is satisfied by `o`", read through the lane
map `δ` already fixed in `def:ob`: each `g` contributes no denial at `o`. -/
def AllSatisfied (G : List Obligation) (o : Obs) : Prop :=
  ∀ g ∈ G, δ g o = DenialPolarity.Adml

/-- Auxiliary: folding `compose` starting from an already-`Adml`
accumulator lands on `Adml` iff the accumulator was `Adml` and every
remaining obligation is satisfied. -/
theorem foldl_compose_eq_Adml_iff (G : List Obligation) (o : Obs)
    (acc : DenialPolarity) :
    G.foldl (fun acc g => DenialPolarity.compose acc (δ g o)) acc =
        DenialPolarity.Adml ↔
      acc = DenialPolarity.Adml ∧ AllSatisfied G o := by
  induction G generalizing acc with
  | nil => simp [AllSatisfied]
  | cons g gs ih =>
    simp only [List.foldl_cons]
    rw [ih, compose_eq_Adml_iff]
    constructor
    · rintro ⟨⟨hacc, hg⟩, hall⟩
      exact ⟨hacc, fun g' hg' => by
        rcases List.mem_cons.mp hg' with h | h
        · exact h ▸ hg
        · exact hall g' h⟩
    · rintro ⟨hacc, hall⟩
      refine ⟨⟨hacc, hall g (List.mem_cons_self g gs)⟩, ?_⟩
      exact fun g' hg' => hall g' (List.mem_cons_of_mem g hg')

/-- **prop:bottom** (the `d_G(o) = Adml ⟺ every gᵢ satisfied by o` half of
the statement, the mathematical content still to prove after `def:ob`): the
total denial fold `d_G(o)` is `Adml` exactly when every obligation in the
battery `G` is satisfied by `o`. -/
theorem dG_eq_Adml_iff (G : List Obligation) (o : Obs) :
    dG G o = DenialPolarity.Adml ↔ AllSatisfied G o := by
  unfold dG
  rw [foldl_compose_eq_Adml_iff]
  simp

end Obligation
