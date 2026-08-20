//! `crown-bribery-case` -- Stage 2 of a 4-stage build: the first real,
//! non-test entry point driving the Solvane Global bribery-compliance case
//! fixture (`fixtures/bribery-case/`, Stage 1) through RDF admission,
//! Knowledge-Hook obligation derivation, PDDL8 planning, and POWL v2
//! projection, ending in a real Arazzo 1.1.x artifact written to disk and
//! compiled to an AIR program.
//!
//! # Chain driven (in order, each `?`-gated on the previous)
//!
//! 1. **F02 admission** ([`admit_observation`]) -- admits `case.ttl` (Stage
//!    1's fixture, embedded verbatim via `include_str!`) as a
//!    [`RawObservation`] against the exact [`AdmissionPolicy`]
//!    `tests/bribery_case_fixture.rs` designed and proved this file against.
//! 2. **Knowledge Hook obligation derivation** -- runs `hook.ttl`'s real
//!    `kh:Hook` SPARQL-CONSTRUCT action against the admitted case graph via
//!    `praxis_graphlaw::TripleStore::load_hook_pack` + `.materialize()` (the
//!    same real mechanism Stage 1 proved), deriving `sc:hasObligation`
//!    triples.
//! 3. **DESIGN.md's Stage-2 projector** (closed by this file, not yet built
//!    anywhere in this repo before this pass) -- turns the derived
//!    obligation local names into `pddl:init` PDDL8 atom-literal strings
//!    (`(has-obligation ...)` + `(requires-evidence ...)`, the latter read
//!    from `hook.ttl`'s own static `sc:requiresEvidenceType` catalog),
//!    mints a fresh runtime `pddl:Problem` RDF fragment carrying them, and
//!    manufactures real, bound-checked PDDL8 domain/problem text from it
//!    via `my_conforming_project::mfg::manufacture` -- the exact mechanism
//!    `tests/bribery_case_pddl.rs` (repo root) already proved against
//!    `pddl-domain.ttl`, now driven by hook-derived facts instead of a
//!    hand-authored problem file.
//! 4. **F08 planning** ([`run_pipeline`]) -- real PDDL8 grounding + BFS plan
//!    search over the manufactured domain/problem text, with every grounded
//!    action bound to a real hook capability from this file's own
//!    [`ACTION_HOOK_PACK_TTL`] (one `kh:Hook` per PDDL8 action schema this
//!    domain declares -- a *different* hook pack from `hook.ttl`, which
//!    derives obligations from RDF observations, not PDDL action
//!    capabilities; see that constant's own doc comment).
//! 5. **F09 growth / F10 geometry** ([`manufacture_and_bind_child`], which
//!    internally gates on `f10_powl_geometry::manufacture_powl_v2` -- see
//!    `crown_local.rs`'s own module doc for why this edge is transitive, not
//!    a second direct call) -- grafts the manufactured plan as a child of a
//!    fixed 2-leaf `PartialOrder` growth root (this fixture defines no
//!    surrounding workflow topology of its own -- the same disclosed
//!    architecture gap `crown-local-cli.rs`'s own module doc names), real
//!    POWL v2 geometry built from the *same* plan tape.
//! 6. **F13 Arazzo manufacture** ([`ArazzoProjectionReceipt::project_and_compile`])
//!    -- projects F10's real POWL v2 geometry into a real Arazzo 1.1.0 JSON
//!    document, **then writes it to disk** (`07-arazzo-artifact.json`).
//!    `project_and_compile` itself only ever builds the artifact in memory
//!    (confirmed absent this session: `grep -n "fs::write\|File::create"
//!    crates/praxis-core/src/arazzo.rs` returns zero hits) -- this file adds
//!    the missing disk-persistence caller, not new manufacture logic.
//! 7. **F14 compile** ([`f14_wasm4pm_arazzo::compile`]) -- compiles the
//!    just-written Arazzo document into a real `AirProgram` (parse -> URI
//!    resolve -> lower -> normalize -> digest), the same document
//!    `project_and_compile` already round-tripped internally to produce its
//!    own `air_digest`/`air_wasm` -- so this stage's success is expected,
//!    not merely hoped for, and is reported honestly either way.
//!
//! # RDF lifecycle authority
//!
//! Every meaningful object/transition above is written to a real file under
//! `target/crown-bribery-case/<run-id>/` before/after it occurs -- the
//! admitted case graph, the hook-derived obligation graph, the runtime PDDL
//! problem RDF fragment, the manufactured PDDL8 text, the computed plan
//! tape, the POWL v2 geometry, and the Arazzo artifact -- not just printed
//! to stdout. See [`run`]'s own body for the exact file list.
//!
//! # Extensibility (Stage 3 will extend this same binary)
//!
//! This file is written as a driver, not a one-off script: every stage is a
//! separate, independently testable function returning a typed
//! [`CliError`] variant naming exactly which boundary refused, and
//! [`run`]'s body is a linear `?`-chain a Stage-3 pass can append to (real
//! Erlang dispatch, per this task's brief) without restructuring anything
//! upstream -- the same discipline `crown-local-cli.rs` already established
//! for this crate's other bin target.
//!
//! Exit codes: 0 success. 1: a real pipeline stage refused (F02/hook
//! materialize/mfg::manufacture/F08/F09/F10/F13/F14 -- the refusal is
//! printed verbatim). 2: usage or internal-setup error (before any pipeline
//! stage ran).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use bumpalo::Bump;

use multifractal_workflow::crown_external::{
    drive_external_readmit_transition, drive_external_witness_tail_through_f16,
    drive_f16_completion_through_f18_broker, drive_f18_completion_through_f20_dispatch,
    ExternalF18Refused, ExternalF20Refused, ExternalReadmitTransitionOutcome,
    ExternalReadmitTransitionRefused, ExternalReentryRun, ExternalWitnessF16Refused,
};
use multifractal_workflow::f02_observation_admission::{
    admit_observation, AdmissionLedger, AdmissionPolicy, AdmissionReceipt,
    ObservationAdmissionRefused, RawObservation,
};
use multifractal_workflow::f08_pddl_planning::projector::{
    AdmittedTriple, HOOK_PACK_PREDICATE, PDDL_DOMAIN_PREDICATE, PDDL_PROBLEM_PREDICATE,
};
use multifractal_workflow::f08_pddl_planning::refusal::Refusal as PlanningRefusal;
use multifractal_workflow::f08_pddl_planning::{run_pipeline, PipelineOutcome};
use multifractal_workflow::f09_mfw_growth::{
    manufacture_and_bind_child, plan_growth, resolve_continuation_goal, DescentMeter,
    MFWGrowthRefused, ResidueState,
};
use multifractal_workflow::f13_arazzo_artifact::{ArazzoProjectionReceipt, CoreError};
use multifractal_workflow::f14_wasm4pm_arazzo::{self, ArazzoCompileRefused};
use multifractal_workflow::f15_air_transition_core::bridge::{
    BridgeEvent, BridgeStepDef, BridgeWorkflow,
};
use multifractal_workflow::f16_otp_runner::bridge::DispatchStatemOutcome;
use multifractal_workflow::f18_broker_law::{ActionId, Broker, BrokerReceipt, BrokerSecret};
use multifractal_workflow::f20_external_dispatch::{
    Powl as CngPowl, SubworkflowDispatchOutcome, SubworkflowPlan,
};

use powl2_decompose::{ParentChildClosure, Powl, SocketKind, SocketPath, WorkflowSocketId};
use praxis_graphlaw::chatman::closure::{ClosureLaw, RecursiveSocketClosure};
use praxis_graphlaw::parser::{Parser, Syntax};
use praxis_graphlaw::triples::{Term, VarOrTerm};
use praxis_graphlaw::TripleStore;

// ---------------------------------------------------------------------------
// Fixture content (Stage 1, embedded verbatim -- never re-derived by hand)
// ---------------------------------------------------------------------------

const CASE_TTL: &str = include_str!("../../fixtures/bribery-case/case.ttl");
const HOOK_TTL: &str = include_str!("../../fixtures/bribery-case/hook.ttl");
const SHAPES_TTL: &str = include_str!("../../fixtures/bribery-case/shapes.ttl");
const DOMAIN_TTL: &str = include_str!("../../fixtures/bribery-case/pddl-domain.ttl");

/// Same constants `tests/bribery_case_fixture.rs` designed `case.ttl`
/// against -- reused verbatim, not re-derived, so this binary's
/// [`AdmissionPolicy`] is byte-for-byte the one that fixture already proved.
const SOURCE_ID: &str = "solvane-case-intake-1";
const SOURCE_PRINCIPAL_IRI: &str = "https://intake.solvane-global.example.org/case-intake-1";
const SUBJECT: &str = "https://cases.solvane-global.example.org/case/BRB-2026-0417";
const SC: &str = "https://cases.solvane-global.example.org/vocab#";

/// F08's Action-Hook Binder catalog -- **distinct from `hook.ttl`**.
/// `hook.ttl`'s hook derives compliance obligations from an RDF
/// *observation* (`kh:kind "sparql"`, a SELECT+CONSTRUCT trigger over
/// `prov:`/`vcard:` facts); this catalog instead declares, per
/// `pddl-domain.ttl` PDDL8 *action schema*, that a registered capability
/// exists to actuate it (`kh:effect "ground-action"` + `kh:action
/// <urn:pddl:action:{schema_name}>`, matched by
/// `crate::f19_hooks::capability_action_iri` on `action.schema_name` alone
/// -- one hook covers every grounding of that schema, not one hook per
/// ground instance). Shape mirrors `f19_hooks.rs`'s own proven
/// `LOCAL_HOOK_PACK` test fixture; every one of `pddl-domain.ttl`'s 9
/// action schemas is covered so F08's Action-Hook Binder never refuses a
/// grounded action for want of a capability.
const ACTION_HOOK_PACK_TTL: &str = r#"
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <urn:mfw:crown-bribery-case:hooks#> .

ex:hook-supply-evidence a kh:Hook ;
  kh:name "supply-evidence-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-supply-evidence" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:supply-evidence> ;
  kh:reason "solvane-compliance-officer-authority-supply-evidence" ;
  kh:priority 1 .

ex:hook-clear-transaction-obligation a kh:Hook ;
  kh:name "clear-transaction-obligation-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-clear-transaction-obligation" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:clear-transaction-obligation> ;
  kh:reason "solvane-compliance-officer-authority-clear-transaction-obligation" ;
  kh:priority 1 .

ex:hook-clear-authorization-obligation a kh:Hook ;
  kh:name "clear-authorization-obligation-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-clear-authorization-obligation" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:clear-authorization-obligation> ;
  kh:reason "solvane-compliance-officer-authority-clear-authorization-obligation" ;
  kh:priority 1 .

ex:hook-clear-policy-obligation a kh:Hook ;
  kh:name "clear-policy-obligation-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-clear-policy-obligation" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:clear-policy-obligation> ;
  kh:reason "solvane-compliance-officer-authority-clear-policy-obligation" ;
  kh:priority 1 .

ex:hook-close-obligations a kh:Hook ;
  kh:name "close-obligations-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-close-obligations" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:close-obligations> ;
  kh:reason "solvane-compliance-officer-authority-close-obligations" ;
  kh:priority 1 .

ex:hook-judge a kh:Hook ;
  kh:name "judge-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-judge" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:judge> ;
  kh:reason "solvane-compliance-officer-authority-judge" ;
  kh:priority 1 .

ex:hook-admit a kh:Hook ;
  kh:name "admit-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-admit" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:admit> ;
  kh:reason "solvane-general-counsel-authority-admit" ;
  kh:priority 1 .

ex:hook-receipt a kh:Hook ;
  kh:name "receipt-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-receipt" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:receipt> ;
  kh:reason "solvane-general-counsel-authority-receipt" ;
  kh:priority 1 .

ex:hook-block-for-missing-evidence a kh:Hook ;
  kh:name "block-for-missing-evidence-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-block-for-missing-evidence" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:block-for-missing-evidence> ;
  kh:reason "solvane-compliance-officer-authority-block-for-missing-evidence" ;
  kh:priority 1 .
"#;

// ---------------------------------------------------------------------------
// Stage 3: real Erlang/OTP dispatch, broker, OCEL, receipt/replay, case
// closure. See this file's module doc "Extensibility" section -- Stage 3
// extends this same binary, driving the F08 plan tape (already real, already
// computed by the Stage 2 chain above) through
// F15(AIR transition)->F16(dispatch-statem)->F18(broker)->F20(direct
// dispatch), then the full F20->F02(re-admit)->F15->F21->F24->F25 loop-back
// tail, via `multifractal_workflow::crown_external`'s real, already-proven
// composition functions (reused verbatim, not reimplemented).
// ---------------------------------------------------------------------------

/// Deterministic Stage-3 [`BrokerSecret`] -- BLAKE3 over a fixed,
/// domain-separated label, never wall-clock/random (repo invariant #3/#5).
/// Mirrors `crown_external_test.rs`'s own fixed `[9u8; 32]` broker-secret
/// fixture pattern, but derived from a disclosed label rather than a bare
/// magic literal.
fn stage3_broker_secret() -> BrokerSecret {
    BrokerSecret::new(*blake3::hash(b"crown-bribery-case:stage3:broker-secret:v1").as_bytes())
}

/// Fixed, disclosed seed for `engine_serve`'s deterministic identity
/// derivation (`splitmix64(seed ^ blake3(engine_id))`) -- never a PID or
/// wall clock. Matches this crate's other F20 fixtures' convention of a
/// fixed seed.
const STAGE3_ENGINE_SEED: u64 = 26_07_12;
/// Poll budget shared by every Stage-3 `engine_serve`/collect round trip.
const STAGE3_MAX_POLLS: u64 = 16;

/// F02 re-admission source id for the `F20 -> F02(re-admit)` edge -- the
/// identity asserting "F20 collected this real consequence" for this case,
/// distinct from [`SOURCE_ID`] (the case-intake principal).
const EXTERNAL_REENTRY_SOURCE: &str = "crown-bribery-case-external-reentry-1";
/// The principal IRI [`EXTERNAL_REENTRY_SOURCE`] maps to in Stage 3's own
/// [`AdmissionPolicy`] (see [`build_external_reentry_policy`]).
const EXTERNAL_REENTRY_PRINCIPAL: &str =
    "https://cases.solvane-global.example.org/crown-bribery-case/external-reentry-authority";
/// A real but vacuous SHACL shape (`sh:targetClass` matches no admitted
/// node) -- mirrors `crown_external_test.rs::reentry_policy`'s own
/// `REENTRY_VACUOUS_SHAPES` fixture: F02 gate 4 genuinely runs and
/// genuinely conforms (no node matches the absent target class), not a
/// placeholder.
const EXTERNAL_REENTRY_VACUOUS_SHAPES: &str = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <urn:mfw:crown-bribery-case:reentry#> .
ex:ReentryShape a sh:NodeShape ;
    sh:targetClass ex:AbsentClass .
"#;

/// Typed, exhaustive failure surface for this binary. No panics, no unwraps
/// on external input or fallible library calls.
#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("usage error: {0}")]
    Usage(String),
    #[error("could not write {path}: {reason}")]
    Io { path: String, reason: String },
    #[error("internal admission policy invalid: {0}")]
    PolicyInvalid(String),
    #[error("F02 admission refused: {0}")]
    Admission(#[from] ObservationAdmissionRefused),
    #[error("case.ttl is malformed: {0}")]
    ObservationMalformed(String),
    #[error("hook.ttl failed to load as a kh: hook pack: {0}")]
    HookPackMalformed(String),
    #[error("hook materialize() failed: {0}")]
    HookMaterializeFailed(String),
    #[error(
        "hook.ttl derived zero sc:hasObligation triples for <{subject}> -- the cross-border \
         trigger pattern did not match this admitted case graph"
    )]
    NoObligationsDerivedForCase { subject: String },
    #[error("hook.ttl failed to re-parse for its sc:requiresEvidenceType catalog: {0}")]
    HookCatalogUnparsable(String),
    #[error(
        "hook.ttl declares no sc:requiresEvidenceType fact for obligation {obligation_local_name:?}"
    )]
    EvidenceTypeCatalogMissing { obligation_local_name: String },
    #[error("mfg::manufacture (RDF pddl: instance data -> PDDL8 text) failed: {0}")]
    Manufacture(String),
    #[error("F08 planning refused: {0}")]
    Planning(#[from] PlanningRefusal),
    #[error("internal growth-root closure invalid: {0}")]
    ClosureInvalid(String),
    #[error("F09/F10 growth refused: {0}")]
    Growth(#[from] MFWGrowthRefused),
    #[error("F13 Arazzo manufacture refused: {0}")]
    Arazzo(#[from] CoreError),
    #[error("F14 Arazzo->AIR compile refused: {0}")]
    Compile(#[from] ArazzoCompileRefused),
    #[error("failed to serialize {what} to JSON: {reason}")]
    Serialize { what: &'static str, reason: String },
    #[error(
        "Stage 3 bridge-workflow construction refused: the plan tape has zero steps to \
             chain (nothing for F15/F16 to dispatch)"
    )]
    EmptyPlanForBridgeWorkflow,
    #[error("F15->F16 dispatch refused: {0}")]
    F16Dispatch(#[from] ExternalWitnessF16Refused),
    #[error(
        "F15/F16 chain produced zero dispatch_step commands for completed step {step_id:?} -- \
         nothing to drive through F16/F18/F20"
    )]
    NoF16DispatchCommands { step_id: String },
    #[error("F16 dispatch for step {step_id} was refused by the real gen_statem: {refusal_atom}")]
    F16DispatchRefused {
        step_id: String,
        refusal_atom: String,
    },
    #[error("F16->F18 broker actuation refused: {0}")]
    F18Broker(Box<ExternalF18Refused>),
    #[error("F18->F20 direct dispatch refused: {0}")]
    F20Direct(#[from] ExternalF20Refused),
    #[error("F20->F02 re-admission policy invalid: {0}")]
    ReentryPolicyInvalid(String),
    #[error("F20->F02->F15->F21->F24->F25 readmit-transition chain refused: {0}")]
    ReadmitTransition(#[from] ExternalReadmitTransitionRefused),
    #[error("case-closure RDF malformed: {reason}")]
    ClosureRdfMalformed { reason: String },
    /// 80/20 gap sweep gap-2: `consequence_digest` was `None` despite `consequence_turtle` being
    /// `Some`. `SubworkflowDispatchOutcome`'s own doc comment (`dispatch_bridge.rs`) guarantees
    /// these two fields are set together -- both `None` or both `Some` -- so reaching this
    /// refusal means that invariant broke upstream. Refused before folding into the Stage-3 crown
    /// receipt or the case-closure Turtle/JSON summary, rather than silently substituting an
    /// empty-string digest via `unwrap_or_default()`.
    #[error(
        "dispatch {dispatch_id}'s consequence_digest was missing despite a collected \
         consequence_turtle"
    )]
    MissingConsequenceDigest { dispatch_id: String },
}

impl From<ExternalF18Refused> for CliError {
    // Manual instead of #[from]: ExternalF18Refused's largest variant is
    // >=128 bytes, which clippy::result_large_err flags on every
    // Result<_, CliError>-returning function in this file (CliError's own
    // size is bounded by its largest field). Boxing here keeps every `?`
    // call site unchanged -- this impl supplies the same conversion
    // #[from] would have generated, just via an owned allocation instead
    // of an inline large variant.
    fn from(e: ExternalF18Refused) -> Self {
        CliError::F18Broker(Box::new(e))
    }
}

impl CliError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_)
            | Self::Io { .. }
            | Self::PolicyInvalid(_)
            | Self::ObservationMalformed(_)
            | Self::HookPackMalformed(_)
            | Self::HookCatalogUnparsable(_)
            | Self::ClosureInvalid(_)
            | Self::Serialize { .. }
            | Self::EmptyPlanForBridgeWorkflow
            | Self::ReentryPolicyInvalid(_)
            | Self::ClosureRdfMalformed { .. } => 2,
            _ => 1,
        }
    }
}

/// This binary's own fixed [`AdmissionPolicy`] -- byte-for-byte
/// `tests/bribery_case_fixture.rs::bribery_case_policy()`, duplicated here
/// (that function is `fn`-private to its own test binary, not part of this
/// crate's public API) rather than imported.
fn build_policy() -> Result<AdmissionPolicy, CliError> {
    use std::collections::BTreeMap;

    let mut known_principals = BTreeMap::new();
    known_principals.insert(SOURCE_ID.to_string(), SOURCE_PRINCIPAL_IRI.to_string());

    let mut authorized = BTreeSet::new();
    authorized.insert("http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string());
    authorized.insert("http://www.w3.org/ns/prov#wasAssociatedWith".to_string());
    authorized.insert("http://www.w3.org/ns/prov#used".to_string());
    authorized.insert("http://www.w3.org/ns/prov#startedAtTime".to_string());
    authorized.insert("http://purl.org/dc/terms/identifier".to_string());
    authorized.insert("http://purl.org/dc/terms/description".to_string());
    authorized.insert(format!("{SC}caseStatus"));
    let mut authorized_predicates = BTreeMap::new();
    authorized_predicates.insert(SOURCE_ID.to_string(), authorized);

    AdmissionPolicy::new(
        known_principals,
        authorized_predicates,
        vec![
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
            "http://www.w3.org/ns/prov#".to_string(),
            "http://purl.org/dc/terms/".to_string(),
            SC.to_string(),
        ],
        vec!["https://".to_string()],
        SHAPES_TTL,
    )
    .map_err(CliError::PolicyInvalid)
}

/// A real, open recursive-socket closure over a 2-leaf `PartialOrder` root
/// -- identical fixture shape to `crown-local-cli.rs`'s own
/// `open_growth_root_and_closure()` (this fixture defines no surrounding
/// workflow topology of its own; see this file's module doc).
fn open_growth_root_and_closure() -> Result<(Powl, RecursiveSocketClosure), CliError> {
    let children = (0..2)
        .map(|i| Powl::Leaf(Some(format!("leaf-{i}"))))
        .collect();
    let root = Powl::PartialOrder {
        children,
        order: BTreeSet::new(),
    };
    let pcc = ParentChildClosure::from_model(&root);
    let socket = WorkflowSocketId {
        path: SocketPath::root(),
        kind: SocketKind::PartialOrder,
    };
    let closure = RecursiveSocketClosure::declare(&pcc, socket, ClosureLaw::AllRequired)
        .map_err(|e| CliError::ClosureInvalid(e.to_string()))?;
    Ok((root, closure))
}

/// Extracts a bare (unbracketed) IRI string from an RDF term, or `None` if
/// the term is not `Term::Iri`. Same pattern `crown-local-cli.rs`'s own
/// private `bare_iri` helper uses (IRI `Display` is a plain `<...>`
/// bracket-trim, no escaping ambiguity, unlike literals).
///
/// # Complexity
/// O(len(iri)) for the bracket trim.
fn bare_iri(vt: &VarOrTerm) -> Option<String> {
    match vt {
        VarOrTerm::Term(t @ Term::Iri(_)) => {
            let displayed = t.to_string();
            Some(
                displayed
                    .trim_start_matches('<')
                    .trim_end_matches('>')
                    .to_string(),
            )
        }
        _ => None,
    }
}

/// Direct-text extraction of `dcterms:identifier "..."` from the admitted
/// case Turtle (same "no public lexical-value accessor" constraint
/// `crown-local-cli.rs`'s module doc discloses for `Term`), lower-cased and
/// `case-`-prefixed to match `pddl-problem-closable.ttl`'s own established
/// object-naming convention (see that file's header): `"BRB-2026-0417"` ->
/// `"case-brb-2026-0417"`.
///
/// # Complexity
/// O(len(case_ttl)) for the two substring searches.
fn extract_case_local_name(case_ttl: &str) -> Result<String, CliError> {
    let marker = "dcterms:identifier \"";
    let after = case_ttl
        .find(marker)
        .map(|i| &case_ttl[i + marker.len()..])
        .ok_or_else(|| {
            CliError::ObservationMalformed(
                "no dcterms:identifier literal found in case.ttl".to_string(),
            )
        })?;
    let end = after.find('"').ok_or_else(|| {
        CliError::ObservationMalformed("dcterms:identifier literal is never closed".to_string())
    })?;
    Ok(format!("case-{}", after[..end].to_lowercase()))
}

/// Real hook-derived `sc:hasObligation` graph for [`SUBJECT`], plus the
/// obligation local names it carries (sorted, deduped).
struct DerivedObligations {
    /// The derived `sc:hasObligation` triples, in the store's own canonical
    /// (sorted) N-Triples-like text -- real RDF, not a summary.
    turtle: String,
    /// Sorted, deduped local names (e.g. `"assess-policy-violation"`).
    local_names: Vec<String>,
}

/// Runs `hook.ttl`'s real `kh:Hook` SPARQL-CONSTRUCT action against the
/// admitted case graph via `TripleStore::load_hook_pack` + `.materialize()`
/// -- the exact mechanism `tests/bribery_case_fixture.rs` proved. Reads
/// back derived facts via direct triple-set inspection
/// (`store.content_to_string()`), not a follow-up SPARQL SELECT -- Stage 1
/// found and disclosed a real `praxis-graphlaw` engine limitation where a
/// `store.query()` SELECT does not see triples a hook's CONSTRUCT added in
/// the same `materialize()` call (see `hook.ttl`'s and
/// `bribery_case_fixture.rs`'s own doc comments).
///
/// # Errors
/// [`CliError::HookPackMalformed`] / [`CliError::HookMaterializeFailed`] on
/// a real engine failure; [`CliError::NoObligationsDerivedForCase`] if the
/// hook's trigger pattern genuinely did not match (a real, typed non-result,
/// not fabricated).
///
/// # Complexity
/// O(materialize) (the hook engine's own documented cost) plus O(D) over
/// the materialized store's dump text, D = dump length.
fn derive_obligations(hook_ttl: &str, case_ttl: &str) -> Result<DerivedObligations, CliError> {
    let mut store = TripleStore::new();
    store
        .load_hook_pack(hook_ttl)
        .map_err(CliError::HookPackMalformed)?;
    store
        .load_triples(case_ttl, Syntax::Turtle)
        .map_err(CliError::ObservationMalformed)?;
    store
        .materialize()
        .map_err(CliError::HookMaterializeFailed)?;

    let dump = store.content_to_string();
    let predicate = format!("{SC}hasObligation");
    let mut names = BTreeSet::new();
    let mut lines = Vec::new();
    for line in dump.lines() {
        if line.contains(SUBJECT) && line.contains(&predicate) {
            if let Some(obj_iri) = extract_last_angle_iri(line) {
                if let Some(local) = obj_iri.rsplit('#').next() {
                    names.insert(local.to_string());
                }
            }
            lines.push(line);
        }
    }
    if names.is_empty() {
        return Err(CliError::NoObligationsDerivedForCase {
            subject: SUBJECT.to_string(),
        });
    }
    let mut turtle = String::new();
    for line in &lines {
        turtle.push_str(line);
        turtle.push('\n');
    }
    Ok(DerivedObligations {
        turtle,
        local_names: names.into_iter().collect(),
    })
}

/// Extracts the IRI inside the LAST `<...>` bracket pair on `line` -- the
/// object position of a `TripleStore::content_to_string()` dump line
/// (`<s> <p> <o>.`).
///
/// # Complexity
/// O(len(line)).
fn extract_last_angle_iri(line: &str) -> Option<String> {
    let after_last_lt = line.rsplit_once('<')?.1;
    let iri = after_last_lt.split('>').next()?;
    Some(iri.to_string())
}

/// Reads `hook.ttl`'s static `sc:requiresEvidenceType` catalog fact for
/// `obligation_local_name` (a simple triple scan over a fresh re-parse of
/// `hook_ttl` -- deliberately not `store.query()`, for the same reason
/// [`derive_obligations`] does not use it).
///
/// # Errors
/// [`CliError::HookCatalogUnparsable`] if `hook_ttl` fails to parse (should
/// be unreachable for the checked-in fixture; defensive, not `.expect()`).
/// [`CliError::EvidenceTypeCatalogMissing`] if no catalog fact exists for
/// this obligation -- a real catalog gap, never silently skipped.
///
/// # Complexity
/// O(len(hook_ttl)) to parse, O(T) to scan (T = parsed triple count).
fn evidence_type_for_obligation(
    hook_ttl: &str,
    obligation_local_name: &str,
) -> Result<String, CliError> {
    let triples =
        Parser::parse_triples(hook_ttl, Syntax::Turtle).map_err(CliError::HookCatalogUnparsable)?;
    let predicate = VarOrTerm::convert(format!("{SC}requiresEvidenceType"));
    for t in &triples {
        if t.p != predicate {
            continue;
        }
        let Some(subject_iri) = bare_iri(&t.s) else {
            continue;
        };
        if subject_iri.rsplit('#').next() != Some(obligation_local_name) {
            continue;
        }
        if let Some(object_iri) = bare_iri(&t.o) {
            if let Some(local) = object_iri.rsplit('#').next() {
                return Ok(local.to_string());
            }
        }
    }
    Err(CliError::EvidenceTypeCatalogMissing {
        obligation_local_name: obligation_local_name.to_string(),
    })
}

/// DESIGN.md's Stage-2 wiring contract, built for real: turns hook-derived
/// `sc:hasObligation` object local names into the two `pddl:init` PDDL8 atom
/// families `pddl-problem-closable.ttl` hand-authored --
/// `(has-obligation <case> <ob>)` for each derived obligation, plus the
/// `(requires-evidence <ob> <etype>)` fact read from `hook.ttl`'s own
/// static `sc:requiresEvidenceType` catalog (joined on the same
/// obligation).
///
/// # Errors
/// [`CliError::EvidenceTypeCatalogMissing`] (via
/// [`evidence_type_for_obligation`]) if `hook_ttl` declares no evidence-type
/// fact for some derived obligation.
///
/// # Complexity
/// O(derived.len() * len(hook_ttl)): one `hook_ttl` re-parse per obligation
/// (bounded -- this fixture's catalog is small and `derived` is bounded by
/// its fixed 3-obligation catalog).
fn project_obligations_to_pddl_init(
    hook_ttl: &str,
    case_local_name: &str,
    derived: &[String],
) -> Result<Vec<String>, CliError> {
    let mut atoms = Vec::with_capacity(derived.len() * 2);
    for obligation in derived {
        atoms.push(format!("(has-obligation {case_local_name} {obligation})"));
    }
    for obligation in derived {
        let evidence_type = evidence_type_for_obligation(hook_ttl, obligation)?;
        atoms.push(format!("(requires-evidence {obligation} {evidence_type})"));
    }
    Ok(atoms)
}

/// Builds a runtime `pddl:Problem` RDF Turtle fragment -- the same shape
/// `pddl-problem-closable.ttl` hand-authored (see that file's header), but
/// with `pddl:init`'s obligation/evidence atoms and the corresponding
/// `pddl:object` obligation/evidence-type entries driven entirely by
/// `derived`/`evidence_types`/`init_atoms` (this run's real, live hook
/// output), not hardcoded -- this is DESIGN.md's disclosed Stage-2 gap,
/// closed here. Concatenate with [`DOMAIN_TTL`] before calling
/// `mfg::manufacture` (that function requires exactly one `pddl:Problem`
/// instance per graph, satisfied by construction: `DOMAIN_TTL` declares
/// none).
///
/// # Complexity
/// O(derived.len() + evidence_types.len() + init_atoms.len()) string
/// building.
fn build_pddl_problem_fragment(
    case_local_name: &str,
    derived: &[String],
    evidence_types: &[String],
    init_atoms: &[String],
) -> String {
    let mut out = String::new();
    out.push_str("@prefix pddl: <http://seanchatmangpt.github.io/praxis/pddl#> .\n\n");
    out.push_str(&format!(
        "<urn:mfw:crown-bribery-case:problem:{case_local_name}>\n    a pddl:Problem ;\n"
    ));
    out.push_str(&format!(
        "    pddl:name \"bribery-case-{case_local_name}-runtime\" ;\n"
    ));
    out.push_str("    pddl:domain \"solvane-bribery-compliance-pddl8\" ;\n");

    let mut objects: Vec<String> = vec![format!(
        "[ pddl:name \"{case_local_name}\" ; pddl:ofType \"law-object\" ]"
    )];
    for o in derived {
        objects.push(format!(
            "[ pddl:name \"{o}\" ; pddl:ofType \"obligation\" ]"
        ));
    }
    for e in evidence_types {
        objects.push(format!(
            "[ pddl:name \"{e}\" ; pddl:ofType \"evidence-type\" ]"
        ));
    }
    objects.push(
        "[ pddl:name \"compliance-officer-shreya-patel\" ; pddl:ofType \"validator\" ]".to_string(),
    );
    objects.push(
        "[ pddl:name \"general-counsel-marcus-webb\" ; pddl:ofType \"authority\" ]".to_string(),
    );
    objects.push(format!(
        "[ pddl:name \"tok-genesis-{case_local_name}\" ; pddl:ofType \"chain-token\" ]"
    ));
    for stage in ["raw", "validated", "admitted", "receipted", "blocked"] {
        objects.push(format!(
            "[ pddl:name \"{stage}\" ; pddl:ofType \"lifecycle-stage\" ]"
        ));
    }
    out.push_str("    pddl:object ");
    out.push_str(&objects.join(" ,\n               "));
    out.push_str(" ;\n");

    let mut init: Vec<String> = vec![format!("\"(in-stage {case_local_name} raw)\"")];
    for atom in init_atoms {
        init.push(format!("\"{atom}\""));
    }
    init.push(format!(
        "\"(prev-chain-valid tok-genesis-{case_local_name})\""
    ));
    out.push_str("    pddl:init ");
    out.push_str(&init.join(" ,\n             "));
    out.push_str(" ;\n");

    out.push_str(&format!(
        "    pddl:goal \"(in-stage {case_local_name} receipted)\" .\n"
    ));
    out
}

/// Manufactures real, bound-checked PDDL8 domain/problem text from
/// [`DOMAIN_TTL`] concatenated with a runtime problem fragment, via
/// `my_conforming_project::mfg::manufacture` -- the exact real pipeline
/// (`SPARQL extraction + enforce_pddl8 bound-checking + PDDL8 text
/// emission`) `tests/bribery_case_pddl.rs` already proved against this
/// domain.
///
/// # Errors
/// [`CliError::Manufacture`] wrapping `mfg::MfgError`'s `Display` text --
/// e.g. a PDDL8 bound violation, or an RDF shape error.
fn manufacture_pddl(
    problem_fragment_ttl: &str,
    source_label: &str,
) -> Result<my_conforming_project::mfg::AdmittedPlanningTask, CliError> {
    let combined = format!("{DOMAIN_TTL}\n{problem_fragment_ttl}");
    my_conforming_project::mfg::manufacture(&combined, source_label)
        .map_err(|e| CliError::Manufacture(e.to_string()))
}

/// Writes `contents` to `run_dir/name`, returning the written path.
fn write_file(run_dir: &Path, name: &str, contents: &str) -> Result<PathBuf, CliError> {
    let path = run_dir.join(name);
    fs::write(&path, contents).map_err(|e| CliError::Io {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    Ok(path)
}

// ---------------------------------------------------------------------------
// Stage 3 functions
// ---------------------------------------------------------------------------

/// Builds a linear [`BridgeWorkflow`] directly from crown-bribery-case's own real F08 plan-tape
/// order: `"{i:02}-{name}"` step ids, each chained to the next (`BridgeStepDef.next`) except the
/// last, which has no successor.
///
/// **Why not build it from F13/F14's own output**: `crown_external.rs`'s own module doc discloses
/// that F13's Arazzo projection template (`crates/praxis-core/templates/arazzo_projection.tera`)
/// emits no `onSuccess` routing at all -- every step is a flat root, no `goto` edges (proven by
/// `crown_external_test.rs::external_tail_f10_output_has_no_successor_edges_to_dispatch_to_f16`).
/// Feeding this case's real F13/F14 output through `drive_external_witness_tail_through_f16`
/// would legitimately dispatch **zero** F16 commands. `BridgeWorkflow`/`BridgeStepDef` are plain
/// public-field structs (`f15_air_transition_core::bridge`), constructible directly without going
/// through F13/F14's template -- so this function builds real routing edges directly from the
/// plan's own real, already-computed linear execution order (Stage 2's `run_pipeline` tape),
/// bypassing F13/F14's template only for the routing *shape*. The plan itself is 100% Stage 2's
/// already-real, already-verified tape, not fabricated.
///
/// # Errors
/// [`CliError::EmptyPlanForBridgeWorkflow`] if `schema_names` is empty (defensive; this binary's
/// own real F08 plan always has >= 1 step, so this should be unreachable in practice).
///
/// # Complexity
/// O(n) over `schema_names`.
fn build_bridge_workflow_from_plan(
    schema_names: &[&str],
) -> Result<(BridgeWorkflow, String), CliError> {
    if schema_names.is_empty() {
        return Err(CliError::EmptyPlanForBridgeWorkflow);
    }
    let ids: Vec<String> = schema_names
        .iter()
        .enumerate()
        .map(|(i, name)| format!("{i:02}-{name}"))
        .collect();
    let mut steps = BTreeMap::new();
    for (i, id) in ids.iter().enumerate() {
        let next = match ids.get(i + 1) {
            Some(next_id) => vec![next_id.clone()],
            None => Vec::new(),
        };
        steps.insert(id.clone(), BridgeStepDef { next });
    }
    let root_step_id = ids[0].clone();
    Ok((BridgeWorkflow { steps }, root_step_id))
}

/// Stage 3's own [`AdmissionPolicy`] for the `F20 -> F02(re-admit)` edge -- mirrors
/// `crown_external_test.rs::reentry_policy()`'s shape exactly (same 3-predicate authorized set,
/// same vacuous-SHACL-shape pattern), reproduced here since that function is private to its own
/// test module, not part of this crate's public API.
///
/// **Resolves this design's one open verification item** (whether `prov:wasDerivedFrom` needs to
/// be in `authorized_predicates`): reading `f02_observation_admission.rs`'s gate 3 (Authority
/// Checker) in full shows it explicitly skips the `prov:wasDerivedFrom` bookkeeping triple before
/// building `distinct_predicates` (`if t.s == declared_subject_term && t.p == prov_predicate {
/// continue; }`), and gate 5 (Semantic Conformance) only ever iterates that same
/// `distinct_predicates` set -- so `prov:wasDerivedFrom` is never checked against
/// `authorized_predicates` at all. Gate 2 (Provenance Checker) checks it by its own, separate
/// mechanism: the asserted derivation IRI must equal `policy.known_principals[source_id]`. So the
/// 3-predicate set below (`dispatchId`/`consequenceDigest`/`consequenceTurtle`) is sufficient;
/// adding `prov:wasDerivedFrom` to `authorized_predicates` would be harmless but is not required.
///
/// # Errors
/// [`CliError::ReentryPolicyInvalid`] if [`EXTERNAL_REENTRY_VACUOUS_SHAPES`] fails to parse
/// (defensive: hand-verified compile-time SHACL Turtle).
fn build_external_reentry_policy() -> Result<AdmissionPolicy, CliError> {
    let mut known_principals = BTreeMap::new();
    known_principals.insert(
        EXTERNAL_REENTRY_SOURCE.to_string(),
        EXTERNAL_REENTRY_PRINCIPAL.to_string(),
    );

    let mut authorized = BTreeSet::new();
    authorized.insert("urn:mfw:f20#dispatchId".to_string());
    authorized.insert("urn:mfw:f20#consequenceDigest".to_string());
    authorized.insert("urn:mfw:f20#consequenceTurtle".to_string());
    let mut authorized_predicates = BTreeMap::new();
    authorized_predicates.insert(EXTERNAL_REENTRY_SOURCE.to_string(), authorized);

    AdmissionPolicy::new(
        known_principals,
        authorized_predicates,
        vec!["urn:mfw:f20#".to_string()],
        vec!["https://".to_string()],
        EXTERNAL_REENTRY_VACUOUS_SHAPES,
    )
    .map_err(CliError::ReentryPolicyInvalid)
}

/// The real, composed output of one Stage-3 run: every real hop's own genuine output, plus a
/// deterministic crown receipt binding them.
struct Stage3Outcome {
    /// The real F15/F16-dispatched step id (`"01-clear-authorization-obligation"`).
    f16_step_id: String,
    /// F16's real, non-empty dispatch token.
    f16_dispatch_token: String,
    /// F16's real 7-state gen_statem transition log.
    f16_transition_log: Vec<String>,
    /// F18's real broker receipt actuating F16's dispatch token.
    broker_receipt: BrokerReceipt,
    /// F18 -> F20 direct dispatch's real outcome (a separate, independent dispatch/serve/collect
    /// round trip from `readmit`'s own F20 dispatch -- see this file's module doc / final report
    /// for why two independent dispatches of the same logical identity is deliberate, not
    /// redundant).
    f20_direct: SubworkflowDispatchOutcome,
    /// `f20_direct.consequence_digest`, extracted and validated once (80/20 gap sweep gap-2: a
    /// `None` here despite a collected `consequence_turtle` is refused as
    /// [`CliError::MissingConsequenceDigest`] before this struct is ever constructed, rather than
    /// silently defaulted to an empty string every place the digest is read below).
    f20_direct_consequence_digest: String,
    /// The full `F20 -> F02(re-admit) -> F15 -> F21 -> F24 -> F25` loop-back tail's real outcome.
    readmit: ExternalReadmitTransitionOutcome,
    /// BLAKE3-hex over every Stage-3 hop's real digest, in canonical sorted order (no wall clock,
    /// no randomness).
    crown_receipt: String,
}

/// Fold every Stage-3 real digest into one deterministic BLAKE3-hex crown receipt, matching
/// `crown_external.rs`'s own `compute_reentry_crown_receipt`/`compute_external_crown_receipt`
/// pattern (sorted lines, BLAKE3, no wall clock).
///
/// # Complexity
/// O(1): a small, fixed number of digest lines.
fn compute_stage3_crown_receipt(
    f16_dispatch_token: &str,
    broker_receipt: &BrokerReceipt,
    f20_direct: &SubworkflowDispatchOutcome,
    f20_direct_consequence_digest: &str,
    readmit: &ExternalReadmitTransitionOutcome,
) -> String {
    let mut lines = vec![
        format!("f16.dispatch_token={f16_dispatch_token}"),
        format!("f18.receipt_hash={}", broker_receipt.receipt_hash_hex),
        format!("f20_direct.dispatch_id={}", f20_direct.dispatch_id),
        format!("f20_direct.consequence_digest={f20_direct_consequence_digest}"),
        format!(
            "f02_readmit.receipt_hash={}",
            readmit.reentry.reentry_admission.receipt_hash
        ),
        format!("f24.receipt_head={}", readmit.ocel_outcome.receipt_head),
        format!(
            "f25.receipt_root={}",
            readmit.replay_outcome.receipt.receipt_root.as_str()
        ),
        format!(
            "f25.replayed_receipt_root={}",
            readmit.replay_outcome.replayed.receipt_root.as_str()
        ),
    ];
    lines.sort();
    let mut hasher = blake3::Hasher::new();
    for line in &lines {
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

/// Drives Stage 3's full real chain, in order: build a [`BridgeWorkflow`] from the real F08 plan
/// tape -> complete its root step through the real `air_core` bridge and drive every resulting
/// `dispatch_step` command through the real F16 gen_statem
/// ([`drive_external_witness_tail_through_f16`]) -> actuate the completed F16 dispatch through
/// the real F18 [`Broker`] ([`drive_f16_completion_through_f18_broker`]) -> drive that receipt
/// through a real, direct F20 dispatch/serve/collect round trip
/// ([`drive_f18_completion_through_f20_dispatch`]) -> re-dispatch the same logical subworkflow
/// identity through the full real `F20 -> F02(re-admit) -> F15 -> F21 -> F24 -> F25` loop-back
/// tail ([`drive_external_readmit_transition`]).
///
/// Only the **first** plan-tape step transition is driven through the real Erlang chain this
/// pass (matching `crown_external_test.rs::f15_transition_command_drives_a_real_f16_dispatch_
/// statem_to_completion`'s own established one-step precedent) -- a disclosed scope bound, not
/// an overclaim.
///
/// # Errors
/// [`CliError`], carrying the first real hop's own typed refusal verbatim. A real F16 gen_statem
/// refusal (`DispatchStatemOutcome::Refused`) is propagated as [`CliError::F16DispatchRefused`],
/// never silently treated as success.
fn run_stage3(
    run_dir: &Path,
    run_id: &str,
    plan_steps: &[&str],
    admission_receipt_hash: &str,
    workflow_id: &str,
) -> Result<Stage3Outcome, CliError> {
    let (bridge_workflow, root_step_id) = build_bridge_workflow_from_plan(plan_steps)?;
    let bridge_events = vec![BridgeEvent::StepCompleted {
        step_id: root_step_id.clone(),
        result: serde_json::Value::String(admission_receipt_hash.to_string()),
    }];
    let crown_receipt_seed =
        format!("crown-bribery-case-{run_id}-f16-seed-{admission_receipt_hash}");
    let f16_outer = drive_external_witness_tail_through_f16(
        &bridge_workflow,
        std::slice::from_ref(&root_step_id),
        &bridge_events,
        &crown_receipt_seed,
    )?;

    let (f16_step_id, f16_dispatch_statem_outcome) = f16_outer
        .dispatch_outcomes
        .into_iter()
        .next()
        .ok_or_else(|| CliError::NoF16DispatchCommands {
            step_id: root_step_id.clone(),
        })?;

    let (f16_dispatch_token, f16_transition_log) = match &f16_dispatch_statem_outcome {
        DispatchStatemOutcome::Completed {
            dispatch_token,
            transition_log,
            ..
        } => (dispatch_token.clone(), transition_log.clone()),
        DispatchStatemOutcome::Refused { refusal_atom, .. } => {
            return Err(CliError::F16DispatchRefused {
                step_id: f16_step_id,
                refusal_atom: refusal_atom.clone(),
            });
        }
    };

    let broker = Broker::new(stage3_broker_secret());
    let action = ActionId::new(
        workflow_id,
        f16_step_id.clone(),
        format!("crown-bribery-case-{run_id}-idempotency-1"),
    );
    let broker_receipt = drive_f16_completion_through_f18_broker(
        &broker,
        action,
        "compliance-officer-shreya-patel",
        true,
        "solvane-compliance-officer-authority-external-dispatch",
        &format!("crown-bribery-case-{run_id}-f18-correlation-1"),
        &f16_step_id,
        &f16_dispatch_statem_outcome,
    )?;

    let f20_direct = drive_f18_completion_through_f20_dispatch(
        &run_dir.join("f20-direct-dispatch"),
        &broker_receipt,
        "crown-bribery-case-engine-direct",
        STAGE3_ENGINE_SEED,
        STAGE3_MAX_POLLS,
        None,
    )?;
    let f20_direct_consequence_digest = f20_direct.consequence_digest.clone().ok_or_else(|| {
        CliError::MissingConsequenceDigest {
            dispatch_id: f20_direct.dispatch_id.clone(),
        }
    })?;

    let subworkflow = SubworkflowPlan {
        id: format!(
            "f18-{}-{}",
            broker_receipt.workflow_id, broker_receipt.step_id
        ),
        role: "single".to_string(),
        tape: bcinr_pddl::Pddl8Tape { ops: Vec::new() },
        model: CngPowl::Leaf(None),
        problem_pddl: String::new(),
        problem_digest: format!("blake3:{}", broker_receipt.consequence_hash_hex),
    };

    let reentry_policy = build_external_reentry_policy()?;
    let reentry_ledger = AdmissionLedger::new();

    let readmit = drive_external_readmit_transition(ExternalReentryRun {
        root: &run_dir.join("f20-readmit-transition"),
        subworkflow: &subworkflow,
        target_engine: "crown-bribery-case-engine-full".to_string(),
        engine_seed: STAGE3_ENGINE_SEED,
        max_polls: STAGE3_MAX_POLLS,
        poll_wait_ms: None,
        policy: &reentry_policy,
        ledger: &reentry_ledger,
        reentry_source_id: EXTERNAL_REENTRY_SOURCE.to_string(),
        reentry_principal_iri: EXTERNAL_REENTRY_PRINCIPAL.to_string(),
        reentry_subject_base_iri: format!("{SUBJECT}/external-reentry"),
        correlation_id: format!("crown-bribery-case-{run_id}-reentry-correlation-1"),
    })?;

    let crown_receipt = compute_stage3_crown_receipt(
        &f16_dispatch_token,
        &broker_receipt,
        &f20_direct,
        &f20_direct_consequence_digest,
        &readmit,
    );

    Ok(Stage3Outcome {
        f16_step_id,
        f16_dispatch_token,
        f16_transition_log,
        broker_receipt,
        f20_direct,
        f20_direct_consequence_digest,
        readmit,
        crown_receipt,
    })
}

/// Derives this run's `run-id` from `run_dir`'s own final path component -- the identical
/// computation `run()`'s own local `run_id` uses, kept as a shared helper so both call sites can
/// never drift.
///
/// # Complexity
/// O(1).
fn run_id_from_dir(run_dir: &Path) -> String {
    run_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown-run")
        .to_string()
}

/// Small machine-readable mirror of `08-case-closure.ttl`'s own real values, matching the
/// `07-arazzo-receipt.json` sibling-file precedent already established in this binary.
#[derive(serde::Serialize)]
struct CaseClosureSummary {
    case_status: &'static str,
    run_id: String,
    case_local_name: String,
    subject: String,
    admission_receipt_hash: String,
    f08_plan_chain_hash: String,
    f08_goal_reached: bool,
    stage3: Option<Stage3ClosureSummary>,
    blocked_reason: Option<String>,
}

#[derive(serde::Serialize)]
struct Stage3ClosureSummary {
    f16_dispatch_step_id: String,
    f16_dispatch_token: String,
    f18_broker_receipt_hash: String,
    f18_consequence_hash: String,
    f20_direct_dispatch_id: String,
    f20_direct_consequence_digest: String,
    f20_readmit_dispatch_id: String,
    f02_readmit_receipt_hash: String,
    f24_ocel_receipt_head: String,
    f25_receipt_root: String,
    f25_replayed_receipt_root: String,
    f25_receipt_root_matched: bool,
    stage3_crown_receipt: String,
}

/// Builds the case-closure Turtle text: a real PROV-O `prov:Activity` (`cls:closure-decision-
/// <run_id>`) plus a SKOS lifecycle `ConceptScheme` mirroring `pddl-problem-closable.ttl`'s own
/// `raw`/`validated`/`admitted`/`receipted`/`blocked` `lifecycle-stage` local names verbatim.
/// Asserts `cls:receipted` only if `stage3` is `Ok` (the real chain genuinely succeeded); asserts
/// `cls:blocked` plus the exact real typed refusal reason otherwise -- never silently converted
/// to `receipted` (this repo's "Bounded must never be reported as Exhausted" invariant).
///
/// # Complexity
/// O(1): a small, fixed number of `format!`/`push_str` calls.
fn build_case_closure_turtle(
    run_id: &str,
    case_local_name: &str,
    plan: &PipelineOutcome,
    stage3: Result<&Stage3Outcome, &CliError>,
) -> String {
    let mut out = String::new();
    out.push_str("@prefix prov: <http://www.w3.org/ns/prov#> .\n");
    out.push_str("@prefix skos: <http://www.w3.org/2004/02/skos/core#> .\n");
    out.push_str(&format!("@prefix sc: <{SC}> .\n"));
    out.push_str("@prefix cls: <urn:mfw:crown-bribery-case:closure#> .\n\n");

    out.push_str("cls:CaseLifecycle a skos:ConceptScheme .\n\n");
    out.push_str(
        "cls:raw a skos:Concept ; skos:inScheme cls:CaseLifecycle ; skos:prefLabel \"raw\" .\n",
    );
    out.push_str(
        "cls:validated a skos:Concept ; skos:inScheme cls:CaseLifecycle ; \
         skos:prefLabel \"validated\" ; skos:broader cls:raw .\n",
    );
    out.push_str(
        "cls:admitted a skos:Concept ; skos:inScheme cls:CaseLifecycle ; \
         skos:prefLabel \"admitted\" ; skos:broader cls:validated .\n",
    );
    out.push_str(
        "cls:receipted a skos:Concept ; skos:inScheme cls:CaseLifecycle ; \
         skos:prefLabel \"receipted\" ; skos:broader cls:admitted .\n",
    );
    out.push_str(
        "cls:blocked a skos:Concept ; skos:inScheme cls:CaseLifecycle ; \
         skos:prefLabel \"blocked\" ; skos:broader cls:admitted .\n\n",
    );

    let activity = format!("cls:closure-decision-{run_id}");
    match stage3 {
        Ok(s) => {
            out.push_str(&format!("<{SUBJECT}> sc:caseStatus cls:receipted .\n\n"));
            out.push_str(&format!("{activity} a prov:Activity ;\n"));
            out.push_str(&format!("  prov:used <{SUBJECT}> ;\n"));
            out.push_str(&format!("  cls:caseLocalName \"{case_local_name}\" ;\n"));
            out.push_str(&format!(
                "  cls:f08PlanReceiptHash \"{}\" ;\n",
                plan.receipt.chain_hash
            ));
            out.push_str(&format!(
                "  cls:f08GoalReached \"{}\"^^<http://www.w3.org/2001/XMLSchema#boolean> ;\n",
                plan.receipt.goal_reached
            ));
            out.push_str(&format!(
                "  cls:f16DispatchStepId \"{}\" ;\n",
                s.f16_step_id
            ));
            out.push_str(&format!(
                "  cls:f16DispatchToken \"{}\" ;\n",
                s.f16_dispatch_token
            ));
            out.push_str(&format!(
                "  cls:f18BrokerReceiptHash \"{}\" ;\n",
                s.broker_receipt.receipt_hash_hex
            ));
            out.push_str(&format!(
                "  cls:f18ConsequenceHash \"{}\" ;\n",
                s.broker_receipt.consequence_hash_hex
            ));
            out.push_str(&format!(
                "  cls:f20DirectDispatchId \"{}\" ;\n",
                s.f20_direct.dispatch_id
            ));
            out.push_str(&format!(
                "  cls:f20DirectConsequenceDigest \"{}\" ;\n",
                s.f20_direct_consequence_digest
            ));
            out.push_str(&format!(
                "  cls:f20ReadmitDispatchId \"{}\" ;\n",
                s.readmit.reentry.dispatch_outcome.dispatch_id
            ));
            out.push_str(&format!(
                "  cls:f02ReadmitReceiptHash \"{}\" ;\n",
                s.readmit.reentry.reentry_admission.receipt_hash
            ));
            out.push_str(&format!(
                "  cls:f24OcelReceiptHead \"{}\" ;\n",
                s.readmit.ocel_outcome.receipt_head
            ));
            out.push_str(&format!(
                "  cls:f25ReceiptRoot \"{}\" ;\n",
                s.readmit.replay_outcome.receipt.receipt_root.as_str()
            ));
            out.push_str(&format!(
                "  cls:f25ReplayedReceiptRoot \"{}\" ;\n",
                s.readmit.replay_outcome.replayed.receipt_root.as_str()
            ));
            out.push_str(&format!(
                "  cls:f25ReceiptRootMatched \
                 \"{}\"^^<http://www.w3.org/2001/XMLSchema#boolean> ;\n",
                s.readmit.replay_outcome.report.receipt_root_matched
            ));
            out.push_str(&format!(
                "  cls:stage3CrownReceipt \"{}\" .\n",
                s.crown_receipt
            ));
        }
        Err(e) => {
            let reason = e
                .to_string()
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            out.push_str(&format!("<{SUBJECT}> sc:caseStatus cls:blocked .\n\n"));
            out.push_str(&format!("{activity} a cls:RefusedClosure ;\n"));
            out.push_str(&format!("  cls:caseLocalName \"{case_local_name}\" ;\n"));
            out.push_str(&format!("  cls:refusalReason \"{reason}\" ;\n"));
            out.push_str(&format!("  prov:used <{SUBJECT}> .\n"));
        }
    }
    out
}

/// Writes the final case-closure (or typed refusal) RDF -- **regardless of whether Stage 3
/// succeeded or refused**, so a caller always finds a receipted-or-blocked verdict on disk -- plus
/// a machine-readable JSON summary sibling (`08-case-closure-summary.json`), matching the
/// `07-arazzo-receipt.json` precedent already established in this binary. See
/// [`build_case_closure_turtle`] for the RDF shape.
///
/// # Errors
/// [`CliError::ClosureRdfMalformed`] if the built Turtle fails to re-parse (defensive self-check,
/// matching `crown_external.rs`'s own `evidence_turtle` pattern: built from a compile-time-
/// controlled format string plus real digests, should be unreachable). [`CliError::Io`] /
/// [`CliError::Serialize`] on write/serialize failure.
fn write_case_closure(
    run_dir: &Path,
    case_local_name: &str,
    admission: &AdmissionReceipt,
    plan: &PipelineOutcome,
    stage3: Result<&Stage3Outcome, &CliError>,
) -> Result<(), CliError> {
    let run_id = run_id_from_dir(run_dir);
    let turtle = build_case_closure_turtle(&run_id, case_local_name, plan, stage3);

    Parser::parse_triples(&turtle, Syntax::Turtle).map_err(|reason| {
        CliError::ClosureRdfMalformed {
            reason: format!("case-closure Turtle self-check failed to re-parse: {reason}"),
        }
    })?;

    write_file(run_dir, "08-case-closure.ttl", &turtle)?;

    let summary = CaseClosureSummary {
        case_status: if stage3.is_ok() {
            "receipted"
        } else {
            "blocked"
        },
        run_id,
        case_local_name: case_local_name.to_string(),
        subject: SUBJECT.to_string(),
        admission_receipt_hash: admission.receipt_hash.clone(),
        f08_plan_chain_hash: plan.receipt.chain_hash.clone(),
        f08_goal_reached: plan.receipt.goal_reached,
        stage3: stage3.ok().map(|s| Stage3ClosureSummary {
            f16_dispatch_step_id: s.f16_step_id.clone(),
            f16_dispatch_token: s.f16_dispatch_token.clone(),
            f18_broker_receipt_hash: s.broker_receipt.receipt_hash_hex.clone(),
            f18_consequence_hash: s.broker_receipt.consequence_hash_hex.clone(),
            f20_direct_dispatch_id: s.f20_direct.dispatch_id.clone(),
            f20_direct_consequence_digest: s.f20_direct_consequence_digest.clone(),
            f20_readmit_dispatch_id: s.readmit.reentry.dispatch_outcome.dispatch_id.clone(),
            f02_readmit_receipt_hash: s.readmit.reentry.reentry_admission.receipt_hash.clone(),
            f24_ocel_receipt_head: s.readmit.ocel_outcome.receipt_head.clone(),
            f25_receipt_root: s
                .readmit
                .replay_outcome
                .receipt
                .receipt_root
                .as_str()
                .to_string(),
            f25_replayed_receipt_root: s
                .readmit
                .replay_outcome
                .replayed
                .receipt_root
                .as_str()
                .to_string(),
            f25_receipt_root_matched: s.readmit.replay_outcome.report.receipt_root_matched,
            stage3_crown_receipt: s.crown_receipt.clone(),
        }),
        blocked_reason: stage3.err().map(|e| e.to_string()),
    };
    let summary_json = serde_json::to_string_pretty(&summary).map_err(|e| CliError::Serialize {
        what: "case-closure summary",
        reason: e.to_string(),
    })?;
    write_file(run_dir, "08-case-closure-summary.json", &summary_json)?;

    Ok(())
}

/// Drives the full Stage-2 chain once, writing every stage's real
/// intermediate/final state under `run_dir`.
fn run(run_dir: &Path) -> Result<(), CliError> {
    fs::create_dir_all(run_dir).map_err(|e| CliError::Io {
        path: run_dir.display().to_string(),
        reason: e.to_string(),
    })?;

    // ---- Stage 1: F02 admission -------------------------------------------
    let policy = build_policy()?;
    let ledger = AdmissionLedger::new();
    let obs = RawObservation {
        correlation_id: "crown-bribery-case-run-1-intake".to_string(),
        source_id: SOURCE_ID.to_string(),
        declared_subject: SUBJECT.to_string(),
        payload_turtle: CASE_TTL.to_string(),
    };
    let admission = admit_observation(&policy, &ledger, obs)?;
    let admitted_path = write_file(run_dir, "01-admitted-case.ttl", CASE_TTL)?;
    println!(
        "[F02 admission]      state={:?} subject={} triples={} receipt_hash={} -> {}",
        admission.state,
        admission.subject_iri,
        admission.triple_count,
        admission.receipt_hash,
        admitted_path.display()
    );

    // ---- Stage 2: Knowledge Hook obligation derivation ---------------------
    let derived = derive_obligations(HOOK_TTL, CASE_TTL)?;
    let obligations_path = write_file(run_dir, "02-derived-obligations.ttl", &derived.turtle)?;
    println!(
        "[Knowledge Hook]     derived {} sc:hasObligation triples for <{}>: {:?} -> {}",
        derived.local_names.len(),
        SUBJECT,
        derived.local_names,
        obligations_path.display()
    );

    // ---- Stage 3: project obligations -> PDDL :init, manufacture PDDL8 ----
    let case_local_name = extract_case_local_name(CASE_TTL)?;
    let init_atoms =
        project_obligations_to_pddl_init(HOOK_TTL, &case_local_name, &derived.local_names)?;
    let evidence_types: Vec<String> = derived
        .local_names
        .iter()
        .map(|o| evidence_type_for_obligation(HOOK_TTL, o))
        .collect::<Result<Vec<_>, _>>()?;
    let problem_fragment = build_pddl_problem_fragment(
        &case_local_name,
        &derived.local_names,
        &evidence_types,
        &init_atoms,
    );
    let fragment_path = write_file(run_dir, "03-pddl-problem-fragment.ttl", &problem_fragment)?;
    println!(
        "[pddl: problem fragment] case_local_name={case_local_name} init_atoms={} -> {}",
        init_atoms.len() + 2,
        fragment_path.display()
    );

    let manufactured = manufacture_pddl(&problem_fragment, "crown-bribery-case runtime problem")?;
    let domain_path = write_file(
        run_dir,
        "04-pddl-domain.pddl",
        &manufactured.project_domain_text(),
    )?;
    let problem_path = write_file(
        run_dir,
        "04-pddl-problem.pddl",
        &manufactured.project_problem_text(),
    )?;
    println!(
        "[mfg::manufacture]   domain_bytes={} problem_bytes={} graph_hash={} -> {}, {}",
        manufactured.project_domain_text().len(),
        manufactured.project_problem_text().len(),
        manufactured.receipt.graph_hash,
        domain_path.display(),
        problem_path.display()
    );

    // ---- Stage 4: F08 planning ----------------------------------------------
    let f08_graph = vec![
        AdmittedTriple {
            subject: SUBJECT.to_string(),
            predicate: PDDL_DOMAIN_PREDICATE.to_string(),
            object_literal: manufactured.project_domain_text().clone(),
        },
        AdmittedTriple {
            subject: SUBJECT.to_string(),
            predicate: PDDL_PROBLEM_PREDICATE.to_string(),
            object_literal: manufactured.project_problem_text().clone(),
        },
        AdmittedTriple {
            subject: SUBJECT.to_string(),
            predicate: HOOK_PACK_PREDICATE.to_string(),
            object_literal: ACTION_HOOK_PACK_TTL.to_string(),
        },
    ];
    let case_id = format!(
        "crown-bribery-{}",
        admission
            .receipt_hash
            .get(..48)
            .unwrap_or(admission.receipt_hash.as_str())
    );
    let plan = run_pipeline(&f08_graph, &case_id)?;
    let plan_json = serde_json::to_string_pretty(&plan.tape).map_err(|e| CliError::Serialize {
        what: "F08 plan tape",
        reason: e.to_string(),
    })?;
    let plan_path = write_file(run_dir, "05-plan-tape.json", &plan_json)?;
    let plan_steps: Vec<&str> = plan
        .tape
        .ops
        .iter()
        .map(|op| op.action.schema_name.as_str())
        .collect();
    println!(
        "[F08 plan]           ops={} goal_reached={} steps={:?} -> {}",
        plan.tape.ops.len(),
        plan.receipt.goal_reached,
        plan_steps,
        plan_path.display()
    );

    // ---- Stage 5: F09 growth / F10 geometry (transitive) --------------------
    let (growth_root, growth_closure) = open_growth_root_and_closure()?;
    let residue = ResidueState {
        socket: growth_closure.socket().clone(),
        description: format!("crown-bribery-case continuation for {SUBJECT}"),
        domain_pddl: manufactured.project_domain_text().clone(),
        problem_pddl: manufactured.project_problem_text().clone(),
    };
    let goal = resolve_continuation_goal(&residue)?;
    let mut meter = DescentMeter::new(4);
    let growth_plan = plan_growth(true, &growth_closure, &goal, &mut meter)?;
    let growth = manufacture_and_bind_child(&growth_root, &growth_plan, ClosureLaw::AllRequired)?;
    let geometry_path = write_file(run_dir, "06-powl-v2-model.ttl", &growth.geometry_turtle)?;
    println!(
        "[F09/F10 growth]     leaves={} partial_orders={} choices={} child_bindings={} turtle_len={} -> {}",
        growth.geometry_shape.leaves,
        growth.geometry_shape.partial_orders,
        growth.geometry_shape.choices,
        growth.geometry_shape.child_bindings,
        growth.geometry_turtle.len(),
        geometry_path.display()
    );

    // ---- Stage 6: F13 Arazzo manufacture + REAL disk write -------------------
    let artifact = ArazzoProjectionReceipt::project_and_compile(
        &growth.geometry.root,
        "urn:mfw:crown-bribery-case/arazzo",
        Some(SUBJECT),
        "crown-bribery-case-workflow",
        "Solvane Global bribery-compliance case -- crown-bribery-case Stage 2 workflow",
        "26.7.12",
    )?;
    let arazzo_path = write_file(
        run_dir,
        "07-arazzo-artifact.json",
        &artifact.arazzo_document,
    )?;
    let receipt_json =
        serde_json::to_string_pretty(&artifact.receipt).map_err(|e| CliError::Serialize {
            what: "F13 Arazzo projection receipt",
            reason: e.to_string(),
        })?;
    let receipt_path = write_file(run_dir, "07-arazzo-receipt.json", &receipt_json)?;
    let snippet: String = artifact.arazzo_document.chars().take(240).collect();
    println!(
        "[F13 Arazzo]         bytes={} arazzo_digest={} air_digest_hex={} -> {}, {}",
        artifact.arazzo_document.len(),
        artifact.receipt.arazzo_digest_hex,
        artifact.receipt.air_digest_hex,
        arazzo_path.display(),
        receipt_path.display()
    );
    println!("[F13 Arazzo snippet] {snippet}...");

    // ---- Stage 7: F14 compile (Arazzo JSON -> AirProgram) --------------------
    let bump = Bump::new();
    let compiled = f14_wasm4pm_arazzo::compile(
        &artifact.arazzo_document,
        "https://cases.solvane-global.example.org/crown-bribery-case/arazzo-manufactured",
        &bump,
    )?;
    println!(
        "[F14 compile]        workflows={} steps={} air_digest_hex={}",
        compiled.program.workflows.len(),
        compiled
            .program
            .workflows
            .iter()
            .map(|w| w.steps.len())
            .sum::<usize>(),
        hex::encode(compiled.digest.0)
    );

    // ---- Stage 3: real F15->F16->F18->F20->F02(re-admit)->F15->F21->F24->F25 tail -----------
    // Built directly from this run's own real F08 plan tape (see
    // `build_bridge_workflow_from_plan`'s own doc comment for why: F13's projection template
    // emits no forward-edge routing). Runs regardless of whether it succeeds or refuses; the
    // case-closure RDF (and its JSON summary sibling) is written either way, never silently
    // skipped or converted to a fake `receipted` on refusal.
    // SAFETY note: `run_dir` is always constructed by this file's own `main()` as
    // `PathBuf::from(format!("target/crown-bribery-case/{run_id}"))`, so `file_name()` is always
    // `Some(valid utf8)`; still handled via `unwrap_or` inside `run_id_from_dir`, not `.expect()`,
    // as a defensive non-panic fallback for any future caller that passes a different `run_dir`.
    let run_id = run_id_from_dir(run_dir);
    let stage3_result = run_stage3(
        run_dir,
        &run_id,
        &plan_steps,
        &admission.receipt_hash,
        "crown-bribery-case-workflow",
    );
    write_case_closure(
        run_dir,
        &case_local_name,
        &admission,
        &plan,
        stage3_result.as_ref(),
    )?;
    match &stage3_result {
        Ok(s) => println!(
            "[F15->F25 EXTERNAL tail] OK -- f16_step={} f16_transition_log={:?} \
             f16_token_len={} broker_receipt={} f20_direct_dispatch={} readmit_dispatch={} \
             f24_receipt_head={} f25_receipt_root={} f25_matched={} crown_receipt={}",
            s.f16_step_id,
            s.f16_transition_log,
            s.f16_dispatch_token.len(),
            s.broker_receipt.receipt_hash_hex,
            s.f20_direct.dispatch_id,
            s.readmit.reentry.dispatch_outcome.dispatch_id,
            s.readmit.ocel_outcome.receipt_head,
            s.readmit.replay_outcome.receipt.receipt_root.as_str(),
            s.readmit.replay_outcome.report.receipt_root_matched,
            s.crown_receipt
        ),
        Err(e) => println!("[F15->F25 EXTERNAL tail] BLOCKED at: {e}"),
    }
    stage3_result?;

    println!("crown-bribery-case: OK -- full chain (F02 -> hook -> F08 -> F09/F10 -> F13 -> F14 -> F15/F16/F18/F20 -> F02(re-admit)/F15/F21/F24/F25) composed for real; artifacts under {}", run_dir.display());
    Ok(())
}

fn parse_args(argv: &[String]) -> Result<String, CliError> {
    let mut run_id = "bribery-case-run-1".to_string();
    let mut iter = argv.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!(
                    "crown-bribery-case: drive the Solvane Global bribery-compliance case \
                     fixture through F02 admission -> Knowledge Hook obligation derivation -> \
                     F08 PDDL planning -> F09/F10 growth+geometry -> F13 Arazzo manufacture -> \
                     F14 AIR compile, writing every real intermediate/final artifact to \
                     target/crown-bribery-case/<run-id>/"
                );
                println!("usage: crown-bribery-case [--run-id ID]");
                std::process::exit(0);
            }
            "--run-id" => {
                let value = iter
                    .next()
                    .ok_or_else(|| CliError::Usage("--run-id requires a value".to_string()))?;
                run_id = value.clone();
            }
            other => {
                return Err(CliError::Usage(format!(
                    "unexpected extra argument: {other}"
                )))
            }
        }
    }
    Ok(run_id)
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let outcome = parse_args(&argv).and_then(|run_id| {
        let run_dir = PathBuf::from(format!("target/crown-bribery-case/{run_id}"));
        run(&run_dir)
    });
    match outcome {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            eprintln!("crown-bribery-case: {err}");
            std::process::exit(err.exit_code());
        }
    }
}
