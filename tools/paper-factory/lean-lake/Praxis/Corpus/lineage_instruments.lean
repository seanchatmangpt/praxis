/-
Label: lineage:instruments
Kind: lineage (methodological/narrative statement, not a formal mathematical claim)

Source statement:
"The LHC does not record every collision; its trigger system encodes, in advance,
the signatures of events worth keeping. The EHT did not photograph a black hole;
it reconstructed one from lawful projections gathered by an instrument designed
around a predicted signal. The discipline common to both: enumerate the event
geometry before the event, instrument for exactly those signatures, and treat
anything outside the geometry as a first-class finding -- never as noise to be
discarded silently."

This is a lineage/methodological note, not a mathematical statement with a
formalizable proof obligation. It is migrated as a structural record: an
`Instrument` composed from pre-built Mathlib/core types (no new axioms), whose
fields capture the discipline described above -- a predicate for the enumerated
event geometry, an encoding of what counts as a "signature", and a totality
witness that nothing outside the geometry is silently discarded (i.e. everything
is classified as either a signature match or an explicit finding, never dropped).

No axioms are introduced: `Instrument` is built entirely from `Prop`-valued
predicates and pre-built `Or`/`Decidable` structure already in core/Mathlib.
-/

namespace Praxis.Corpus.LineageInstruments

/-- An instrument designed around an enumerated event geometry `geometry`,
    a decidable "signature" predicate `isSignature`, and a totality obligation:
    every event is either a signature match or is explicitly classified as a
    finding (`isFinding`) -- nothing is silently discarded. -/
structure Instrument (Event : Type) where
  /-- the enumerated event geometry, decided in advance of any event -/
  geometry : Event → Prop
  /-- the signature predicate the instrument is built to detect -/
  isSignature : Event → Prop
  /-- events outside the geometry are treated as first-class findings -/
  isFinding : Event → Prop
  /-- totality: nothing falls outside signature-or-finding, i.e. nothing is
      discarded silently -/
  total : ∀ e : Event, isSignature e ∨ isFinding e

end Praxis.Corpus.LineageInstruments
