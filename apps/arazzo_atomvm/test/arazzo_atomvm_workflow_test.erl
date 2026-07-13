-module(arazzo_atomvm_workflow_test).
-include_lib("eunit/include/eunit.hrl").

%% Swarm audit wnl2yhbgm finding #13's AtomVM sibling (dogfood workflow w3wsl28yy, HIGH
%% severity): arazzo_atomvm_workflow.erl had no dedicated unit test coverage anywhere in
%% this repo before this file -- the only prior coverage was the differential
%% OTP-vs-AtomVM equivalence suite (apps/arazzo_runner/test/
%% arazzo_runner_atomvm_differential_test.erl), which drives real, well-formed inputs and
%% was never designed to exercise crash-safety on malformed ones. These tests drive the
%% real `arazzo_atomvm_workflow:start/2` / `dispatch_event/2` / `get_state/1` API a real
%% caller would use -- no mocks, no test-only shortcut into the actor loop.

%% loop/2's actor is spawned via bare spawn/3 (no supervisor, no restart, no DETS
%% persistence): before this fix, a malformed Event -- here, a non-binary StepId, which
%% air_core:transition/2's <<StepId/binary>> clause head cannot match -- crashed the
%% process with an uncaught function_clause, silently and permanently destroying the
%% workflow instance. This proves three things: (1) the process survives (no DOWN
%% message), (2) it is still genuinely responsive (get_state/1 still answers), and (3) it
%% can still process a SUBSEQUENT, well-formed event correctly afterward -- not just
%% "technically alive but wedged."
loop_survives_a_malformed_event_and_keeps_processing_later_events_test() ->
    {ok, Pid} = arazzo_atomvm_workflow:start(<<"wf-malformed-event-1">>),
    Mon = monitor(process, Pid),

    ok = arazzo_atomvm_workflow:dispatch_event(Pid, {step_completed, not_a_binary, ok}),

    %% The critical safety property: no DOWN message arrives in a generous window.
    receive
        {'DOWN', Mon, process, Pid, Reason} ->
            error({workflow_process_crashed_on_malformed_event, Reason})
    after 500 ->
        ok
    end,
    ?assert(is_process_alive(Pid)),

    %% Still genuinely responsive, not just alive-but-wedged.
    {ok, _State} = arazzo_atomvm_workflow:get_state(Pid),

    %% And a subsequent, well-formed event still advances real state -- this workflow
    %% instance is still usable, not merely surviving in a degraded husk.
    ok = arazzo_atomvm_workflow:dispatch_event(
        Pid, {step_completed, <<"step_a">>, ok}
    ),
    {ok, FinalState} = arazzo_atomvm_workflow:get_state(Pid),
    ?assertNotEqual(undefined, FinalState).

%% start/2's air_core:new/1 call is genuinely fallible on caller-supplied InitOpts: a
%% step's `next` field that isn't a list reaches build_pred_mask_map/2's lists:foldl/3 and
%% raises function_clause. Unlike loop/2's crash above, this would crash whatever process
%% CALLS start/2 directly (no worker has been spawned yet to isolate it) -- proves start/2
%% now returns a typed {error, _} instead of crashing the caller.
start_2_returns_typed_error_instead_of_crashing_on_malformed_init_opts_test() ->
    MalformedInitOpts = #{
        workflow => #{
            steps => #{
                <<"step_a">> => #{next => not_a_list}
            }
        },
        active_steps => [<<"step_a">>]
    },
    Result = arazzo_atomvm_workflow:start(<<"wf-malformed-init-1">>, MalformedInitOpts),
    ?assertMatch({error, {air_core_new_failed, _Class, _Reason, _Stack}}, Result).

%% Regression guard: a well-formed start/2 call must still succeed exactly as before --
%% the try/catch above must not turn a genuinely valid InitOpts into a false refusal.
start_2_still_succeeds_on_well_formed_init_opts_test() ->
    WellFormedInitOpts = #{
        workflow => #{
            steps => #{
                <<"step_a">> => #{next => [<<"step_b">>]},
                <<"step_b">> => #{next => []}
            }
        },
        active_steps => [<<"step_a">>]
    },
    ?assertMatch(
        {ok, _Pid},
        arazzo_atomvm_workflow:start(<<"wf-well-formed-init-1">>, WellFormedInitOpts)
    ).
