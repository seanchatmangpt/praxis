//! Family F15 -- "AIR Single Semantic Core (shared OTP/AtomVM transition
//! machine)" (atlas ticket V12-015).
//!
//! Survey verdict: **MIXED**. Per `.claude/rules/no-overclaiming.md`, this
//! doc comment states plainly which parts are real (verified this session
//! by reading the dependency source and by running `just erlang-test`) and
//! which parts are honest disclosed gaps -- no part is dressed up to look
//! more complete than it is.
//!
//! ## Why this module does not "thinly wrap" `air_core.erl` in Rust
//!
//! The survey asked for a thin wrap of the ALREADY_BUILT semantic core.
//! That is not mechanically possible for this family, and this is a
//! **BLOCKED** finding, not a silently-skipped one:
//!
//! - The real, single-source AIR transition machine --
//!   `context()`/`event()`/`command()`/`transition/2`, the bitmask AND/join
//!   readiness logic (PROJ-756) -- is pure Erlang:
//!   `apps/air_core/src/air_core.erl:1-339` (read in full this session).
//! - The one Rust piece of `air_core`, `apps/air_core/native/air_core_nif/
//!   src/lib.rs`, is a workspace member
//!   (`/Users/sac/praxis/Cargo.toml:242`, confirmed by grep this session)
//!   but its `Cargo.toml` declares `crate-type = ["cdylib"]` only -- no
//!   `rlib`/`lib` target, so no other Rust crate in this workspace can
//!   `use air_core_nif::...` against it; `cdylib` produces a dynamic
//!   library for foreign-language loading, not an rlib for Rust-to-Rust
//!   linking. Its sole function, `eval_expr_nif`, additionally takes
//!   `rustler::Term<'a>` values bound to a live `rustler::Env<'a>` that
//!   only exists inside a loaded BEAM NIF call -- it cannot be invoked from
//!   ordinary Rust code outside that context.
//! - Building a real bridge (Erlang distribution protocol client, a port,
//!   or a second reversed NIF) would be genuinely new infrastructure, not
//!   a thin wrap -- and per `arazzo_runner_broker.erl:141-165`'s own
//!   documented reasoning for the same class of problem
//!   (`RETURN_SEMANTIC_REFUSED`'s disclosed gap), inventing a narrow,
//!   purpose-built bridge for one family risks becoming exactly the kind
//!   of second/third reinterpretation of AIR semantics the family's own
//!   "single semantic core" invariant exists to prevent. Out of this
//!   pass's reasonable-effort scope; tracked under V12-015, not faked.
//!
//! ## What IS real in this module
//!
//! - **[`AirLifecycleState`]**: the family's explicit 8-state lifecycle +
//!   `REFUSED`, hand-written here (naming/ordering only -- no transition
//!   logic). Cross-checked against `f15_air_transition_core_generated.rs`'s
//!   `LIFECYCLE_STATE_CATALOG` (ggen-generated from
//!   `packs/f15-air-core-pack/ontology.ttl`) by
//!   [`tests::lifecycle_catalog_matches_enum_variants`] below, so the two
//!   cannot silently drift.
//! - **[`AIRTransitionRefused`]**: a typed refusal taxonomy. Every variant
//!   mirrors a refusal atom this session actually found returned by
//!   `apps/arazzo_runner/src/arazzo_runner_broker.erl` (exact file:line
//!   citations in `f15_air_transition_core_generated.rs::REFUSAL_CATALOG`,
//!   cross-checked by [`tests::refusal_catalog_matches_enum_variants`]) --
//!   this is an honest disclosure that the refusal machinery lives one
//!   layer above `air_core:transition/2` itself (which is TOTAL and never
//!   refuses, confirmed by reading its full source this session), not a
//!   claim that this enum enforces anything at runtime today.
//!   `ReturnSemanticRefused` and `NotYetImplemented` are marked
//!   `DISCLOSED_GAP` in the generated catalog, matching the broker's own
//!   `?UNENFORCED_RETURN_STAGES` disclosure -- not invented here.
//! - **[`AirEvent`] / [`AirCommand`]**: minimal Rust mirrors of
//!   `air_core.erl`'s `event()` (`{step_completed, StepId, Result}` /
//!   `{step_failed, StepId, Reason}`) and `command()`
//!   (`{dispatch_step, StepId, StepDef}`) types (`air_core.erl:60-72`).
//!   `Result`/`Reason`/`StepDef` are Erlang `term()`/`map()` with no closed
//!   Rust shape; this module represents them as opaque `String` debug
//!   text, not a faithful term encoding -- disclosed, not hidden. These
//!   types carry no transition behavior; they exist purely as a citable,
//!   compiling data shape for future cross-language tooling.
//! - **[`TransitionReceiptFields`]**: a field-for-field mirror of Erlang's
//!   `#event_receipt{}` (`apps/arazzo_runner/include/
//!   arazzo_event_receipt.hrl`, all 10 PRD-declared fields plus the
//!   derived `receipt_head`), `serde`-round-trippable (both `serde` and
//!   `serde_json` are already crate dependencies, added for F07; reused
//!   here, not re-added). It deliberately does **not** attempt to
//!   recompute `receipt_head`: doing so byte-identically would require
//!   reimplementing Erlang's External Term Format encoder
//!   (`erlang:term_to_binary/2`), which is out of this pass's scope --
//!   disclosed as a non-goal, not silently approximated with a different
//!   hash that would look plausible but verify nothing real.
//! - **[`f15_air_transition_core_generated.rs`]**: real `ggen sync` output
//!   (not hand-typed) from `packs/f15-air-core-pack/ontology.ttl`'s
//!   `air:LifecycleState`, `air:RefusalKind`, and `air:ProvEntity`
//!   individuals -- the L6 PROV data-model catalog
//!   (`AIRProgram -> WorkflowState -> ReadySet -> ExpressionResult ->
//!   CriterionResult -> RouteAction -> BoundOutputs -> TransitionReceipt`)
//!   plus the two catalogs above. Generated twice this session from an
//!   isolated scratch ggen project (not registered in the shared root
//!   `ggen.toml`, zero blast radius on other families' packs mid-wave) and
//!   confirmed byte-identical via `diff`.
//!
//! ## Exit-evidence markers (honest, hand-authored claim ceiling)
//!
//! See [`EXIT_EVIDENCE_MARKERS`]. None of the three named markers
//! (`AIR_SINGLE_SEMANTIC_CORE_PROVEN`, `DETERMINISTIC_READY_SET`,
//! `AIR_TRANSITION_REPLAYABLE`) previously existed anywhere in the repo
//! (grepped this session -- zero hits before this pass). This module
//! records them as data (name + status + evidence citation), not as a
//! claim this Rust crate itself proves them: the real evidence, where it
//! exists, is Erlang-side (`just erlang-test`, this session: 55 tests, 0
//! failures, including
//! `apps/arazzo_runner/test/arazzo_runner_atomvm_differential_test.erl`'s
//! cross-shell digest/refusal-class/command-trail comparison over one
//! corpus). These are deliberately hand-authored judgments, not
//! ggen-generated data, since "is this claim earned" is not a mechanical
//! projection.
//!
//! ## What is HAND_WRITE_REQUIRED and NOT done here (disclosed, not faked)
//!
//! - **A literal Rust<->Erlang reuse bridge** for the transition core (see
//!   above) -- genuinely new infrastructure, not attempted this pass.
//! - **The L7 chaos/recovery test suite** (duplicate events, process/engine
//!   restart, stale results) the family survey names: the underlying
//!   idempotency/dedup ETS machinery already exists in
//!   `arazzo_runner_broker.erl` and restart-survival in
//!   `arazzo_runner_workflow.erl` (real, per the survey's own citations),
//!   but no dedicated test drives those exact chaos scenarios, and such a
//!   test would be Erlang (`apps/arazzo_runner/test/`), not something this
//!   Rust crate can contain. Out of scope for this module; tracked under
//!   V12-015, not simulated here with a fake pass.
//! - **A dedicated `TERMINAL_CHECKED` predicate**: disclosed in
//!   `f15_air_transition_core_generated.rs::LIFECYCLE_STATE_CATALOG` as a
//!   gap (no such function exists in `air_core.erl` or either shell,
//!   grepped this session).
//!
//! Survey-cited paths for F15 (re-verified by direct read this session,
//! not merely re-cited):
//! - /Users/sac/praxis/apps/air_core/src/air_core.erl
//! - /Users/sac/praxis/apps/air_core/native/air_core_nif/src/lib.rs
//! - /Users/sac/praxis/apps/air_core/native/air_core_nif/Cargo.toml
//! - /Users/sac/praxis/apps/arazzo_runner/src/arazzo_runner_workflow.erl
//! - /Users/sac/praxis/apps/arazzo_runner/src/arazzo_runner_event_receipt.erl
//! - /Users/sac/praxis/apps/arazzo_runner/include/arazzo_event_receipt.hrl
//! - /Users/sac/praxis/apps/arazzo_runner/src/arazzo_runner_broker.erl
//! - /Users/sac/praxis/apps/arazzo_atomvm/src/arazzo_atomvm_workflow.erl
//! - /Users/sac/praxis/apps/atomvm_runner/src/atomvm_runner.erl
//! - /Users/sac/praxis/apps/arazzo_runner/test/arazzo_runner_atomvm_differential_test.erl
//! - /Users/sac/praxis/apps/arazzo_atomvm/PROOF_OF_EQUIVALENCE.md
//! - /Users/sac/praxis/justfile
//! - /Users/sac/praxis/crates/wasm4pm-arazzo/src/air.rs (a separate,
//!   unrelated static-structural "AIR" compiled to WASM bytecode -- no
//!   relation to `air_core.erl`'s runtime semantics; flagged, not used
//!   here, per the survey's own caution against a third reinterpretation).
//! - /Users/sac/praxis/packs/f15-air-core-pack/ (this wiring pass, new)

include!("f15_air_transition_core_generated.rs");

/// The F15 8-state AIR transition lifecycle, plus the parallel `REFUSED`
/// terminal. Naming/ordering only -- carries no transition behavior; the
/// real state advancement is `air_core:transition/2`
/// (`apps/air_core/src/air_core.erl:191-195`), which this module does not
/// reimplement (see the module doc comment's "why this module does not
/// thinly wrap" section).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AirLifecycleState {
    ProgramLoaded,
    ReadyComputed,
    ExpressionsEvaluated,
    CriteriaEvaluated,
    ActionApplied,
    OutputsBound,
    TerminalChecked,
    Receipted,
    /// Parallel terminal, reachable from any of the 8 named rungs when the
    /// layer above `air_core` (see [`AIRTransitionRefused`]) refuses.
    Refused,
}

impl AirLifecycleState {
    /// The 8 named lifecycle states in chain order (excludes `Refused`,
    /// which is a parallel terminal, not a rung on this ladder).
    pub const CHAIN_ORDER: [AirLifecycleState; 8] = [
        AirLifecycleState::ProgramLoaded,
        AirLifecycleState::ReadyComputed,
        AirLifecycleState::ExpressionsEvaluated,
        AirLifecycleState::CriteriaEvaluated,
        AirLifecycleState::ActionApplied,
        AirLifecycleState::OutputsBound,
        AirLifecycleState::TerminalChecked,
        AirLifecycleState::Receipted,
    ];

    /// The `rdfs:label` name this variant corresponds to in
    /// `f15_air_transition_core_generated.rs::LIFECYCLE_STATE_CATALOG`.
    pub const fn catalog_name(self) -> &'static str {
        match self {
            AirLifecycleState::ProgramLoaded => "PROGRAM_LOADED",
            AirLifecycleState::ReadyComputed => "READY_COMPUTED",
            AirLifecycleState::ExpressionsEvaluated => "EXPRESSIONS_EVALUATED",
            AirLifecycleState::CriteriaEvaluated => "CRITERIA_EVALUATED",
            AirLifecycleState::ActionApplied => "ACTION_APPLIED",
            AirLifecycleState::OutputsBound => "OUTPUTS_BOUND",
            AirLifecycleState::TerminalChecked => "TERMINAL_CHECKED",
            AirLifecycleState::Receipted => "RECEIPTED",
            AirLifecycleState::Refused => "REFUSED",
        }
    }
}

/// F15's typed refusal taxonomy. Every variant names a concrete offender
/// in its payload and mirrors a real refusal atom this session found
/// actually returned by `arazzo_runner_broker.erl` -- see
/// `f15_air_transition_core_generated.rs::REFUSAL_CATALOG` for the
/// ontology-sourced file:line citation of each, cross-checked against this
/// enum by [`tests::refusal_catalog_matches_enum_variants`] below.
///
/// This enum is a **typed mirror for citation/interop purposes**, not a
/// runtime refusal path this Rust crate itself enforces today -- the real
/// enforcement is the Erlang code cited per-variant.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AIRTransitionRefused {
    /// Mirrors `'CORRELATION_MISSING'` (dispatch stage,
    /// `arazzo_runner_broker.erl:193`: no correlation_id on
    /// `#workflow_identity{}`; or the correlation stage,
    /// `arazzo_runner_broker.erl:483`: an unknown `dispatch_token` at
    /// return time). `step_id`/`dispatch_token` are `None` for whichever
    /// call site did not supply them.
    #[error(
        "correlation missing: workflow_id={workflow_id} step_id={step_id:?} \
         dispatch_token={dispatch_token:?}"
    )]
    CorrelationMissing {
        workflow_id: String,
        step_id: Option<String>,
        dispatch_token: Option<String>,
    },
    /// Mirrors `'CORRELATION_MISMATCH'`
    /// (`arazzo_runner_broker.erl:487-491`): a known `dispatch_token` whose
    /// claimed correlation ID does not match the ledger's
    /// `D#dispatch.correlation_id`.
    #[error(
        "correlation mismatch for dispatch_token={dispatch_token}: expected \
         {expected_correlation_id}, got {returned_correlation_id}"
    )]
    CorrelationMismatch {
        dispatch_token: String,
        expected_correlation_id: String,
        returned_correlation_id: String,
    },
    /// Mirrors `'BROKER_RECEIPT_PRECONDITION_MISSING'`
    /// (`arazzo_runner_broker.erl:212-217`): `#workflow_identity.receipt_head`
    /// is not a non-empty binary, so no prior receipt chain is attached.
    #[error("broker receipt precondition missing: workflow_id={workflow_id} step_id={step_id}")]
    BrokerReceiptPreconditionMissing {
        workflow_id: String,
        step_id: String,
    },
    /// Mirrors `'RETURN_PROVENANCE_MISSING'`
    /// (`arazzo_runner_broker.erl:516-524`): `D#dispatch.status` is
    /// `dispatched` or `dispatch_failed`, so no consequence was ever
    /// captured for this `dispatch_token` to have provenance over.
    #[error("return provenance missing for dispatch_token={dispatch_token} (status={status})")]
    ReturnProvenanceMissing {
        dispatch_token: String,
        status: String,
    },
    /// Mirrors `'RETURN_AUTHORITY_REFUSED'`
    /// (`arazzo_runner_broker.erl:529-544`): the returner's authority token
    /// does not match `D#dispatch.return_authority_token`.
    #[error("return authority refused for dispatch_token={dispatch_token}")]
    ReturnAuthorityRefused { dispatch_token: String },
    /// Mirrors `'RETURN_STRUCTURE_REFUSED'`
    /// (`arazzo_runner_broker.erl:546-574`): `raw_consequence` fails the
    /// type requirement `required_result_types/1` derived from the step's
    /// own `outputs` bind rules.
    #[error(
        "return structure refused for dispatch_token={dispatch_token}: required \
         {required_types}, got {actual_type}"
    )]
    ReturnStructureRefused {
        dispatch_token: String,
        required_types: String,
        actual_type: String,
    },
    /// Mirrors `return_semantic_refused`
    /// (`arazzo_runner_broker.erl:141-165`, `?UNENFORCED_RETURN_STAGES`):
    /// SHACL/admission-layer semantic conformance is a **disclosed,
    /// unenforced gap** in the Erlang codebase itself -- this variant
    /// exists so a caller can name that gap explicitly rather than the
    /// stage silently passing.
    #[error("return semantic refused for dispatch_token={dispatch_token} (disclosed gap: no bridge from this Erlang codebase to the SHACL admission layer)")]
    ReturnSemanticRefused { dispatch_token: String },
    /// Mirrors `'DIRECT_ACTUATION_REFUSED'`
    /// (`arazzo_runner_broker.erl:344,708-722`, enforced in
    /// `arazzo_runner_workflow:enqueue_io/2`): an io-worker attempted
    /// actuation without a valid, broker-minted actuation token.
    #[error("direct actuation refused: workflow_id={workflow_id} step_id={step_id}")]
    DirectActuationRefused {
        workflow_id: String,
        step_id: String,
    },
    /// Mirrors `'OTP_ATOMVM_SEMANTIC_DRIFT'`
    /// (`arazzo_runner_atomvm_differential_test.erl:511-546`): the OTP and
    /// AtomVM shells produced a different value along `dimension`
    /// (`state_digest` / `result_digest` / `refusal_class` /
    /// `command_trail`) for the identical ordered event corpus -- this is
    /// a **differential-test comparator finding**, not a live runtime
    /// refusal either shell emits on its own.
    #[error("OTP/AtomVM semantic drift on {dimension}: otp={otp_value:?} atomvm={atomvm_value:?}")]
    OtpAtomvmSemanticDrift {
        dimension: String,
        otp_value: String,
        atomvm_value: String,
    },
    /// A pipeline stage or bridge this family's own diagram requires (the
    /// Rust<->Erlang core reuse bridge, or a dedicated `TERMINAL_CHECKED`
    /// predicate) that is genuinely `HAND_WRITE_REQUIRED` and not yet
    /// built. Fails loud with the gap named, instead of faking success.
    #[error("not yet implemented: {gap}")]
    NotYetImplemented { gap: String },
}

/// Minimal Rust mirror of `air_core.erl`'s `event()` type
/// (`air_core.erl:60-61`). `result`/`reason` are opaque debug text, not a
/// faithful Erlang `term()` encoding -- see the module doc comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AirEvent {
    StepCompleted { step_id: String, result: String },
    StepFailed { step_id: String, reason: String },
}

/// Minimal Rust mirror of `air_core.erl`'s `command()` type
/// (`air_core.erl:72`, the only concrete command shape today).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirDispatchStepCommand {
    pub step_id: String,
    /// Opaque debug text of the Erlang `StepDef` map -- see the module doc
    /// comment.
    pub step_def: String,
}

/// Field-for-field mirror of Erlang's `#event_receipt{}`
/// (`apps/arazzo_runner/include/arazzo_event_receipt.hrl`), in the record's
/// own declaration order. `serde`-round-trippable for cross-language JSON
/// interop tooling. Does **not** recompute `receipt_head` -- see the
/// module doc comment for why that is a disclosed non-goal, not a silent
/// approximation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TransitionReceiptFields {
    pub workflow_semantic_id: String,
    pub parent_semantic_id: Option<String>,
    pub event_type: String,
    pub event_digest: String,
    pub prior_receipt_head: String,
    pub resulting_state_digest: String,
    pub command_digest: String,
    pub runtime_profile: String,
    pub logical_clock: u64,
    pub replay_id: String,
    /// Derived field, not one of the 10 PRD-declared fields (see the
    /// `.hrl`'s own comment) -- the value the Erlang side calls "receipt
    /// head".
    pub receipt_head: String,
}

/// One named v26.7.12 exit-evidence marker for F15. Hand-authored, not
/// ggen-generated (see the module doc comment for why). `status` is an
/// honest claim ceiling, not a pass/fail badge this crate computes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitEvidenceMarker {
    pub name: &'static str,
    /// `PROVEN_ELSEWHERE` (real evidence exists, but Erlang-side, not
    /// produced by this Rust crate), `PARTIAL` (some real evidence, named
    /// gaps remain), or `NOT_CLAIMED` (no evidence exists for this yet).
    pub status: &'static str,
    pub evidence: &'static str,
}

/// The three F15 exit-evidence markers named by the v26.7.12 atlas. None
/// existed anywhere in the repo before this pass (grepped this session).
pub const EXIT_EVIDENCE_MARKERS: &[ExitEvidenceMarker] = &[
    ExitEvidenceMarker {
        name: "AIR_SINGLE_SEMANTIC_CORE_PROVEN",
        status: "PARTIAL",
        evidence: "air_core:transition/2 is the sole transition function both shells call \
                   (grepped: no second transition implementation anywhere in apps/). But \
                   'proven' would require the Lean formalization PROOF_OF_EQUIVALENCE.md's own \
                   banner names as still outstanding (PROJ-769) -- today's evidence is the \
                   differential test corpus (one corpus, not a general proof), not a \
                   machine-checked equivalence proof.",
    },
    ExitEvidenceMarker {
        name: "DETERMINISTIC_READY_SET",
        status: "PARTIAL",
        evidence: "newly_ready_successors/5 (air_core.erl:259-277) computes readiness from \
                   bitmask operations and a map fold over a step's own literal `next` list \
                   (order-preserving over NextSteps, not iterated as an unordered set) -- no \
                   HashMap-order-dependent step is visible in the read source. Not \
                   independently stress-tested this session across many random step-graph \
                   shapes; the differential test's fixed 3-corpus repeat-run check is the only \
                   direct determinism evidence found.",
    },
    ExitEvidenceMarker {
        name: "AIR_TRANSITION_REPLAYABLE",
        status: "PROVEN_ELSEWHERE (Erlang-side; not produced by this Rust crate)",
        evidence: "arazzo_runner_atomvm_differential_test.erl drives a shared ordered corpus \
                   through both shells and asserts state digest, result digest, refusal class, \
                   and command sequence match; PROOF_OF_EQUIVALENCE.md's own banner records 5 \
                   consecutive full-suite runs byte-identical. Verified fresh this session: \
                   `just erlang-test` -> 55 tests, 0 failures.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Cross-checks `AirLifecycleState`'s 9 variants (8 chain states +
    /// `Refused`) against `LIFECYCLE_STATE_CATALOG`'s names by set
    /// membership, so the hand-written enum and the ggen-generated catalog
    /// cannot silently drift apart.
    #[test]
    fn lifecycle_catalog_matches_enum_variants() {
        let enum_names: BTreeSet<&str> = AirLifecycleState::CHAIN_ORDER
            .iter()
            .copied()
            .map(AirLifecycleState::catalog_name)
            .chain(std::iter::once(AirLifecycleState::Refused.catalog_name()))
            .collect();
        let catalog_names: BTreeSet<&str> =
            LIFECYCLE_STATE_CATALOG.iter().map(|e| e.name).collect();
        assert_eq!(enum_names, catalog_names);
        assert_eq!(enum_names.len(), 9, "8 chain states + REFUSED");
    }

    /// The 8-state chain order in `CHAIN_ORDER` matches the ontology's own
    /// `state_order` field exactly (0..7, in `CHAIN_ORDER`'s array order).
    #[test]
    fn chain_order_matches_catalog_state_order() {
        for (i, state) in AirLifecycleState::CHAIN_ORDER.iter().enumerate() {
            let entry = LIFECYCLE_STATE_CATALOG
                .iter()
                .find(|e| e.name == state.catalog_name())
                .expect("catalog entry must exist (checked by the prior test)");
            assert_eq!(
                entry.state_order,
                Some(i as u8),
                "state={}",
                state.catalog_name()
            );
        }
    }

    /// Cross-checks `AIRTransitionRefused`'s 10 variants against
    /// `REFUSAL_CATALOG`'s names by set membership.
    #[test]
    fn refusal_catalog_matches_enum_variants() {
        let enum_names: BTreeSet<&str> = [
            "CorrelationMissing",
            "CorrelationMismatch",
            "BrokerReceiptPreconditionMissing",
            "ReturnProvenanceMissing",
            "ReturnAuthorityRefused",
            "ReturnStructureRefused",
            "ReturnSemanticRefused",
            "DirectActuationRefused",
            "OtpAtomvmSemanticDrift",
            "NotYetImplemented",
        ]
        .into_iter()
        .collect();
        let catalog_names: BTreeSet<&str> = REFUSAL_CATALOG.iter().map(|e| e.name).collect();
        assert_eq!(enum_names, catalog_names);
        assert_eq!(enum_names.len(), 10);
    }

    /// Every `DISCLOSED_GAP`-status catalog entry corresponds to a variant
    /// this module's own doc comment discloses as a gap too (not silently
    /// upgraded to look enforced).
    #[test]
    fn disclosed_gaps_are_the_expected_two() {
        let disclosed: BTreeSet<&str> = REFUSAL_CATALOG
            .iter()
            .filter(|e| e.status == "DISCLOSED_GAP")
            .map(|e| e.name)
            .collect();
        let expected: BTreeSet<&str> = ["ReturnSemanticRefused", "NotYetImplemented"]
            .into_iter()
            .collect();
        assert_eq!(disclosed, expected);
    }

    /// The PROV chain is well-formed: 8 entries, contiguous order 0..7,
    /// and each non-root entry's `derived_from` names its immediate
    /// predecessor by label (a real `prov:wasDerivedFrom` linkage check,
    /// not just a length check).
    #[test]
    fn prov_chain_is_well_formed() {
        assert_eq!(PROV_CHAIN.len(), 8);
        for (i, entry) in PROV_CHAIN.iter().enumerate() {
            assert_eq!(entry.prov_order, i as u8);
            if i == 0 {
                assert_eq!(entry.derived_from, "");
            } else {
                assert_eq!(entry.derived_from, PROV_CHAIN[i - 1].name);
            }
        }
        assert_eq!(PROV_CHAIN[0].name, "AIRProgram");
        assert_eq!(PROV_CHAIN[7].name, "TransitionReceipt");
    }

    /// `TransitionReceiptFields` round-trips through JSON (the interop
    /// shape it exists for), using field values structurally shaped like
    /// what `arazzo_runner_event_receipt:receipt_to_map/1` produces (hex
    /// BLAKE3 digests, an `otp`/`atomvm` runtime_profile tag) -- not a
    /// verification that these are real digests from a live Erlang run.
    #[test]
    fn transition_receipt_fields_round_trips_through_json() {
        let fields = TransitionReceiptFields {
            workflow_semantic_id: "wf-1".to_string(),
            parent_semantic_id: None,
            event_type: "step_completed".to_string(),
            event_digest: "a".repeat(64),
            prior_receipt_head: "b".repeat(64),
            resulting_state_digest: "c".repeat(64),
            command_digest: "d".repeat(64),
            runtime_profile: "otp".to_string(),
            logical_clock: 1,
            replay_id: "replay-1".to_string(),
            receipt_head: "e".repeat(64),
        };
        let json = serde_json::to_string(&fields).expect("serialize");
        let round_tripped: TransitionReceiptFields =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(fields, round_tripped);
    }

    /// `AIRTransitionRefused` variants are usable as a real `std::error::
    /// Error` (thiserror-derived `Display`), matching this repo's typed-
    /// Refusal convention.
    #[test]
    fn air_transition_refused_variants_display_with_context() {
        let refusal = AIRTransitionRefused::CorrelationMissing {
            workflow_id: "wf-1".to_string(),
            step_id: Some("step_a".to_string()),
            dispatch_token: None,
        };
        let message = refusal.to_string();
        assert!(message.contains("wf-1"));
        assert!(message.contains("step_a"));

        let drift = AIRTransitionRefused::OtpAtomvmSemanticDrift {
            dimension: "state_digest".to_string(),
            otp_value: "abc".to_string(),
            atomvm_value: "def".to_string(),
        };
        assert!(drift.to_string().contains("state_digest"));
    }

    /// `EXIT_EVIDENCE_MARKERS` names exactly the three markers the atlas
    /// requires, no more, no fewer -- and none silently claims `PROVEN`
    /// (all three carry a hedge or an explicit "Erlang-side, not this
    /// crate" scope).
    #[test]
    fn exit_evidence_markers_are_the_named_three_and_none_overclaim() {
        let names: BTreeSet<&str> = EXIT_EVIDENCE_MARKERS.iter().map(|m| m.name).collect();
        let expected: BTreeSet<&str> = [
            "AIR_SINGLE_SEMANTIC_CORE_PROVEN",
            "DETERMINISTIC_READY_SET",
            "AIR_TRANSITION_REPLAYABLE",
        ]
        .into_iter()
        .collect();
        assert_eq!(names, expected);
        for marker in EXIT_EVIDENCE_MARKERS {
            assert_ne!(
                marker.status, "PROVEN",
                "marker {} must not claim unscoped PROVEN status",
                marker.name
            );
        }
    }

    /// `AirEvent`/`AirDispatchStepCommand` are real, constructible,
    /// comparable Rust values (the compiling-data-shape claim this module
    /// makes for them, nothing more).
    #[test]
    fn air_event_and_command_are_constructible_and_comparable() {
        let a = AirEvent::StepCompleted {
            step_id: "step_a".to_string(),
            result: "true".to_string(),
        };
        let b = a.clone();
        assert_eq!(a, b);

        let cmd = AirDispatchStepCommand {
            step_id: "step_b".to_string(),
            step_def: "#{next => []}".to_string(),
        };
        assert_eq!(cmd.step_id, "step_b");
    }
}
