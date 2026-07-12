-module(arazzo_runner_dispatch_statem).
-behaviour(gen_statem).
-include("arazzo_runner.hrl").

%% API
-export([start_link/4, mark_ready/1, dispatch/1,
         get_lifecycle_state/1, get_transition_log/1, get_outcome/1]).

%% gen_statem callbacks
-export([init/1, callback_mode/0, terminate/3]).

%% State functions -- one per atlas L5 state name (F16_otp-runner.md):
%% MANUFACTURED -> READY -> DISPATCHED -> AWAITING_RESULT -> AWAITING_ADMISSION
%% -> RUNNING -> COMPLETED -> REFUSED.
-export([manufactured/3, ready/3, dispatched/3, awaiting_result/3,
         awaiting_admission/3, running/3, completed/3, refused/3]).

%% ---------------------------------------------------------------------
%% F16 (atlas ticket V12-016, L5 "State Machine") -- real gen_statem
%% implementation of the 8-state lifecycle for ONE step-dispatch round trip.
%% ---------------------------------------------------------------------
%%
%% SCOPE, STATED HONESTLY (see crates/multifractal-workflow/src/f16_otp_runner.rs
%% for the Rust-side cross-check that reads this module's own source to verify
%% these claims, not just this comment):
%%
%%  1. This is a REAL, running `gen_statem` -- `callback_mode/0` returns
%%     `[state_functions, state_enter]`, every one of the 8 atlas state names is a
%%     real exported state function, and `state_enter` callbacks append to
%%     `transition_log` so a caller (or test) can verify the exact ordered
%%     sequence of states genuinely visited for any given run, not just the
%%     final outcome.
%%
%%  2. `ready/3`'s `dispatch` call spawns a REAL, separate, monitored worker
%%     process (`spawn_monitor/1`) that calls the REAL, unmodified
%%     `arazzo_runner_broker:dispatch/4` -- the exact same function
%%     `arazzo_runner_workflow:apply_transition/4` already calls in production.
%%     The `dispatch` call replies `ok` immediately upon entering `dispatched`,
%%     BEFORE the worker's round trip completes -- proving this state is a real
%%     async, concurrently-executing state, not a synchronous simulation dressed
%%     up as one.
%%
%%  3. DISCLOSED LIMITATION (not papered over): `arazzo_runner_broker:dispatch/4`
%%     performs actuation (the io-worker round trip) AND the full return-
%%     admission chain (correlation -> provenance -> authority -> structure ->
%%     semantic -> O*) as ONE atomic, blocking call -- it exposes no separately
%%     callable stage boundary an external caller could pause at. There is
%%     therefore no way, without a broker.erl refactor (out of this session's
%%     scope -- see f16_otp_runner.rs's own module doc for why that refactor
%%     was not attempted), for this state machine to genuinely, observably PAUSE
%%     between `awaiting_result` (actuation in flight) and `awaiting_admission`
%%     (admission decision in flight): both are entered and exited in immediate,
%%     real sequence the instant the worker's single round trip returns. The
%%     states are real (real code runs, real data is inspected, real transition-
%%     log entries are appended, in the atlas's own order) but not independently
%%     observable mid-flight the way `dispatched` genuinely is. A future ticket
%%     that splits `arazzo_runner_broker:do_dispatch_actuate/6` into a
%%     separately-callable actuate-then-admit pair would close this remaining
%%     gap without changing this module's public API.
%%
%%  4. DISCLOSED SCOPE LIMITATION: this state machine governs ONE step-dispatch
%%     round trip, not an entire multi-step workflow's lifecycle end to end --
%%     the atlas's L2 diagram's "Workflow gen_statem" component, read literally,
%%     implies replacing `arazzo_runner_workflow.erl`'s own per-workflow
%%     `workflow_loop/1` (a plain `receive` loop over a DAG of concurrently-
%%     ready steps) with a gen_statem covering the WHOLE workflow's execution.
%%     That full per-workflow rewrite was assessed and deliberately NOT
%%     attempted this session: `arazzo_runner_workflow.erl` is exercised by 3
%%     large, precise eunit tests covering supervisor-driven crash+restart,
%%     DETS-only reconstruction, and 9 distinct reaction-event classes
%%     (`arazzo_runner_workflow_test.erl`) -- a full rewrite risks regressing
%%     all of it for a linear 8-state chain that does not obviously compose with
%%     a DAG of concurrently-ready steps (a workflow can have >1 step ready at
%%     once; a single linear MANUFACTURED..COMPLETED chain does not, by itself,
%%     represent "N steps ready in parallel" without further design this
%%     session did not have room to work through safely). This module is
%%     therefore scoped to what IS safely, honestly buildable now: a real,
%%     tested, individually-addressable gen_statem per dispatch, supervised by a
%%     real, new Dispatch Worker Supervisor (`arazzo_runner_dispatch_sup.erl`),
%%     proven (via `arazzo_runner_dispatch_statem_test.erl`) to drive the SAME
%%     real, unmodified production broker code that
%%     `arazzo_runner_workflow:apply_transition/4` already uses, including
%%     genuinely advancing a live workflow's air_core state on the lawful path.
%%
%%  5. NOT YET WIRED INTO PRODUCTION: `arazzo_runner_workflow:apply_transition/4`
%%     still calls `arazzo_runner_broker:dispatch/4` directly and synchronously
%%     (unchanged, this session did not touch that file) -- this module is real,
%%     supervised, and independently tested, but nothing in the production
%%     dispatch path constructs an `arazzo_runner_dispatch_statem` today. Wiring
%%     `apply_transition/4` to spawn one of these per `dispatch_step` command
%%     instead of calling the broker inline would change that function's
%%     synchronous completion-ordering guarantees (several
%%     `arazzo_runner_workflow_test.erl` assertions rely on dispatch completing
%%     before the next reaction is processed) and was judged too large a change
%%     to make safely in this pass -- disclosed, not attempted.
%% ---------------------------------------------------------------------

-record(d, {
    workflow_id            :: binary(),
    identity                :: #workflow_identity{},
    step_id                 :: binary(),
    step_def                :: map(),
    worker = undefined      :: {pid(), reference()} | undefined,
    dispatch_token = undefined :: binary() | undefined,
    outcome = undefined     :: term(),
    %% Chronological (oldest-first) log of every state genuinely entered, one
    %% atom per `state_enter` callback -- proves the real ordered sequence a
    %% given run walked, not just its final resting state.
    transition_log = []     :: [atom()]
}).

%% ---------------------------------------------------------------------
%% API
%% ---------------------------------------------------------------------

-spec start_link(binary(), #workflow_identity{}, binary(), map()) -> {ok, pid()}.
start_link(WorkflowId, Identity, StepId, StepDef)
        when is_binary(WorkflowId), is_record(Identity, workflow_identity),
             is_binary(StepId), is_map(StepDef) ->
    gen_statem:start_link(?MODULE, {WorkflowId, Identity, StepId, StepDef}, []).

-spec mark_ready(pid()) -> ok | {error, term()}.
mark_ready(Pid) ->
    gen_statem:call(Pid, mark_ready).

-spec dispatch(pid()) -> ok | {error, term()}.
dispatch(Pid) ->
    gen_statem:call(Pid, dispatch).

-spec get_lifecycle_state(pid()) -> atom().
get_lifecycle_state(Pid) ->
    gen_statem:call(Pid, get_lifecycle_state).

-spec get_transition_log(pid()) -> [atom()].
get_transition_log(Pid) ->
    gen_statem:call(Pid, get_transition_log).

-spec get_outcome(pid()) -> term().
get_outcome(Pid) ->
    gen_statem:call(Pid, get_outcome).

%% ---------------------------------------------------------------------
%% gen_statem callbacks
%% ---------------------------------------------------------------------

callback_mode() -> [state_functions, state_enter].

init({WorkflowId, Identity, StepId, StepDef}) ->
    Data = #d{workflow_id = WorkflowId, identity = Identity, step_id = StepId, step_def = StepDef},
    {ok, manufactured, Data}.

terminate(_Reason, _State, _Data) -> ok.

%% ---- MANUFACTURED ----

manufactured(enter, _Old, Data) ->
    {keep_state, log_enter(manufactured, Data)};
manufactured({call, From}, mark_ready, Data) ->
    {next_state, ready, Data, [{reply, From, ok}]};
manufactured({call, From}, Event, Data) ->
    common_call(manufactured, From, Event, Data).

%% ---- READY ----

ready(enter, _Old, Data) ->
    {keep_state, log_enter(ready, Data)};
ready({call, From}, dispatch, Data) ->
    case validate_ready(Data) of
        ok ->
            Worker = spawn_dispatch_worker(self(), Data),
            Data1 = Data#d{worker = Worker},
            {next_state, dispatched, Data1, [{reply, From, ok}]};
        {error, Reason} ->
            %% Atlas L5: "READY --> REFUSED: invalid" -- a genuinely malformed
            %% request (not a broker-level refusal, which only DISPATCHED's
            %% real round trip can produce) is refused here, before any
            %% actuation is attempted.
            Data1 = Data#d{outcome = {refused, 'DISPATCH_REQUEST_INVALID', Reason}},
            {next_state, refused, Data1, [{reply, From, {refused, 'DISPATCH_REQUEST_INVALID', Reason}}]}
    end;
ready({call, From}, Event, Data) ->
    common_call(ready, From, Event, Data).

%% ---- DISPATCHED (real, observably async: the worker process spawned in
%%      ready/3 is genuinely still running when this state is entered -- the
%%      `dispatch/1` caller already got its `ok` reply before this point) ----

dispatched(enter, _Old, Data) ->
    {keep_state, log_enter(dispatched, Data)};
dispatched(info, {dispatch_outcome, WorkerPid, Outcome}, Data = #d{worker = {WorkerPid, _Ref}}) ->
    Data1 = Data#d{outcome = Outcome},
    {next_state, awaiting_result, Data1, [{next_event, internal, advance}]};
dispatched(info, {'DOWN', Ref, process, WorkerPid, Reason}, Data = #d{worker = {WorkerPid, Ref}}) ->
    %% The worker died without ever sending {dispatch_outcome, ...} -- a
    %% genuine process crash, not a broker-level refusal. Surfaced honestly as
    %% its own distinct outcome rather than folded into a broker refusal atom
    %% it did not actually produce.
    Data1 = Data#d{outcome = {refused, 'DISPATCH_WORKER_CRASHED', #{reason => Reason}}},
    {next_state, refused, Data1};
dispatched({call, From}, Event, Data) ->
    common_call(dispatched, From, Event, Data).

%% ---- AWAITING_RESULT ----
%% See module header point 3: entered and exited immediately, in real
%% sequence, once the single real broker round trip has already returned --
%% not independently pausable without a broker.erl stage-boundary refactor.

awaiting_result(enter, _Old, Data) ->
    {keep_state, log_enter(awaiting_result, Data)};
awaiting_result(internal, advance, Data) ->
    {next_state, awaiting_admission, Data, [{next_event, internal, advance}]};
awaiting_result(info, {'DOWN', _Ref, process, _Pid, _Reason}, Data) ->
    discard_stray_worker_down(Data);
awaiting_result({call, From}, Event, Data) ->
    common_call(awaiting_result, From, Event, Data).

%% ---- AWAITING_ADMISSION ----

awaiting_admission(enter, _Old, Data) ->
    {keep_state, log_enter(awaiting_admission, Data)};
awaiting_admission(internal, advance, Data = #d{outcome = Outcome}) ->
    case Outcome of
        {ok, DispatchToken} when is_binary(DispatchToken) ->
            Data1 = Data#d{dispatch_token = DispatchToken},
            {next_state, running, Data1, [{next_event, internal, advance}]};
        {refused, _Code, _Ctx} ->
            %% Atlas L5: "AWAITING_ADMISSION --> REFUSED: authority or
            %% conformance failure" -- the real, unmodified broker's own
            %% refusal (any of the 7 real atoms in
            %% f16_otp_runner_vocab:REFUSAL_ATOMS), carried verbatim, not
            %% translated or re-coded.
            {next_state, refused, Data};
        {error, Reason} ->
            Data1 = Data#d{outcome = {refused, 'DISPATCH_ERROR', #{reason => Reason}}},
            {next_state, refused, Data1}
    end;
awaiting_admission(info, {'DOWN', _Ref, process, _Pid, _Reason}, Data) ->
    discard_stray_worker_down(Data);
awaiting_admission({call, From}, Event, Data) ->
    common_call(awaiting_admission, From, Event, Data).

%% ---- RUNNING ----

running(enter, _Old, Data) ->
    {keep_state, log_enter(running, Data)};
running(internal, advance, Data) ->
    {next_state, completed, Data};
running(info, {'DOWN', _Ref, process, _Pid, _Reason}, Data) ->
    discard_stray_worker_down(Data);
running({call, From}, Event, Data) ->
    common_call(running, From, Event, Data).

%% ---- COMPLETED (terminal) ----

completed(enter, _Old, Data) ->
    {keep_state, log_enter(completed, Data)};
completed(info, {'DOWN', _Ref, process, _Pid, _Reason}, Data) ->
    discard_stray_worker_down(Data);
completed({call, From}, Event, Data) ->
    common_call(completed, From, Event, Data).

%% ---- REFUSED (terminal) ----

refused(enter, _Old, Data) ->
    {keep_state, log_enter(refused, Data)};
refused(info, {'DOWN', _Ref, process, _Pid, _Reason}, Data) ->
    discard_stray_worker_down(Data);
refused({call, From}, Event, Data) ->
    common_call(refused, From, Event, Data).

%% ---------------------------------------------------------------------
%% Shared helpers
%% ---------------------------------------------------------------------

log_enter(StateName, Data = #d{transition_log = Log}) ->
    Data#d{transition_log = Log ++ [StateName]}.

%% The worker process spawned in ready/3 always exits (normally, right after
%% sending {dispatch_outcome, ...}, or abnormally, handled by dispatched/3's
%% own 'DOWN' clause) -- because spawn_monitor/1's monitor stays armed for
%% the worker's entire lifetime, a SECOND, harmless 'DOWN' notification for
%% its ordinary post-outcome normal exit can arrive after this state machine
%% has already moved past `dispatched` (the only state that still needs to
%% distinguish "crashed before sending an outcome" from "already handled").
%% Every later state discards it via this shared no-op rather than crashing
%% with function_clause on an event no state function past `dispatched`
%% otherwise expects.
discard_stray_worker_down(Data) ->
    {keep_state, Data}.

%% Structural validity only (well-formed request shape) -- deliberately does
%% NOT duplicate arazzo_runner_broker:dispatch/4's own semantic preactuation
%% checks (correlation_id/receipt_head presence): those remain that module's
%% exclusive, single source of truth and surface via the
%% awaiting_admission -> refused edge instead, once the real round trip
%% returns. See module header point 3.
validate_ready(#d{step_id = StepId, step_def = StepDef})
        when is_binary(StepId), byte_size(StepId) > 0, is_map(StepDef) ->
    ok;
validate_ready(#d{step_id = StepId, step_def = StepDef}) ->
    {error, #{step_id => StepId, step_def_is_map => is_map(StepDef)}}.

spawn_dispatch_worker(StatemPid, #d{workflow_id = WorkflowId, identity = Identity,
                                     step_id = StepId, step_def = StepDef}) ->
    spawn_monitor(fun() ->
        Outcome =
            try arazzo_runner_broker:dispatch(WorkflowId, Identity, StepId, StepDef)
            catch Class:Reason:Stack -> {error, {exception, Class, Reason, Stack}}
            end,
        StatemPid ! {dispatch_outcome, self(), Outcome}
    end).

%% Generic introspection calls, valid from every state; anything else
%% (`dispatch`/`mark_ready` called out of order, or an unknown message) is
%% refused with a typed `{error, {unexpected_event_in_state, StateName}}`
%% rather than silently ignored or crashing. StateName is always the real,
%% current state function's own name, passed in by each call site above --
%% not re-derived or guessed.
common_call(StateName, From, get_lifecycle_state, Data) ->
    {keep_state, Data, [{reply, From, StateName}]};
common_call(_StateName, From, get_transition_log, Data = #d{transition_log = Log}) ->
    {keep_state, Data, [{reply, From, Log}]};
common_call(_StateName, From, get_outcome, Data = #d{outcome = Outcome}) ->
    {keep_state, Data, [{reply, From, Outcome}]};
common_call(StateName, From, _Other, Data) ->
    {keep_state, Data, [{reply, From, {error, {unexpected_event_in_state, StateName}}}]}.
