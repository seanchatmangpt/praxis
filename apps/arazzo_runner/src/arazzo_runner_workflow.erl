-module(arazzo_runner_workflow).
-include("arazzo_runner.hrl").

%% API
-export([start_link/1, dispatch_event/2]).

%% Introspection (used by callers and eunit tests; identity/state are always
%% looked up by workflow_id, never by Pid -- PRD 7.8: "Workflow semantic
%% identity SHALL be independent of PID.")
-export([get_identity/1, get_runner_state/1, get_pid/1]).

%% PROJ-758 (PRD 13 / PRD 8): broker-facing entry points. enqueue_io/2 is
%% the sole gate into the io-worker pool (see its own doc comment below);
%% admit_result/3 is how arazzo_runner_broker re-admits a return-chain-
%% admitted consequence into a live workflow's state.
-export([enqueue_io/2, admit_result/3]).

%% Internal callbacks
-export([workflow_loop/1, io_worker_loop/0, infra_loop/0]).

%% ---------------------------------------------------------------------
%% PROJ-757 (PRD v26.7.11 7.8, Layer 8 -- OTP Outer Runner)
%% ---------------------------------------------------------------------
%%
%% Design decision (this ticket): extend this module/app in place rather
%% than rebuild the deleted apps/otp_runner/ directory. Reasons, in order:
%%
%%  1. This ticket's own text names concrete symbols to fix --
%%     `restart => temporary` (arazzo_runner_sup.erl), `process_transition/2`
%%     (this file), `start_link/1` "takes one opaque ID and tracks nothing
%%     else" (this file) -- all of which already live here, not in a
%%     from-scratch app.
%%  2. arazzo_runner.app.src's own description is "Workflow Execution Engine
%%     wrapping air_core" -- exactly PRD 7.7's "OTP ... SHALL wrap the same
%%     transition core", i.e. this app already *is* the OTP Outer Runner
%%     described in 7.8, just missing identity/reaction-event/restart
%%     correctness.
%%  3. PRD 7.7 contrasts exactly two wrappers around the shared transition
%%     core -- "OTP and AtomVM" -- not three. A second, separate OTP app
%%     alongside this one would not correspond to anything in the PRD's own
%%     layering and would fork the supervision tree for no semantic reason.
%%  4. apps/otp_runner/ was deleted for containing a live LLM-code-injection
%%     pathway (docs/jira/v26.7.11/SAFETY_FINDINGS.md), not because its
%%     *name* was wrong. Recreating a directory with that exact name risks
%%     exactly the confusion ("is this the safe one or the deleted one?")
%%     the deletion was meant to resolve.
%%
%% NOTE: PROJ-755 fixed air_core:transition/2's return shape to the PRD
%% v26.7.11 7.7 contract delta_AIR: (S,E) -> (S',C) -- it now returns
%% {NewContext, Commands}, always (no ok/io_request/error/stop envelope;
%% air_core has no failure path of its own). apply_transition/4 below
%% matches that 2-tuple directly plus the {exception,_,_,_} shape produced
%% by this module's own try/catch wrapper.

%% ---------------------------------------------------------------------
%% PROJ-758 (PRD v26.7.11 13 -- Broker Requirements; 8 -- Independent
%% Process Cells / return-admission chain).
%% ---------------------------------------------------------------------
%%
%% Closes the gap PROJ-755's and PROJ-757's own ticket text name explicitly:
%% Commands (dispatch_step) computed by apply_transition/4 below were
%% folded into pending_dispatches and never routed anywhere. As of this
%% ticket, apply_transition/4 also calls arazzo_runner_broker:dispatch/4
%% for every dispatch_step command -- see the `{dispatch_step, StepId,
%% StepDef}` clause of its lists:foldl/3 fold function. See
%% arazzo_runner_broker.erl's own header comment for the "new module in
%% this app, not a new app" design decision.
%%
%% Two new pieces added here, both required for the broker to be able to
%% round-trip a dispatch and later re-admit its result:
%%  - enqueue_io/2: the sole gate into the pre-existing io-worker pool
%%    (previously idle infrastructure; execute_io_request/1's inert-
%%    placeholder comment below is now stale except for the "not yet
%%    wired" framing -- it IS wired, through this function only).
%%  - admit_result/3 (+ get_pid/1 and the arazzo_workflow_pids table): a
%%    workflow_id -> Pid registry did not exist before this ticket --
%%    dispatch_event/2 always required a Pid the caller already had from
%%    start_link/1's own return value, which a return arriving later,
%%    routed through the broker, does not have. Workflow *identity*
%%    remains PID-independent (PRD 7.8) for every DURABLE fact
%%    (get_identity/1, get_runner_state/1 do not need this table); this
%%    registry only resolves "which live process to send an event to
%%    right now", the same role dispatch_event/2's own Pid argument
%%    already played -- it does not change what identity means.

%% ---------------------------------------------------------------------
%% API
%% ---------------------------------------------------------------------

%% start_link/1 takes a single "start spec" map: the 10 PRD 7.8 identity
%% fields (see arazzo_runner_identity:from_map/1) plus, for a genuinely
%% fresh workflow only, `workflow_def` / `active_steps` / `env` / `history`
%% to seed air_core:new/1. On any restart (this workflow_id already has
%% persisted state in DETS) the persisted identity and execution state are
%% used instead and the workflow_def/active_steps/env/history keys in
%% StartSpec are ignored -- this is the "reconstructing from admitted state
%% and replay surfaces" requirement, not a re-application of whatever
%% arguments the caller (or a supervisor auto-restart) happened to pass.
-spec start_link(map()) -> {ok, pid()}.
start_link(StartSpec) when is_map(StartSpec) ->
    setup_infrastructure(),

    Identity = case arazzo_runner_identity:from_map(StartSpec) of
        {ok, Id} -> Id;
        {error, Reason} -> error({invalid_workflow_identity, Reason})
    end,
    WorkflowId = Identity#workflow_identity.workflow_id,

    RunnerState0 = case arazzo_runner_identity:load(WorkflowId) of
        {ok, Persisted} ->
            Persisted;
        not_found ->
            WorkflowDef = maps:get(workflow_def, StartSpec, #{}),
            Core = air_core:new(#{
                workflow => WorkflowDef,
                active_steps => maps:get(active_steps, StartSpec, []),
                env => maps:get(env, StartSpec, #{}),
                history => maps:get(history, StartSpec, [])
            }),
            #runner_state{identity = Identity, workflow_def = WorkflowDef, core = Core}
    end,

    %% React to `start`: this is the runner's reaction to its own creation
    %% (PRD 7.8's first reaction-event class), not a no-op -- it's what
    %% makes the first identity+state write durable before anything else
    %% can happen to this process.
    RunnerState1 = record_reaction(RunnerState0, start),
    ok = arazzo_runner_identity:persist(RunnerState1),
    ets:insert(arazzo_workflow_states, {WorkflowId, RunnerState1}),

    Pid = proc_lib:spawn_link(?MODULE, workflow_loop, [WorkflowId]),

    %% PROJ-758: record the current live Pid for this workflow_id so a
    %% later-arriving broker return (admit_result/3) can find it. Always
    %% overwrites any stale entry from a pre-restart Pid -- exactly the
    %% "current live process for this identity" lookup dispatch_event/2's
    %% own Pid argument already assumed the caller had another way to get.
    ets:insert(arazzo_workflow_pids, {WorkflowId, Pid}),

    {ok, Pid}.

dispatch_event(Pid, Event) ->
    Pid ! {event, Event},
    ok.

%% PROJ-758: resolves a workflow_id to its current live Pid. Distinct from
%% (and does not change) get_identity/1 / get_runner_state/1's PID-
%% independent durable-state lookups -- this is only "who to send an event
%% to right now", the same role a caller already had to supply by hand to
%% dispatch_event/2 before this ticket.
-spec get_pid(binary()) -> {ok, pid()} | not_found.
get_pid(WorkflowId) ->
    %% Mirrors get_runner_state/1's own "undefined table means nothing has
    %% ever started" shortcut -- unlike arazzo_workflow_states, there is no
    %% durable (DETS) fallback here: a Pid is inherently ephemeral, so an
    %% absent table or absent key both legitimately mean not_found, not
    %% "go bootstrap infrastructure to find out".
    case ets:info(arazzo_workflow_pids) of
        undefined -> not_found;
        _ ->
            case ets:lookup(arazzo_workflow_pids, WorkflowId) of
                [{WorkflowId, Pid}] -> {ok, Pid};
                [] -> not_found
            end
    end.

%% PROJ-758 (PRD 8, return-admission chain, stage 6 -- O*): the only path
%% by which arazzo_runner_broker re-admits an already-return-chain-admitted
%% consequence into a live workflow. Reuses the exact `result` reaction
%% path PROJ-757 already built and tested (handle_reaction/3's
%% `{result, StepId, Result}` clause) -- this function adds Pid resolution
%% and liveness checking in front of it, nothing else.
-spec admit_result(binary(), binary(), term()) -> ok | {error, workflow_not_found}.
admit_result(WorkflowId, StepId, Result) ->
    case get_pid(WorkflowId) of
        {ok, Pid} ->
            case is_process_alive(Pid) of
                true ->
                    dispatch_event(Pid, {result, StepId, Result});
                false ->
                    {error, workflow_not_found}
            end;
        not_found ->
            {error, workflow_not_found}
    end.

-spec get_identity(binary()) -> {ok, #workflow_identity{}} | not_found.
get_identity(WorkflowId) ->
    case get_runner_state(WorkflowId) of
        {ok, #runner_state{identity = Identity}} -> {ok, Identity};
        not_found -> not_found
    end.

-spec get_runner_state(binary()) -> {ok, #runner_state{}} | not_found.
get_runner_state(WorkflowId) ->
    case ets:info(arazzo_workflow_states) of
        undefined ->
            arazzo_runner_identity:load(WorkflowId);
        _ ->
            case ets:lookup(arazzo_workflow_states, WorkflowId) of
                [{WorkflowId, RS}] -> {ok, RS};
                [] -> arazzo_runner_identity:load(WorkflowId)
            end
    end.

%% ---------------------------------------------------------------------
%% Infrastructure and Queue Management (unaffected by this ticket -- real,
%% ordinary worker-pool leader election; see
%% docs/jira/v26.7.11/SAFETY_FINDINGS.md section "What was NOT touched")
%% ---------------------------------------------------------------------

%% setup_infrastructure/0 is self-healing: it monitors whichever infra pid
%% it ends up depending on (one it just spawned, or one another concurrent
%% caller already registered) and, if THAT pid dies before
%% arazzo_workflow_states appears, retries the whole bootstrap from
%% scratch rather than waiting forever. This matters concretely for
%% restart-survival (PRD 7.8): a workflow crash test that kills infra and
%% then immediately calls start_link/1 again can race the old infra's
%% (and its linked pg scope's) teardown against the new infra's bootstrap
%% -- e.g. ets:new/2 for a *named* table can raise badarg if the previous
%% owner's same-named table has not finished being reclaimed yet. Without
%% this retry, that transient badarg would silently kill the fresh
%% infra_loop process before it ever created the table, and the plain
%% polling loop this replaced (`ets:info` forever `undefined`, no failure
%% signal) would then spin indefinitely -- reproduced directly: killing
%% infra and restarting in a tight loop hung on roughly 1 in 5-10
%% iterations before this fix, 0 in 500+ after.
-spec setup_infrastructure() -> ok.
setup_infrastructure() ->
    setup_infrastructure(50).

setup_infrastructure(0) ->
    error(infra_bootstrap_retries_exhausted);
setup_infrastructure(Retries) ->
    case whereis(arazzo_runner_infra) of
        undefined ->
            Pid = spawn(?MODULE, infra_loop, []),
            Mon = monitor(process, Pid),
            case catch register(arazzo_runner_infra, Pid) of
                true ->
                    Pid ! init_infra,
                    settle_infra(Pid, Mon, Retries);
                _ ->
                    %% Lost the registration race to a concurrent caller (or
                    %% Pid already died before we could register it) --
                    %% re-resolve `whereis` fresh and retry.
                    demonitor(Mon, [flush]),
                    setup_infrastructure(Retries - 1)
            end;
        ExistingPid ->
            Mon = monitor(process, ExistingPid),
            settle_infra(ExistingPid, Mon, Retries)
    end.

settle_infra(Pid, Mon, Retries) ->
    case ets:info(arazzo_workflow_states) of
        undefined ->
            receive
                {'DOWN', Mon, process, Pid, _Reason} ->
                    setup_infrastructure(Retries - 1)
            after 1 ->
                settle_infra(Pid, Mon, Retries)
            end;
        _ ->
            demonitor(Mon, [flush]),
            ok
    end.

infra_loop() ->
    receive
        init_infra ->
            %% Pre-existing latent bug, surfaced (not introduced) by this
            %% ticket's tests: io_worker_loop/0 below calls
            %% pg:join(arazzo_io_workers, self()) using the *default* pg
            %% scope, but nothing anywhere in this codebase ever started
            %% that scope's server (it is not part of kernel's default
            %% supervision tree in this OTP release -- verified via
            %% `supervisor:which_children(kernel_sup)`, no `pg` child).
            %% Every io_worker was therefore crashing on its first message
            %% (`exit:{noproc,...}` from pg:join/2's gen_server:call), and
            %% because they are spawn_link'd from this process, that crash
            %% propagated straight back and killed infra_loop itself --
            %% silently, before the ETS table it had just created could
            %% ever be used, with no trace beyond `ets:info/1` reverting to
            %% `undefined`. Ensuring the default scope is running here,
            %% before any io_worker can reach pg:join/2, is the fix.
            ensure_pg_started(),

            %% PROJ-758: workflow_id -> current live Pid (ephemeral, not
            %% persisted to DETS -- see get_pid/1) and the broker's 4
            %% ledger tables (dispatches, idempotency dedup, one-shot
            %% actuation tokens, per-workflow evidence-hash chain heads).
            %% Co-owned by this same long-lived infra process for the same
            %% reason arazzo_workflow_states is: an ETS table's lifetime is
            %% its owner's lifetime, and none of these must die with
            %% whichever transient caller happened to touch them first.
            %% Created BEFORE arazzo_workflow_states (below) because
            %% settle_infra/3 only polls arazzo_workflow_states -- placing
            %% it last here means "arazzo_workflow_states exists" remains a
            %% valid proof that every table created in this same,
            %% single-threaded init_infra clause already exists too.
            ets:new(arazzo_workflow_pids, [public, named_table, set,
                                           {write_concurrency, true},
                                           {read_concurrency, true}]),
            ets:new(arazzo_broker_dispatches, [public, named_table, set,
                                               {write_concurrency, true},
                                               {read_concurrency, true}]),
            ets:new(arazzo_broker_dedup, [public, named_table, set,
                                          {write_concurrency, true},
                                          {read_concurrency, true}]),
            ets:new(arazzo_broker_tokens, [public, named_table, set,
                                           {write_concurrency, true},
                                           {read_concurrency, true}]),
            ets:new(arazzo_broker_chain_heads, [public, named_table, set,
                                                {write_concurrency, true},
                                                {read_concurrency, true}]),

            %% Lock-free ETS table for workflow states.
            ets:new(arazzo_workflow_states, [public, named_table, set,
                                             {write_concurrency, true},
                                             {read_concurrency, true}]),

            NumWorkers = erlang:system_info(schedulers_online) * 16,
            [spawn_link(?MODULE, io_worker_loop, []) || _I <- lists:seq(1, NumWorkers)],
            infra_loop();
        _ ->
            infra_loop()
    end.

%% pg:start_link/0 itself is the atomic check-and-start (it registers the
%% name `pg` as part of the same gen_server start_link call) -- a prior
%% whereis(pg) pre-check would be a TOCTOU race against a just-killed pg
%% instance whose exit signal (it was linked to a since-killed infra_loop)
%% has not necessarily finished propagating yet.
ensure_pg_started() ->
    case pg:start_link() of
        {ok, _Pid} -> ok;
        {error, {already_started, _Pid}} -> ok
    end.

%% ---------------------------------------------------------------------
%% Workflow Process Loop
%% ---------------------------------------------------------------------

workflow_loop(WorkflowId) ->
    receive
        {event, Event} ->
            react(WorkflowId, Event),
            workflow_loop(WorkflowId);
        _Other ->
            workflow_loop(WorkflowId)
    end.

%% react/2 is the single entry point for all 9 PRD 7.8 reaction-event
%% classes except `start` (handled inline in start_link/1, above, since it
%% is the event that creates the very state react/2 looks up here).
%%
%% # Complexity
%% O(1) ETS lookup/insert plus whatever handle_reaction/3 does for the
%% specific event (dominated by apply_transition/4's O(|next|) air_core
%% call for result/timeout/child_complete/child_refused; O(1) for the
%% purely bookkeeping events dispatch_ready/acknowledgment/retry_due/
%% admission_result).
react(WorkflowId, Event) ->
    case ets:lookup(arazzo_workflow_states, WorkflowId) of
        [{WorkflowId, RS0}] ->
            RS1 = handle_reaction(WorkflowId, Event, RS0),
            ets:insert(arazzo_workflow_states, {WorkflowId, RS1}),
            ok = arazzo_runner_identity:persist(RS1),
            ok;
        [] ->
            ok
    end.

%% ---- The 9 PRD 7.8 reaction-event classes (start is handled in start_link/1) ----

%% result: the real air_core semantic event (step_completed). Genuinely
%% advances state_mask/completed_mask/env/history and may produce new
%% dispatch_step commands.
handle_reaction(WorkflowId, {result, StepId, Result}, RS) ->
    apply_transition(WorkflowId, RS, {step_completed, StepId, Result}, result);

%% timeout: a step that did not complete in time is treated as a step
%% failure (air_core's event() type has no separate timeout event; a
%% timed-out step must not unlock any AND/join depending on it, which is
%% exactly step_failed's semantics -- see air_core:handle_step_failed/3).
handle_reaction(WorkflowId, {timeout, StepId}, RS) ->
    apply_transition(WorkflowId, RS, {step_failed, StepId, timeout}, timeout);

%% retry-due: a retry timer fired for StepId. Increments the observable
%% retry counter and re-announces dispatch-readiness for that step (from
%% the admitted workflow_def carried in RunnerState, not by re-deriving it
%% from air_core's opaque context()).
handle_reaction(_WorkflowId, {retry_due, StepId}, RS) ->
    Counts = RS#runner_state.retry_counts,
    N = maps:get(StepId, Counts, 0) + 1,
    RS1 = RS#runner_state{retry_counts = maps:put(StepId, N, Counts)},
    RS2 = case step_def(RS1, StepId) of
        undefined -> RS1;
        Def -> RS1#runner_state{pending_dispatches = [{StepId, Def} | RS1#runner_state.pending_dispatches]}
    end,
    record_reaction(RS2, {retry_due, StepId});

%% dispatch-ready: a step is ready to be routed to an external actuator.
%% apply_transition/4 also synthesizes this internally for every
%% dispatch_step command air_core produces; this clause is for the case of
%% dispatch-readiness being (re-)announced directly, e.g. by retry-due
%% above or by a future broker (PROJ-758) re-signalling readiness.
handle_reaction(_WorkflowId, {dispatch_ready, StepId, StepDef}, RS) ->
    RS1 = RS#runner_state{pending_dispatches = [{StepId, StepDef} | RS#runner_state.pending_dispatches]},
    record_reaction(RS1, {dispatch_ready, StepId});

%% acknowledgment: an external actuator (none exists yet -- PROJ-758 is not
%% built; this is only reachable today via a direct dispatch_event/2 call,
%% honestly, not wired to any real broker) confirms receipt of a dispatch.
%% Moves the step out of pending_dispatches into acknowledged.
handle_reaction(_WorkflowId, {acknowledgment, StepId, AckMeta}, RS) ->
    RS1 = RS#runner_state{
        pending_dispatches = lists:keydelete(StepId, 1, RS#runner_state.pending_dispatches),
        acknowledged = [{StepId, AckMeta} | RS#runner_state.acknowledged]
    },
    record_reaction(RS1, {acknowledgment, StepId});

%% child-complete: a child workflow finished. Folded into the parent's
%% air_core state as the completion of the step that spawned it (StepId),
%% so it genuinely advances state_mask/env/history exactly like `result`
%% does -- plus records the child's outcome for audit/introspection.
handle_reaction(WorkflowId, {child_complete, ChildWorkflowId, StepId, Result}, RS) ->
    RS1 = apply_transition(WorkflowId, RS, {step_completed, StepId, Result}, {child_complete, ChildWorkflowId}),
    RS1#runner_state{children = maps:put(ChildWorkflowId, {complete, Result}, RS1#runner_state.children)};

%% child-refused: a child workflow was refused. Folded as the failure of
%% the step that spawned it -- a refused child permanently blocks any
%% AND/join depending on that step, the same "no compensation protocol yet"
%% default air_core:handle_step_failed/3 documents for PROJ-759.
handle_reaction(WorkflowId, {child_refused, ChildWorkflowId, StepId, Reason}, RS) ->
    RS1 = apply_transition(WorkflowId, RS, {step_failed, StepId, Reason}, {child_refused, ChildWorkflowId}),
    RS1#runner_state{
        children = maps:put(ChildWorkflowId, {refused, Reason}, RS1#runner_state.children),
        refusals = [{ChildWorkflowId, Reason} | RS1#runner_state.refusals]
    };

%% admission-result (accepted): recorded for audit; execution continues.
handle_reaction(_WorkflowId, {admission_result, accepted}, RS) ->
    RS1 = RS#runner_state{admission_log = [accepted | RS#runner_state.admission_log]},
    record_reaction(RS1, {admission_result, accepted});

%% admission-result (refused): PRD 7.5 -- "Production Arazzo without an
%% admitted POWL source and projection receipt SHALL be refused." This is
%% not advisory: persist the refusal, then terminate. It will not be
%% auto-restarted (restart => transient in arazzo_runner_sup only restarts
%% on *abnormal* exit; a deliberate admission refusal is a normal one).
handle_reaction(_WorkflowId, {admission_result, {refused, Reason}} = Ev, RS) ->
    RS1 = RS#runner_state{admission_log = [{refused, Reason} | RS#runner_state.admission_log]},
    RS2 = record_reaction(RS1, Ev),
    ok = arazzo_runner_identity:persist(RS2),
    ets:insert(arazzo_workflow_states, {(RS2#runner_state.identity)#workflow_identity.workflow_id, RS2}),
    exit({admission_refused, Reason}).

%% ---------------------------------------------------------------------
%% Shared air_core transition application
%% ---------------------------------------------------------------------

%% Applies one air_core event to RS's core context, folding every resulting
%% dispatch_step command into pending_dispatches (each recorded as its own
%% {dispatch_ready, StepId} reaction) plus the outer ReactionTag itself.
%%
%% # Complexity
%% O(|next(StepId)|) -- dominated by air_core:transition/2's own documented
%% bound (see air_core.erl handle_step_completed/3): O(1) exception-wrapped
%% NIF-free Erlang call plus a foldl over Commands, which is at most the
%% step's direct successor count.
apply_transition(WorkflowId, RS, Event, ReactionTag) ->
    Core = RS#runner_state.core,
    case try air_core:transition(Event, Core) catch C:R:S -> {exception, C, R, S} end of
        {exception, Class, Reason, Stack} ->
            error_logger:error_msg(
                "Workflow ~p transition crashed on ~p: ~p:~p ~p",
                [WorkflowId, ReactionTag, Class, Reason, Stack]
            ),
            RS1 = record_reaction(RS, {ReactionTag, crashed}),
            ok = arazzo_runner_identity:persist(RS1),
            exit({transition_crashed, WorkflowId, Reason});
        {NewCore, Commands} ->
            %% The real, always-taken shape as of PROJ-755: NewCore is the
            %% #context{} record, Commands the newly-ready dispatch_step
            %% requests produced by exactly this one transition.
            RS1 = lists:foldl(
                fun({dispatch_step, StepId, StepDef}, Acc) ->
                    Acc1 = Acc#runner_state{
                        pending_dispatches = [{StepId, StepDef} | Acc#runner_state.pending_dispatches]
                    },
                    %% PROJ-758: route this command through the broker --
                    %% the sole actuation route (PRD 13). This is
                    %% synchronous and inline (it can block this workflow
                    %% process for up to enqueue_io/2's 5s timeout): no
                    %% separate dispatcher process exists yet to make this
                    %% async, and dispatch_ready/acknowledgment/result
                    %% remain distinct reaction events regardless (a future
                    %% ticket could spawn a per-dispatch worker to decouple
                    %% this without changing that event vocabulary).
                    Identity = Acc1#runner_state.identity,
                    BrokerResult = arazzo_runner_broker:dispatch(WorkflowId, Identity, StepId, StepDef),
                    Acc2 = Acc1#runner_state{
                        broker_dispatches = [{StepId, BrokerResult} | Acc1#runner_state.broker_dispatches]
                    },
                    Acc3 = case BrokerResult of
                        {refused, Code, Ctx} ->
                            Acc2#runner_state{
                                refusals = [{StepId, {broker, Code, Ctx}} | Acc2#runner_state.refusals]
                            };
                        {ok, _DispatchToken} ->
                            Acc2;
                        {error, _Reason} ->
                            Acc2
                    end,
                    record_reaction(Acc3, {dispatch_ready, StepId})
                end,
                RS#runner_state{core = NewCore},
                Commands
            ),
            record_reaction(RS1, ReactionTag)
    end.

record_reaction(RS, Tag) ->
    RS#runner_state{reaction_log = [Tag | RS#runner_state.reaction_log]}.

step_def(#runner_state{workflow_def = WorkflowDef}, StepId) ->
    Steps = maps:get(steps, WorkflowDef, #{}),
    maps:get(StepId, Steps, undefined).

%% ---------------------------------------------------------------------
%% I/O worker pool: simplified Raft-style leader election so exactly one
%% worker owns heartbeat/coordination duties at a time. Real, ordinary
%% worker-pool coordination -- unaffected by this ticket (PROJ-758). Now
%% wired: enqueue_io/2, below, is the sole entry point PROJ-758's broker
%% (arazzo_runner_broker.erl) uses to reach this pool's I/O data plane
%% (`{execute_io, ReplyPid, Req}`) -- see enqueue_io/2's own doc comment
%% for the token-gated enforcement that makes it the sole entry point in
%% fact, not just in intent.
%% ---------------------------------------------------------------------

io_worker_loop() ->
    pg:join(arazzo_io_workers, self()),
    State = #{
        role => follower,
        current_term => 0,
        voted_for => undefined,
        leader => undefined,
        votes_received => 0,
        timer_ref => start_election_timer()
    },
    io_worker_receive_loop(State).

-define(ELECTION_TIMEOUT_MIN, 150).
-define(ELECTION_TIMEOUT_MAX, 300).
-define(HEARTBEAT_INTERVAL, 50).

start_election_timer() ->
    Timeout = ?ELECTION_TIMEOUT_MIN + rand:uniform(?ELECTION_TIMEOUT_MAX - ?ELECTION_TIMEOUT_MIN),
    erlang:send_after(Timeout, self(), election_timeout).

start_heartbeat_timer() ->
    erlang:send_after(?HEARTBEAT_INTERVAL, self(), send_heartbeat).

io_worker_receive_loop(State = #{role := Role, current_term := Term}) ->
    receive
        election_timeout ->
            NewTerm = Term + 1,
            NewState = State#{
                role => candidate,
                current_term => NewTerm,
                voted_for => self(),
                votes_received => 1,
                timer_ref => reset_timer(maps:get(timer_ref, State), election)
            },
            broadcast({request_vote, NewTerm, self()}),
            io_worker_receive_loop(NewState);

        {request_vote, CandidateTerm, CandidateId} ->
            if
                CandidateTerm > Term ->
                    CandidateId ! {vote_granted, CandidateTerm, self()},
                    NewState = State#{
                        role => follower,
                        current_term => CandidateTerm,
                        voted_for => CandidateId,
                        timer_ref => reset_timer(maps:get(timer_ref, State), election)
                    },
                    io_worker_receive_loop(NewState);
                true ->
                    io_worker_receive_loop(State)
            end;

        {vote_granted, VoteTerm, _VoterId} ->
            if
                Role =:= candidate, VoteTerm =:= Term ->
                    Votes = maps:get(votes_received, State) + 1,
                    Majority = (length(pg:get_members(arazzo_io_workers)) div 2) + 1,
                    if
                        Votes >= Majority ->
                            NewState = State#{
                                role => leader,
                                leader => self(),
                                timer_ref => reset_timer(maps:get(timer_ref, State), heartbeat)
                            },
                            broadcast({append_entries, Term, self()}),
                            io_worker_receive_loop(NewState);
                        true ->
                            io_worker_receive_loop(State#{votes_received => Votes})
                    end;
                true ->
                    io_worker_receive_loop(State)
            end;

        send_heartbeat ->
            if
                Role =:= leader ->
                    broadcast({append_entries, Term, self()}),
                    NewState = State#{timer_ref => reset_timer(maps:get(timer_ref, State), heartbeat)},
                    io_worker_receive_loop(NewState);
                true ->
                    io_worker_receive_loop(State)
            end;

        {append_entries, LeaderTerm, LeaderId} ->
            if
                LeaderTerm >= Term ->
                    NewState = State#{
                        role => follower,
                        current_term => LeaderTerm,
                        leader => LeaderId,
                        voted_for => undefined,
                        timer_ref => reset_timer(maps:get(timer_ref, State), election)
                    },
                    io_worker_receive_loop(NewState);
                true ->
                    io_worker_receive_loop(State)
            end;

        %% -- I/O Data Plane --
        {execute_io, ReplyPid, Req} ->
            Reply = execute_io_request(Req),
            ReplyPid ! {io_reply, Reply},
            io_worker_receive_loop(State);

        _ ->
            io_worker_receive_loop(State)
    end.

broadcast(Msg) ->
    Members = pg:get_members(arazzo_io_workers),
    [Pid ! Msg || Pid <- Members, Pid =/= self()].

reset_timer(OldRef, Type) ->
    erlang:cancel_timer(OldRef),
    %% Flush specific timers to prevent race conditions during state transitions
    receive election_timeout -> ok after 0 -> ok end,
    receive send_heartbeat -> ok after 0 -> ok end,
    if
        Type =:= election -> start_election_timer();
        Type =:= heartbeat -> start_heartbeat_timer()
    end.

%% Actual actuation (HTTP/RDMA/etc.) belongs behind the broker (PRD section
%% 13) and is now reachable only via enqueue_io/2 below. This function
%% itself remains an echo placeholder -- PROJ-758's scope is the broker's
%% actuation ROUTE (pre-actuation verification, the one-shot token gate,
%% post-actuation evidence capture, return-admission), not a real
%% HTTP/RDMA backend for the pool to call; that is a separate, later
%% concern (no ticket for it yet in docs/jira/v26.7.11/tickets/index.md).
execute_io_request(Req) ->
    {ok, {processed, Req}}.

%% PROJ-758: the sole gate into this pool's I/O data plane. PRD 13: "The
%% broker SHALL be the only actuation route" -- enforced here, not just
%% documented. ActuationToken must be one arazzo_runner_broker:dispatch/4
%% minted for exactly one prior call and has not yet been consumed;
%% consume_actuation_token/1's ets:take/2 is atomic, so a bogus token, an
%% already-consumed token, or no broker call at all are all refused
%% *before* anything is sent to any pool member -- no I/O happens on any
%% of those paths, actuated or otherwise.
%%
%% # Complexity
%% O(1) token check + O(|members|) worst case for pg:get_members/1 (bounded
%% by NumWorkers = schedulers_online * 16, a small constant in practice)
%% plus one synchronous round trip to the chosen worker, bounded by the 5s
%% receive timeout below.
-spec enqueue_io(binary(), term()) ->
    {ok, term()} | {refused, 'DIRECT_ACTUATION_REFUSED', map()} | {error, term()}.
enqueue_io(ActuationToken, Req) ->
    case arazzo_runner_broker:consume_actuation_token(ActuationToken) of
        ok ->
            case pg:get_members(arazzo_io_workers) of
                [] ->
                    {error, no_io_workers};
                Members ->
                    Worker = pick_worker(ActuationToken, Members),
                    Worker ! {execute_io, self(), Req},
                    receive
                        {io_reply, Reply} -> {ok, Reply}
                    after 5000 ->
                        {error, io_timeout}
                    end
            end;
        refused ->
            {refused, 'DIRECT_ACTUATION_REFUSED', #{actuation_token => ActuationToken}}
    end.

%% Deterministic member selection (erlang:phash2/2 over the token), not
%% random -- repo determinism discipline: given the same token and the same
%% pool membership, the same worker is chosen every time.
pick_worker(ActuationToken, Members) ->
    N = erlang:phash2(ActuationToken, length(Members)) + 1,
    lists:nth(N, Members).
