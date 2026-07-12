%% PROJ-758 (PRD v26.7.11 section 13 -- Broker Requirements; section 8 --
%% Independent Process Cells / return-admission chain).
%%
%% One ledger entry per broker-issued dispatch, keyed by dispatch_token in
%% the arazzo_broker_dispatches ETS table (owned by the same long-lived
%% infra process that owns arazzo_workflow_states -- see
%% arazzo_runner_workflow.erl's infra_loop/0 -- so the ledger survives the
%% death of whichever caller happened to invoke dispatch/4).
%%
%% Field-by-field mapping to PRD 13's pre-actuation bullet list and PRD 8's
%% return-admission chain is documented in arazzo_runner_broker.erl next to
%% the functions that populate/consume each field. Not every PRD-13 bullet
%% has a field here -- see ?UNENFORCED_PREACTUATION_CHECKS in that module
%% for the ones this ticket does not independently enforce (named gaps,
%% PROJ-785 and later).
-record(dispatch, {
    dispatch_token                 :: binary(),
    workflow_id                    :: binary(),
    step_id                        :: binary(),
    %% PRD 13 pre-actuation bullet "correlation ID" -- the enforced one
    %% (CORRELATION_MISSING is a real, tested refusal; see dispatch/4).
    %% PROJ-785: this same field is also the ground truth admit_return/3's
    %% Stage 1 checks a returner's claimed CorrelationId against
    %% (CORRELATION_MISMATCH on mismatch).
    correlation_id                 :: binary(),
    %% PRD 13 pre-actuation bullet "idempotency key" -- enforced as real
    %% deduplication (a repeated dispatch for the same
    %% {workflow_id, step_id, idempotency_key} returns the existing
    %% dispatch_token instead of re-actuating), not as a refusal.
    idempotency_key                :: binary(),
    %% One-shot ticket minted at dispatch time and required by
    %% arazzo_runner_workflow:enqueue_io/2 to reach the io-worker pool at
    %% all -- the concrete mechanism behind DIRECT_ACTUATION_REFUSED
    %% ("the broker SHALL be the only actuation route", PRD 13). Consumed
    %% (ets:take/2, atomic) on first use; a second use or a bogus/unissued
    %% token is refused.
    actuation_token                :: binary(),
    %% One-shot proof minted at dispatch time and required by
    %% admit_return/3's authority stage (PRD 8's return-admission chain) --
    %% the concrete mechanism behind RETURN_AUTHORITY_REFUSED: only a
    %% caller presenting the token minted for THIS dispatch may supply its
    %% returned consequence.
    return_authority_token         :: binary(),
    %% PRD 7.8 identity field, carried through so post-actuation evidence
    %% and the return-admission chain both "preserve replay identity"
    %% (PRD 13's post-actuation bullet) without re-deriving it.
    replay_id                      :: binary(),
    %% PROJ-785: also the real data source for admit_return/3's Stage 2
    %% provenance check -- only `actuated` proves a consequence genuinely
    %% originated from this broker's own do_dispatch/6 actuation; `dispatched`
    %% or `dispatch_failed` refuse RETURN_PROVENANCE_MISSING.
    status = dispatched            :: dispatched | actuated | dispatch_failed | admitted,
    %% Captured post-actuation (PRD 13: "capture the real consequence").
    %% undefined until status moves to `actuated`.
    raw_consequence = undefined    :: term(),
    %% PRD 13 post-actuation bullet "hash the consequence" -- sha256 of
    %% (prev_evidence_hash || term_to_binary(raw_consequence)), hex-encoded.
    %% Local Erlang-side evidence hash for this broker's own ledger; not a
    %% substitute for and not asserting compliance with the BLAKE3
    %% canonical-N-Quads receipt discipline that governs
    %% crates/praxis-graphlaw specifically (out of this ticket's scope,
    %% touches no Rust code).
    consequence_hash = undefined   :: binary() | undefined,
    %% PRD 13 post-actuation bullet "extend the receipt chain" -- the prior
    %% chain head for this workflow_id at the moment this hash was
    %% computed (<<>> for the first dispatch of a given workflow).
    prev_evidence_hash = <<>>      :: binary(),
    %% Names, per PRD 13's own bullet list, exactly which pre-actuation
    %% checks this dispatch did NOT independently verify (see
    %% ?UNENFORCED_PREACTUATION_CHECKS) -- attached to every ledger entry
    %% so the gap is receipt-visible per-dispatch, not just a code comment.
    unenforced_preactuation_checks = [] :: [atom()],
    %% PROJ-785: PRD 8's return-admission "structure" stage
    %% (RETURN_STRUCTURE_REFUSED). Derived at dispatch time (do_dispatch/6)
    %% from the SAME StepDef `outputs` bind-rule list air_core:bind_outputs/3
    %% will evaluate against this dispatch's eventual raw_consequence -- not
    %% an invented schema field. Each element is an Erlang type atom
    %% (`integer` | `boolean`) that raw_consequence must satisfy for that
    %% future bind_outputs/3 evaluation to not fail on a type-decode error
    %% inside air_core's eval_expr_nif (arithmetic ops decode::<i64>,
    %% and/or/not decode::<bool>()) -- see
    %% arazzo_runner_broker:required_result_types/1 and
    %% admit_return_structure/1. Empty list means the step's own outputs
    %% never reference `__result__` under a type-coercing operator, so any
    %% raw_consequence structurally conforms (vacuously) -- the honest
    %% default for the majority of steps in this codebase's corpus, whose
    %% outputs are literal-only.
    required_result_types = [] :: ['integer' | 'boolean']
}).
