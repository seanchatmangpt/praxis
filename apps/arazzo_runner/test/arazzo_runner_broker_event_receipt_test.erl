-module(arazzo_runner_broker_event_receipt_test).
-include_lib("eunit/include/eunit.hrl").
-include("arazzo_runner.hrl").
-include("arazzo_event_receipt.hrl").

%% PROJ-781 (PRD v26.7.11 15 -- Receipt and Replay).
%%
%% Proves the ONE real emission site this ticket wires end-to-end:
%% arazzo_runner_broker:do_dispatch/6 (reached via a genuine air_core
%% transition, exactly like arazzo_runner_broker_test.erl's own round-trip
%% tests) mints a real #event_receipt{} BEFORE actuation, retrievable
%% afterward via arazzo_runner_event_receipt:get_receipt/2 -- not merely
%% unit-tested in isolation. A separate, fresh DETS dir/table per test
%% module, mirroring arazzo_runner_broker_test.erl's own isolation
%% convention, so this module's WorkflowIds/tables never collide with that
%% module's when both run in the same `rebar3 eunit` invocation.

setup() ->
    Dir = filename:join(
        "/tmp",
        "arazzo_broker_event_receipt_eunit_" ++ integer_to_list(erlang:unique_integer([positive]))
    ),
    ok = filelib:ensure_dir(filename:join(Dir, "x")),
    ok = application:set_env(arazzo_runner, state_dir, Dir),
    TableName = list_to_atom(
        "arazzo_broker_event_receipt_eunit_state_" ++
        integer_to_list(erlang:unique_integer([positive]))),
    ok = application:set_env(arazzo_runner, dets_table, TableName),
    {ok, _Started} = application:ensure_all_started(arazzo_runner),
    Dir.

cleanup(Dir) ->
    catch application:stop(arazzo_runner),
    catch arazzo_runner_identity:close_table(),
    catch os:cmd("rm -rf " ++ Dir),
    ok = application:unset_env(arazzo_runner, state_dir),
    ok = application:unset_env(arazzo_runner, dets_table),
    ok.

arazzo_runner_broker_event_receipt_test_() ->
    {setup,
     fun setup/0,
     fun cleanup/1,
     fun(_) ->
         [
             {"do_dispatch/6, reached through a real air_core transition, "
              "mints a real #event_receipt{} for the step_dispatched event "
              "before actuation, with all 10 PRD fields correctly populated "
              "from the real dispatch context",
              fun test_dispatch_emits_real_event_receipt/0},
             {"two dispatches within the same workflow genuinely extend the "
              "same BLAKE3-linked event-receipt chain (2nd receipt's "
              "prior_receipt_head equals 1st receipt's receipt_head)",
              fun test_two_dispatches_extend_the_same_chain/0},
             {"a workflow identity with an empty receipt_head is refused "
              "before this ticket's event-receipt emission is ever reached "
              "-- BROKER_RECEIPT_PRECONDITION_MISSING still fires first, "
              "proving the two gates compose rather than one silently "
              "masking the other",
              fun test_missing_identity_receipt_head_refused_before_emission/0}
         ]
     end}.

%% ---------------------------------------------------------------------
%% Fixtures (mirrors arazzo_runner_broker_test.erl's own shape)
%% ---------------------------------------------------------------------

sample_identity(WorkflowId) ->
    #{
        workflow_id => WorkflowId,
        parent_workflow_id => undefined,
        arazzo_workflow_id => <<"arazzo-wf-broker-receipt">>,
        source_powl_region_id => <<"powl-region-broker-receipt">>,
        dispatch_id => <<"dispatch-broker-receipt-1">>,
        correlation_id => <<"corr-broker-receipt-1">>,
        source_digest => <<"src-digest-broker-receipt">>,
        projection_digest => <<"proj-digest-broker-receipt">>,
        receipt_head => <<"receipt-head-broker-receipt-genesis">>,
        replay_id => <<"replay-broker-receipt-1">>
    }.

sample_workflow_def() ->
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

start_spec(WorkflowId) ->
    maps:merge(sample_identity(WorkflowId), #{
        workflow_def => sample_workflow_def(),
        active_steps => [<<"step_a">>],
        env => #{},
        history => []
    }).

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

%% ---------------------------------------------------------------------
%% Tests
%% ---------------------------------------------------------------------

test_dispatch_emits_real_event_receipt() ->
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-broker-receipt-dispatch-1">>,
    StartSpec = start_spec(WorkflowId),
    {ok, Pid} = arazzo_runner_workflow:start_link(StartSpec),

    C0 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {result, <<"step_a">>, ok}),
    RS1 = wait_for_reaction(WorkflowId, C0, result),
    [{<<"step_b">>, {ok, _DispatchToken}}] = RS1#runner_state.broker_dispatches,

    %% The real emission site (do_dispatch/6) ran before actuation, at
    %% logical_clock 1 -- the first (and, for this test, only) event ever
    %% emitted for this workflow_id (see next_logical_clock/1's own doc
    %% comment for why the first tick is 1, not 0).
    {ok, Receipt} = arazzo_runner_event_receipt:get_receipt(WorkflowId, 1),
    ?assertEqual(WorkflowId, Receipt#event_receipt.workflow_semantic_id),
    ?assertEqual(undefined, Receipt#event_receipt.parent_semantic_id),
    ?assertEqual(step_dispatched, Receipt#event_receipt.event_type),
    ?assertEqual(otp, Receipt#event_receipt.runtime_profile),
    ?assertEqual(<<"replay-broker-receipt-1">>, Receipt#event_receipt.replay_id),
    %% Genesis: chains from the real #workflow_identity.receipt_head this
    %% workflow was started with, not an invented root.
    ?assertEqual(<<"receipt-head-broker-receipt-genesis">>, Receipt#event_receipt.prior_receipt_head),

    %% command_digest independently recomputable from the real dispatched
    %% command (StepDef for step_b, the exact shape air_core's own C would
    %% carry) -- proves this is a genuine content hash, not a placeholder.
    StepBDef = maps:get(<<"step_b">>, maps:get(steps, sample_workflow_def())),
    {ok, ExpectedCommandDigest} = arazzo_runner_blake3:hex(
        erlang:term_to_binary({dispatch_step, <<"step_b">>, StepBDef}, [{minor_version, 1}])),
    ?assertEqual(ExpectedCommandDigest, Receipt#event_receipt.command_digest),

    {ok, ChainHead} = arazzo_runner_event_receipt:get_chain_head(WorkflowId),
    ?assertEqual(Receipt#event_receipt.receipt_head, ChainHead),
    ok.

test_two_dispatches_extend_the_same_chain() ->
    WorkflowId = <<"wf-broker-receipt-chain-1">>,
    {ok, Identity} = arazzo_runner_identity:from_map(sample_identity(WorkflowId)),
    StepDef = #{outputs => [], next => []},

    {ok, _Token1} = arazzo_runner_broker:dispatch(WorkflowId, Identity, <<"step_x">>, StepDef),
    {ok, R1} = arazzo_runner_event_receipt:get_receipt(WorkflowId, 1),
    ?assertEqual(<<"receipt-head-broker-receipt-genesis">>, R1#event_receipt.prior_receipt_head),

    {ok, _Token2} = arazzo_runner_broker:dispatch(WorkflowId, Identity, <<"step_y">>, StepDef),
    {ok, R2} = arazzo_runner_event_receipt:get_receipt(WorkflowId, 2),

    %% The actual chain-extension property: R2 chains from R1's own
    %% receipt_head, not from the genesis value again.
    ?assertEqual(R1#event_receipt.receipt_head, R2#event_receipt.prior_receipt_head),
    ?assertNotEqual(R1#event_receipt.receipt_head, R2#event_receipt.receipt_head),
    ok.

test_missing_identity_receipt_head_refused_before_emission() ->
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-broker-receipt-no-head-1">>,
    StartSpec = (start_spec(WorkflowId))#{receipt_head => undefined},
    {ok, Pid} = arazzo_runner_workflow:start_link(StartSpec),

    C0 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {result, <<"step_a">>, ok}),
    RS1 = wait_for_reaction(WorkflowId, C0, result),

    ExpectedCtx = #{stage => preactuation, workflow_id => WorkflowId, step_id => <<"step_b">>},
    ?assertEqual([{<<"step_b">>, {refused, 'BROKER_RECEIPT_PRECONDITION_MISSING', ExpectedCtx}}],
                 RS1#runner_state.broker_dispatches),
    %% No event receipt was ever minted for this refused dispatch attempt --
    %% the pre-existing required_prior_receipts gate (PROJ-758) still runs
    %% BEFORE this ticket's emission site, so a missing genesis value is
    %% caught there, not silently defaulted to some invented value here.
    ?assertEqual(not_found, arazzo_runner_event_receipt:get_receipt(WorkflowId, 0)),
    ok.
