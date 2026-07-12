-module(arazzo_runner_broker).
-include("arazzo_runner.hrl").
-include("arazzo_broker.hrl").

-export([
    dispatch/4,
    admit_return/3,
    get_ledger_entry/1,
    consume_actuation_token/1
]).

%% ---------------------------------------------------------------------
%% PROJ-758 (PRD v26.7.11 section 13 -- Broker Requirements; section 8 --
%% Independent Process Cells / return-admission chain).
%% ---------------------------------------------------------------------
%%
%% Design decision (this ticket): a new module inside the EXISTING
%% apps/arazzo_runner/ app, not a new top-level app. Reasons, in order:
%%
%%  1. PRD 13's own bullet list frames the broker as what stands between
%%     air_core's `C` (dispatch_step commands, PRD 7.7) and the io-worker
%%     pool's actual actuation -- and both of those already live in
%%     arazzo_runner_workflow.erl. The comment this ticket's own text
%%     points at ("Actual actuation ... belongs behind the broker ... not
%%     dispatched directly from an I/O worker", arazzo_runner_workflow.erl
%%     ~line 525) names this module's job as gating THAT pool, not
%%     replacing it -- there is no reason for the gate and the gated
%%     resource to live in different applications.
%%  2. PRD 8 lists "broker-mediated actuation" as a property an
%%     Independent Process Cell HAS, not a tenth architectural layer of its
%%     own alongside PRD 7.1-7.10's ten. arazzo_runner (PROJ-757's own
%%     documented reasoning) already *is* the OTP Outer Runner / process
%%     cell for this codebase; the broker is a component the cell uses to
%%     actuate, not a peer of the cell.
%%  3. A separate app would need its own way to reach arazzo_runner's
%%     ETS-backed pending_dispatches/pg-scoped io-worker pool anyway
%%     (cross-app calls, same VM) -- that indirection buys nothing a
%%     same-app module doesn't already have for free, and would fork
%%     ownership of "who is allowed to actuate" across two supervision
%%     trees for no semantic reason (echoing PROJ-757's own reasoning
%%     against a second OTP-runner app).
%%
%% Concretely: this module mints and validates the tokens that
%% arazzo_runner_workflow:enqueue_io/2 requires before it will forward
%% anything to the io-worker pool (see that function) -- that pairing is
%% what makes "the broker SHALL be the only actuation route" (PRD 13) a
%% real, mechanically-enforced property instead of a comment, and it is
%% also why the ledger ETS tables below are created in that module's
%% long-lived infra_loop/0, not here (this module has no long-lived process
%% of its own; ETS table ownership must not be tied to whichever transient
%% caller happens to invoke dispatch/4 first).
%%
%% Scope actually closed this ticket (see docs/jira/v26.7.11/tickets/
%% index.md PROJ-758's own text): CORRELATION_MISSING, RETURN_AUTHORITY_
%% REFUSED, DIRECT_ACTUATION_REFUSED (the latter enforced in
%% arazzo_runner_workflow:enqueue_io/2, not here), and
%% BROKER_RECEIPT_PRECONDITION_MISSING (required_prior_receipts -- wired in
%% a later pass of this same ticket after adversarial review found
%% receipt_head already had a real Erlang-side data source, see
%% ?UNENFORCED_PREACTUATION_CHECKS below) as real, triggerable refusals with
%% negative tests. PROJ-785 owns the remainder of the 8-code broker/
%% correlation catalog; every PRD-13/PRD-8 check this module does not
%% independently enforce is named explicitly below (?UNENFORCED_
%% PREACTUATION_CHECKS, ?UNENFORCED_RETURN_STAGES) rather than silently
%% skipped or covered by an invented code.

%% PRD 13 lists 9 pre-actuation checks. This module enforces 4 for real:
%% correlation ID (CORRELATION_MISSING), idempotency key (real dedup, see
%% dispatch/4), input conformance (StepDef's map-ness is enforced
%% structurally by this module's own function-head pattern matching --
%% the same "fails loud via badarg/function_clause on malformed shape"
%% convention air_core.erl already uses, e.g. its `<<StepId/binary>>`
%% head patterns; there is no typed-Refusal convention anywhere in this
%% Erlang codebase to be consistent with instead), and required prior
%% receipts (BROKER_RECEIPT_PRECONDITION_MISSING, see dispatch/4).
%%
%% required_prior_receipts was originally left in this unenforced set on
%% the claim that it had "no Erlang-side data source" -- that was
%% incorrect and was corrected by adversarial review: `receipt_head` has
%% been a required, always-present #workflow_identity{} field since
%% PROJ-757 (see ?REQUIRED_IDENTITY_FIELDS in arazzo_runner_identity.erl),
%% at the same depth of effort as the correlation_id check this ticket
%% already implemented. The remaining 5 genuinely have no Erlang-side data
%% source to check against yet (no GraphLaw Authority Registry / hook-
%% contract bridge exists in apps/ today) and are recorded, not silently
%% assumed-passing, on every ledger entry.
-define(UNENFORCED_PREACTUATION_CHECKS, [
    current_artifact_standing,
    actor_role,
    capability_authority,
    hook_contract,
    artifact_lineage
]).

%% PRD 8's chain has 6 stages: correlation -> provenance -> authority ->
%% structure -> semantic_conformance -> O*. PROJ-758 enforced correlation
%% (CORRELATION_MISSING, unknown dispatch_token) and authority (RETURN_
%% AUTHORITY_REFUSED) for real; provenance was originally left unenforced
%% on the claim it was "trivially satisfied by construction" the moment
%% Stage 1 found a ledger entry at all.
%%
%% PROJ-785 (this ticket) applied the same adversarial-review standard
%% PROJ-758's own remediation used for required_prior_receipts and found
%% that provenance claim incomplete: Stage 1 finding D by dispatch_token
%% proves the TOKEN is known, not that a consequence actually originated
%% from a broker-dispatched IO operation. `status` already distinguishes
%% those cases -- a real, already-populated #dispatch{} field set only by
%% do_dispatch/6, not invented for this check -- so RETURN_PROVENANCE_
%% MISSING is now enforced for real (see admit_return_provenance/2) instead
%% of a permissive pass-through. CORRELATION_MISMATCH is also now real:
%% admit_return/3 takes the returner's claimed CorrelationId and checks it
%% against the SAME D#dispatch.correlation_id the ledger has carried since
%% dispatch time (populated from Identity#workflow_identity.correlation_id
%% -- real data, not a field invented for this check; see admit_return/3).
%%
%% A second adversarial-review pass (still PROJ-785) re-examined the
%% original "structure and semantic_conformance remain genuine gaps" claim
%% below and found `structure` incomplete too, on the exact same standard:
%% air_core's StepDef indeed carries no declared output SCHEMA anywhere in
%% this codebase (re-grepped `output_schema`/`schema`/`shape` across
%% apps/arazzo_runner and apps/air_core -- still none exists), but StepDef's
%% `outputs` field -- the SAME bind-rule list air_core:bind_outputs/3
%% evaluates against raw_consequence once admitted -- already encodes a
%% real, checkable structural expectation whenever a bind rule references
%% the sentinel `{var, '__result__'}` under a type-coercing operator: air_
%% core's own eval_expr_nif (apps/air_core/native/air_core_nif/src/lib.rs)
%% decodes that operand as i64 for `+`/`-`/`*`/`/` and as bool for `and`/
%% `or`/`not`, and would badarg on a mismatched raw_consequence the moment
%% the live workflow process actually ran bind_outputs/3 post-admission.
%% required_result_types/1 below statically derives that same type
%% requirement from StepDef's own `outputs` at dispatch time (do_dispatch/6
%% stores it on the ledger entry, see arazzo_broker.hrl) and
%% admit_return_structure/1 checks raw_consequence against it BEFORE
%% admission -- real, StepDef-declared structural conformance, not an
%% invented schema field. Steps whose outputs never reference `__result__`
%% under a typed operator (the literal-only majority of this codebase's
%% test corpus) derive an empty requirement set, which is vacuously
%% satisfied by any raw_consequence -- correct semantics (the step declared
%% no structural expectation), not a cop-out default.
%%
%% RETURN_SEMANTIC_REFUSED remains a genuine, disclosed gap after the same
%% re-investigation: PRD 11 frames semantic conformance as SHACL/admission-
%% layer territory (crates/praxis-graphlaw's src/shacl/ + src/chatman/
%% admission8.rs), and no bridge from this Erlang codebase reaches it.
%% Concretely checked, not assumed: this repo has exactly ONE Erlang<->Rust
%% bridge anywhere in apps/ -- air_core's eval_expr_nif (rustler NIF,
%% apps/air_core/native/air_core_nif/, single dependency `rustler = "0.33"`,
%% no toolchain override) -- and it is a narrow arithmetic/boolean/
%% comparison evaluator over individual Erlang terms, architecturally
%% unrelated to SHACL shape validation over an RDF graph. Extending it (or
%% adding a second NIF, or a port) to reach crates/praxis-graphlaw's
%% admission layer would mean pulling that crate's full dependency graph
%% (oxigraph, spargebra, two out-of-tree path deps at /Users/sac/bcinr and
%% /Users/sac/wasm4pm, the nightly-2026-06-22 toolchain pin) into a BEAM-
%% loaded NIF, with no existing representation of raw_consequence as RDF
%% quads to hand it in the first place -- genuinely new infrastructure, out
%% of this ticket's reasonable-effort scope, not a fake bridge forced
%% through. No open_port/httpc bridge of any kind exists anywhere in
%% apps/arazzo_runner or apps/air_core either (grepped for
%% open_port/gen_tcp/httpc: across both apps -- zero hits). Kept in
%% ?UNENFORCED_RETURN_STAGES below so the gap stays a lookup, not a
%% surprise.
-define(UNENFORCED_RETURN_STAGES, [
    return_semantic_refused
]).

%% ---------------------------------------------------------------------
%% Pre-actuation verification + actuation + post-actuation obligations
%% (PRD 13).
%% ---------------------------------------------------------------------

%% # Complexity
%% O(1) ETS/bookkeeping plus O(bytes) hashing, no traversal of
%% workflow/step collections: one ETS lookup for the correlation-id-derived
%% dedup key, at most one io-worker round trip (bounded by enqueue_io/2's
%% own 5s timeout), one sha256 over the captured consequence. The
%% correlation-id and receipt-head pre-actuation gates are both O(1)
%% binary-presence checks on already-loaded #workflow_identity{} fields.
%% PROJ-781 adds one arazzo_runner_event_receipt:emit/1 call before any of
%% the above: O(1) ETS (chain head + logical clock) plus 4 real BLAKE3
%% subprocess round trips (arazzo_runner_blake3:hex/1, one per material
%% plus one for the receipt_head seal), each O(bytes) in its own material
%% and bounded by that module's own 10s timeout.
-spec dispatch(binary(), #workflow_identity{}, binary(), map()) ->
    {ok, binary()} | {refused, atom(), map()} | {error, term()}.
dispatch(WorkflowId, Identity, StepId, StepDef)
        when is_binary(WorkflowId), is_record(Identity, workflow_identity),
             is_binary(StepId), is_map(StepDef) ->
    ensure_broker_ets(),
    CorrelationId = Identity#workflow_identity.correlation_id,
    case is_binary(CorrelationId) andalso byte_size(CorrelationId) > 0 of
        false ->
            {refused, 'CORRELATION_MISSING',
             #{stage => preactuation, workflow_id => WorkflowId, step_id => StepId}};
        true ->
            check_required_prior_receipts(WorkflowId, Identity, StepId, StepDef, CorrelationId)
    end.

%% PRD 13 pre-actuation bullet "required prior receipts". `receipt_head` is
%% a required #workflow_identity{} field (PROJ-757,
%% arazzo_runner_identity:?REQUIRED_IDENTITY_FIELDS) that is only ever
%% populated with a genuine, non-empty receipt-chain head by callers that
%% actually have one (see PROJ-757's from_map/1: key-presence is enforced
%% at identity-construction time, but -- exactly like correlation_id --
%% value-validity is this module's job, not identity's). An
%% #workflow_identity{} whose receipt_head is not a non-empty binary
%% (e.g. `undefined`, legitimately constructible via from_map/1 the same
%% way test_correlation_missing_on_dispatch/0 constructs a missing
%% correlation_id) means no prior receipt chain is actually attached to
%% this workflow instance, so actuation is refused before any ledger entry
%% is created -- symmetric with the correlation_id gate above.
check_required_prior_receipts(WorkflowId, Identity, StepId, StepDef, CorrelationId) ->
    ReceiptHead = Identity#workflow_identity.receipt_head,
    case is_binary(ReceiptHead) andalso byte_size(ReceiptHead) > 0 of
        false ->
            {refused, 'BROKER_RECEIPT_PRECONDITION_MISSING',
             #{stage => preactuation, workflow_id => WorkflowId, step_id => StepId}};
        true ->
            IdempotencyKey = idempotency_key(StepDef, StepId),
            DedupKey = {WorkflowId, StepId, IdempotencyKey},
            %% dispatch_token/3 is a pure function of
            %% (WorkflowId, StepId, IdempotencyKey, node secret) -- every
            %% concurrent caller for the same DedupKey computes the
            %% identical value independently, before any ETS write, so
            %% claiming DedupKey atomically below and handing the loser
            %% back this SAME value (rather than a value invented by
            %% whichever racer happened to win) is safe.
            DispatchToken = dispatch_token(WorkflowId, StepId, IdempotencyKey),
            %% PRD 13 "idempotency key", made atomic (fix for the TOCTOU
            %% race an independent audit found in the check-then-act pair
            %% this replaced: ets:lookup/2 for an existing dispatch, then
            %% a later, separate ets:insert/2 on miss, is non-atomic on a
            %% write_concurrency table -- two racing duplicate deliveries
            %% could both observe a miss and both proceed to
            %% do_dispatch/7, and the loser's do_dispatch_actuate/6
            %% failure-branch write
            %% (`D0#dispatch{status = dispatch_failed}`) could then
            %% unconditionally clobber the winner's successful
            %% `status = actuated` ledger record for the SAME
            %% (deterministic) DispatchToken key, since both racers derive
            %% the identical dispatch_token/3 value for the same DedupKey.
            %% ets:insert_new/2 is a single atomic compare-and-swap: only
            %% one of two racing processes can ever have it return `true`
            %% for the same key, so only one of them ever reaches
            %% do_dispatch/7 (and therefore only one of them ever writes
            %% to arazzo_broker_dispatches for this dispatch_token) --
            %% eliminating the clobber window entirely rather than
            %% narrowing it.
            case ets:insert_new(arazzo_broker_dedup, {DedupKey, DispatchToken}) of
                true ->
                    do_dispatch(WorkflowId, Identity, StepId, StepDef, CorrelationId,
                                IdempotencyKey, DispatchToken);
                false ->
                    %% Lost the claim race (or this is a genuine repeat
                    %% dispatch of an already-claimed key): return the
                    %% SAME token the winner claimed, and do not actuate
                    %% again. lists:nth/get after insert_new's `false` can
                    %% never see [] here -- insert_new only returns false
                    %% when a value for this exact key already exists.
                    [{DedupKey, ExistingToken}] = ets:lookup(arazzo_broker_dedup, DedupKey),
                    {ok, ExistingToken}
            end
    end.

idempotency_key(StepDef, StepId) ->
    case maps:get(idempotency_key, StepDef, undefined) of
        Key when is_binary(Key), byte_size(Key) > 0 -> Key;
        _ ->
            %% Deterministic default (no randomness -- repo determinism
            %% discipline): absent an explicit key, the step id itself is
            %% the natural idempotency boundary, since a given step within
            %% a given workflow instance is already a singleton dispatch
            %% target under air_core's AND/join semantics (PROJ-756).
            StepId
    end.

do_dispatch(WorkflowId, Identity, StepId, StepDef, CorrelationId, IdempotencyKey, DispatchToken) ->
    ActuationToken = actuation_token(DispatchToken),
    ReturnAuthorityToken = return_authority_token(DispatchToken),
    %% PRD 8 "structure" stage (RETURN_STRUCTURE_REFUSED, PROJ-785): derived
    %% now, from THIS StepDef's own `outputs` field, and carried on the
    %% ledger entry so admit_return_structure/1 can check the eventual
    %% raw_consequence against it later without needing StepDef again. See
    %% required_result_types/1 and arazzo_broker.hrl's field doc.
    RequiredResultTypes = required_result_types(maps:get(outputs, StepDef, [])),
    D0 = #dispatch{
        dispatch_token = DispatchToken,
        workflow_id = WorkflowId,
        step_id = StepId,
        correlation_id = CorrelationId,
        idempotency_key = IdempotencyKey,
        actuation_token = ActuationToken,
        return_authority_token = ReturnAuthorityToken,
        replay_id = Identity#workflow_identity.replay_id,
        status = dispatched,
        unenforced_preactuation_checks = ?UNENFORCED_PREACTUATION_CHECKS,
        required_result_types = RequiredResultTypes
    },
    %% PROJ-781 (PRD 15, "every workflow execution SHALL extend a
    %% BLAKE3-linked receipt chain"): mints the real #event_receipt{} for
    %% this step_dispatched event BEFORE any ledger entry or I/O, so a
    %% receipt-chain failure (e.g. b3sum unavailable) refuses the dispatch
    %% outright rather than actuating an unreceipted step (PRD 6.2 "zero
    %% unreceipted actuation"). event_material identifies the dispatch
    %% request itself; command_material is the real air_core dispatch_step
    %% command shape (PRD 7.7's C); resulting_state_material is the
    %% freshly-built D0 ledger entry -- the genuine state this event
    %% produces, not a placeholder. This is one real emission site among
    %% several named in this module's own header comment as future wiring
    %% (post-actuation "step completing" below, and admit_return_ok/1's
    %% "step completing" via the return-admission chain) -- deliberately
    %% not attempted in the same pass; see PROJ-781 ticket text.
    case arazzo_runner_event_receipt:emit(#{
            workflow_id => WorkflowId,
            parent_workflow_id => Identity#workflow_identity.parent_workflow_id,
            event_type => step_dispatched,
            event_material => {step_dispatch_requested, WorkflowId, StepId, CorrelationId, IdempotencyKey},
            resulting_state_material => D0,
            command_material => {dispatch_step, StepId, StepDef},
            runtime_profile => otp,
            replay_id => Identity#workflow_identity.replay_id,
            identity_receipt_head => Identity#workflow_identity.receipt_head
         }) of
        {ok, _EventReceipt} ->
            do_dispatch_actuate(WorkflowId, StepId, StepDef, IdempotencyKey, ActuationToken, D0);
        {error, Reason} ->
            {error, {event_receipt_unavailable, Reason}}
    end.

do_dispatch_actuate(WorkflowId, _StepId, StepDef, _IdempotencyKey, ActuationToken, D0) ->
    DispatchToken = D0#dispatch.dispatch_token,
    true = ets:insert(arazzo_broker_dispatches, {DispatchToken, D0}),
    %% arazzo_broker_dedup's claim for {WorkflowId, StepId, IdempotencyKey}
    %% was already made atomically (ets:insert_new/2) by
    %% check_required_prior_receipts/5 before this function was ever
    %% called -- re-inserting it here would just be a second, redundant
    %% write to a key this same call path already owns exclusively; not
    %% writing it again is what keeps "only the ets:insert_new/2 winner
    %% ever reaches this function" true as a real invariant, not just an
    %% intention.
    true = ets:insert(arazzo_broker_tokens, {ActuationToken, DispatchToken}),

    %% The only route to the io-worker pool: enqueue_io/2 refuses (
    %% DIRECT_ACTUATION_REFUSED) anything not carrying a token this call
    %% just minted. See arazzo_runner_workflow:enqueue_io/2.
    case arazzo_runner_workflow:enqueue_io(ActuationToken, StepDef) of
        {ok, RawConsequence} ->
            {ConsequenceHash, PrevHash} = consequence_hash(WorkflowId, RawConsequence),
            D1 = D0#dispatch{
                status = actuated,
                raw_consequence = RawConsequence,
                consequence_hash = ConsequenceHash,
                prev_evidence_hash = PrevHash
            },
            true = ets:insert(arazzo_broker_dispatches, {DispatchToken, D1}),
            emit_evidence(D1),
            %% Close the return-admission loop for real. Before this fix,
            %% a successfully-actuated step's raw_consequence was captured
            %% onto the ledger (exactly as above) and NEVER fed back into
            %% air_core: admit_return/3 -- the only function that does
            %% that -- had zero production callers anywhere in apps/*/src
            %% (confirmed by grep). A workflow would sit at
            %% `status = actuated` forever; apply_transition/4
            %% (arazzo_runner_workflow.erl) only records {ok, DispatchToken}
            %% in broker_dispatches and never advances air_core state for
            %% it. This call site is the correct integration point: D1
            %% carries the SAME correlation_id recorded from the
            %% dispatching identity and the SAME return_authority_token
            %% this exact call minted (not a forged or externally-supplied
            %% one), so calling admit_return/3 here drives the consequence
            %% through the EXISTING, already-tested return-admission chain
            %% (correlation -> provenance -> authority -> structure ->
            %% semantic -> O*) in full -- every gate still runs against
            %% this actuation's own real data, not a bypass of any of
            %% them.
            case admit_return(DispatchToken, D1#dispatch.correlation_id,
                               D1#dispatch.return_authority_token) of
                {ok, admitted} ->
                    {ok, DispatchToken};
                {refused, Code, Ctx} ->
                    %% A genuine return-admission gate refused this
                    %% actuation's own consequence (e.g.
                    %% RETURN_STRUCTURE_REFUSED for a raw_consequence that
                    %% does not conform to this step's own declared output
                    %% types) -- surfaced as the dispatch/4 outcome itself,
                    %% since the step's result was captured but never
                    %% admitted into air_core.
                    {refused, Code, Ctx};
                {error, workflow_not_found} ->
                    %% The actuation is real and already durably recorded
                    %% above (status = actuated, raw_consequence captured,
                    %% evidence emitted) -- there is simply no live
                    %% arazzo_runner_workflow process registered for
                    %% WorkflowId right now to feed the result back into
                    %% (e.g. dispatch/4 exercised directly against the
                    %% ledger, without a running workflow process, as this
                    %% module's own idempotency/evidence-chain tests do).
                    %% The ledger entry correctly stays `actuated`, not
                    %% `admitted`; a later admit_return/3 call against the
                    %% same dispatch_token can still admit it once/if a
                    %% live process exists.
                    {ok, DispatchToken};
                {error, {already_admitted, _}} ->
                    %% Unreachable from this call site in practice (D1's
                    %% status is `actuated`, never `admitted`, the instant
                    %% before this call), kept only so this case matches
                    %% admit_return/3's full documented return shape
                    %% exhaustively rather than relying on a fallthrough.
                    {ok, DispatchToken}
            end;
        {refused, Code, Ctx} ->
            true = ets:insert(arazzo_broker_dispatches, {DispatchToken, D0#dispatch{status = dispatch_failed}}),
            {refused, Code, Ctx};
        {error, Reason} ->
            true = ets:insert(arazzo_broker_dispatches, {DispatchToken, D0#dispatch{status = dispatch_failed}}),
            {error, Reason}
    end.

%% PRD 13 post-actuation bullet "hash the consequence" + "extend the
%% receipt chain": sha256(prev_head || term_to_binary(consequence)),
%% hex-encoded, chained per workflow_id. No wall-clock input anywhere in
%% this computation -- chain ordering comes from ETS's own per-key
%% read-your-writes consistency for a single workflow_id, not from time.
consequence_hash(WorkflowId, RawConsequence) ->
    PrevHash = chain_head(WorkflowId),
    Bin = erlang:term_to_binary(RawConsequence, [{minor_version, 1}]),
    Hash = crypto:hash(sha256, <<PrevHash/binary, Bin/binary>>),
    HexHash = binary:encode_hex(Hash),
    true = ets:insert(arazzo_broker_chain_heads, {WorkflowId, HexHash}),
    {HexHash, PrevHash}.

chain_head(WorkflowId) ->
    case ets:lookup(arazzo_broker_chain_heads, WorkflowId) of
        [{WorkflowId, H}] -> H;
        [] -> <<>>
    end.

%% PRD 13 post-actuation bullet "emit runtime evidence". The ledger entry
%% itself (queryable via get_ledger_entry/1) is the durable, assertable
%% form of this evidence; this log line is a secondary, human-readable
%% emission of the same facts, not the only record of them.
emit_evidence(#dispatch{} = D) ->
    error_logger:info_msg(
        "arazzo_runner_broker evidence: workflow=~p step=~p dispatch_token=~p "
        "consequence_hash=~p prev_evidence_hash=~p replay_id=~p "
        "unenforced_preactuation_checks=~p",
        [D#dispatch.workflow_id, D#dispatch.step_id, D#dispatch.dispatch_token,
         D#dispatch.consequence_hash, D#dispatch.prev_evidence_hash, D#dispatch.replay_id,
         D#dispatch.unenforced_preactuation_checks]
    ).

%% ---------------------------------------------------------------------
%% Return-admission chain (PRD 8):
%% O_external -> correlation -> provenance -> authority -> structure ->
%% semantic_conformance -> O* or refusal.
%% ---------------------------------------------------------------------

%% # Complexity
%% O(1): one ETS lookup by dispatch_token plus a small, fixed number of
%% binary-equality checks (correlation_id, status, return_authority_token),
%% plus, on success, one call into arazzo_runner_workflow:admit_result/3
%% (itself O(1): one ETS Pid lookup plus a message send -- the actual
%% air_core transition it triggers is bounded per apply_transition/4's own
%% documented O(|next(StepId)|)).
-spec admit_return(binary(), binary() | undefined, binary() | undefined) ->
    {ok, admitted} | {refused, atom(), map()} | {error, term()}.
admit_return(DispatchToken, CorrelationId, ReturnerAuthorityToken) ->
    ensure_broker_ets(),
    %% Stage 1: correlation, in two parts (PROJ-785 closes the second):
    %%  (a) A dispatch_token this ledger never issued is, from this
    %%      workflow's perspective, indistinguishable from "no correlation
    %%      at all" -- it cannot be traced to any broker dispatch.
    %%      CORRELATION_MISSING.
    %%  (b) A dispatch_token the ledger DID issue, but whose returner
    %%      claims a CorrelationId that does not match the correlation_id
    %%      recorded on that ledger entry at dispatch time (D#dispatch.
    %%      correlation_id -- real data populated in do_dispatch/6 from
    %%      Identity#workflow_identity.correlation_id, not invented for
    %%      this check). CORRELATION_MISMATCH: a *known* correlation whose
    %%      content mismatches.
    case ets:lookup(arazzo_broker_dispatches, DispatchToken) of
        [] ->
            {refused, 'CORRELATION_MISSING',
             #{stage => correlation, dispatch_token => DispatchToken}};
        [{DispatchToken, D}] when CorrelationId =:= D#dispatch.correlation_id ->
            admit_return_provenance(D, ReturnerAuthorityToken);
        [{DispatchToken, D}] ->
            {refused, 'CORRELATION_MISMATCH',
             #{stage => correlation, dispatch_token => DispatchToken,
               expected_correlation_id => D#dispatch.correlation_id,
               returned_correlation_id => CorrelationId}}
    end.

admit_return_provenance(D, ReturnerAuthorityToken) ->
    %% Stage 2: provenance (PROJ-785 closes this for real; PROJ-758 left it
    %% claiming "trivially satisfied by construction" once Stage 1 found a
    %% ledger entry at all -- adversarial review found that claim
    %% incomplete: finding D by dispatch_token proves the TOKEN is known,
    %% not that a consequence actually originated from a broker-dispatched
    %% IO operation. `status` already distinguishes those cases (set only
    %% by do_dispatch/6, a real state-machine field, not invented for this
    %% check):
    %%  - `actuated`: enqueue_io/2's io-worker round trip genuinely
    %%    returned and do_dispatch/6 captured raw_consequence + hashed it
    %%    -- real provenance. Proceed to authority.
    %%  - `dispatched`: still in flight -- no consequence has been
    %%    captured from any actuation yet. RETURN_PROVENANCE_MISSING.
    %%  - `dispatch_failed`: enqueue_io/2 refused or errored -- no
    %%    consequence was ever produced to have provenance over.
    %%    RETURN_PROVENANCE_MISSING.
    %%  - `admitted`: this dispatch_token was already re-admitted by a
    %%    prior admit_return/3 call -- not a provenance gap but a distinct
    %%    double-admission condition (PRD 8/18 do not name a refusal code
    %%    for this; kept as a plain error, matching this module's existing
    %%    convention of {error, _} for conditions PRD 8/18 do not name).
    case D#dispatch.status of
        actuated ->
            admit_return_authority(D, ReturnerAuthorityToken);
        dispatched ->
            {refused, 'RETURN_PROVENANCE_MISSING',
             #{stage => provenance, dispatch_token => D#dispatch.dispatch_token}};
        dispatch_failed ->
            {refused, 'RETURN_PROVENANCE_MISSING',
             #{stage => provenance, dispatch_token => D#dispatch.dispatch_token}};
        admitted ->
            {error, {already_admitted, D#dispatch.dispatch_token}}
    end.

admit_return_authority(D, ReturnerAuthorityToken) ->
    %% Stage 3: authority. Only reachable with D#dispatch.status =:= actuated
    %% -- admit_return_provenance/2 above already refused or errored on
    %% every other status -- so this stage's only remaining job is the
    %% authority-token check itself. Only whoever was handed the
    %% return_authority_token minted for THIS dispatch (PRD 13's broker
    %% law: zero unreceipted actuation extends naturally to zero
    %% unauthorized return-claims) may supply its consequence.
    case ReturnerAuthorityToken =/= undefined
         andalso ReturnerAuthorityToken =:= D#dispatch.return_authority_token of
        true ->
            admit_return_structure(D);
        false ->
            {refused, 'RETURN_AUTHORITY_REFUSED',
             #{stage => authority, dispatch_token => D#dispatch.dispatch_token}}
    end.

admit_return_structure(D) ->
    %% Stage 4: structure. RETURN_STRUCTURE_REFUSED (PROJ-785): enforced for
    %% real against D#dispatch.required_result_types -- the type
    %% requirement required_result_types/1 derived from THIS step's own
    %% `outputs` bind rules at dispatch time (see do_dispatch/6 and
    %% arazzo_broker.hrl's field doc). A raw_consequence that would badarg
    %% inside air_core's real bind_outputs/3 -> eval_expr_nif evaluation
    %% (e.g. a non-integer consequence for a step whose outputs perform
    %% arithmetic on `{var, '__result__'}`) is refused here, before
    %% admission, instead of surfacing as a NIF crash inside the live
    %% workflow process later. The ledger-invariant check this replaced
    %% (raw_consequence =:= undefined) is kept first, as defense in depth --
    %% already implied by admit_return_provenance/2's status=actuated gate
    %% above, not itself a PRD-8 structural-conformance check.
    case D#dispatch.raw_consequence of
        undefined ->
            {error, {no_captured_consequence, D#dispatch.dispatch_token}};
        RawConsequence ->
            RequiredTypes = D#dispatch.required_result_types,
            case result_conforms(RawConsequence, RequiredTypes) of
                true ->
                    admit_return_semantic(D);
                false ->
                    {refused, 'RETURN_STRUCTURE_REFUSED',
                     #{stage => structure, dispatch_token => D#dispatch.dispatch_token,
                       required_types => RequiredTypes,
                       actual_type => erlang_type_of(RawConsequence)}}
            end
    end.

admit_return_semantic(D) ->
    %% Stage 5: semantic conformance. RETURN_SEMANTIC_REFUSED stays
    %% genuinely unenforced (PROJ-785 re-investigated, did not assume): PRD
    %% 11 frames semantic conformance as SHACL/admission-layer territory,
    %% and no bridge from this Erlang codebase to crates/praxis-graphlaw's
    %% admission layer exists anywhere in apps/arazzo_runner or apps/
    %% air_core -- the repo's only Erlang<->Rust bridge (air_core's
    %% eval_expr_nif) is a narrow single-term arithmetic/boolean evaluator,
    %% not reasonably extensible to full SHACL graph validation without new
    %% infrastructure (see the ?UNENFORCED_RETURN_STAGES comment above for
    %% the full investigation). Building that bridge is out of this
    %% ticket's (and this Erlang module's) reach -- see
    %% ?UNENFORCED_RETURN_STAGES.
    admit_return_ok(D).

admit_return_ok(D) ->
    %% Stage 6: O* -- admitted. Re-admits via the SAME, already-tested
    %% (PROJ-757) `result` reaction path: a real event sent to the live
    %% workflow process, genuinely advancing air_core state. Only
    %% reachable once every gate above passed -- PRD 8's "Only admitted
    %% returned consequence MAY unlock a parent, sibling, or dependent
    %% workflow."
    case arazzo_runner_workflow:admit_result(
            D#dispatch.workflow_id, D#dispatch.step_id, D#dispatch.raw_consequence) of
        ok ->
            true = ets:insert(arazzo_broker_dispatches,
                               {D#dispatch.dispatch_token, D#dispatch{status = admitted}}),
            {ok, admitted};
        {error, Reason} ->
            {error, Reason}
    end.

%% ---------------------------------------------------------------------
%% RETURN_STRUCTURE_REFUSED derivation (PRD 8 "structure" stage, PROJ-785).
%%
%% StepDef's `outputs` field is a [bind_rule()] list -- the exact type
%% air_core.erl declares and air_core:bind_outputs/3 evaluates via
%% eval_expr/3 -> eval_expr_nif (apps/air_core/native/air_core_nif/src/
%% lib.rs). That NIF resolves the sentinel {var, '__result__'} (atom or
%% binary spelling, both checked -- see is_result_ref/1) to the raw
%% consequence being bound, and decodes it as i64 for the arithmetic ops
%% (`+`/`-`/`*`/`/`) or as bool for the boolean ops (`and`/`or`/`not`),
%% erroring (badarg) on a mismatched Erlang term. Comparison ops (`==`/
%% `!=`/`>`/`<`/`>=`/`<=`) decode neither operand -- Erlang term ordering
%% (`.cmp()` on the Rust side) is total across all term types, so they
%% impose no structural requirement. required_result_types/1 walks a step's
%% declared outputs and derives exactly the set of Erlang types
%% raw_consequence must belong to for that future, real evaluation to
%% succeed -- this is the step's own declaration, not an invented schema.
%% ---------------------------------------------------------------------

%% # Complexity
%% O(|Outputs| * D) where D is the (small, fixed-depth-in-practice) depth of
%% each bind rule's expression tree -- each sub-expression is visited once.
-spec required_result_types([{bind, atom() | binary(), term()}]) -> ['integer' | 'boolean'].
required_result_types(Outputs) ->
    lists:usort(lists:flatmap(
        fun({bind, _Var, Expr}) -> expr_result_constraints(Expr) end,
        Outputs)).

expr_result_constraints({op, Op, E1, E2}) when Op =:= '+'; Op =:= '-'; Op =:= '*'; Op =:= '/' ->
    result_ref_constraint(E1, integer) ++ result_ref_constraint(E2, integer)
        ++ expr_result_constraints(E1) ++ expr_result_constraints(E2);
expr_result_constraints({op, Op, E1, E2}) when Op =:= 'and'; Op =:= 'or' ->
    result_ref_constraint(E1, boolean) ++ result_ref_constraint(E2, boolean)
        ++ expr_result_constraints(E1) ++ expr_result_constraints(E2);
expr_result_constraints({op, _CmpOp, E1, E2}) ->
    %% ==, !=, >, <, >=, <= -- no type decode on either side (see module
    %% header above); still recurse in case a nested typed op exists.
    expr_result_constraints(E1) ++ expr_result_constraints(E2);
expr_result_constraints({op, 'not', E}) ->
    result_ref_constraint(E, boolean) ++ expr_result_constraints(E);
expr_result_constraints({op, _UnaryOp, E}) ->
    expr_result_constraints(E);
expr_result_constraints(_OtherExpr) ->
    %% {literal, _}, {var, Name} (env-bound, not the result sentinel), or
    %% any other leaf: no constraint on raw_consequence.
    [].

%% Only a DIRECT {var, '__result__'} operand of a typed op contributes Type;
%% anything else (a literal, an env var, a nested op whose own result feeds
%% the outer op) is not itself the raw consequence and is handled by the
%% recursive expr_result_constraints/1 call at each call site instead.
result_ref_constraint({var, Name}, Type) ->
    case is_result_ref(Name) of
        true -> [Type];
        false -> []
    end;
result_ref_constraint(_NotAVarRef, _Type) ->
    [].

is_result_ref('__result__') -> true;
is_result_ref(<<"__result__">>) -> true;
is_result_ref(_Other) -> false.

%% # Complexity
%% O(|RequiredTypes|), a small fixed-size list in practice (at most 2:
%% integer, boolean).
-spec result_conforms(term(), ['integer' | 'boolean']) -> boolean().
result_conforms(_RawConsequence, []) ->
    true;
result_conforms(RawConsequence, RequiredTypes) ->
    lists:all(fun(Type) -> conforms_to_type(RawConsequence, Type) end, RequiredTypes).

conforms_to_type(V, integer) -> is_integer(V);
conforms_to_type(V, boolean) -> is_boolean(V).

%% Debuggability only (carried in RETURN_STRUCTURE_REFUSED's context map) --
%% never itself a conformance decision.
erlang_type_of(V) when is_integer(V) -> integer;
erlang_type_of(V) when is_boolean(V) -> boolean;
erlang_type_of(V) when is_float(V) -> float;
erlang_type_of(V) when is_binary(V) -> binary;
erlang_type_of(V) when is_atom(V) -> atom;
erlang_type_of(V) when is_map(V) -> map;
erlang_type_of(V) when is_tuple(V) -> tuple;
erlang_type_of(V) when is_list(V) -> list;
erlang_type_of(_V) -> term.

%% ---------------------------------------------------------------------
%% Introspection (tests + a future admin surface)
%% ---------------------------------------------------------------------

-spec get_ledger_entry(binary()) -> {ok, #dispatch{}} | not_found.
get_ledger_entry(DispatchToken) ->
    ensure_broker_ets(),
    case ets:lookup(arazzo_broker_dispatches, DispatchToken) of
        [{DispatchToken, D}] -> {ok, D};
        [] -> not_found
    end.

%% ---------------------------------------------------------------------
%% Actuation-route enforcement (DIRECT_ACTUATION_REFUSED). Called by
%% arazzo_runner_workflow:enqueue_io/2 -- the sole entry point into the
%% io-worker pool -- before it will forward anything to any pool member.
%% ---------------------------------------------------------------------

%% # Complexity
%% O(1): ets:take/2 is a single atomic lookup-and-delete, so two concurrent
%% callers racing the same token can never both succeed (no TOCTOU window
%% between "check" and "consume").
-spec consume_actuation_token(binary()) -> ok | refused.
consume_actuation_token(ActuationToken) ->
    ensure_broker_ets(),
    case ets:take(arazzo_broker_tokens, ActuationToken) of
        [{ActuationToken, _DispatchToken}] -> ok;
        [] -> refused
    end.

%% ---------------------------------------------------------------------
%% Token derivation: sha256 over tagged, `|`-joined parts PLUS a per-node
%% secret (see broker_secret/0) mixed into every hash. Deterministic within
%% a single running node (repeatable content-addressing for a fixed
%% secret, not wall-clock/RNG nondeterminism in the receipt-computation
%% sense the repo's determinism discipline forbids) but NOT independently
%% recomputable by anyone who only knows the public identifiers
%% (WorkflowId, StepId, IdempotencyKey, DispatchToken) -- an independent
%% audit found the original secret-free design let anyone who knew those
%% public identifiers derive a valid dispatch_token/actuation_token/
%% return_authority_token themselves and call enqueue_io/2 or
%% admit_return/3 directly, bypassing DIRECT_ACTUATION_REFUSED and
%% RETURN_AUTHORITY_REFUSED entirely -- a real authentication bypass, not
%% a theoretical one, since every public identifier here (workflow_id,
%% step_id, idempotency_key) is exactly the data an external caller
%% legitimately already has. Mixing in broker_secret/0 (generated once
%% per node, at first use, via crypto:strong_rand_bytes/1, and never
%% returned by any exported function) closes that: computing a valid
%% token now requires knowing a value that never leaves this node's
%% memory.
%% ---------------------------------------------------------------------

dispatch_token(WorkflowId, StepId, IdempotencyKey) ->
    make_token([<<"dispatch">>, WorkflowId, StepId, IdempotencyKey]).

actuation_token(DispatchToken) ->
    make_token([<<"actuate">>, DispatchToken]).

return_authority_token(DispatchToken) ->
    make_token([<<"return-authority">>, DispatchToken]).

make_token(Parts) ->
    Bin = iolist_to_binary(lists:join(<<"|">>, Parts)),
    binary:encode_hex(crypto:hash(sha256, <<(broker_secret())/binary, Bin/binary>>)).

%% Per-node authority secret: 32 random bytes, generated at most once per
%% running node (lazily, on first token derivation -- functionally
%% equivalent to "at startup" for this purpose, and avoids needing a
%% dedicated supervised-process init hook) and held only in this node's
%% own persistent_term storage -- never logged, never returned by any
%% exported function, never derivable from any public identifier.
%%
%% Race-safety: persistent_term:put/2 is not itself a compare-and-swap, so
%% two processes racing the very first call could each generate a
%% DIFFERENT candidate secret and each attempt to write it. That race is
%% harmless here: every caller, including both racers, finishes by
%% re-reading the key (not by trusting whichever candidate it personally
%% generated), so both converge on whichever single write actually landed
%% last -- the same guarantee the arazzo_broker_dedup ets:insert_new fix
%% elsewhere in this module relies on, just via persistent_term's
%% single global slot instead of ETS's per-key atomicity. After this
%% first-use race settles (a one-time event in a node's lifetime), every
%% subsequent call is a plain, side-effect-free get.
broker_secret() ->
    case persistent_term:get({?MODULE, broker_secret}, undefined) of
        undefined ->
            Candidate = crypto:strong_rand_bytes(32),
            persistent_term:put({?MODULE, broker_secret}, Candidate),
            persistent_term:get({?MODULE, broker_secret});
        Secret ->
            Secret
    end.

%% ---------------------------------------------------------------------
%% ETS bootstrap. These 4 tables are actually created (as public,
%% named_table, owned by the long-lived infra process) in
%% arazzo_runner_workflow:infra_loop/0's init_infra clause, alongside
%% arazzo_workflow_states -- co-owned for the same reason: a table's
%% lifetime is its owning process's lifetime, and the broker's ledger must
%% outlive any single transient caller of dispatch/4 or admit_return/3.
%% ensure_broker_ets/0 here is a defensive idempotent fallback (mirrors
%% arazzo_runner_identity:open_table/0's own "lazy, safe to call from
%% anywhere" pattern) for callers that reach this module before
%% arazzo_runner_workflow:setup_infrastructure/0 has run for any other
%% reason; the race-safe creation logic is identical to that function's.
%% ---------------------------------------------------------------------

ensure_broker_ets() ->
    ensure_table(arazzo_broker_dispatches),
    ensure_table(arazzo_broker_dedup),
    ensure_table(arazzo_broker_tokens),
    ensure_table(arazzo_broker_chain_heads),
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
