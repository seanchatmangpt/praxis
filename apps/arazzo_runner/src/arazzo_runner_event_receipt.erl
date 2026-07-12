-module(arazzo_runner_event_receipt).
-include("arazzo_event_receipt.hrl").

-export([
    build/1,
    emit/1,
    chain_head/2,
    get_chain_head/1,
    get_receipt/2,
    receipt_to_map/1
]).

%% PROJ-781 (PRD v26.7.11 15 -- Receipt and Replay, PRD.md:704-716).
%%
%% Real construction + chaining logic for #event_receipt{} (see
%% arazzo_event_receipt.hrl for the field-by-field PRD mapping). Two public
%% entry points at two different levels:
%%
%%  - build/1: pure construction from an already-fully-supplied field map
%%    (including an explicit prior_receipt_head and logical_clock) --
%%    no ETS, no side effects, the unit the determinism tests exercise
%%    directly.
%%  - emit/1: the real per-workflow chaining wrapper a caller (e.g.
%%    arazzo_runner_broker:do_dispatch/6) actually uses -- looks up this
%%    workflow's current chain head (or seeds it from the caller-supplied
%%    identity_receipt_head, PRD 7.8's existing genesis value, for a fresh
%%    workflow), derives the next logical clock tick, calls build/1, then
%%    durably advances both the chain-head pointer and the append-only
%%    receipt log.
%%
%% Not the only "receipt" concept in this codebase -- see
%% arazzo_event_receipt.hrl's header comment for how this differs from both
%% #workflow_identity.receipt_head (PRD 7.8, the identity-level genesis
%% value this module reads, never overwrites) and #dispatch{}'s own
%% sha256-based consequence_hash chain (PRD 13, PROJ-758, a narrower,
%% already-disclaimed-as-not-BLAKE3 local ledger hash).

%% Every key below must be present in the map passed to build/1 (values may
%% legitimately be `undefined`, e.g. parent_semantic_id for a root
%% workflow -- mirrors arazzo_runner_identity's REQUIRED_IDENTITY_FIELDS
%% key-presence-not-value-validity convention).
-define(REQUIRED_BUILD_FIELDS, [
    workflow_semantic_id,
    parent_semantic_id,
    event_type,
    event_material,
    prior_receipt_head,
    resulting_state_material,
    command_material,
    runtime_profile,
    logical_clock,
    replay_id
]).

-define(REQUIRED_EMIT_FIELDS, [
    workflow_id,
    parent_workflow_id,
    event_type,
    event_material,
    resulting_state_material,
    command_material,
    runtime_profile,
    replay_id,
    identity_receipt_head
]).

-define(CHAIN_HEAD_TABLE, arazzo_event_receipt_chain_heads).
-define(CLOCK_TABLE, arazzo_event_receipt_clocks).
-define(RECEIPT_LOG_TABLE, arazzo_event_receipt_log).

%% ---------------------------------------------------------------------
%% Pure construction
%% ---------------------------------------------------------------------

%% # Complexity
%% O(1): the required-field check walks a fixed-length (10-element) literal
%% list regardless of input size; the three material hashes are each
%% O(bytes) in arazzo_runner_blake3:hex/1 (one BLAKE3 pass per material,
%% dominated by the caller-supplied term sizes, not by anything this
%% function iterates).
-spec build(map()) -> {ok, #event_receipt{}} | {error, term()}.
build(Fields) when is_map(Fields) ->
    Missing = [F || F <- ?REQUIRED_BUILD_FIELDS, not maps:is_key(F, Fields)],
    case Missing of
        [] -> build_checked(Fields);
        _ -> {error, {missing_event_receipt_fields, Missing}}
    end.

build_checked(Fields) ->
    EventMaterial = maps:get(event_material, Fields),
    ResultingStateMaterial = maps:get(resulting_state_material, Fields),
    CommandMaterial = maps:get(command_material, Fields),
    EventDigestResult = arazzo_runner_blake3:hex(canonical_term_bytes(EventMaterial)),
    StateDigestResult = arazzo_runner_blake3:hex(canonical_term_bytes(ResultingStateMaterial)),
    CommandDigestResult = arazzo_runner_blake3:hex(canonical_term_bytes(CommandMaterial)),
    case {EventDigestResult, StateDigestResult, CommandDigestResult} of
        {{ok, EventDigest}, {ok, StateDigest}, {ok, CommandDigest}} ->
            seal_receipt(Fields, EventDigest, StateDigest, CommandDigest);
        _ ->
            {error, {digest_computation_failed, #{
                event => EventDigestResult,
                resulting_state => StateDigestResult,
                command => CommandDigestResult
            }}}
    end.

seal_receipt(Fields, EventDigest, StateDigest, CommandDigest) ->
    R0 = #event_receipt{
        workflow_semantic_id = maps:get(workflow_semantic_id, Fields),
        parent_semantic_id = maps:get(parent_semantic_id, Fields),
        event_type = maps:get(event_type, Fields),
        event_digest = EventDigest,
        prior_receipt_head = maps:get(prior_receipt_head, Fields),
        resulting_state_digest = StateDigest,
        command_digest = CommandDigest,
        runtime_profile = maps:get(runtime_profile, Fields),
        logical_clock = maps:get(logical_clock, Fields),
        replay_id = maps:get(replay_id, Fields),
        receipt_head = <<>>
    },
    case arazzo_runner_blake3:hex(chain_material(R0)) of
        {ok, ReceiptHead} -> {ok, R0#event_receipt{receipt_head = ReceiptHead}};
        {error, Reason} -> {error, {receipt_head_hash_failed, Reason}}
    end.

%% Canonical, deterministic byte representation of one already-digested
%% receipt's 10 declared fields, in a FIXED order (the record's own
%% declaration order) -- the same term_to_binary-then-hash technique
%% PROJ-756's air_core_corpus_test.erl established for this codebase's
%% Erlang side (no raw maps, no HashMap-equivalent iteration order feeding
%% the hash). This is what receipt_head actually binds.
chain_material(#event_receipt{} = R) ->
    erlang:term_to_binary({
        R#event_receipt.workflow_semantic_id,
        R#event_receipt.parent_semantic_id,
        R#event_receipt.event_type,
        R#event_receipt.event_digest,
        R#event_receipt.prior_receipt_head,
        R#event_receipt.resulting_state_digest,
        R#event_receipt.command_digest,
        R#event_receipt.runtime_profile,
        R#event_receipt.logical_clock,
        R#event_receipt.replay_id
    }, [{minor_version, 1}]).

canonical_term_bytes(Term) ->
    erlang:term_to_binary(Term, [{minor_version, 1}]).

%% ---------------------------------------------------------------------
%% Chaining wrapper (the real per-workflow event stream integration point)
%% ---------------------------------------------------------------------

%% # Complexity
%% O(1): two ETS lookups (chain head, logical clock) plus one atomic ETS
%% counter increment, then build/1's own O(bytes) work; on success, two ETS
%% inserts (chain-head pointer update, append-only receipt-log entry).
-spec emit(map()) -> {ok, #event_receipt{}} | {error, term()}.
emit(Params) when is_map(Params) ->
    Missing = [F || F <- ?REQUIRED_EMIT_FIELDS, not maps:is_key(F, Params)],
    case Missing of
        [] -> emit_checked(Params);
        _ -> {error, {missing_event_receipt_emit_fields, Missing}}
    end.

emit_checked(Params) ->
    ensure_event_receipt_ets(),
    WorkflowId = maps:get(workflow_id, Params),
    IdentityReceiptHead = maps:get(identity_receipt_head, Params),
    PriorHead = chain_head(WorkflowId, IdentityReceiptHead),
    LogicalClock = next_logical_clock(WorkflowId),
    BuildFields = #{
        workflow_semantic_id => WorkflowId,
        parent_semantic_id => maps:get(parent_workflow_id, Params),
        event_type => maps:get(event_type, Params),
        event_material => maps:get(event_material, Params),
        prior_receipt_head => PriorHead,
        resulting_state_material => maps:get(resulting_state_material, Params),
        command_material => maps:get(command_material, Params),
        runtime_profile => maps:get(runtime_profile, Params),
        logical_clock => LogicalClock,
        replay_id => maps:get(replay_id, Params)
    },
    case build(BuildFields) of
        {ok, Receipt} ->
            true = ets:insert(?CHAIN_HEAD_TABLE, {WorkflowId, Receipt#event_receipt.receipt_head}),
            true = ets:insert(?RECEIPT_LOG_TABLE, {{WorkflowId, LogicalClock}, Receipt}),
            {ok, Receipt};
        {error, Reason} ->
            %% Deliberately does NOT advance the chain-head pointer or the
            %% logical clock's externally-observable meaning on failure: no
            %% #event_receipt{} was ever minted for this attempt, so there
            %% is nothing for a later event to have chained from. The
            %% underlying ETS counter itself may still have incremented
            %% (see next_logical_clock/1) -- a harmless numbering gap, not
            %% an orphaned receipt, exactly like a failed-transaction ID
            %% generator skipping a value.
            {error, Reason}
    end.

%% Current chain head for WorkflowId, or IdentityReceiptHead (the genesis
%% value already required and validated by
%% arazzo_runner_broker:check_required_prior_receipts/5, PRD 7.8) if this
%% is the first event ever emitted for this workflow.
chain_head(WorkflowId, IdentityReceiptHead) ->
    case ets:lookup(?CHAIN_HEAD_TABLE, WorkflowId) of
        [{WorkflowId, Head}] -> Head;
        [] -> IdentityReceiptHead
    end.

-spec get_chain_head(binary()) -> {ok, binary()} | not_found.
get_chain_head(WorkflowId) ->
    ensure_event_receipt_ets(),
    case ets:lookup(?CHAIN_HEAD_TABLE, WorkflowId) of
        [{WorkflowId, Head}] -> {ok, Head};
        [] -> not_found
    end.

-spec get_receipt(binary(), non_neg_integer()) -> {ok, #event_receipt{}} | not_found.
get_receipt(WorkflowId, LogicalClock) ->
    ensure_event_receipt_ets(),
    case ets:lookup(?RECEIPT_LOG_TABLE, {WorkflowId, LogicalClock}) of
        [{{WorkflowId, LogicalClock}, Receipt}] -> {ok, Receipt};
        [] -> not_found
    end.

%% Debuggability/introspection only, e.g. for logging or a future replay
%% verifier (PROJ-782) to serialize -- never itself hashed (chain_material/1
%% hashes the record's own fields directly, not this map).
-spec receipt_to_map(#event_receipt{}) -> map().
receipt_to_map(#event_receipt{} = R) ->
    #{
        workflow_semantic_id => R#event_receipt.workflow_semantic_id,
        parent_semantic_id => R#event_receipt.parent_semantic_id,
        event_type => R#event_receipt.event_type,
        event_digest => R#event_receipt.event_digest,
        prior_receipt_head => R#event_receipt.prior_receipt_head,
        resulting_state_digest => R#event_receipt.resulting_state_digest,
        command_digest => R#event_receipt.command_digest,
        runtime_profile => R#event_receipt.runtime_profile,
        logical_clock => R#event_receipt.logical_clock,
        replay_id => R#event_receipt.replay_id,
        receipt_head => R#event_receipt.receipt_head
    }.

%% Monotonically increasing per-workflow event counter -- a logic tick, not
%% wall time (repo-wide no-wall-clock-in-receipt-paths invariant). The
%% 4-arg ets:update_counter/4 form atomically inserts {WorkflowId, 0} as
%% the default object on first use, THEN applies the {2, 1} increment to
%% it (ets:update_counter/4's documented behavior: Default is inserted
%% first, and the update is still applied to the resulting object) -- so
%% the first event for a given workflow_id is logical_clock 1, not 0.
%% Still strictly monotonic from a fixed, known start; no
%% read-then-write race between two concurrent emit/1 callers for the same
%% workflow_id.
next_logical_clock(WorkflowId) ->
    ets:update_counter(?CLOCK_TABLE, WorkflowId, {2, 1}, {WorkflowId, 0}).

%% ---------------------------------------------------------------------
%% ETS bootstrap. Deliberately this module's OWN tables, not
%% arazzo_runner_broker's ensure_broker_ets/0 -- additive, does not touch
%% that module's table set or its infra_loop/0 ownership, mirroring
%% arazzo_runner_identity:open_table/0's "lazy, safe to call from anywhere,
%% idempotent" pattern.
%% ---------------------------------------------------------------------

ensure_event_receipt_ets() ->
    ensure_table(?CHAIN_HEAD_TABLE),
    ensure_table(?CLOCK_TABLE),
    ensure_table(?RECEIPT_LOG_TABLE),
    ok.

ensure_table(Name) ->
    case ets:info(Name) of
        undefined ->
            try ets:new(Name, [public, named_table, set,
                                {write_concurrency, true}, {read_concurrency, true}]) of
                Name -> ok
            catch
                error:badarg -> ok  %% lost the creation race to a concurrent caller
            end;
        _ ->
            ok
    end.
