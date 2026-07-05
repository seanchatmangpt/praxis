/- def:cost
   The router's CostVector is the tuple
     CostV = (c0,...,c5)
           = (admitted_bar, risk, attention_seconds, tokens, latency, switches)
   ordered lexicographically, cheapest-first. The refused vector
     refused() = (unadmitted, 255, ∞, ∞, ∞, 255)
   is the top element.

   We model each coordinate that can take the value ∞ as `Option Nat`,
   with `none` standing for ∞ and `some n` standing for the finite value `n`.
   The admission flag `c0` (written `admitted_bar` in the source, i.e. "not
   admitted") is modeled as a `Bool`, where `true` means "unadmitted".
-/

/-- A single cost coordinate: either a finite natural number, or ∞. -/
abbrev ExtNat := Option Nat

/-- The router's cost vector `CostV = (c0,...,c5)`. -/
structure CostVector where
  /-- c0 : whether the request was *not* admitted (`admitted_bar`). -/
  unadmitted        : Bool
  /-- c1 : risk score. -/
  risk              : ExtNat
  /-- c2 : attention seconds consumed. -/
  attentionSeconds  : ExtNat
  /-- c3 : tokens consumed. -/
  tokens            : ExtNat
  /-- c4 : latency. -/
  latency           : ExtNat
  /-- c5 : number of switches. -/
  switches          : Nat

/-- The distinguished refused vector `refused() = (unadmitted, 255, ∞, ∞, ∞, 255)`. -/
def refused : CostVector :=
  { unadmitted := true
  , risk := some 255
  , attentionSeconds := none
  , tokens := none
  , latency := none
  , switches := 255 }
