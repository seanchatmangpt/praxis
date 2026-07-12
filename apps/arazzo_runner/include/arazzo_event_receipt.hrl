%% PROJ-781 (PRD v26.7.11 15 -- Receipt and Replay, PRD.md:704-716 "Minimum
%% event receipt fields").
%%
%% Distinct from #workflow_identity{} (PRD 7.8, arazzo_runner.hrl) -- that
%% record identifies a workflow INSTANCE once, independent of PID. This
%% record receipts a single workflow EXECUTION EVENT (a step dispatching, a
%% step completing, a reaction firing) as it flows through the runtime, and
%% a new one is minted per event, chained to the previous one via
%% prior_receipt_head -- the Erlang-side realization of "every workflow
%% execution SHALL extend a BLAKE3-linked receipt chain" (PRD.md:702).
%%
%% Also distinct from #dispatch{}'s own sha256-based consequence_hash /
%% prev_evidence_hash chain (arazzo_broker.hrl, PROJ-758): that chain covers
%% only the captured actuation consequence and is explicitly disclaimed
%% there as "not a substitute for... the BLAKE3 canonical-N-Quads receipt
%% discipline that governs crates/praxis-graphlaw specifically". This
%% record IS that BLAKE3 discipline's Erlang-side event-receipt analog --
%% real BLAKE3 (via arazzo_runner_blake3, the b3sum-CLI technique PROJ-756
%% established for this codebase), not sha256, and covering the full event
%% (not just the actuation payload).
%%
%% Field-by-field mapping to PRD.md:704-716's 10-field list:
-record(event_receipt, {
    %% "workflow semantic ID" -- the PID-independent workflow identity (PRD
    %% 7.8's own #workflow_identity.workflow_id).
    workflow_semantic_id   :: binary(),
    %% "parent semantic ID" -- undefined for a root workflow, same
    %% always-present-key convention as #workflow_identity.parent_workflow_id.
    parent_semantic_id     :: binary() | undefined,
    %% "event type" -- e.g. step_dispatched, step_completed, step_failed,
    %% reaction_fired. An atom tag, not a free-form string, so callers and
    %% a future replay verifier (PROJ-782) can pattern-match it.
    event_type              :: atom(),
    %% "event digest" -- BLAKE3 hex digest of the canonical byte
    %% representation of the event's own identifying content (what
    %% happened), distinct from command_digest (what was dispatched as a
    %% consequence) and resulting_state_digest (what the system became).
    event_digest             :: binary(),
    %% "prior receipt head" -- the receipt_head of the immediately
    %% preceding #event_receipt{} for this workflow_semantic_id, or (for
    %% the first event of a workflow) the genesis value carried on
    %% #workflow_identity.receipt_head -- the same real, already-required
    %% Erlang-side field PROJ-757/758 established, not a new root scheme.
    prior_receipt_head       :: binary(),
    %% "resulting state digest" -- BLAKE3 hex digest of the canonical byte
    %% representation of the runtime state produced as a result of this
    %% event (e.g. the freshly-minted #dispatch{} ledger entry for a
    %% step_dispatched event).
    resulting_state_digest   :: binary(),
    %% "command digest" -- BLAKE3 hex digest of the canonical byte
    %% representation of the command this event dispatched or otherwise
    %% carried (e.g. the {dispatch_step, StepId, StepDef} command air_core's
    %% delta_AIR: (S,E)->(S',C) produced, PRD 7.7).
    command_digest           :: binary(),
    %% "runtime profile" -- which runtime executed this event: `otp` (PRD
    %% 7.8 Layer 8) or `atomvm` (PRD 7.9 Layer 9). A real, structural fact
    %% (which module emitted the receipt), never guessed or defaulted.
    runtime_profile          :: atom(),
    %% "timestamp or declared logical clock" -- this codebase's Chatman
    %% Constant discipline is logic ticks, never wall time (see
    %% docs/CHATMAN_EQUATION.md and the repo-wide no-wall-clock-in-receipt-
    %% paths invariant): a monotonically increasing, per-workflow event
    %% sequence number, not os:timestamp()/erlang:system_time().
    logical_clock            :: non_neg_integer(),
    %% "replay ID" -- the same #workflow_identity.replay_id already
    %% required and carried by #dispatch{} (PROJ-757/758).
    replay_id                :: binary(),
    %% Derived, NOT one of the PRD's declared 10 fields: the BLAKE3 hex
    %% digest binding all 10 fields above together (chain material order
    %% fixed by arazzo_runner_event_receipt:chain_material/1), computed
    %% once at construction time. This IS the value the PRD calls "receipt
    %% head" elsewhere (PRD.md:723 "verify receipt-head equivalence") --
    %% what the NEXT event's prior_receipt_head reads back. Named
    %% `receipt_head` to match #workflow_identity.receipt_head's existing
    %% Erlang-side field name, not a third, differently-spelled scheme.
    receipt_head              :: binary()
}).
