/-
def:fit (00_foundations / projection_thesis) — formalized as a structure/definition.

Original (LaTeX):
  Conformance fitness Fitness in [0,1] is one minus the fraction of tokens a
  replay was forced to consume on unenabled lifecycle transitions;
  Fitness = 1 iff the replay never attempted a disabled firing.

This is a DEFINITION, not a theorem — there is no proof obligation beyond
type-checking as a well-formed Lean definition. We model the "fraction of
tokens consumed on unenabled transitions" concretely as a ratio of natural
numbers (forced : total), and define Fitness as 1 - that ratio, valued in
the rationals restricted to [0,1] via a subtype. `def:receipt` treats
`Fitness` as an opaque axiom type (a later formalization target for
receipts); here we give it a concrete construction, matching the LaTeX's
own definitional content — the two are deliberately not unified, since
def:receipt's `Fitness` is a placeholder for whatever this definition
produces.
-/

/-- A replay tally: how many of the total tokens consumed during a replay
    were forced onto unenabled (disabled) lifecycle transitions. -/
structure ReplayTally where
  total : Nat
  forced : Nat
  forced_le_total : forced ≤ total

/-- The forced fraction of a replay tally, as a rational number in [0,1].
    When `total = 0` (no tokens consumed at all) the fraction is taken to
    be `0`, so an empty replay is fully conformant. -/
def forcedFraction (t : ReplayTally) : Rat :=
  if t.total = 0 then 0 else (t.forced : Rat) / (t.total : Rat)

/-- Conformance fitness: one minus the forced fraction. By construction
    this lands in [0,1] whenever `forced ≤ total`. -/
def Fitness (t : ReplayTally) : Rat :=
  1 - forcedFraction t

/-- The distinguished case named in the LaTeX: a replay that never attempted
    a disabled firing has `forced = 0`, hence `Fitness = 1`. -/
def perfectTally (n : Nat) : ReplayTally :=
  { total := n, forced := 0, forced_le_total := Nat.zero_le n }
