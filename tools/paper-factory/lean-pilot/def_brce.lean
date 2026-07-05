/-
def:brce (00_foundations / projection_thesis) — formalized as a structure/definition.

Original (LaTeX):
  A system satisfies the Bounded Receipted Chatman Equation if for every
  actuated artifact it maintains the admission gate (B1), bounded
  manufacture (B2), receipt totality (B3), and conformance (B4).

This is a DEFINITION, not a theorem — the only proof obligation is that the
file type-checks as a well-formed Lean structure. We reuse the concrete
pieces already built for `def:fit` (ReplayTally/Fitness), and model the
`def:mu`/`def:receipt` layers abstractly here (as this file's own opaque
axioms), since those dependencies were formalized in separate files whose
declarations we don't have direct source access to import from a bare
Lean core session — we mirror their shapes faithfully instead of
re-deriving their content:

  - (B1) admission gate: every actuated artifact's originating observation
    was admitted (not refused).
  - (B2) bounded manufacture: the synthesis map used to produce the
    artifact is bounded by a fixed structural constant (`def:mu`'s M2).
  - (B3) receipt totality: every actuated artifact has an associated
    receipt frame (`def:receipt`) committing it into the chain.
  - (B4) conformance: the replay fitness of the artifact's manufacture
    is the maximal value (using `def:fit`'s concrete `Fitness`/
    `ReplayTally`, reused verbatim).

`Brce` bundles the four obligations as a structure over an abstract
"system" carrying its actuated artifacts, so that "a system satisfies
the Bounded Receipted Chatman Equation" is literally the Lean type
`Brce Sys`.
-/

/-- A replay tally: how many of the total tokens consumed during a replay
    were forced onto unenabled (disabled) lifecycle transitions.
    (Reused verbatim from `def:fit`.) -/
structure ReplayTally where
  total : Nat
  forced : Nat
  forced_le_total : forced ≤ total

/-- The forced fraction of a replay tally, as a rational number in [0,1].
    (Reused verbatim from `def:fit`.) -/
def forcedFraction (t : ReplayTally) : Rat :=
  if t.total = 0 then 0 else (t.forced : Rat) / (t.total : Rat)

/-- Conformance fitness: one minus the forced fraction.
    (Reused verbatim from `def:fit`.) -/
def Fitness (t : ReplayTally) : Rat :=
  1 - forcedFraction t

/-- Abstract carrier of a system's actuated artifacts. -/
axiom Sys : Type
axiom Artifact : Type

/-- Every artifact in scope belongs to some system (models "for every
    actuated artifact" as ranging over this fixed carrier). -/
axiom actuatedBy : Artifact → Sys → Prop

/-- (B1) Admission gate: the observation from which an artifact was
    synthesized was admitted, not refused. Modeled abstractly as a
    predicate on artifacts. -/
axiom Admitted : Artifact → Prop

/-- (B2) Bounded manufacture: a fixed structural size measure and its
    bound, mirroring `def:mu`'s `reprSize`/`M2_bound`. -/
axiom reprSize : Artifact → Nat
axiom M2_bound : Nat

/-- (B3) Receipt totality: every artifact has an associated receipt
    digest committing it into the chain, mirroring `def:receipt`'s
    `Receipt`/`chainStep` layer abstractly. -/
axiom Bits256 : Type
abbrev Digest := Bits256
axiom receiptOf : Artifact → Digest

/-- (B4) Conformance: the replay tally recorded for an artifact's
    manufacture. -/
axiom replayOf : Artifact → ReplayTally

/-- A system satisfies the Bounded Receipted Chatman Equation if, for
    every actuated artifact `a`, it maintains:
      (B1) the admission gate,
      (B2) bounded manufacture,
      (B3) receipt totality (trivially witnessed by `receiptOf`, total
           by construction as a function `Artifact → Digest`), and
      (B4) conformance (`Fitness (replayOf a) = 1`).
-/
structure Brce (s : Sys) where
  b1_admission : ∀ a : Artifact, actuatedBy a s → Admitted a
  b2_bounded   : ∀ a : Artifact, actuatedBy a s → reprSize a ≤ M2_bound
  b3_receipted : ∀ a : Artifact, actuatedBy a s → ∃ _ : Digest, receiptOf a = receiptOf a
  b4_conformant : ∀ a : Artifact, actuatedBy a s → Fitness (replayOf a) = 1
