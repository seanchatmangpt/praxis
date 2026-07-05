/-
def:pipeline

Let Stage be the set of admission stages; fix `o` and let `φ_o : Stage → Deny`
send a stage to the denial word it emits on `o`; a pipeline is a finite
sequence `w = s_1 ⋯ s_k ∈ Stage*`, an element of the free monoid on Stage;
its aggregate denial is `Φ_o(w) = ⋁_{i=1}^{k} φ_o(s_i)`, `Φ_o(ε) = Adml`.
-/

/-- `DenialPolarity` is a transparent newtype over a 64-bit word, carved into
eight byte-lanes (one bit-group per named denial reason). Reused verbatim from
`def:denialcode` / `def:ob` so this file type-checks standalone. -/
structure DenialPolarity where
  val : UInt64
deriving DecidableEq, Repr

namespace DenialPolarity

/-- The clean word: no denial reasons set. -/
def Adml : DenialPolarity := ⟨0⟩

/-- The product: bitwise OR of the underlying words. -/
def compose (a b : DenialPolarity) : DenialPolarity := ⟨a.val ||| b.val⟩

end DenialPolarity

/-- The denial word `Deny` a stage contributes on a fixed payload `o`; this is
`DenialPolarity` from `def:denialcode`/`def:ob`. -/
abbrev Deny := DenialPolarity

/-- `Stage`, the set of admission stages, kept abstract. -/
opaque Stage : Type

/-- A pipeline is a finite sequence `w = s_1 ⋯ s_k ∈ Stage*`, i.e. an element
of the free monoid on `Stage` — a `List Stage`. The empty pipeline `ε` is
`[]`. -/
abbrev Pipeline := List Stage

/-- The aggregate denial `Φ_o(w) = ⋁_{i=1}^{k} φ_o(s_i)`, folded left-to-right
via the denial product `compose`, with `Φ_o(ε) = Adml`. -/
def aggregateDenial (φ : Stage → Deny) (w : Pipeline) : Deny :=
  w.foldl (fun acc s => DenialPolarity.compose acc (φ s)) DenialPolarity.Adml

/-- `Φ_o(ε) = Adml`, the base case of the definition, holds definitionally. -/
example (φ : Stage → Deny) : aggregateDenial φ [] = DenialPolarity.Adml := rfl
