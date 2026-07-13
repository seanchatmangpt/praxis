-module(arazzo_runner_workflow_test).
-include_lib("eunit/include/eunit.hrl").
-include("arazzo_runner.hrl").

%% PROJ-757 (PRD v26.7.11 7.8, Layer 8 -- OTP Outer Runner) proof suite.
%%
%% Each test uses a fresh, isolated DETS directory (see setup/0) so runs
%% never see another test's (or another run's) persisted state.

%% ---------------------------------------------------------------------
%% Fixture
%% ---------------------------------------------------------------------

arazzo_runner_workflow_test_() ->
    {setup,
     fun setup/0,
     fun cleanup/1,
     fun(_) ->
         [
             {"10 identity fields are present and PID-independent across a "
              "real supervisor-driven crash+restart",
              fun test_identity_survives_supervisor_restart/0},
             {"start/result/child_complete (and other) reactions genuinely "
              "change observable state, not just accepted-and-ignored",
              fun test_reactions_change_observable_state/0},
             {"a genuine crash of BOTH the workflow process and the ETS-owning "
              "infra process still reconstructs full execution state, purely "
              "from durable (DETS) persistence",
              fun test_crash_restart_reconstructs_from_dets/0},
             {"swarm audit wnl2yhbgm finding #13: a genuinely malformed step "
              "outputs field that crashes required_result_types/1 inside the "
              "broker dispatch call is caught, not left to crash the whole "
              "workflow process",
              fun test_broker_dispatch_exception_does_not_crash_the_workflow/0},
             {"swarm audit wnl2yhbgm finding #13's OTP-runner reaction-dispatch "
              "sibling: an unrecognized event tag reaching handle_reaction/3 is "
              "caught by a catch-all clause, not left to crash the workflow "
              "process via an uncaught function_clause",
              fun test_unrecognized_reaction_event_does_not_crash_the_workflow/0}
         ]
     end}.

setup() ->
    Dir = filename:join(
        "/tmp",
        "arazzo_runner_eunit_" ++ integer_to_list(erlang:unique_integer([positive]))
    ),
    ok = filelib:ensure_dir(filename:join(Dir, "x")),
    ok = application:set_env(arazzo_runner, state_dir, Dir),
    Dir.

cleanup(Dir) ->
    catch application:stop(arazzo_runner),
    catch arazzo_runner_identity:close_table(),
    catch os:cmd("rm -rf " ++ Dir),
    ok = application:unset_env(arazzo_runner, state_dir),
    ok.

%% ---------------------------------------------------------------------
%% Shared fixtures
%% ---------------------------------------------------------------------

sample_identity(WorkflowId) ->
    #{
        workflow_id => WorkflowId,
        parent_workflow_id => undefined,
        arazzo_workflow_id => <<"arazzo-wf-42">>,
        source_powl_region_id => <<"powl-region-7">>,
        dispatch_id => <<"dispatch-1">>,
        correlation_id => <<"corr-1">>,
        source_digest => <<"src-digest-abc">>,
        projection_digest => <<"proj-digest-def">>,
        receipt_head => <<"receipt-head-ghi">>,
        replay_id => <<"replay-1">>
    }.

%% step_b's outputs deliberately reference the real air_core sentinel
%% {var, '__result__'} under a type-coercing op ('and'), not a bare
%% {literal, true} -- this is what keeps step_b genuinely pending after
%% step_a's `result` reaction, for the two tests below that need to drive
%% step_b's completion themselves (acknowledgment/child_complete in
%% test_reactions_change_observable_state/0, a second explicit `result` in
%% test_crash_restart_reconstructs_from_dets/0).
%%
%% Since PROJ-758's broker fix (this ticket) wired admit_return/3 into the
%% real production dispatch path (do_dispatch_actuate/6), EVERY
%% dispatch_step command apply_transition/4 produces is routed through a
%% REAL round trip: arazzo_runner_broker:dispatch/4 -> the echo io-worker
%% (execute_io_request/1, arazzo_runner_workflow.erl) -> admit_return/3.
%% The echo worker always actuates to a `{processed, StepDef}` tuple; a
%% step whose declared outputs never reference `__result__` under a typed
%% op (like step_a's own {literal, true} above) derives an EMPTY
%% required_result_types set (arazzo_runner_broker:required_result_types/1),
%% which that tuple vacuously satisfies -- so such a step's dispatch
%% auto-admits and the step self-completes the instant it becomes ready,
%% with no test-controlled event involved at all (see
%% arazzo_runner_broker_test.erl's test_full_dispatch_correlation_return_
%% round_trip/0, which relies on exactly that for step_b there). step_b
%% HERE needs the opposite: required_result_types/1 derives `[boolean]`
%% from the 'and' op below, and a tuple never satisfies `is_boolean/1`, so
%% admit_return/3's structure stage (RETURN_STRUCTURE_REFUSED, PROJ-785)
%% refuses every broker-synthesized auto-dispatch of step_b, deterministically,
%% every run -- not a race against how fast the echo round trip completes.
%% Only this file's own explicit `child_complete`/`result` reaction events
%% (which pass Result = true, a real boolean) ever genuinely admit step_b.
sample_workflow_def() ->
    #{steps => #{
        <<"step_a">> => #{
            outputs => [{bind, <<"step_a_done">>, {literal, true}}],
            next => [<<"step_b">>]
        },
        <<"step_b">> => #{
            outputs => [{bind, <<"step_b_done">>, {op, 'and', {var, '__result__'}, {literal, true}}}],
            next => []
        }
    }}.

start_spec(WorkflowId) ->
    maps:merge(sample_identity(WorkflowId), #{
        workflow_def => sample_workflow_def(),
        active_steps => [<<"step_a">>],
        env => #{},
        history => []
    }).

%% Reaction tags (e.g. `result`) are not unique across a test -- the same
%% tag legitimately recurs (e.g. once before a crash, again after restart on
%% the next step). A plain "is Tag present anywhere in reaction_log" check
%% would race: it can be satisfied instantly by a *stale* entry from before
%% the event now in flight was even sent, letting the caller's assertions
%% run before the real, new reaction has been processed. Instead: capture
%% reaction_count/1 (the log length) right before triggering an event, then
%% wait for the log to actually grow, and verify the new head is Tag --
%% record_reaction/2 always prepends, so whatever handle_reaction/3 (or
%% apply_transition/4's own trailing record_reaction/2 call) added last for
%% this event is guaranteed to be at the head once the length has grown.
reaction_count(WorkflowId) ->
    case arazzo_runner_workflow:get_runner_state(WorkflowId) of
        {ok, #runner_state{reaction_log = Log}} -> length(Log);
        not_found -> 0
    end.

wait_for_reaction(WorkflowId, PriorCount, Tag) ->
    wait_for_reaction(WorkflowId, PriorCount, Tag, 200).

wait_for_reaction(WorkflowId, PriorCount, Tag, 0) ->
    error({timeout_waiting_for_reaction, WorkflowId, PriorCount, Tag});
wait_for_reaction(WorkflowId, PriorCount, Tag, N) ->
    case arazzo_runner_workflow:get_runner_state(WorkflowId) of
        {ok, #runner_state{reaction_log = Log} = RS} when length(Log) > PriorCount ->
            ?assertEqual(Tag, hd(Log)),
            RS;
        _ ->
            timer:sleep(10),
            wait_for_reaction(WorkflowId, PriorCount, Tag, N - 1)
    end.

wait_for_new_child(Sup, OldPid) ->
    wait_for_new_child(Sup, OldPid, 200).

wait_for_new_child(_Sup, _OldPid, 0) ->
    error(timeout_waiting_for_supervisor_restart);
wait_for_new_child(Sup, OldPid, N) ->
    Children = supervisor:which_children(Sup),
    case [Pid || {_, Pid, _, _} <- Children, is_pid(Pid), Pid =/= OldPid] of
        [NewPid | _] -> NewPid;
        [] -> timer:sleep(10), wait_for_new_child(Sup, OldPid, N - 1)
    end.

%% ---------------------------------------------------------------------
%% Proof 1: identity fields present + PID-independent across a real
%% supervisor-driven crash+restart (not a simulated one -- we really
%% `exit(Pid, kill)` a real process and observe the real supervisor
%% respawn it under `restart => transient`).
%% ---------------------------------------------------------------------

test_identity_survives_supervisor_restart() ->
    {ok, _Started} = application:ensure_all_started(arazzo_runner),
    WorkflowId = <<"wf-restart-1">>,
    StartSpec = start_spec(WorkflowId),

    {ok, Pid1} = arazzo_runner_sup:start_workflow(StartSpec),

    {ok, Id1} = arazzo_runner_workflow:get_identity(WorkflowId),
    %% All 10 PRD 7.8 fields are present (record tag + 10 fields = 11).
    ?assertEqual(11, tuple_size(Id1)),
    IdFields = record_info(fields, workflow_identity),
    ?assertEqual(10, length(IdFields)),
    ?assertEqual(WorkflowId, Id1#workflow_identity.workflow_id),
    ?assertEqual(undefined, Id1#workflow_identity.parent_workflow_id),
    ?assertEqual(<<"arazzo-wf-42">>, Id1#workflow_identity.arazzo_workflow_id),
    ?assertEqual(<<"powl-region-7">>, Id1#workflow_identity.source_powl_region_id),
    ?assertEqual(<<"dispatch-1">>, Id1#workflow_identity.dispatch_id),
    ?assertEqual(<<"corr-1">>, Id1#workflow_identity.correlation_id),
    ?assertEqual(<<"src-digest-abc">>, Id1#workflow_identity.source_digest),
    ?assertEqual(<<"proj-digest-def">>, Id1#workflow_identity.projection_digest),
    ?assertEqual(<<"receipt-head-ghi">>, Id1#workflow_identity.receipt_head),
    ?assertEqual(<<"replay-1">>, Id1#workflow_identity.replay_id),

    %% Real crash: an untrappable kill signal, not a manufactured "stop" --
    %% `restart => transient` restarts on exactly this kind of abnormal exit.
    Mon = monitor(process, Pid1),
    true = erlang:exit(Pid1, kill),
    receive
        {'DOWN', Mon, process, Pid1, killed} -> ok
    after 1000 ->
        error(workflow_process_did_not_die)
    end,

    %% Real supervisor behavior, observed via supervisor:which_children/1 --
    %% not a direct call into arazzo_runner_workflow internals.
    Pid2 = wait_for_new_child(arazzo_runner_sup, Pid1),
    ?assertNotEqual(Pid1, Pid2),

    {ok, Id2} = arazzo_runner_workflow:get_identity(WorkflowId),
    ?assertEqual(Id1, Id2),
    ok.

%% ---------------------------------------------------------------------
%% Proof 2: reaction events genuinely change observable state.
%% ---------------------------------------------------------------------

test_reactions_change_observable_state() ->
    %% start_link/1 is called directly (not via the supervisor) in this
    %% test, which links Pid to this process (proc_lib:spawn_link inside
    %% start_link/1 links to its *caller*). This test deliberately drives
    %% Pid to a non-normal exit (the admission-refusal case below), so trap
    %% exits to observe that via a message instead of dying ourselves.
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-react-1">>,
    StartSpec = start_spec(WorkflowId),
    {ok, Pid} = arazzo_runner_workflow:start_link(StartSpec),

    %% start
    {ok, RS0} = arazzo_runner_workflow:get_runner_state(WorkflowId),
    ?assertEqual([start], RS0#runner_state.reaction_log),
    ?assertEqual([<<"step_a">>], air_core:ready_steps(RS0#runner_state.core)),

    %% result: real air_core transition, env/ready-steps genuinely advance,
    %% and a dispatch_ready is genuinely synthesized for step_b.
    C0 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {result, <<"step_a">>, ok}),
    RS1 = wait_for_reaction(WorkflowId, C0, result),
    ?assertEqual(true, maps:get(<<"step_a_done">>, air_core:get_env(RS1#runner_state.core))),
    ?assertEqual([<<"step_b">>], air_core:ready_steps(RS1#runner_state.core)),
    ?assertEqual([{<<"step_b">>, maps:get(<<"step_b">>, maps:get(steps, sample_workflow_def()))}],
                 RS1#runner_state.pending_dispatches),
    %% step_b's own broker-synthesized dispatch (apply_transition/4's foldl
    %% over the dispatch_step command air_core just produced) really did
    %% round-trip through the broker and get genuinely, deterministically
    %% refused at the return-admission structure stage -- see
    %% sample_workflow_def/0's own doc comment for why. This is what proves
    %% step_b staying ready (assertion above) is not a race this test
    %% happened to win, but a real gate; the acknowledgment/child_complete
    %% reactions below are what actually drive step_b to completion.
    ?assertMatch([{<<"step_b">>, {broker, 'RETURN_STRUCTURE_REFUSED', _}}],
                 RS1#runner_state.refusals),

    %% acknowledgment: moves step_b out of pending_dispatches.
    C1 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {acknowledgment, <<"step_b">>, #{seen => true}}),
    RS2 = wait_for_reaction(WorkflowId, C1, {acknowledgment, <<"step_b">>}),
    ?assertEqual([], RS2#runner_state.pending_dispatches),
    ?assertEqual([{<<"step_b">>, #{seen => true}}], RS2#runner_state.acknowledged),

    %% child_complete: folds into the parent's air_core state as the
    %% completion of step_b (the step that "spawned" the child), and
    %% records the child's outcome. Result is `true` (not an arbitrary
    %% atom like `done`) because step_b's own outputs
    %% (sample_workflow_def/0) now do real bool-typed arithmetic
    %% (`{op, 'and', {var, '__result__'}, {literal, true}}`) on whatever
    %% Result this event admits -- this is step_b's FIRST genuine
    %% completion (the broker's own auto-dispatch attempt for step_b was
    %% refused above, never admitted), so this is exactly the real
    %% bind_outputs/3 -> eval_expr_nif evaluation the assertion below
    %% checks.
    C2 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {child_complete, <<"child-1">>, <<"step_b">>, true}),
    RS3 = wait_for_reaction(WorkflowId, C2, {child_complete, <<"child-1">>}),
    ?assertEqual(true, maps:get(<<"step_b_done">>, air_core:get_env(RS3#runner_state.core))),
    ?assertEqual([], air_core:ready_steps(RS3#runner_state.core)),
    ?assertEqual({complete, true}, maps:get(<<"child-1">>, RS3#runner_state.children)),
    %% Exactly 2 air_core transitions have been applied so far: `result`
    %% (step_a) and `child_complete` (folded as step_b's completion).
    %% `acknowledgment` between them is pure runner bookkeeping and applies
    %% no air_core transition, so it does not add to history.
    ?assertEqual(2, length(air_core:get_history(RS3#runner_state.core))),

    %% timeout: treated as a step failure -- observable via history growth
    %% and the reaction log, on a step that hasn't run yet.
    HistBeforeTimeout = length(air_core:get_history(RS3#runner_state.core)),
    C3 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {timeout, <<"step_c_never_dispatched">>}),
    RS4 = wait_for_reaction(WorkflowId, C3, timeout),
    ?assertEqual(HistBeforeTimeout + 1, length(air_core:get_history(RS4#runner_state.core))),

    %% retry_due: observable retry counter increments.
    C4 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {retry_due, <<"step_c_never_dispatched">>}),
    RS5 = wait_for_reaction(WorkflowId, C4, {retry_due, <<"step_c_never_dispatched">>}),
    ?assertEqual(#{<<"step_c_never_dispatched">> => 1}, RS5#runner_state.retry_counts),

    %% admission_result (refused): terminates the process deliberately.
    Mon = monitor(process, Pid),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {admission_result, {refused, <<"no projection receipt">>}}),
    receive
        {'DOWN', Mon, process, Pid, {admission_refused, <<"no projection receipt">>}} -> ok
    after 1000 ->
        error(admission_refusal_did_not_terminate_process)
    end,
    {ok, RS6} = arazzo_runner_workflow:get_runner_state(WorkflowId),
    ?assertEqual([{refused, <<"no projection receipt">>}], RS6#runner_state.admission_log),
    ok.

%% ---------------------------------------------------------------------
%% Proof 3: genuine crash-and-restart reconstructs state purely from DETS.
%%
%% Kills BOTH the workflow process AND the infra process that owns the ETS
%% cache table, so nothing whatsoever survives in memory -- the only thing
%% that can possibly answer the next start_link/1 call is what was
%% actually flushed to disk (arazzo_runner_identity:persist/1's
%% dets:sync/1) before the kill.
%% ---------------------------------------------------------------------

test_crash_restart_reconstructs_from_dets() ->
    %% Same trap_exit rationale as test_reactions_change_observable_state/0:
    %% start_link/1 is called directly, linking Pid1 (and later Pid2) to
    %% this process, and this test deliberately kills Pid1.
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-crash-1">>,
    StartSpec = start_spec(WorkflowId),

    {ok, Pid1} = arazzo_runner_workflow:start_link(StartSpec),
    C0 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid1, {result, <<"step_a">>, ok}),
    RS1 = wait_for_reaction(WorkflowId, C0, result),
    ?assertEqual(true, maps:get(<<"step_a_done">>, air_core:get_env(RS1#runner_state.core))),
    %% step_b stays genuinely ready (not auto-completed by the broker's own
    %% closed loop) for the same reason test_reactions_change_observable_
    %% state/0 documents: sample_workflow_def/0's step_b outputs require a
    %% real boolean `__result__`, which the echo io-worker's tuple response
    %% never supplies, so admit_return/3 refuses the auto-dispatch
    %% (RETURN_STRUCTURE_REFUSED) instead of silently completing it here.
    ?assertEqual([<<"step_b">>], air_core:ready_steps(RS1#runner_state.core)),
    ?assertMatch([{<<"step_b">>, {broker, 'RETURN_STRUCTURE_REFUSED', _}}],
                 RS1#runner_state.refusals),

    InfraPid = whereis(arazzo_runner_infra),
    ?assert(is_pid(InfraPid)),
    MonWorkflow = monitor(process, Pid1),
    MonInfra = monitor(process, InfraPid),
    true = erlang:exit(Pid1, kill),
    true = erlang:exit(InfraPid, kill),
    receive
        {'DOWN', MonWorkflow, process, Pid1, _} -> ok
    after 1000 ->
        error(workflow_did_not_die)
    end,
    receive
        {'DOWN', MonInfra, process, InfraPid, _} -> ok
    after 1000 ->
        error(infra_did_not_die)
    end,

    %% The ETS cache really is gone -- it was owned by InfraPid.
    ?assertEqual(undefined, ets:info(arazzo_workflow_states)),

    %% Restart: a brand-new, unrelated process. start_link/1 is called
    %% directly here (not via the supervisor) specifically so this test
    %% cannot be accused of relying on the supervisor remembering the
    %% pre-crash args in memory -- the supervisor process was never
    %% involved in this test at all. Everything must come from DETS.
    {ok, Pid2} = arazzo_runner_workflow:start_link(StartSpec),
    ?assertNotEqual(Pid1, Pid2),

    {ok, RS2} = arazzo_runner_workflow:get_runner_state(WorkflowId),
    ?assertEqual(RS1#runner_state.identity, RS2#runner_state.identity),

    Core2 = RS2#runner_state.core,
    ?assertEqual(true, maps:get(<<"step_a_done">>, air_core:get_env(Core2))),
    ?assertEqual([<<"step_b">>], air_core:ready_steps(Core2)),
    ?assertEqual(1, length(air_core:get_history(Core2))),

    %% And the reconstructed instance is still live: it can keep processing
    %% events past the point the crashed instance stopped at. Result is
    %% `true` (a real boolean), not `ok` -- step_b's outputs
    %% (sample_workflow_def/0) do real bool-typed arithmetic on
    %% `__result__`, and this is step_b's first genuine completion (its
    %% broker-synthesized auto-dispatch was refused above, never admitted).
    C2 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid2, {result, <<"step_b">>, true}),
    RS3 = wait_for_reaction(WorkflowId, C2, result),
    Core3 = RS3#runner_state.core,
    ?assertEqual(true, maps:get(<<"step_b_done">>, air_core:get_env(Core3))),
    ?assertEqual([], air_core:ready_steps(Core3)),
    ?assertEqual(2, length(air_core:get_history(Core3))),
    ok.

%% ---------------------------------------------------------------------
%% Proof 4: swarm audit wnl2yhbgm finding #13 -- a genuine broker-dispatch
%% exception is caught, not left to crash the whole workflow process.
%% ---------------------------------------------------------------------

%% step_a's outputs are well-formed and trivial (no result-sentinel typing
%% needed); step_b_malformed's `outputs` field is genuinely malformed --
%% `{bind, <<"step_b_done">>}` is an arity-2 tuple, not the arity-3
%% `{bind, Var, Expr}` arazzo_runner_broker:required_result_types/1's sole
%% function clause matches on. This is real, unvalidated caller-supplied
%% data flowing through the SAME path every real workflow_def does -- no
%% mock, no fault injection into the broker itself.
malformed_outputs_workflow_def() ->
    #{steps => #{
        <<"step_a">> => #{
            outputs => [{bind, <<"step_a_done">>, {literal, true}}],
            next => [<<"step_b_malformed">>]
        },
        <<"step_b_malformed">> => #{
            outputs => [{bind, <<"step_b_done">>}],
            next => []
        }
    }}.

%% step_a's completion (a real {result, ...} reaction event, driven exactly
%% like test_reactions_change_observable_state/0 drives step_a's own
%% completion) makes step_b_malformed newly ready -- air_core:transition/2
%% genuinely produces a {dispatch_step, <<"step_b_malformed">>, StepDef}
%% command, which apply_transition/4's foldl then dispatches through the
%% REAL arazzo_runner_broker:dispatch/4 -> do_dispatch/7 ->
%% required_result_types/1 call chain, inside this SAME test's single
%% reaction event -- no test-only shortcut into the broker.
test_broker_dispatch_exception_does_not_crash_the_workflow() ->
    {ok, _Started} = application:ensure_all_started(arazzo_runner),
    WorkflowId = <<"wf-malformed-outputs-1">>,
    StartSpec = maps:merge(sample_identity(WorkflowId), #{
        workflow_def => malformed_outputs_workflow_def(),
        active_steps => [<<"step_a">>],
        env => #{},
        history => []
    }),
    {ok, Pid} = arazzo_runner_sup:start_workflow(StartSpec),
    Mon = monitor(process, Pid),

    C0 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {result, <<"step_a">>, true}),
    _RS1 = wait_for_reaction(WorkflowId, C0, result),

    %% The critical safety property this fix exists for: the workflow
    %% process did not crash on the malformed step_b_malformed dispatch --
    %% no DOWN message arrives in a generous window past the reaction
    %% having already completed (proving this isn't a race against a
    %% not-yet-delivered exit signal).
    receive
        {'DOWN', Mon, process, Pid, Reason} ->
            error({workflow_process_crashed_on_malformed_step_outputs, Reason})
    after 500 ->
        ok
    end,
    ?assert(is_process_alive(Pid)),

    %% Not just "didn't crash" -- the exception was genuinely caught and
    %% recorded in broker_dispatches, the same durable field ordinary
    %% broker results use, not silently dropped.
    {ok, RS} = arazzo_runner_workflow:get_runner_state(WorkflowId),
    {value, {<<"step_b_malformed">>, BrokerResult}} =
        lists:keysearch(<<"step_b_malformed">>, 1, RS#runner_state.broker_dispatches),
    ?assertMatch({error, {exception, _Class, _Reason, _Stack}}, BrokerResult),

    %% And step_a's own outputs -- unaffected by step_b_malformed's failure
    %% -- genuinely completed first, proving this is a real two-step chain,
    %% not a workflow that never got past step_a.
    ?assertEqual(true, maps:get(<<"step_a_done">>, air_core:get_env(RS#runner_state.core))).

%% swarm audit wnl2yhbgm finding #13's OTP-runner reaction-dispatch sibling
%% (deferred from commit f22c1db4, precisely scoped by dogfood workflow
%% w8ckcazm7): before the handle_reaction/3 catch-all clause, no prior clause
%% matched an unrecognized event tag -- react/2 calls handle_reaction/3
%% directly, with no try/catch of its own, so the resulting function_clause
%% propagated straight through react/2 into workflow_loop/1's receive,
%% crashing the whole (supervised) workflow process and losing whatever
%% other commands this in-flight event would have produced. This drives the
%% REAL dispatch_event/2 -> workflow_loop/1 -> react/2 -> handle_reaction/3
%% path with a real, unrecognized event tuple -- no test-only shortcut into
%% the reaction handler.
test_unrecognized_reaction_event_does_not_crash_the_workflow() ->
    {ok, _Started} = application:ensure_all_started(arazzo_runner),
    WorkflowId = <<"wf-unrecognized-event-1">>,
    StartSpec = start_spec(WorkflowId),
    {ok, Pid} = arazzo_runner_sup:start_workflow(StartSpec),
    Mon = monitor(process, Pid),

    ok = arazzo_runner_workflow:dispatch_event(Pid, {totally_unrecognized_event_tag, 1, 2, 3}),

    %% The critical safety property: no DOWN message arrives in a generous
    %% window -- proves the catch-all clause returned RS unchanged instead of
    %% letting the function_clause propagate.
    receive
        {'DOWN', Mon, process, Pid, Reason} ->
            error({workflow_process_crashed_on_unrecognized_event, Reason})
    after 500 ->
        ok
    end,
    ?assert(is_process_alive(Pid)),

    %% Not just "alive but wedged" -- a subsequent, well-formed `result`
    %% event still genuinely advances state, proving this workflow instance
    %% is still fully usable (RS_/reaction_log untouched by the dropped
    %% unrecognized event, not left in some half-applied condition).
    C0 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {result, <<"step_a">>, ok}),
    RS1 = wait_for_reaction(WorkflowId, C0, result),
    ?assertEqual(true, maps:get(<<"step_a_done">>, air_core:get_env(RS1#runner_state.core))),
    ?assertEqual([<<"step_b">>], air_core:ready_steps(RS1#runner_state.core)).
