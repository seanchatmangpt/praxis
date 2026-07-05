/-
Label: refusal:noknhkpath
Kind: refusal

No path dependencies on knhk: verified by live probes, recorded as the standing
reason -- the leanest candidate crate drags tokio/reqwest/opentelemetry
transitively; the mu-kernel ships property-testing libraries as regular
dependencies and uses unsafe transmutes; the workspace does not build as a
whole. Ports of 50--200-line designs, tagged PORT(knhk) with per-item deltas,
cost less than the supply chain. Drift is greppable.
-/

/-- A concrete objection blocking a `knhk` path dependency. -/
inductive KnhkObjection where
  /-- The leanest candidate crate drags in `tokio`/`reqwest`/`opentelemetry`
      transitively. -/
  | heavyTransitiveDeps
  /-- The mu-kernel ships property-testing libraries as regular (non-dev)
      dependencies. -/
  | proptestAsRegularDep
  /-- The mu-kernel uses unsafe transmutes. -/
  | unsafeTransmute
  /-- The workspace does not build as a whole. -/
  | workspaceDoesNotBuild
  deriving Repr, DecidableEq

/-- The standing refusal: `knhk` is rejected as a path dependency, justified by
    at least one recorded objection. -/
structure NoKnhkPathRefusal where
  reason : KnhkObjection

/-- The refusal is witnessed, e.g. by the workspace build failure. -/
def noKnhkPathRefusal : NoKnhkPathRefusal :=
  { reason := KnhkObjection.workspaceDoesNotBuild }

/-- Alternative remediation: port 50--200-line designs, each tagged
    `PORT(knhk)` with a per-item delta, rather than take the dependency. -/
structure PortedItem where
  lineCount : Nat
  lineCountLo : lineCount ≥ 50
  lineCountHi : lineCount ≤ 200
  tag : String := "PORT(knhk)"
  delta : String
