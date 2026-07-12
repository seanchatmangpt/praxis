%% PROJ-757: PRD v26.7.11 section 7.8 (Layer 8 -- OTP Outer Runner) record
%% definitions, shared between arazzo_runner_identity, arazzo_runner_workflow,
%% arazzo_runner_sup, and their eunit tests.

%% Minimum workflow identity (PRD 7.8): 10 fields, independent of PID, that
%% together identify a workflow instance across process restarts. This is
%% looked up by workflow_id (never by Pid) -- see
%% arazzo_runner_workflow:get_identity/1.
-record(workflow_identity, {
    workflow_id           :: binary(),
    %% undefined for a root workflow with no parent; the key/field is
    %% still always present, per PRD's "minimum workflow identity" list.
    parent_workflow_id    :: binary() | undefined,
    arazzo_workflow_id    :: binary(),
    source_powl_region_id :: binary(),
    dispatch_id           :: binary(),
    correlation_id        :: binary(),
    source_digest         :: binary(),
    projection_digest     :: binary(),
    receipt_head          :: binary(),
    replay_id             :: binary()
}).

%% Durable, PID-independent runner state: identity plus everything needed to
%% reconstruct a workflow instance after an execution-process restart (PRD
%% 7.8: "SHALL survive execution-process restart by reconstructing from
%% admitted state and replay surfaces"). Persisted to DETS (real disk-backed
%% OTP term storage -- survives the death of every Erlang process that had
%% it open, not just a single process crash) keyed by workflow_id via
%% arazzo_runner_identity:persist/1 and load/1.
-record(runner_state, {
    identity                :: #workflow_identity{},
    %% Copy of the admitted workflow definition (air_core's `workflow` map),
    %% carried alongside identity so a reconstruction can answer
    %% "what does this step definition look like" (e.g. for retry-due
    %% re-dispatch) without reaching into air_core's opaque context() record.
    workflow_def = #{}      :: map(),
    %% air_core:context() -- opaque outside air_core.erl; never pattern
    %% matched here, only threaded through air_core:transition/2 and the
    %% air_core:get_env/1, get_history/1, ready_steps/1 accessors.
    core                     :: term(),
    %% {StepId, StepDef} pairs from dispatch_step commands (air_core's C)
    %% that have not yet been acknowledged. PROJ-758 (the broker that would
    %% route these to a real external actuator) is not built yet, so this
    %% list is the honest record of "computed, not yet routed anywhere".
    pending_dispatches = [] :: [{binary(), map()}],
    acknowledged = []       :: [{binary(), term()}],
    retry_counts = #{}      :: #{binary() => non_neg_integer()},
    children = #{}          :: #{binary() => term()},
    refusals = []           :: [term()],
    admission_log = []      :: [term()],
    %% PROJ-758: outcome of routing each dispatch_step command through
    %% arazzo_runner_broker:dispatch/4 -- {StepId, {ok, DispatchToken}} on
    %% success or {StepId, {refused, Code, Ctx} | {error, Reason}}
    %% otherwise. Colocated with the rest of this record's observable audit
    %% trail (acknowledged/refusals/admission_log) rather than requiring
    %% callers to reach into broker internals to see what happened.
    broker_dispatches = []  :: [{binary(), term()}],
    %% Ordered (most-recent-first) log of every reaction-event tag this
    %% instance has processed. Exists so tests can assert a reaction
    %% genuinely happened (observable state changed) rather than being
    %% silently accepted-and-ignored.
    reaction_log = []       :: [term()]
}).
