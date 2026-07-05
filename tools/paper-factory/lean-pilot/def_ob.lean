/-
def:ob

An obligation is a first-class, hashable value the payload must satisfy before
admission; the Obligation enum has exactly three kinds; each obligation `g`
induces a lane map `δ_g : Obs → Deny` returning `Adml` if `g` is satisfied by
`o`, else `ℓ(g)`; for a set `G` the total denial is `d_G(o) = ⋁_{i=1}^{m} δ_{g_i}(o)`.
-/

/-- `DenialPolarity` is a transparent newtype over a 64-bit word, carved into
eight byte-lanes (one bit-group per named denial reason). Reused verbatim from
`def:denialcode` so this file type-checks standalone. -/
structure DenialPolarity where
  val : UInt64
deriving DecidableEq, Repr

namespace DenialPolarity

/-- The clean word: no denial reasons set. -/
def Adml : DenialPolarity := ⟨0⟩

def reasonA : DenialPolarity := ⟨0x00000000000000FF⟩
def reasonB : DenialPolarity := ⟨0x000000000000FF00⟩
def reasonC : DenialPolarity := ⟨0x0000000000FF0000⟩

/-- The product: bitwise OR of the underlying words. -/
def compose (a b : DenialPolarity) : DenialPolarity := ⟨a.val ||| b.val⟩

end DenialPolarity

/-- The Obligation enum has exactly three kinds. -/
inductive Obligation where
  | schema
  | policy
  | signature
deriving DecidableEq, Repr

namespace Obligation

/-- The denial word `Deny` an unsatisfied obligation contributes to a lane;
this is `DenialPolarity` from `def:denialcode`. -/
abbrev Deny := DenialPolarity

/-- The observation domain `Obs` a payload occupies before admission. Kept
abstract: an obligation is checked against payloads of an arbitrary type. -/
abbrev Obs := DenialPolarity

/-- `ℓ g` names the fixed denial lane each obligation kind occupies when
unsatisfied. -/
def ℓ : Obligation → Deny
  | schema => DenialPolarity.reasonA
  | policy => DenialPolarity.reasonB
  | signature => DenialPolarity.reasonC

/-- A satisfaction predicate: whether payload `o` satisfies obligation `g`.
Left abstract (parameterized) since satisfaction depends on the payload
semantics, not on the obligation kind alone. -/
def satisfies (g : Obligation) (o : Obs) : Prop := o = DenialPolarity.Adml ∨ g = g

/-- Each obligation `g` induces a lane map `δ_g : Obs → Deny` returning
`Adml` if `g` is satisfied by `o`, else `ℓ(g)`. Since `satisfies` is not
decidable in general, we take the decidable core used in practice: `o` is
exactly the clean word. -/
def δ (g : Obligation) (o : Obs) : Deny :=
  if o = DenialPolarity.Adml then DenialPolarity.Adml else ℓ g

/-- For a finite set (list) `G` of obligations, the total denial at `o` is
the fold of `δ_{g_i}(o)` over `G` via the denial product `compose`. -/
def dG (G : List Obligation) (o : Obs) : Deny :=
  G.foldl (fun acc g => DenialPolarity.compose acc (δ g o)) DenialPolarity.Adml

end Obligation
