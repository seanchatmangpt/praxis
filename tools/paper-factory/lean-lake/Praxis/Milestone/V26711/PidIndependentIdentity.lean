/-!
# PROJ-769 / PRD v26.7.11 §7.8 — Semantic Identity Independent of PID

Target 7 of the 9 declared Lean/Lake formalization targets at `PRD.md:1035-1043`:
"semantic identity independence from runtime PID."

PRD §7.8 (`docs/jira/v26.7.11/PRD.md:392-409`), verbatim:

> Each external workflow instance SHALL run as a supervised process.
> Workflow semantic identity SHALL be independent of PID.
> Minimum workflow identity: workflow ID; parent workflow ID; Arazzo workflow ID;
> source POWL region ID; dispatch ID; correlation ID; source digest; projection
> digest; receipt head; replay ID.
> The OTP runner SHALL survive execution-process restart by reconstructing from
> admitted state and replay surfaces.

## Real correspondence

Models `apps/arazzo_runner/include/arazzo_runner.hrl`'s `#workflow_identity{}`
record (the real OTP implementation,
`apps/arazzo_runner/src/arazzo_runner_workflow.erl`) field-for-field:
`workflow_id`, `parent_workflow_id`, `arazzo_workflow_id`, `source_powl_region_id`,
`dispatch_id`, `correlation_id`, `source_digest`, `projection_digest`,
`receipt_head`, `replay_id` — exactly PRD §7.8's ten-field "minimum workflow
identity" list, and notably absent any PID field. The Erlang source's own comment
(`arazzo_runner_workflow.erl:76-83`) states this directly: "Workflow *identity*
remains PID-independent (PRD 7.8) for every DURABLE fact ... a `workflow_id -> Pid`
registry (`arazzo_workflow_pids`, a separate ETS table) only resolves 'which live
process to send an event to right now' ... it does not change what identity
means." `RunnerState`/`restart` below model that same separation: identity is a
field of `RunnerState` whose type structurally cannot mention `Pid` (it has no such
field), and `restart` — standing in for a supervisor respawn under a fresh `Pid`,
per `start_link/1`'s `ets:insert(arazzo_workflow_pids, {WorkflowId, Pid})` always
overwriting the prior live-process entry — provably leaves `identity` untouched.

No axioms: `WorkflowIdentity`, `RunnerState`, `restart` are plain
structures/functions over an arbitrary `Pid` type (universally quantified, not
axiomatized into existence — Erlang's `pid()` is opaque and this file does not need
to know anything about it beyond "some type").
-/

/-- `#workflow_identity{}` (`apps/arazzo_runner/include/arazzo_runner.hrl`), field
for field: PRD §7.8's ten-field "minimum workflow identity." `parentWorkflowId` is
`Option String` matching the Erlang field's `binary() | undefined` (absent for a
root workflow). -/
structure WorkflowIdentity where
  workflowId         : String
  parentWorkflowId   : Option String
  arazzoWorkflowId   : String
  sourcePowlRegionId : String
  dispatchId         : String
  correlationId      : String
  sourceDigest       : String
  projectionDigest   : String
  receiptHead        : String
  replayId           : String
deriving DecidableEq, Repr

/-- A live OTP runner's durable state: its `WorkflowIdentity` plus a currently-live
`Pid`, standing in for `#runner_state{}` plus the `arazzo_workflow_pids` ETS entry
resolving to it. `Pid` is left an arbitrary type parameter, not axiomatized — this
file needs nothing about it beyond "some identifier type," matching Erlang's
`pid()` being opaque outside the BEAM. -/
structure RunnerState (Pid : Type) where
  identity : WorkflowIdentity
  livePid  : Pid

/-- A supervisor restart: reconstructs the runner under a fresh live `Pid`
(`start_link/1` always overwrites the `arazzo_workflow_pids` entry for this
`workflow_id`, per the Erlang source's own comment), leaving `identity` untouched —
"the OTP runner SHALL survive execution-process restart by reconstructing from
admitted state." -/
def restart {Pid : Type} (rs : RunnerState Pid) (newPid : Pid) : RunnerState Pid :=
  { rs with livePid := newPid }

/-- `thm:pid_independent_identity`: a restart never changes `identity`, for *any*
choice of new `Pid` — "workflow semantic identity SHALL be independent of PID,"
proved as a real equation over the concrete `restart` model above, not merely
declared. -/
theorem restart_preserves_identity {Pid : Type} (rs : RunnerState Pid)
    (newPid : Pid) : (restart rs newPid).identity = rs.identity := rfl

/-- Corollary, stated the other direction: two runner states sharing the same
`identity` but arbitrary (possibly distinct) `livePid` values are, as far as
`identity` is concerned, indistinguishable — restart or crash-and-respawn under any
`Pid` never produces a different semantic identity. -/
theorem identity_indifferent_to_pid {Pid : Type} (id : WorkflowIdentity)
    (p1 p2 : Pid) :
    (RunnerState.mk id p1).identity = (RunnerState.mk id p2).identity := rfl
