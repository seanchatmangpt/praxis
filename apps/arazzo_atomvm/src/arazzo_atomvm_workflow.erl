-module(arazzo_atomvm_workflow).
-export([start/1, start/2, dispatch_event/2, stop/1, get_state/1]).
-export([loop/2, loop_waiting_for_io/2, execute_io_worker/2]).

%% API

%% PROJ-760: start/1's previous body checked
%% `erlang:function_exported(air_core, initial_state, 0)` -- air_core has
%% never exported an `initial_state/0` function (see its `-export([...])`
%% list in apps/air_core/src/air_core.erl: new/1, ready_steps/1,
%% transition/2, eval_expr/2,3, eval_criteria/2, apply_action/2,
%% bind_outputs/3, get_env/1, get_history/1, set_active_steps/2 -- no
%% initial_state/0), so that check was always false and CoreState was
%% always the atom `undefined`. Reproduced empirically this session: a
%% fresh `start/1` process died on its first dispatched event with
%% `{function_clause, [{air_core,handle_step_completed,
%% [<<"step_a">>,ok,undefined], ...}]}`, since `undefined` cannot match
%% air_core's `#context{}` record pattern in the handler function heads.
%% This is the "genuine execution path through air_core:transition/2, not
%% a simulated or hardcoded response" requirement PROJ-760 exists to
%% satisfy -- a wrapper that crashes on the first real event does not meet
%% it. Fixed here by constructing a real, valid, empty context via
%% air_core:new/1 (the same constructor arazzo_runner_workflow.erl uses
%% for a fresh workflow) instead of relying on a function that never
%% existed.
start(WorkflowId) ->
    start(WorkflowId, #{}).

%% start/2 accepts an air_core:new/1-shaped Opts map (workflow,
%% active_steps, env, history) so a real, non-trivial workflow definition
%% can be supplied -- mirroring arazzo_runner_workflow:start_link/1's
%% StartSpec-driven construction on the OTP side. start/1 above is the
%% zero-workflow convenience case (Opts = #{}, i.e. air_core:new(#{}), a
%% valid empty context with no steps).
%%
%% Swarm audit wnl2yhbgm finding #13's AtomVM sibling (related exposure,
%% dogfood workflow w3wsl28yy): air_core:new/1 is genuinely fallible on a
%% caller-supplied InitOpts shape (e.g. a step's `next` field that isn't a
%% list reaches build_pred_mask_map/2's lists:foldl/3 and raises). Unlike
%% loop/2's crash below, an exception here would crash whatever process
%% CALLS start/2 directly -- no worker process has been spawned yet to
%% isolate it, and this module has no supervisor. Caught and returned as a
%% typed {error, _}, matching Erlang's own constructor-that-can-fail
%% convention, rather than crashing the caller. Confirmed no production
%% caller pattern-matches this return in a way that would break: the only
%% non-test caller is atomvm_runner:start/2, a thin pass-through delegation
%% facade with no {ok, _} = ... match on the result.
-spec start(binary(), map()) -> {ok, pid()} | {error, term()}.
start(WorkflowId, InitOpts) when is_map(InitOpts) ->
    try air_core:new(InitOpts) of
        InitialCoreState ->
            Pid = spawn(?MODULE, loop, [WorkflowId, InitialCoreState]),
            {ok, Pid}
    catch
        Class:Reason:Stack ->
            {error, {air_core_new_failed, Class, Reason, Stack}}
    end.

dispatch_event(Pid, Event) ->
    Pid ! {event, Event},
    ok.

stop(Pid) ->
    Pid ! stop,
    ok.

%% get_state/1: synchronous introspection of the actor's current
%% air_core:context(). This bare `receive` loop implements no `sys`
%% protocol (it is spawned with plain spawn/3, not proc_lib), so
%% sys:get_state/1 does not work against it; this is the real accessor
%% that lets callers (tests, a future differential harness) observe genuine
%% state advancement instead of only inferring liveness from the absence
%% of a crash.
-spec get_state(pid()) -> {ok, term()}.
get_state(Pid) ->
    Pid ! {get_state, self()},
    receive
        {state, State} -> {ok, State}
    after 1000 ->
        error(get_state_timeout)
    end.

%% Actor loop

%% NOTE: PROJ-755 fixed air_core:transition/2's return shape to the PRD
%% v26.7.11 7.7 contract delta_AIR: (S,E) -> (S',C) -- it now always
%% returns {NewContext, Commands} (a plain 2-tuple; air_core itself never
%% produces an ok/io_request/error/stop envelope). The `{ok, _}` /
%% `{io_request, _, _}` / `{error, _}` / `{stop, normal, _}` clauses below
%% were already unreachable before this fix (the prior bare-context() return
%% only ever matched the old fallback clause; see git history) and remain
%% unreachable now for the same reason -- transition/2's first tuple element
%% is always a #context{} record, never one of those atoms. They are left in
%% place, unchanged, as the previously-established target shape for a future
%% protocol layer; only the fallback clause is updated, to the shape
%% transition/2 actually produces today. Commands is currently unconsumed:
%% PROJ-758 (the broker that routes C through lawful surfaces) is not yet
%% built, so this actor only advances state.

loop(WorkflowId, CoreState) ->
    receive
        {event, Event} ->
            %% Swarm audit wnl2yhbgm finding #13's AtomVM sibling (dogfood workflow
            %% w3wsl28yy, HIGH severity): air_core:transition/2 is genuinely fallible on
            %% caller-supplied Event/CoreState shapes -- a malformed StepId, or a step's
            %% outputs bind-rule with the wrong arity, the identical trigger class
            %% arazzo_runner_workflow.erl's apply_transition/4 was fixed against in commit
            %% 2966330f. Unlike that OTP sibling, THIS actor is spawned via bare spawn/3
            %% (start/2 above) -- no supervisor, no restart, no DETS persistence -- so an
            %% uncaught exception here does not just crash a supervised process that gets
            %% respawned; it silently, permanently destroys the workflow instance with no
            %% trace. Caught here, logged, and the one malformed event is discarded: the
            %% loop continues with CoreState UNCHANGED, so every other in-flight/future
            %% event for this workflow instance is unaffected -- matching
            %% apply_transition/4's own established try/catch-then-case idiom (a 4-tuple
            %% {exception, Class, Reason, Stack} sentinel cannot collide with any of this
            %% case's other, real air_core:transition/2 return shapes below, all of which
            %% are 2- or 3-tuples).
            case try air_core:transition(Event, CoreState)
                 catch C:R:S -> {exception, C, R, S}
                 end of
                {exception, Class, Reason, Stack} ->
                    error_logger:error_msg(
                        "arazzo_atomvm_workflow ~p: air_core:transition/2 crashed on "
                        "event ~p: ~p:~p ~p",
                        [WorkflowId, Event, Class, Reason, Stack]
                    ),
                    loop(WorkflowId, CoreState);
                {ok, NewCoreState} ->
                    loop(WorkflowId, NewCoreState);
                {io_request, Req, NewCoreState} ->
                    spawn_worker(Req),
                    loop_waiting_for_io(WorkflowId, NewCoreState);
                {error, Reason} ->
                    exit({error, Reason});
                {stop, normal, _NewCoreState} ->
                    exit(normal);
                {NewCoreState, _Commands} ->
                    %% The real, always-taken shape as of PROJ-755.
                    loop(WorkflowId, NewCoreState)
            end;
        {get_state, From} ->
            From ! {state, CoreState},
            loop(WorkflowId, CoreState);
        stop ->
            exit(normal);
        _Other ->
            loop(WorkflowId, CoreState)
    end.

loop_waiting_for_io(WorkflowId, CoreState) ->
    receive
        {io_reply, _Reply} = Msg ->
            case air_core:transition(Msg, CoreState) of
                {ok, NewCoreState} ->
                    loop(WorkflowId, NewCoreState);
                {io_request, Req, NewCoreState} ->
                    spawn_worker(Req),
                    loop_waiting_for_io(WorkflowId, NewCoreState);
                {error, Reason} ->
                    exit({error, Reason});
                {stop, normal, _NewCoreState} ->
                    exit(normal);
                {NewCoreState, _Commands} ->
                    loop(WorkflowId, NewCoreState)
            end;
        {get_state, From} ->
            From ! {state, CoreState},
            loop_waiting_for_io(WorkflowId, CoreState);
        stop ->
            exit(normal);
        _Other ->
            loop_waiting_for_io(WorkflowId, CoreState)
    end.

spawn_worker(Req) ->
    spawn(?MODULE, execute_io_worker, [self(), Req]).

execute_io_worker(Parent, Req) ->
    Reply = execute_io_request(Req),
    Parent ! {io_reply, Reply}.

execute_io_request(Req) ->
    %% Placeholder for real active I/O execution
    {ok, {processed, Req}}.
