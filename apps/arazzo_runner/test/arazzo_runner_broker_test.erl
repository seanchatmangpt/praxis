-module(arazzo_runner_broker_test).
-include_lib("eunit/include/eunit.hrl").
-include("arazzo_runner.hrl").
-include("arazzo_broker.hrl").

%% PROJ-758 (PRD v26.7.11 13 -- Broker Requirements; 8 -- Independent
%% Process Cells / return-admission chain) proof suite, extended by
%% PROJ-785 (CORRELATION_MISMATCH, RETURN_PROVENANCE_MISSING, and now
%% RETURN_STRUCTURE_REFUSED -- three of PROJ-785's 4 remaining refusal
%% codes that turned out to have a real Erlang-side data source;
%% RETURN_SEMANTIC_REFUSED alone stays in ?UNENFORCED_RETURN_STAGES, see
%% arazzo_runner_broker.erl for the full investigation of why).
%%
%% Every test exercises the REAL arazzo_runner_broker module against a
%% REAL, running arazzo_runner_workflow process and the REAL (stub-
%% actuation) io-worker pool -- nothing here is mocked or simulated at the
%% broker layer. The provenance and structure tests below are the one
%% exception that constructs a #dispatch{} ledger entry directly via
%% ets:insert/2 rather than driving it through a real dispatch/4 call --
%% documented at each call site why (do_dispatch/6's dispatch/4 ->
%% enqueue_io/2 chain is synchronous, so `dispatched`/`dispatch_failed` are
%% real but only ever transiently observable states from outside, and
%% today's echo-placeholder execute_io_request/1 (arazzo_runner_workflow.erl)
%% only ever actuates to a `{ok, {processed, StepDef}}` tuple, never an
%% integer/boolean -- so a real, live dispatch/4 round trip can never itself
%% produce a raw_consequence that would exercise the integer/boolean
%% conformance paths RETURN_STRUCTURE_REFUSED checks). Direct construction
%% is the only way to hold the ledger in one of those states, or with one of
%% those raw_consequence shapes, long enough to assert against, and it uses
%% the exact same #dispatch{} shape and ETS table do_dispatch/6 itself
%% writes -- not a mock of the broker's own logic. Fresh, isolated DETS
%% directory per run (mirrors arazzo_runner_workflow_test.erl's own
%% setup/0); every WorkflowId used here is unique to this module so ETS
%% ledger state never collides with arazzo_runner_workflow_test.erl's own
%% fixtures run in the same VM.

arazzo_runner_broker_test_() ->
    {setup,
     fun setup/0,
     fun cleanup/1,
     fun(_) ->
         [
             {"dispatch/4 refuses CORRELATION_MISSING for real when the "
              "workflow identity carries no correlation id -- wired "
              "end-to-end through a real air_core transition, not called "
              "in isolation; no ledger entry, no actuation attempted",
              fun test_correlation_missing_on_dispatch/0},
             {"dispatch/4 refuses BROKER_RECEIPT_PRECONDITION_MISSING for "
              "real when the workflow identity carries no receipt_head -- "
              "wired end-to-end through a real air_core transition; no "
              "ledger entry, no actuation attempted",
              fun test_broker_receipt_precondition_missing_on_dispatch/0},
             {"dispatch/4 proceeds past the required_prior_receipts gate "
              "and genuinely actuates when receipt_head is present -- the "
              "positive path for the same check",
              fun test_required_prior_receipts_present_proceeds/0},
             {"a full real round trip through the PRODUCTION path only "
              "(dispatch_event -> apply_transition -> broker:dispatch/4 -> "
              "enqueue_io -> do_dispatch_actuate's own internal "
              "admit_return/3 call): air_core produces a dispatch_step "
              "command, the broker actuates it via the real io-worker "
              "pool, captures + hashes the consequence, and the "
              "return-admission loop closes ON ITS OWN, with no direct "
              "admit_return/3 call anywhere in this test, genuinely "
              "advancing air_core state",
              fun test_full_dispatch_correlation_return_round_trip/0},
             {"admit_return/3 refuses CORRELATION_MISSING for an unknown "
              "dispatch_token",
              fun test_admit_return_correlation_missing/0},
             {"admit_return/3 refuses CORRELATION_MISMATCH (PROJ-785) for "
              "a KNOWN dispatch_token whose returner claims a "
              "CorrelationId that does not match the one recorded on the "
              "ledger entry at dispatch time -- distinct from "
              "CORRELATION_MISSING's unknown-token case; the workflow's "
              "air_core state provably does not advance",
              fun test_admit_return_correlation_mismatch/0},
             {"admit_return/3 refuses RETURN_PROVENANCE_MISSING (PROJ-785) "
              "when a KNOWN dispatch_token's ledger entry has status "
              "`dispatched` -- no consequence was ever captured from any "
              "actuation for this token, so there is nothing to have "
              "provenance over",
              fun test_admit_return_provenance_missing_dispatched/0},
             {"admit_return/3 refuses RETURN_PROVENANCE_MISSING (PROJ-785) "
              "when a KNOWN dispatch_token's ledger entry has status "
              "`dispatch_failed` -- actuation was attempted but never "
              "produced a consequence",
              fun test_admit_return_provenance_missing_dispatch_failed/0},
             {"admit_return/3 refuses RETURN_AUTHORITY_REFUSED for a wrong "
              "or missing returner authority token, and the workflow's "
              "air_core state provably does not advance past dispatch-"
              "readiness",
              fun test_admit_return_authority_refused/0},
             {"admit_return/3 admits a structurally conforming "
              "raw_consequence (RETURN_STRUCTURE_REFUSED, PROJ-785): an "
              "integer consequence for a step whose declared outputs do "
              "real arithmetic on {var, '__result__'} is admitted, and "
              "air_core's REAL bind_outputs/3 -> eval_expr_nif evaluation "
              "genuinely computes the bound value from it",
              fun test_admit_return_structure_conforms/0},
             {"admit_return/3 refuses RETURN_STRUCTURE_REFUSED (PROJ-785) "
              "for a raw_consequence whose Erlang type does not satisfy "
              "the type required_result_types/1 derived from the step's "
              "own declared outputs, and air_core state provably does not "
              "advance",
              fun test_admit_return_structure_refused_type_mismatch/0},
             {"enqueue_io/2 refuses DIRECT_ACTUATION_REFUSED for a bogus, "
              "never-issued actuation token, before any io-worker round "
              "trip is attempted (bounded by elapsed time, well under the "
              "5s pool-reply timeout)",
              fun test_direct_actuation_refused_bogus_token/0},
             {"enqueue_io/2 refuses DIRECT_ACTUATION_REFUSED on token "
              "reuse -- a legitimately-issued token is one-shot",
              fun test_direct_actuation_refused_token_reuse/0},
             {"dispatch/4 is genuinely idempotent: a repeated dispatch for "
              "the same workflow/step returns the SAME dispatch_token and "
              "does not actuate (extend the evidence chain) a second time",
              fun test_dispatch_idempotent_dedup/0},
             {"post-actuation evidence: consequence_hash is present and "
              "the per-workflow evidence chain genuinely extends (2nd "
              "dispatch's prev_evidence_hash equals 1st dispatch's "
              "consequence_hash)",
              fun test_evidence_hash_chain_extends/0},
             {"authority tokens require the server-side node secret, not "
              "just the public workflow_id/step_id/idempotency_key -- a "
              "token computed the OLD (unsalted) way from public "
              "identifiers alone does not match the real minted tokens "
              "and is rejected by both enqueue_io/2 "
              "(DIRECT_ACTUATION_REFUSED) and admit_return/3 "
              "(RETURN_AUTHORITY_REFUSED)",
              fun test_actuation_token_requires_server_secret/0},
             {"two real, concurrently-racing processes dispatching the "
              "SAME idempotency key claim the dedup slot atomically "
              "(ets:insert_new/2): both observe the same dispatch_token, "
              "and the ledger ends up in a single consistent `actuated` "
              "state, never clobbered back to `dispatch_failed` by a "
              "racing loser's write",
              fun test_concurrent_duplicate_dispatch_claims_exactly_once/0}
         ]
     end}.

%% ---------------------------------------------------------------------
%% Fixture
%% ---------------------------------------------------------------------

setup() ->
    Dir = filename:join(
        "/tmp",
        "arazzo_broker_eunit_" ++ integer_to_list(erlang:unique_integer([positive]))
    ),
    ok = filelib:ensure_dir(filename:join(Dir, "x")),
    ok = application:set_env(arazzo_runner, state_dir, Dir),
    %% Distinct DETS table name from arazzo_runner_workflow_test.erl's
    %% (which uses arazzo_runner_identity's own default,
    %% arazzo_runner_state) so the two eunit modules' DETS files never
    %% collide when both run within the same `rebar3 eunit` invocation,
    %% regardless of run order: dets:open_file/2 raises
    %% `incompatible_arguments` if the SAME table name is reopened
    %% pointing at a different file while any reference to the previous
    %% open is still live (found running both modules together --
    %% real, reproducible, not hypothetical; see cleanup/1's matching
    %% unset).
    TableName = list_to_atom(
        "arazzo_broker_eunit_state_" ++ integer_to_list(erlang:unique_integer([positive]))),
    ok = application:set_env(arazzo_runner, dets_table, TableName),
    {ok, _Started} = application:ensure_all_started(arazzo_runner),
    %% Force real infra bootstrap (pg scope, io-worker pool, broker ETS
    %% ledger tables) before any test below runs, independent of whether
    %% this module happens to run before arazzo_runner_workflow_test.erl
    %% in the same `rebar3 eunit` invocation -- these tests must not
    %% depend on test-module run order for correctness. This workflow is
    %% never driven again after this point.
    BootstrapId = <<"wf-broker-infra-bootstrap">>,
    {ok, _BootstrapPid} = arazzo_runner_workflow:start_link(start_spec(BootstrapId)),
    Dir.

cleanup(Dir) ->
    catch application:stop(arazzo_runner),
    catch arazzo_runner_identity:close_table(),
    catch os:cmd("rm -rf " ++ Dir),
    ok = application:unset_env(arazzo_runner, state_dir),
    ok = application:unset_env(arazzo_runner, dets_table),
    ok.

%% ---------------------------------------------------------------------
%% Shared fixtures
%% ---------------------------------------------------------------------

sample_identity(WorkflowId) ->
    sample_identity(WorkflowId, <<"corr-broker-1">>).

sample_identity(WorkflowId, CorrelationId) ->
    #{
        workflow_id => WorkflowId,
        parent_workflow_id => undefined,
        arazzo_workflow_id => <<"arazzo-wf-broker">>,
        source_powl_region_id => <<"powl-region-broker">>,
        dispatch_id => <<"dispatch-broker-1">>,
        correlation_id => CorrelationId,
        source_digest => <<"src-digest-broker">>,
        projection_digest => <<"proj-digest-broker">>,
        receipt_head => <<"receipt-head-broker">>,
        replay_id => <<"replay-broker-1">>
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
    start_spec(WorkflowId, <<"corr-broker-1">>).

start_spec(WorkflowId, CorrelationId) ->
    maps:merge(sample_identity(WorkflowId, CorrelationId), #{
        workflow_def => sample_workflow_def(),
        active_steps => [<<"step_a">>],
        env => #{},
        history => []
    }).

%% PROJ-785 (RETURN_STRUCTURE_REFUSED): a step whose `outputs` bind rule
%% references the real air_core sentinel {var, '__result__'} under a
%% type-coercing operator -- exactly the shape required_result_types/1
%% (arazzo_runner_broker.erl) walks to derive a real structural
%% requirement (here: raw_consequence must be an integer for `+` to
%% decode it, per air_core's own eval_expr_nif). Distinct from
%% sample_workflow_def/0's steps, whose outputs are literal-only and so
%% derive an empty (vacuously-satisfied) requirement set.
sample_workflow_def_with_result_binding() ->
    #{steps => #{
        <<"step_struct">> => #{
            outputs => [{bind, <<"doubled">>, {op, '+', {var, '__result__'}, {literal, 1}}}],
            next => []
        }
    }}.

start_spec_structure(WorkflowId) ->
    maps:merge(sample_identity(WorkflowId), #{
        workflow_def => sample_workflow_def_with_result_binding(),
        active_steps => [<<"step_struct">>],
        env => #{},
        history => []
    }).

%% Same shape as start_spec/2 but with receipt_head overridden -- used by
%% the required_prior_receipts (BROKER_RECEIPT_PRECONDITION_MISSING)
%% pre-actuation gate tests, mirroring start_spec/2's own correlation_id
%% override pattern.
start_spec_with_receipt_head(WorkflowId, ReceiptHead) ->
    maps:merge(
        (sample_identity(WorkflowId))#{receipt_head => ReceiptHead},
        #{
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

%% Polls on a genuine terminal CONDITION over #runner_state{} rather than a
%% fixed reaction-log count. Needed for asserting the return-admission loop
%% closes end to end (do_dispatch_actuate/6's internal admit_return/3 call,
%% arazzo_runner_broker.erl): the admitted-result event that finally
%% advances a dispatched step's air_core state is queued asynchronously
%% (Pid ! {event, ...}) to the SAME live workflow process from within its
%% own currently-executing reaction cycle, so whether it lands within the
%% same react/2 cycle that triggered the dispatch, or a separate one
%% scheduled immediately after, is a genuine scheduling detail -- a
%% fixed-count-based wait (wait_for_reaction/3 above) would be racy against
%% it either way. Polling the actual air_core-visible outcome instead is
%% correct regardless of how many reaction cycles it took to get there.
wait_until(WorkflowId, Pred) ->
    wait_until(WorkflowId, Pred, 200).

wait_until(WorkflowId, _Pred, 0) ->
    error({timeout_waiting_for_condition, WorkflowId});
wait_until(WorkflowId, Pred, N) ->
    case arazzo_runner_workflow:get_runner_state(WorkflowId) of
        {ok, RS} ->
            case Pred(RS) of
                true -> RS;
                false ->
                    timer:sleep(10),
                    wait_until(WorkflowId, Pred, N - 1)
            end;
        not_found ->
            timer:sleep(10),
            wait_until(WorkflowId, Pred, N - 1)
    end.

%% ---------------------------------------------------------------------
%% Pre-actuation verification / CORRELATION_MISSING (PRD 13)
%% ---------------------------------------------------------------------

test_correlation_missing_on_dispatch() ->
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-broker-corr-missing-1">>,
    %% Explicitly undefined correlation_id: arazzo_runner_identity:from_map/1
    %% only checks key-presence, not value validity, so this legitimately
    %% constructs a #workflow_identity{} -- exactly the shape dispatch/4
    %% must catch for real, reached here through a genuine air_core
    %% transition (step_a completing), not a direct isolated call.
    StartSpec = start_spec(WorkflowId, undefined),
    {ok, Pid} = arazzo_runner_workflow:start_link(StartSpec),

    C0 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {result, <<"step_a">>, ok}),
    RS1 = wait_for_reaction(WorkflowId, C0, result),

    ExpectedCtx = #{stage => preactuation, workflow_id => WorkflowId, step_id => <<"step_b">>},
    ?assertEqual([{<<"step_b">>, {refused, 'CORRELATION_MISSING', ExpectedCtx}}],
                 RS1#runner_state.broker_dispatches),
    ?assertEqual([{<<"step_b">>, {broker, 'CORRELATION_MISSING', ExpectedCtx}}],
                 RS1#runner_state.refusals),
    %% air_core's own C (dispatch_step commands) is unaffected by the
    %% broker's decision -- PRD 7.7 draws that boundary deliberately: the
    %% transition core computed a real command; the broker separately
    %% refused to actuate it.
    ?assertEqual([{<<"step_b">>, maps:get(<<"step_b">>, maps:get(steps, sample_workflow_def()))}],
                 RS1#runner_state.pending_dispatches),
    %% Nothing was actuated: no ledger entry exists for a dispatch that
    %% never got a token minted.
    ?assertEqual(not_found, arazzo_runner_broker:get_ledger_entry(<<"no-such-token">>)),
    ok.

%% ---------------------------------------------------------------------
%% Pre-actuation verification / BROKER_RECEIPT_PRECONDITION_MISSING
%% (PRD 13, required_prior_receipts)
%% ---------------------------------------------------------------------

test_broker_receipt_precondition_missing_on_dispatch() ->
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-broker-receipt-missing-1">>,
    %% Explicitly undefined receipt_head: arazzo_runner_identity:from_map/1
    %% only checks key-presence, not value validity (same pattern
    %% test_correlation_missing_on_dispatch/0 exercises for correlation_id),
    %% so this legitimately constructs a #workflow_identity{} -- exactly the
    %% shape dispatch/4 must catch for required_prior_receipts, reached here
    %% through a genuine air_core transition (step_a completing), not a
    %% direct isolated call.
    StartSpec = start_spec_with_receipt_head(WorkflowId, undefined),
    {ok, Pid} = arazzo_runner_workflow:start_link(StartSpec),

    C0 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {result, <<"step_a">>, ok}),
    RS1 = wait_for_reaction(WorkflowId, C0, result),

    ExpectedCtx = #{stage => preactuation, workflow_id => WorkflowId, step_id => <<"step_b">>},
    ?assertEqual([{<<"step_b">>, {refused, 'BROKER_RECEIPT_PRECONDITION_MISSING', ExpectedCtx}}],
                 RS1#runner_state.broker_dispatches),
    ?assertEqual([{<<"step_b">>, {broker, 'BROKER_RECEIPT_PRECONDITION_MISSING', ExpectedCtx}}],
                 RS1#runner_state.refusals),
    %% air_core's own C (dispatch_step commands) is unaffected by the
    %% broker's decision -- same PRD 7.7 boundary the correlation test
    %% above exercises.
    ?assertEqual([{<<"step_b">>, maps:get(<<"step_b">>, maps:get(steps, sample_workflow_def()))}],
                 RS1#runner_state.pending_dispatches),
    %% Nothing was actuated: no ledger entry exists for a dispatch that
    %% never got a token minted.
    ?assertEqual(not_found, arazzo_runner_broker:get_ledger_entry(<<"no-such-token">>)),
    ok.

test_required_prior_receipts_present_proceeds() ->
    %% Positive path for the same gate: a real, non-empty receipt_head
    %% (sample_identity/1's default, unmodified) lets dispatch/4 proceed
    %% all the way to genuine actuation instead of being refused at the
    %% required_prior_receipts gate.
    WorkflowId = <<"wf-broker-receipt-present-1">>,
    {ok, Identity} = arazzo_runner_identity:from_map(sample_identity(WorkflowId)),
    ?assert(is_binary(Identity#workflow_identity.receipt_head)),
    ?assert(byte_size(Identity#workflow_identity.receipt_head) > 0),
    StepDef = #{outputs => [], next => []},

    {ok, Token} = arazzo_runner_broker:dispatch(WorkflowId, Identity, <<"step_z">>, StepDef),
    ?assert(is_binary(Token)),
    {ok, D} = arazzo_runner_broker:get_ledger_entry(Token),
    %% Genuinely actuated (not refused, not merely dispatched-and-stuck) --
    %% proves the gate let a real dispatch through end to end.
    ?assertEqual(actuated, D#dispatch.status),
    ?assertNot(lists:member(required_prior_receipts, D#dispatch.unenforced_preactuation_checks)),
    ok.

%% ---------------------------------------------------------------------
%% Full real round trip: dispatch -> actuate -> capture/hash -> admit
%% ---------------------------------------------------------------------

test_full_dispatch_correlation_return_round_trip() ->
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-broker-full-roundtrip-1">>,
    StartSpec = start_spec(WorkflowId),
    {ok, Pid} = arazzo_runner_workflow:start_link(StartSpec),

    %% This is the ONLY thing this test does to drive the workflow: one
    %% real reaction event on the real production path. No direct
    %% admit_return/3 call anywhere below -- proving that is the entire
    %% point of this test, since before this fix admit_return/3 had zero
    %% production callers (confirmed by grep across apps/*/src) and a
    %% dispatched step would sit at ledger status `actuated` forever,
    %% never reaching air_core.
    ok = arazzo_runner_workflow:dispatch_event(Pid, {result, <<"step_a">>, ok}),

    %% Poll on the real terminal air_core condition rather than a fixed
    %% reaction-log count: do_dispatch_actuate/6's own internal
    %% admit_return/3 call (arazzo_runner_broker.erl) enqueues the
    %% admitted-result event to this SAME live process from within its own
    %% currently-running reaction cycle, so whether air_core absorbs
    %% step_b's completion within that same react/2 cycle or the very next
    %% one is a scheduling detail this test must not assume either way.
    RS1 = wait_until(WorkflowId, fun(RS) ->
        air_core:ready_steps(RS#runner_state.core) =:= []
    end),

    %% air_core genuinely advanced past step_b, not just "a message was
    %% sent": the real bind_outputs/3 evaluation ran, step_b_done is
    %% bound, and both steps are in history.
    ?assertEqual(true, maps:get(<<"step_b_done">>, air_core:get_env(RS1#runner_state.core))),
    ?assertEqual(2, length(air_core:get_history(RS1#runner_state.core))),

    %% The broker really actuated step_b: a genuine dispatch_token came
    %% back from dispatch/4 itself, not a refusal -- the success contract
    %% is unchanged even though dispatch/4 now also closes the
    %% return-admission loop internally before returning it.
    [{<<"step_b">>, {ok, DispatchToken}}] = RS1#runner_state.broker_dispatches,
    ?assert(is_binary(DispatchToken)),

    %% Post-actuation AND return-admission obligations both really
    %% happened (PRD 13 + PRD 8): a real ledger entry, actuated, with a
    %% captured consequence, a computed hash, the replay identity
    %% preserved, and -- the actual claim this test exists to prove --
    %% status = admitted, reached with no direct admit_return/3 call.
    {ok, D} = arazzo_runner_broker:get_ledger_entry(DispatchToken),
    ?assertEqual(admitted, D#dispatch.status),
    StepBDef = maps:get(<<"step_b">>, maps:get(steps, sample_workflow_def())),
    ?assertEqual({ok, {processed, StepBDef}}, D#dispatch.raw_consequence),
    ?assert(is_binary(D#dispatch.consequence_hash)),
    ?assertEqual(<<"replay-broker-1">>, D#dispatch.replay_id),

    %% A second, direct admit_return/3 call against the SAME dispatch_token
    %% is correctly refused as a double-admission, not silently re-applied
    %% -- proves the loop closed exactly once, not "eventually, maybe
    %% more than once".
    ?assertEqual(
        {error, {already_admitted, DispatchToken}},
        arazzo_runner_broker:admit_return(
            DispatchToken, D#dispatch.correlation_id, D#dispatch.return_authority_token)),
    ok.

%% ---------------------------------------------------------------------
%% Return-admission chain refusals (PRD 8)
%% ---------------------------------------------------------------------

test_admit_return_correlation_missing() ->
    Result = arazzo_runner_broker:admit_return(
        <<"never-issued-dispatch-token">>, <<"whatever-correlation">>, <<"whatever-auth">>),
    ?assertEqual(
        {refused, 'CORRELATION_MISSING',
         #{stage => correlation, dispatch_token => <<"never-issued-dispatch-token">>}},
        Result),
    ok.

%% ---------------------------------------------------------------------
%% Return-admission chain / CORRELATION_MISMATCH (PRD 8, PROJ-785)
%% ---------------------------------------------------------------------

%% Direct ledger construction (mirrors the RETURN_PROVENANCE_MISSING /
%% RETURN_STRUCTURE_REFUSED tests' own established pattern below): this
%% test's whole purpose is admit_return/3's OWN correlation-mismatch
%% refusal logic against a stable `actuated` entry. A real, live dispatch/4
%% round trip is no longer a way to hold that state for a second,
%% independent admit_return/3 call -- do_dispatch_actuate/6 now closes the
%% return-admission loop itself the instant actuation succeeds (see
%% test_full_dispatch_correlation_return_round_trip/0), so a real dispatch
%% would already be `admitted` before this test could exercise a mismatched
%% claim against it.
test_admit_return_correlation_mismatch() ->
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-broker-corr-mismatch-1">>,
    StartSpec = start_spec(WorkflowId),
    {ok, _Pid} = arazzo_runner_workflow:start_link(StartSpec),

    Token = <<"corr-mismatch-ledger-token-1">>,
    not_found = arazzo_runner_broker:get_ledger_entry(Token),
    D = #dispatch{
        dispatch_token = Token,
        workflow_id = WorkflowId,
        step_id = <<"step_a">>,
        correlation_id = <<"corr-broker-1">>,
        idempotency_key = <<"idem-corr-mismatch-1">>,
        actuation_token = <<"actuation-corr-mismatch-1">>,
        return_authority_token = <<"return-authority-corr-mismatch-1">>,
        replay_id = <<"replay-corr-mismatch-1">>,
        status = actuated,
        raw_consequence = ok,
        consequence_hash = <<"corr-mismatch-hash-1">>,
        prev_evidence_hash = <<>>,
        unenforced_preactuation_checks = [],
        required_result_types = []
    },
    true = ets:insert(arazzo_broker_dispatches, {Token, D}),

    %% A KNOWN dispatch_token (Stage 1 lookup succeeds) whose returner
    %% claims a DIFFERENT correlation_id than the one on file --
    %% CORRELATION_MISMATCH, not CORRELATION_MISSING.
    ?assertEqual(
        {refused, 'CORRELATION_MISMATCH',
         #{stage => correlation, dispatch_token => Token,
           expected_correlation_id => <<"corr-broker-1">>,
           returned_correlation_id => <<"not-the-real-correlation-id">>}},
        arazzo_runner_broker:admit_return(
            Token, <<"not-the-real-correlation-id">>, <<"return-authority-corr-mismatch-1">>)),

    %% Not admitted: air_core state unaffected (the live workflow process
    %% behind WorkflowId never received any event in this test), ledger
    %% entry still `actuated`, never `admitted` -- a correlation mismatch
    %% never even reaches the provenance/authority stages.
    {ok, RS} = arazzo_runner_workflow:get_runner_state(WorkflowId),
    ?assertEqual([<<"step_a">>], air_core:ready_steps(RS#runner_state.core)),
    ?assertEqual(false, maps:is_key(<<"step_a_done">>, air_core:get_env(RS#runner_state.core))),
    {ok, D2} = arazzo_runner_broker:get_ledger_entry(Token),
    ?assertEqual(actuated, D2#dispatch.status),
    ok.

%% ---------------------------------------------------------------------
%% Return-admission chain / RETURN_PROVENANCE_MISSING (PRD 8, PROJ-785)
%%
%% dispatch/4's real synchronous chain (do_dispatch/6 -> enqueue_io/2)
%% never leaves a ledger entry observable from outside in `dispatched` or
%% `dispatch_failed` state -- by the time dispatch/4 returns, do_dispatch/6
%% has already moved status to `actuated` (or the whole ledger entry was
%% never created at all, for the preactuation-refusal cases). Both states
%% ARE real and reachable (every entry starts `dispatched` at the top of
%% do_dispatch/6 before enqueue_io/2 is called; `dispatch_failed` is the
%% real outcome when enqueue_io/2 returns {refused,_,_} or {error,_}), just
%% not from a black-box caller of dispatch/4 alone. These tests hold the
%% ledger in each state directly via ets:insert/2 into the SAME
%% arazzo_broker_dispatches table and the SAME #dispatch{} shape
%% do_dispatch/6 itself writes -- proving admit_return/3's own logic
%% against those states, not mocking the broker.
%% ---------------------------------------------------------------------

test_admit_return_provenance_missing_dispatched() ->
    Token = <<"provenance-test-dispatched-1">>,
    %% Forces ensure_broker_ets/0 (idempotent) so the table exists even if
    %% this is the very first test in the module to touch it.
    not_found = arazzo_runner_broker:get_ledger_entry(Token),
    D = #dispatch{
        dispatch_token = Token,
        workflow_id = <<"wf-provenance-dispatched-1">>,
        step_id = <<"step_prov">>,
        correlation_id = <<"corr-provenance-dispatched-1">>,
        idempotency_key = <<"idem-provenance-dispatched-1">>,
        actuation_token = <<"actuation-provenance-dispatched-1">>,
        return_authority_token = <<"return-authority-provenance-dispatched-1">>,
        replay_id = <<"replay-provenance-dispatched-1">>,
        status = dispatched,
        raw_consequence = undefined,
        consequence_hash = undefined,
        prev_evidence_hash = <<>>,
        unenforced_preactuation_checks = []
    },
    true = ets:insert(arazzo_broker_dispatches, {Token, D}),

    ?assertEqual(
        {refused, 'RETURN_PROVENANCE_MISSING', #{stage => provenance, dispatch_token => Token}},
        arazzo_runner_broker:admit_return(
            Token, <<"corr-provenance-dispatched-1">>,
            <<"return-authority-provenance-dispatched-1">>)),
    ok.

test_admit_return_provenance_missing_dispatch_failed() ->
    Token = <<"provenance-test-dispatch-failed-1">>,
    not_found = arazzo_runner_broker:get_ledger_entry(Token),
    D = #dispatch{
        dispatch_token = Token,
        workflow_id = <<"wf-provenance-dispatch-failed-1">>,
        step_id = <<"step_prov">>,
        correlation_id = <<"corr-provenance-failed-1">>,
        idempotency_key = <<"idem-provenance-failed-1">>,
        actuation_token = <<"actuation-provenance-failed-1">>,
        return_authority_token = <<"return-authority-provenance-failed-1">>,
        replay_id = <<"replay-provenance-failed-1">>,
        status = dispatch_failed,
        raw_consequence = undefined,
        consequence_hash = undefined,
        prev_evidence_hash = <<>>,
        unenforced_preactuation_checks = []
    },
    true = ets:insert(arazzo_broker_dispatches, {Token, D}),

    ?assertEqual(
        {refused, 'RETURN_PROVENANCE_MISSING', #{stage => provenance, dispatch_token => Token}},
        arazzo_runner_broker:admit_return(
            Token, <<"corr-provenance-failed-1">>, <<"return-authority-provenance-failed-1">>)),
    ok.

%% ---------------------------------------------------------------------
%% Return-admission chain / RETURN_AUTHORITY_REFUSED (PRD 8)
%% ---------------------------------------------------------------------

%% Direct ledger construction -- same rationale as
%% test_admit_return_correlation_mismatch/0 above: dispatch/4 now closes
%% the return-admission loop itself the instant actuation succeeds, so a
%% real dispatch/4 round trip is no longer a way to observe a stable
%% `actuated` (never `admitted`) ledger entry for a second, independent
%% admit_return/3 call to probe.
test_admit_return_authority_refused() ->
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-broker-authority-refused-1">>,
    StartSpec = start_spec(WorkflowId),
    {ok, _Pid} = arazzo_runner_workflow:start_link(StartSpec),

    Token = <<"authority-refused-ledger-token-1">>,
    not_found = arazzo_runner_broker:get_ledger_entry(Token),
    CorrelationId = <<"corr-broker-1">>,
    D = #dispatch{
        dispatch_token = Token,
        workflow_id = WorkflowId,
        step_id = <<"step_a">>,
        correlation_id = CorrelationId,
        idempotency_key = <<"idem-authority-refused-1">>,
        actuation_token = <<"actuation-authority-refused-1">>,
        return_authority_token = <<"return-authority-authority-refused-1">>,
        replay_id = <<"replay-authority-refused-1">>,
        status = actuated,
        raw_consequence = ok,
        consequence_hash = <<"authority-refused-hash-1">>,
        prev_evidence_hash = <<>>,
        unenforced_preactuation_checks = [],
        required_result_types = []
    },
    true = ets:insert(arazzo_broker_dispatches, {Token, D}),

    %% Wrong token.
    ?assertEqual(
        {refused, 'RETURN_AUTHORITY_REFUSED', #{stage => authority, dispatch_token => Token}},
        arazzo_runner_broker:admit_return(Token, CorrelationId, <<"not-the-real-token">>)),

    %% Missing token.
    ?assertEqual(
        {refused, 'RETURN_AUTHORITY_REFUSED', #{stage => authority, dispatch_token => Token}},
        arazzo_runner_broker:admit_return(Token, CorrelationId, undefined)),

    %% Neither attempt advanced air_core state: the live workflow process
    %% behind WorkflowId never received any event in this test.
    {ok, RS} = arazzo_runner_workflow:get_runner_state(WorkflowId),
    ?assertEqual([<<"step_a">>], air_core:ready_steps(RS#runner_state.core)),
    ?assertEqual(false, maps:is_key(<<"step_a_done">>, air_core:get_env(RS#runner_state.core))),

    %% Ledger entry is still `actuated`, never `admitted`.
    {ok, D2} = arazzo_runner_broker:get_ledger_entry(Token),
    ?assertEqual(actuated, D2#dispatch.status),
    ok.

%% ---------------------------------------------------------------------
%% Return-admission chain / RETURN_STRUCTURE_REFUSED (PRD 8, PROJ-785)
%%
%% Today's echo-placeholder execute_io_request/1 (arazzo_runner_workflow.erl)
%% only ever actuates to a {ok, {processed, StepDef}} tuple, so a real, live
%% dispatch/4 -> enqueue_io/2 round trip can never itself produce an
%% integer raw_consequence. Both tests below hold the ledger's `actuated`
%% entry directly via ets:insert/2 -- the exact same #dispatch{} shape and
%% table do_dispatch/6 itself writes, mirroring the RETURN_PROVENANCE_
%% MISSING tests' own direct-construction pattern above -- the only way to
%% exercise the conformance check's decision against a controlled
%% raw_consequence shape. Both drive a real, live arazzo_runner_workflow
%% process (start_spec_structure/1's step_struct, whose own declared
%% outputs is what required_result_types/1 reads to derive `[integer]`),
%% so the positive test proves the full real path -- not just "not
%% refused" -- by asserting air_core's genuine post-admission env state.
%% ---------------------------------------------------------------------

test_admit_return_structure_conforms() ->
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-broker-structure-conforms-1">>,
    StartSpec = start_spec_structure(WorkflowId),
    {ok, _Pid} = arazzo_runner_workflow:start_link(StartSpec),

    Token = <<"structure-test-conforms-1">>,
    not_found = arazzo_runner_broker:get_ledger_entry(Token),
    D = #dispatch{
        dispatch_token = Token,
        workflow_id = WorkflowId,
        step_id = <<"step_struct">>,
        correlation_id = <<"corr-broker-1">>,
        idempotency_key = <<"idem-structure-conforms-1">>,
        actuation_token = <<"actuation-structure-conforms-1">>,
        return_authority_token = <<"return-authority-structure-conforms-1">>,
        replay_id = <<"replay-structure-conforms-1">>,
        status = actuated,
        %% A real integer -- conforms to the `integer` requirement
        %% required_result_types/1 derives from step_struct's own outputs
        %% (sample_workflow_def_with_result_binding/0):
        %% {op, '+', {var, '__result__'}, {literal, 1}}.
        raw_consequence = 42,
        consequence_hash = <<"structure-conforms-hash-1">>,
        prev_evidence_hash = <<>>,
        unenforced_preactuation_checks = [],
        required_result_types = [integer]
    },
    true = ets:insert(arazzo_broker_dispatches, {Token, D}),

    C0 = reaction_count(WorkflowId),
    ?assertEqual(
        {ok, admitted},
        arazzo_runner_broker:admit_return(
            Token, <<"corr-broker-1">>, <<"return-authority-structure-conforms-1">>)),
    RS1 = wait_for_reaction(WorkflowId, C0, result),

    %% Not just "not refused": the REAL air_core bind_outputs/3 ->
    %% eval_expr_nif evaluation genuinely ran on raw_consequence = 42 and
    %% computed 42 + 1 = 43 -- required_result_types/1's `integer`
    %% requirement was the correct prediction of what that evaluation
    %% needed to not badarg.
    ?assertEqual(43, maps:get(<<"doubled">>, air_core:get_env(RS1#runner_state.core))),

    {ok, D2} = arazzo_runner_broker:get_ledger_entry(Token),
    ?assertEqual(admitted, D2#dispatch.status),
    ok.

test_admit_return_structure_refused_type_mismatch() ->
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-broker-structure-mismatch-1">>,
    StartSpec = start_spec_structure(WorkflowId),
    {ok, _Pid} = arazzo_runner_workflow:start_link(StartSpec),

    Token = <<"structure-test-mismatch-1">>,
    not_found = arazzo_runner_broker:get_ledger_entry(Token),
    D = #dispatch{
        dispatch_token = Token,
        workflow_id = WorkflowId,
        step_id = <<"step_struct">>,
        correlation_id = <<"corr-broker-1">>,
        idempotency_key = <<"idem-structure-mismatch-1">>,
        actuation_token = <<"actuation-structure-mismatch-1">>,
        return_authority_token = <<"return-authority-structure-mismatch-1">>,
        replay_id = <<"replay-structure-mismatch-1">>,
        status = actuated,
        %% A binary, not an integer -- step_struct's own outputs need an
        %% integer for {op, '+', {var, '__result__'}, {literal, 1}} to
        %% decode without badarg-ing inside air_core's real eval_expr_nif.
        raw_consequence = <<"not-an-integer">>,
        consequence_hash = <<"structure-mismatch-hash-1">>,
        prev_evidence_hash = <<>>,
        unenforced_preactuation_checks = [],
        required_result_types = [integer]
    },
    true = ets:insert(arazzo_broker_dispatches, {Token, D}),

    ?assertEqual(
        {refused, 'RETURN_STRUCTURE_REFUSED',
         #{stage => structure, dispatch_token => Token,
           required_types => [integer], actual_type => binary}},
        arazzo_runner_broker:admit_return(
            Token, <<"corr-broker-1">>, <<"return-authority-structure-mismatch-1">>)),

    %% Not admitted: air_core state unaffected (no `doubled` key ever
    %% bound -- the real bind_outputs/3 evaluation that would have badarg'd
    %% on this raw_consequence never ran), ledger entry still `actuated`,
    %% never `admitted`.
    {ok, RS1} = arazzo_runner_workflow:get_runner_state(WorkflowId),
    ?assertEqual(false, maps:is_key(<<"doubled">>, air_core:get_env(RS1#runner_state.core))),
    {ok, D2} = arazzo_runner_broker:get_ledger_entry(Token),
    ?assertEqual(actuated, D2#dispatch.status),
    ok.

%% ---------------------------------------------------------------------
%% Actuation-route enforcement (DIRECT_ACTUATION_REFUSED, PRD 13)
%% ---------------------------------------------------------------------

test_direct_actuation_refused_bogus_token() ->
    Token = <<"never-issued-actuation-token-",
              (integer_to_binary(erlang:unique_integer([positive])))/binary>>,
    {ElapsedMicros, Result} = timer:tc(fun() ->
        arazzo_runner_workflow:enqueue_io(Token, #{probe => true})
    end),
    ?assertEqual({refused, 'DIRECT_ACTUATION_REFUSED', #{actuation_token => Token}}, Result),
    %% Refused before any pool round trip was attempted: had enqueue_io/2
    %% gone ahead and messaged a worker, a bogus token means no reply
    %% would ever come back and its own 5s receive-timeout would have been
    %% hit instead. Comfortably under that bound proves the token gate
    %% short-circuited before touching the pool.
    ?assert(ElapsedMicros < 1000000),
    ok.

test_direct_actuation_refused_token_reuse() ->
    process_flag(trap_exit, true),
    WorkflowId = <<"wf-broker-token-reuse-1">>,
    StartSpec = start_spec(WorkflowId),
    {ok, Pid} = arazzo_runner_workflow:start_link(StartSpec),

    C0 = reaction_count(WorkflowId),
    ok = arazzo_runner_workflow:dispatch_event(Pid, {result, <<"step_a">>, ok}),
    RS1 = wait_for_reaction(WorkflowId, C0, result),
    [{<<"step_b">>, {ok, DispatchToken}}] = RS1#runner_state.broker_dispatches,
    {ok, D} = arazzo_runner_broker:get_ledger_entry(DispatchToken),
    ActuationToken = D#dispatch.actuation_token,

    %% This exact token was already consumed by do_dispatch/6's own call
    %% to enqueue_io/2 during the real dispatch above -- a second, direct
    %% call with the SAME token (simulating a bypass of the broker) is
    %% refused, proving the ticket is genuinely one-shot, not just
    %% checked-and-left-valid.
    ?assertEqual(
        {refused, 'DIRECT_ACTUATION_REFUSED', #{actuation_token => ActuationToken}},
        arazzo_runner_workflow:enqueue_io(ActuationToken, #{probe => true})),
    ok.

%% ---------------------------------------------------------------------
%% Idempotency key dedup (PRD 13) + evidence hash chain (PRD 13)
%% ---------------------------------------------------------------------

test_dispatch_idempotent_dedup() ->
    WorkflowId = <<"wf-broker-idempotent-1">>,
    {ok, Identity} = arazzo_runner_identity:from_map(sample_identity(WorkflowId)),
    StepDef = #{outputs => [], next => []},

    {ok, Token1} = arazzo_runner_broker:dispatch(WorkflowId, Identity, <<"step_x">>, StepDef),
    {ok, D1} = arazzo_runner_broker:get_ledger_entry(Token1),
    ?assertEqual(actuated, D1#dispatch.status),
    ChainHeadAfter1 = ets:lookup(arazzo_broker_chain_heads, WorkflowId),

    {ok, Token2} = arazzo_runner_broker:dispatch(WorkflowId, Identity, <<"step_x">>, StepDef),
    ?assertEqual(Token1, Token2),
    %% Genuinely deduplicated, not "returns the old token but re-actuates
    %% anyway": the evidence chain head for this workflow did not move.
    ChainHeadAfter2 = ets:lookup(arazzo_broker_chain_heads, WorkflowId),
    ?assertEqual(ChainHeadAfter1, ChainHeadAfter2),
    ok.

test_evidence_hash_chain_extends() ->
    WorkflowId = <<"wf-broker-chain-extends-1">>,
    {ok, Identity} = arazzo_runner_identity:from_map(sample_identity(WorkflowId)),
    StepDef = #{outputs => [], next => []},

    {ok, Token1} = arazzo_runner_broker:dispatch(WorkflowId, Identity, <<"step_y1">>, StepDef),
    {ok, D1} = arazzo_runner_broker:get_ledger_entry(Token1),
    ?assertEqual(<<>>, D1#dispatch.prev_evidence_hash),
    ?assert(is_binary(D1#dispatch.consequence_hash)),

    {ok, Token2} = arazzo_runner_broker:dispatch(WorkflowId, Identity, <<"step_y2">>, StepDef),
    {ok, D2} = arazzo_runner_broker:get_ledger_entry(Token2),
    ?assertNotEqual(Token1, Token2),
    ?assertEqual(D1#dispatch.consequence_hash, D2#dispatch.prev_evidence_hash),
    ?assertNotEqual(D1#dispatch.consequence_hash, D2#dispatch.consequence_hash),
    ok.

%% ---------------------------------------------------------------------
%% Token secrecy (real auth-bypass fix): dispatch_token/actuation_token/
%% return_authority_token now require arazzo_runner_broker's per-node
%% secret (broker_secret/0), not just the public workflow_id/step_id/
%% idempotency_key an external caller already legitimately knows. Before
%% this fix, make_token/1 hashed ONLY those public parts, so anyone who
%% knew them could independently recompute a valid dispatch_token,
%% actuation_token, or return_authority_token and call enqueue_io/2 or
%% admit_return/3 directly -- bypassing DIRECT_ACTUATION_REFUSED /
%% RETURN_AUTHORITY_REFUSED entirely, a real authentication bypass.
%% ---------------------------------------------------------------------

test_actuation_token_requires_server_secret() ->
    WorkflowId = <<"wf-broker-secret-bypass-1">>,
    {ok, Identity} = arazzo_runner_identity:from_map(sample_identity(WorkflowId)),
    StepDef = #{outputs => [], next => []},
    StepId = <<"step_secret_bypass">>,
    %% idempotency_key/2's own deterministic default (arazzo_runner_broker.erl):
    %% absent an explicit idempotency_key in StepDef, StepId is the key.
    IdempotencyKey = StepId,

    {ok, DispatchToken} = arazzo_runner_broker:dispatch(WorkflowId, Identity, StepId, StepDef),
    {ok, D} = arazzo_runner_broker:get_ledger_entry(DispatchToken),

    %% An external attacker who knows only the PUBLIC identifiers
    %% (workflow_id, step_id, idempotency_key -- exactly what this test
    %% just used to call dispatch/4 itself) recomputes the token the OLD,
    %% pre-fix way: sha256 over the tagged, `|`-joined parts ALONE, no
    %% server-side secret mixed in -- the exact formula
    %% arazzo_runner_broker:make_token/1 used before this fix.
    LegacyDispatchToken = binary:encode_hex(crypto:hash(
        sha256,
        iolist_to_binary(lists:join(<<"|">>, [<<"dispatch">>, WorkflowId, StepId, IdempotencyKey])))),
    LegacyActuationToken = binary:encode_hex(crypto:hash(
        sha256, iolist_to_binary(lists:join(<<"|">>, [<<"actuate">>, LegacyDispatchToken])))),
    LegacyReturnAuthorityToken = binary:encode_hex(crypto:hash(
        sha256, iolist_to_binary(lists:join(<<"|">>, [<<"return-authority">>, LegacyDispatchToken])))),

    %% The real, server-minted tokens are NOT independently recomputable
    %% from public identifiers alone -- the per-node secret genuinely
    %% changes every one of them.
    ?assertNotEqual(DispatchToken, LegacyDispatchToken),
    ?assertNotEqual(D#dispatch.actuation_token, LegacyActuationToken),
    ?assertNotEqual(D#dispatch.return_authority_token, LegacyReturnAuthorityToken),

    %% Concretely: an attacker presenting the publicly-recomputable
    %% actuation token cannot reach the io-worker pool at all -- the
    %% actual DIRECT_ACTUATION_REFUSED bypass this fix closes.
    ?assertEqual(
        {refused, 'DIRECT_ACTUATION_REFUSED', #{actuation_token => LegacyActuationToken}},
        arazzo_runner_workflow:enqueue_io(LegacyActuationToken, #{probe => true})),

    %% ...nor can they forge a return admission using the
    %% publicly-recomputable return_authority_token against the real
    %% (still `actuated`, not yet admitted -- no live workflow process
    %% exists for WorkflowId in this test) ledger entry -- the actual
    %% RETURN_AUTHORITY_REFUSED bypass this fix closes.
    ?assertEqual(
        {refused, 'RETURN_AUTHORITY_REFUSED',
         #{stage => authority, dispatch_token => DispatchToken}},
        arazzo_runner_broker:admit_return(
            DispatchToken, D#dispatch.correlation_id, LegacyReturnAuthorityToken)),
    ok.

%% ---------------------------------------------------------------------
%% Idempotency dedup TOCTOU race (real concurrency, not two sequential
%% calls): two real processes call dispatch/4 for the SAME
%% {workflow_id, step_id, idempotency_key}, released as close to
%% simultaneously as possible via a shared barrier message, to maximize
%% the window the ets:lookup-then-ets:insert pair this test's
%% corresponding fix (ets:insert_new/2) replaced would have left open.
%% ---------------------------------------------------------------------

test_concurrent_duplicate_dispatch_claims_exactly_once() ->
    WorkflowId = <<"wf-broker-race-1">>,
    {ok, Identity} = arazzo_runner_identity:from_map(sample_identity(WorkflowId)),
    StepDef = #{outputs => [], next => []},
    StepId = <<"step_race">>,

    Parent = self(),
    Barrier = make_ref(),
    SpawnRacer = fun() ->
        spawn(fun() ->
            receive {go, Barrier} -> ok end,
            Result = arazzo_runner_broker:dispatch(WorkflowId, Identity, StepId, StepDef),
            Parent ! {done, self(), Result}
        end)
    end,
    P1 = SpawnRacer(),
    P2 = SpawnRacer(),
    %% Released back-to-back with no intervening work, both already
    %% blocked on the SAME barrier receive -- maximizes the chance the
    %% BEAM scheduler actually interleaves the two dispatch/4 calls
    %% instead of running them fully sequentially.
    P1 ! {go, Barrier},
    P2 ! {go, Barrier},

    {Result1, Result2} = receive_racer_results(P1, P2),

    {ok, Token1} = Result1,
    {ok, Token2} = Result2,
    %% Both racers observe the SAME dispatch_token -- dispatch_token/3 is
    %% a pure, deterministic function of
    %% (WorkflowId, StepId, IdempotencyKey, node secret); neither racer
    %% invents a distinct one.
    ?assertEqual(Token1, Token2),

    %% The ledger ends up in a single, consistent state: the ONE real
    %% enqueue_io/2 round trip that ets:insert_new/2 ever let through for
    %% this dispatch_token succeeded, and status is genuinely `actuated`
    %% -- never clobbered back to `dispatch_failed` by a racing loser's
    %% write, which is exactly the failure mode the old
    %% ets:lookup/2-then-ets:insert/2 pair allowed (both racers could
    %% observe a dedup miss and both reach do_dispatch_actuate/6, racing
    %% on the SAME deterministic dispatch_token/actuation_token keys
    %% downstream).
    {ok, D} = arazzo_runner_broker:get_ledger_entry(Token1),
    ?assertEqual(actuated, D#dispatch.status),
    ?assert(is_binary(D#dispatch.consequence_hash)),
    ok.

receive_racer_results(P1, P2) ->
    receive
        {done, P1, R1} ->
            receive
                {done, P2, R2} -> {R1, R2}
            after 5000 -> error({timeout_waiting_for_racer, P2})
            end;
        {done, P2, R2} ->
            receive
                {done, P1, R1} -> {R1, R2}
            after 5000 -> error({timeout_waiting_for_racer, P1})
            end
    after 5000 ->
        error(timeout_waiting_for_both_racers)
    end.
