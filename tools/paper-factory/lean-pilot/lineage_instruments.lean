/-
Label: lineage:instruments
Kind: lineage

Statement (informal, from thesis corpus):
The LHC does not record every collision; its trigger system encodes, in advance,
the signatures of events worth keeping. The EHT did not photograph a black hole;
it reconstructed one from lawful projections gathered by an instrument designed
around a predicted signal. The discipline common to both: enumerate the event
geometry before the event, instrument for exactly those signatures, and treat
anything outside the geometry as a first-class finding -- never as noise to be
discarded silently.

This is a lineage/discipline statement, not a theorem: we encode it as a
definition capturing the shared structure (a pre-registered event geometry,
an instrument keyed to that geometry, and an explicit "outside geometry"
case that is a first-class finding rather than discarded noise). No proof
obligation beyond type-checking.
-/

/-- An instrumented discipline over a space of possible events `Event`,
    parameterized by the type `Finding` used to record first-class findings
    for events outside the pre-registered geometry. -/
structure InstrumentedDiscipline (Event : Type u) (Finding : Type v) where
  /-- The event geometry enumerated *before* the event: which events are
      "worth keeping" is decided in advance. -/
  geometry : Event → Prop
  /-- The instrument is designed around exactly this predicted geometry:
      it decides, for each event, whether it lies in the geometry. -/
  instrument : (e : Event) → Decidable (geometry e)
  /-- Anything outside the geometry is not discarded silently: it is
      recorded as a first-class finding. -/
  outsideGeometryFinding : (e : Event) → ¬ geometry e → Finding

/-- The defining discipline: an event is either captured by the
    pre-registered geometry, or it produces a first-class finding.
    Nothing is silently discarded. -/
def InstrumentedDiscipline.noSilentDiscard
    {Event : Type u} {Finding : Type v}
    (D : InstrumentedDiscipline Event Finding) (e : Event) :
    D.geometry e ∨ Nonempty Finding :=
  match D.instrument e with
  | isTrue h => Or.inl h
  | isFalse h => Or.inr ⟨D.outsideGeometryFinding e h⟩
