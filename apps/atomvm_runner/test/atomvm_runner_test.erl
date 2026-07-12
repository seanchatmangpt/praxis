-module(atomvm_runner_test).
-include_lib("eunit/include/eunit.hrl").

%% PROJ-760 (PRD v26.7.11 7.9, Layer 9 -- AtomVM Runner) proof suite.
%%
%% Unlike arazzo_runner_workflow_test.erl (the OTP-side wrapper's proof
%% suite), there is no supervisor, DETS persistence, or crash-restart
%% story here -- arazzo_atomvm_workflow (the actor loop atomvm_runner
%% delegates to) is a bare `receive` loop with none of that, by design
%% (PRD 7.10 / 7.9's constrained-microcontroller framing). What this suite
%% proves instead: atomvm_runner's exported functions reach a genuine,
%% non-crashing execution path through air_core:transition/2 -- real state
%% advancement across multiple steps, not a simulated or hardcoded
%% response -- exercised the same way arazzo_runner_workflow_test.erl
%% exercises the OTP-side wrapper, without needing a live external system
%% (no real AtomVM runtime is installed in this environment; see
%% atomvm_runner.erl's module comment).

sample_workflow() ->
    #{steps => #{
        <<"step_a">> => #{
            outputs => [{bind, <<"step_a_done">>, {literal, true}}],
            next => [<<"step_b">>]
        },
        <<"step_b">> => #{
            outputs => [{bind, <<"step_b_done">>, {literal, true}}],
            next => []
        }
    }}.

get_state(Pid) ->
    {ok, State} = atomvm_runner:get_state(Pid),
    State.

%% ---------------------------------------------------------------------
%% Proof 1: start/2 with a real, non-trivial two-step workflow genuinely
%% advances state across successive air_core:transition/2 calls -- ready
%% steps, bound env vars, and history length all change exactly as
%% air_core's own AND/join semantics (PROJ-756) dictate, reached only
%% through atomvm_runner's public API (start/2, dispatch_event/2,
%% get_state/1), never by calling air_core or arazzo_atomvm_workflow
%% directly.
%% ---------------------------------------------------------------------

real_transition_through_air_core_test() ->
    WorkflowId = <<"wf-atomvm-1">>,
    {ok, Pid} = atomvm_runner:start(WorkflowId, #{
        workflow => sample_workflow(),
        active_steps => [<<"step_a">>]
    }),

    S0 = get_state(Pid),
    ?assertEqual([<<"step_a">>], air_core:ready_steps(S0)),
    ?assertEqual(#{}, air_core:get_env(S0)),
    ?assertEqual(0, length(air_core:get_history(S0))),

    ok = atomvm_runner:dispatch_event(Pid, {step_completed, <<"step_a">>, ok}),
    S1 = wait_for_history_len(Pid, 1),
    ?assertEqual([<<"step_b">>], air_core:ready_steps(S1)),
    ?assertEqual(true, maps:get(<<"step_a_done">>, air_core:get_env(S1))),

    ok = atomvm_runner:dispatch_event(Pid, {step_completed, <<"step_b">>, ok}),
    S2 = wait_for_history_len(Pid, 2),
    ?assertEqual([], air_core:ready_steps(S2)),
    ?assertEqual(true, maps:get(<<"step_a_done">>, air_core:get_env(S2))),
    ?assertEqual(true, maps:get(<<"step_b_done">>, air_core:get_env(S2))),

    ok = atomvm_runner:stop(Pid),
    ok.

%% ---------------------------------------------------------------------
%% Proof 2: a step_failed event never unlocks an AND/join successor that
%% depends on it (air_core:handle_step_failed/3's documented behavior),
%% observed genuinely through atomvm_runner -- proves this wrapper does
%% not paper over or diverge from air_core's real refusal-shaped semantics
%% (PROJ-756's completed_mask discipline).
%% ---------------------------------------------------------------------

step_failed_blocks_join_test() ->
    WorkflowId = <<"wf-atomvm-2">>,
    Workflow = #{steps => #{
        <<"a">> => #{outputs => [], next => [<<"c">>]},
        <<"b">> => #{outputs => [], next => [<<"c">>]},
        <<"c">> => #{outputs => [], next => []}
    }},
    {ok, Pid} = atomvm_runner:start(WorkflowId, #{
        workflow => Workflow,
        active_steps => [<<"a">>, <<"b">>]
    }),

    ok = atomvm_runner:dispatch_event(Pid, {step_completed, <<"a">>, ok}),
    S1 = wait_for_history_len(Pid, 1),
    %% "c" needs both "a" and "b" complete; only "a" has, so "c" stays
    %% unready.
    ?assertEqual([<<"b">>], air_core:ready_steps(S1)),

    ok = atomvm_runner:dispatch_event(Pid, {step_failed, <<"b">>, some_reason}),
    S2 = wait_for_history_len(Pid, 2),
    %% "b" failed rather than completed: "c" must never become ready.
    ?assertEqual([], air_core:ready_steps(S2)),

    ok = atomvm_runner:stop(Pid),
    ok.

%% ---------------------------------------------------------------------
%% Proof 3: regression guard for the initial_state bug found and fixed
%% this session (PROJ-760) -- start/1 (the zero-workflow convenience form)
%% must produce a live, dispatchable actor, not one that crashes on its
%% first event with a #context{} function_clause error.
%% ---------------------------------------------------------------------

start_1_does_not_crash_on_first_event_test() ->
    {ok, Pid} = atomvm_runner:start(<<"wf-atomvm-3">>),
    Mon = monitor(process, Pid),
    ok = atomvm_runner:dispatch_event(Pid, {step_completed, <<"nonexistent_step">>, ok}),
    receive
        {'DOWN', Mon, process, Pid, Reason} ->
            error({regression_start_1_crashed, Reason})
    after 200 ->
        ok
    end,
    S = get_state(Pid),
    ?assertEqual([], air_core:ready_steps(S)),
    ok = atomvm_runner:stop(Pid),
    ok.

%% ---------------------------------------------------------------------
%% Helpers
%% ---------------------------------------------------------------------

wait_for_history_len(Pid, N) ->
    wait_for_history_len(Pid, N, 200).

wait_for_history_len(_Pid, N, 0) ->
    error({timeout_waiting_for_history_len, N});
wait_for_history_len(Pid, N, Retries) ->
    State = get_state(Pid),
    case length(air_core:get_history(State)) of
        Len when Len >= N -> State;
        _ ->
            timer:sleep(10),
            wait_for_history_len(Pid, N, Retries - 1)
    end.
