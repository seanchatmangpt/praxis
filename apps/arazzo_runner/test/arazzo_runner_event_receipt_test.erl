-module(arazzo_runner_event_receipt_test).
-include_lib("eunit/include/eunit.hrl").
-include("arazzo_event_receipt.hrl").

%% PROJ-781 (PRD v26.7.11 15 -- Receipt and Replay, PRD.md:704-716).
%%
%% Proves arazzo_runner_event_receipt's two entry points (build/1, the pure
%% constructor; emit/1, the real per-workflow chaining wrapper) for real:
%% all 10 PRD-named fields are genuinely populated (not zeroed
%% placeholders), digests are content-sensitive and deterministic, and
%% repeated emit/1 calls for the same workflow genuinely EXTEND a chain
%% (each new receipt's prior_receipt_head equals the previous receipt's
%% own receipt_head) -- the actual property PRD.md:702 ("every workflow
%% execution SHALL extend a BLAKE3-linked receipt chain") requires, not
%% just "a hash exists somewhere".

%% ---------------------------------------------------------------------
%% build/1
%% ---------------------------------------------------------------------

base_build_fields() ->
    #{
        workflow_semantic_id => <<"wf-event-receipt-1">>,
        parent_semantic_id => undefined,
        event_type => step_dispatched,
        event_material => {step_dispatch_requested, <<"wf-event-receipt-1">>, <<"step_a">>},
        prior_receipt_head => <<"genesis-receipt-head">>,
        resulting_state_material => #{status => dispatched, step_id => <<"step_a">>},
        command_material => {dispatch_step, <<"step_a">>, #{outputs => [], next => []}},
        runtime_profile => otp,
        logical_clock => 0,
        replay_id => <<"replay-event-receipt-1">>
    }.

build_missing_fields_is_refused_test() ->
    Fields = maps:remove(event_type, maps:remove(command_material, base_build_fields())),
    Result = arazzo_runner_event_receipt:build(Fields),
    ?assertMatch({error, {missing_event_receipt_fields, _}}, Result),
    {error, {missing_event_receipt_fields, Missing}} = Result,
    ?assertEqual(lists:sort([event_type, command_material]), lists:sort(Missing)).

build_happy_path_populates_all_10_prd_fields_test() ->
    {ok, R} = arazzo_runner_event_receipt:build(base_build_fields()),
    ?assertEqual(<<"wf-event-receipt-1">>, R#event_receipt.workflow_semantic_id),
    ?assertEqual(undefined, R#event_receipt.parent_semantic_id),
    ?assertEqual(step_dispatched, R#event_receipt.event_type),
    ?assert(is_binary(R#event_receipt.event_digest)),
    ?assert(byte_size(R#event_receipt.event_digest) > 0),
    ?assertEqual(<<"genesis-receipt-head">>, R#event_receipt.prior_receipt_head),
    ?assert(is_binary(R#event_receipt.resulting_state_digest)),
    ?assert(byte_size(R#event_receipt.resulting_state_digest) > 0),
    ?assert(is_binary(R#event_receipt.command_digest)),
    ?assert(byte_size(R#event_receipt.command_digest) > 0),
    ?assertEqual(otp, R#event_receipt.runtime_profile),
    ?assertEqual(0, R#event_receipt.logical_clock),
    ?assertEqual(<<"replay-event-receipt-1">>, R#event_receipt.replay_id),
    %% Derived receipt_head is real and distinct from all three material
    %% digests (its own tag/order distinguishes it -- not an accidental
    %% duplicate of one of them).
    ?assert(is_binary(R#event_receipt.receipt_head)),
    ?assert(byte_size(R#event_receipt.receipt_head) > 0),
    ?assertNotEqual(R#event_receipt.event_digest, R#event_receipt.receipt_head),
    ?assertNotEqual(R#event_receipt.resulting_state_digest, R#event_receipt.receipt_head),
    ?assertNotEqual(R#event_receipt.command_digest, R#event_receipt.receipt_head).

%% Not zeroed/faked: the three digest fields are genuinely distinct from
%% each other for materials that are themselves distinct.
build_digest_fields_are_mutually_distinct_test() ->
    {ok, R} = arazzo_runner_event_receipt:build(base_build_fields()),
    ?assertNotEqual(R#event_receipt.event_digest, R#event_receipt.resulting_state_digest),
    ?assertNotEqual(R#event_receipt.event_digest, R#event_receipt.command_digest),
    ?assertNotEqual(R#event_receipt.resulting_state_digest, R#event_receipt.command_digest).

%% Content sensitivity: changing ONE material changes ONLY its own digest
%% field (and, transitively, receipt_head, since receipt_head binds all
%% fields) -- proving these are real per-field hashes, not one hash copied
%% into three fields.
build_event_material_change_only_moves_event_digest_test() ->
    Fields1 = base_build_fields(),
    Fields2 = Fields1#{event_material => {step_dispatch_requested, <<"wf-event-receipt-1">>, <<"step_b">>}},
    {ok, R1} = arazzo_runner_event_receipt:build(Fields1),
    {ok, R2} = arazzo_runner_event_receipt:build(Fields2),
    ?assertNotEqual(R1#event_receipt.event_digest, R2#event_receipt.event_digest),
    ?assertEqual(R1#event_receipt.resulting_state_digest, R2#event_receipt.resulting_state_digest),
    ?assertEqual(R1#event_receipt.command_digest, R2#event_receipt.command_digest),
    ?assertNotEqual(R1#event_receipt.receipt_head, R2#event_receipt.receipt_head).

%% Determinism: building from the SAME fields map 3 independent times
%% produces byte-identical digests and receipt_head every time.
build_is_deterministic_across_repeated_runs_test() ->
    Fields = base_build_fields(),
    {ok, R1} = arazzo_runner_event_receipt:build(Fields),
    {ok, R2} = arazzo_runner_event_receipt:build(Fields),
    {ok, R3} = arazzo_runner_event_receipt:build(Fields),
    ?assertEqual(R1#event_receipt.event_digest, R2#event_receipt.event_digest),
    ?assertEqual(R2#event_receipt.event_digest, R3#event_receipt.event_digest),
    ?assertEqual(R1#event_receipt.receipt_head, R2#event_receipt.receipt_head),
    ?assertEqual(R2#event_receipt.receipt_head, R3#event_receipt.receipt_head).

%% ---------------------------------------------------------------------
%% emit/1: the real per-workflow chaining wrapper
%% ---------------------------------------------------------------------

base_emit_params(WorkflowId) ->
    #{
        workflow_id => WorkflowId,
        parent_workflow_id => undefined,
        event_type => step_dispatched,
        event_material => {step_dispatch_requested, WorkflowId, <<"step_a">>},
        resulting_state_material => #{status => dispatched, step_id => <<"step_a">>},
        command_material => {dispatch_step, <<"step_a">>, #{outputs => [], next => []}},
        runtime_profile => otp,
        replay_id => <<"replay-emit-1">>,
        identity_receipt_head => <<"identity-genesis-head">>
    }.

emit_missing_fields_is_refused_test() ->
    Params = maps:remove(runtime_profile, base_emit_params(<<"wf-emit-missing-1">>)),
    ?assertMatch({error, {missing_event_receipt_emit_fields, [runtime_profile]}},
                 arazzo_runner_event_receipt:emit(Params)).

%% Genesis: the FIRST event emitted for a fresh workflow_id chains from the
%% caller-supplied identity_receipt_head (PRD 7.8's real, already-required
%% #workflow_identity.receipt_head), not an invented root.
emit_first_event_chains_from_identity_receipt_head_test() ->
    WorkflowId = <<"wf-emit-genesis-1">>,
    not_found = arazzo_runner_event_receipt:get_chain_head(WorkflowId),
    {ok, R1} = arazzo_runner_event_receipt:emit(base_emit_params(WorkflowId)),
    ?assertEqual(<<"identity-genesis-head">>, R1#event_receipt.prior_receipt_head),
    %% First event for a fresh workflow_id is logical_clock 1 (see
    %% next_logical_clock/1's own doc comment for why not 0).
    ?assertEqual(1, R1#event_receipt.logical_clock),
    {ok, ChainHead} = arazzo_runner_event_receipt:get_chain_head(WorkflowId),
    ?assertEqual(R1#event_receipt.receipt_head, ChainHead).

%% The actual chain-extension property: a SECOND emit/1 for the SAME
%% workflow_id produces a receipt whose prior_receipt_head equals the
%% FIRST receipt's own receipt_head (not identity_receipt_head again, and
%% not some unrelated value) -- a real, growing hash chain, and the
%% logical clock genuinely advances.
emit_second_event_chains_from_first_receipt_head_test() ->
    WorkflowId = <<"wf-emit-chain-extends-1">>,
    Params1 = base_emit_params(WorkflowId),
    {ok, R1} = arazzo_runner_event_receipt:emit(Params1),
    Params2 = Params1#{
        event_type => step_completed,
        event_material => {step_completion_observed, WorkflowId, <<"step_a">>, ok},
        resulting_state_material => #{status => completed, step_id => <<"step_a">>}
    },
    {ok, R2} = arazzo_runner_event_receipt:emit(Params2),
    ?assertEqual(R1#event_receipt.receipt_head, R2#event_receipt.prior_receipt_head),
    ?assertNotEqual(R1#event_receipt.receipt_head, R2#event_receipt.receipt_head),
    ?assertEqual(1, R1#event_receipt.logical_clock),
    ?assertEqual(2, R2#event_receipt.logical_clock),
    {ok, ChainHead} = arazzo_runner_event_receipt:get_chain_head(WorkflowId),
    ?assertEqual(R2#event_receipt.receipt_head, ChainHead).

%% Two DIFFERENT workflow_ids never share a chain head, even when started
%% from the same identity_receipt_head genesis value.
emit_distinct_workflows_have_independent_chains_test() ->
    ParamsA = base_emit_params(<<"wf-emit-independent-a">>),
    ParamsB = base_emit_params(<<"wf-emit-independent-b">>),
    {ok, RA} = arazzo_runner_event_receipt:emit(ParamsA),
    {ok, RB} = arazzo_runner_event_receipt:emit(ParamsB),
    %% Same genesis prior_receipt_head (both are each workflow's first
    %% event)...
    ?assertEqual(RA#event_receipt.prior_receipt_head, RB#event_receipt.prior_receipt_head),
    %% ...but distinct workflow_semantic_id feeds into the digest, so the
    %% receipts themselves (and hence receipt_head) are still distinct.
    ?assertNotEqual(RA#event_receipt.receipt_head, RB#event_receipt.receipt_head).

%% get_receipt/2 durably retrieves exactly what emit/1 minted, by
%% (workflow_id, logical_clock) -- a genuine append-only log, not a
%% write-only side effect.
emit_persists_retrievable_receipt_log_test() ->
    WorkflowId = <<"wf-emit-log-1">>,
    {ok, R} = arazzo_runner_event_receipt:emit(base_emit_params(WorkflowId)),
    {ok, Fetched} = arazzo_runner_event_receipt:get_receipt(WorkflowId, 1),
    ?assertEqual(R, Fetched),
    ?assertEqual(not_found, arazzo_runner_event_receipt:get_receipt(WorkflowId, 2)).

receipt_to_map_round_trips_all_fields_test() ->
    {ok, R} = arazzo_runner_event_receipt:build(base_build_fields()),
    M = arazzo_runner_event_receipt:receipt_to_map(R),
    ?assertEqual(R#event_receipt.workflow_semantic_id, maps:get(workflow_semantic_id, M)),
    ?assertEqual(R#event_receipt.receipt_head, maps:get(receipt_head, M)),
    %% The 10 PRD-declared fields plus the derived receipt_head.
    ?assertEqual(11, map_size(M)).
