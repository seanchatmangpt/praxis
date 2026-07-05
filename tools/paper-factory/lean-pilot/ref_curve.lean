/-
Label: ref:curve
Kind: refusal

Statement (informal):
No novelty-curve-under-faults measurement -- yet: member-level fault injection
recovers without re-running solver work, so a work-proxy re-measurement would
overstate the cache dividend -- the flattering error. The claim is withheld
until node-level injection is wired through the fleet path; the receipt file
names this in its own deferred list.

Formalization approach:
A `refusal` kind carries no proof obligation beyond type-checking. We model
the claim as an inductive type describing the two possible fault-injection
granularities, a predicate on which granularity has been "wired through the
fleet path", and a structure recording that the novelty-curve-under-faults
measurement is withheld (refused) unless node-level injection is available.
-/

/-- The granularity at which fault injection can occur. -/
inductive InjectionGranularity where
  | memberLevel : InjectionGranularity
  | nodeLevel   : InjectionGranularity
deriving DecidableEq, Repr

/-- Whether a given injection granularity has been wired through the fleet path. -/
def wiredThroughFleetPath : InjectionGranularity → Prop
  | InjectionGranularity.memberLevel => True
  | InjectionGranularity.nodeLevel   => False

/-- A measurement claim about the novelty curve under faults, parameterized by
the granularity of fault injection it would be based on. -/
structure NoveltyCurveClaim where
  granularity : InjectionGranularity
  -- The claim is only sound (not a "flattering error") if the recovery
  -- observed under this granularity actually re-runs solver work, i.e. if
  -- the granularity is wired through the fleet path at node level.
  sound : Prop := wiredThroughFleetPath granularity

/-- The refusal: at member-level granularity, recovery happens without
re-running solver work, so a work-proxy re-measurement at that granularity
would overstate the cache dividend. Hence the claim at member-level
granularity is withheld (its `sound` field is not inhabited/asserted). -/
def memberLevelClaim : NoveltyCurveClaim :=
  { granularity := InjectionGranularity.memberLevel }

/-- The node-level claim, the one whose wiring is required before the
measurement can be trusted. -/
def nodeLevelClaim : NoveltyCurveClaim :=
  { granularity := InjectionGranularity.nodeLevel }

/-- The measurement is withheld: node-level fault injection is *not* wired
through the fleet path, which is the formal content of "the claim is
withheld until node-level injection is wired through". -/
theorem noveltyCurveClaim_withheld :
    ¬ wiredThroughFleetPath nodeLevelClaim.granularity := by
  simp [nodeLevelClaim, wiredThroughFleetPath]

/-- The receipt's deferred list names exactly this withheld (node-level) claim. -/
def deferredList : List InjectionGranularity :=
  [InjectionGranularity.nodeLevel]

theorem nodeLevel_in_deferredList :
    InjectionGranularity.nodeLevel ∈ deferredList := by
  simp [deferredList]
