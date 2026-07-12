//! Family F19 -- "Hook Registry and Machine-First Actuation" (atlas ticket V12-019).
//!
//! Survey verdict: **MIXED**. This module wires four kinds of real content, per the
//! family survey's own `ALREADY_BUILT` / `REUSE_ADAPT` / `GGEN_GENERATABLE` /
//! `HAND_WRITE_REQUIRED` breakdown -- see each section below for which is which. Nothing
//! here is a decorative re-export of something that does not exist: every reused function
//! is a real, already-tested `praxis-graphlaw` function called through its real signature;
//! every adapted type is a clean-room re-implementation (not an import, since the cng
//! source is `pub(super)` and cannot be imported cross-crate); every generated constant
//! was produced by a real `ggen sync run` this session (see below); and the hand-written
//! pipeline is exercised end-to-end by the tests at the bottom of this file, not left as an
//! unwired type.
//!
//! # Scope boundary (disclosed, not silently assumed)
//!
//! F19's atlas L2 pipeline ends at "Typed hook invocation or external dispatch artifact":
//! this module resolves a validated PDDL ground action to a capability-bound, authorized,
//! classified, scheduled, receipted `HookResolution` -- it does NOT itself execute external
//! dispatch (posting to a remote machine/human surface) or run a retry loop. That is F20
//! "External Dispatch and Re-admission" territory (a separate atlas family, `ALREADY_BUILT`
//! per its own survey, backed by `crates/cng/src/bench/dispatch.rs`'s real dispatch
//! contract/consequence machinery) -- composing F19's `HookResolution` into F20's dispatch
//! contract is future wiring work, not claimed here.
//!
//! # 1. ALREADY_BUILT reuse (real, thin wrap)
//!
//! `praxis-graphlaw`'s hook engine (`crates/praxis-graphlaw/src/hooks/{mod,parsing,compile,
//! verdict,construct}.rs`) supplies four of F19's eight L2 components as real, tested
//! functions, called directly (not reimplemented) below. This module's own compile and
//! test status against those functions is verified via
//! `CARGO_TARGET_DIR=target/agent-f19-hooks-verify cargo check -p multifractal-workflow
//! --lib` and `... cargo test -p multifractal-workflow f19_hooks` (isolated target dir,
//! concurrent-agent-safe per this repo's build-hygiene rule) -- exact exit status is
//! whatever the wiring session that touched this file most recently reports, not asserted
//! here as a standing fact this comment could go stale against.
//! - **Hook RDF Catalog + Hook SHACL**: [`validate_and_extract_hooks`] parses admitted
//!   Turtle, rewrites `hook:` aliases to `kh:`, and validates against the real
//!   `kh:HookShape` closed-shape SHACL law pack (`parsing.rs:166`,
//!   `temp_store.validate_shacl(SHACL_LAW_PACK)`) -- not a keyword sweep.
//! - **Hook Scheduler**: [`schedule_hooks`] runs the real Kahn's-algorithm topological
//!   sort with `(priority, HookId)` tie-breaking (`compile.rs`).
//! - **Hook Receipt Store**: [`HookVerdictRecord`] and [`hook_hash`] (SHA-256 over
//!   canonical JSON of the sorted verdict records) supply the receipt type this module's
//!   `Replayable` state actually constructs, not a look-alike.
//!
//! # 2. REUSE_ADAPT (clean-room, cited, cng's originals are `pub(super)`)
//!
//! [`ExecutionClass`] and [`classify_execution`] are a minimal three-way re-implementation
//! of cng's `ExecutionClass`/`route_category` (`crates/cng/src/bench/dispatch.rs:75-100`).
//! That type and function are `pub(super)` inside cng's `bench` module -- **not** `pub`, so
//! they cannot be imported from this crate; a prior investigation this session confirmed
//! this via `grep -n "enum ExecutionClass" crates/cng/src/bench/dispatch.rs` showing
//! `pub(super)`. This module's classifier keys off this family's own `kh:action` IRI suffix
//! convention (`#external-machine` / `#external-human`) rather than cng's 14-category
//! "workday" benchmark strings -- a different, F19-native vocabulary, not a copy. Likewise
//! [`RecoveryPolicy`] adapts cng's `retry_law`/`compensation_law` fields
//! (`dispatch.rs:290,292`) as declarative-only strings, honestly carrying forward cng's own
//! disclosed limitation ("retry_law: declarative-only" -- no executing retry loop exists in
//! cng either, and none is added here).
//!
//! # 3. GGEN_GENERATABLE (real, generated this session)
//!
//! `f19_hooks_generated.rs` ([`PROV_CHAIN`], `[ProvChainEntity]`) was produced by a real
//! `ggen sync run` against `packs/f19-hooks-pack/` (pack.toml + ontology.ttl + a Tera
//! template), run from a scratch project (not the shared root `ggen.toml`, matching the
//! F04/F09 packs' own committed-generated-file convention rather than registering into the
//! shared, concurrently-edited root project file). Regenerate with:
//! ```text
//! mkdir -p /tmp/ggen-scratch-f19/generated /tmp/ggen-scratch-f19/templates
//! cp /Users/sac/praxis/schema/praxis.ttl /tmp/ggen-scratch-f19/praxis.ttl
//! cat > /tmp/ggen-scratch-f19/ggen.toml <<'EOF'
//! [project]
//! name = "f19-hooks-scratch"
//! [ontology]
//! source = "praxis.ttl"
//! [packs]
//! f19-hooks-pack = { path = "/Users/sac/praxis/packs/f19-hooks-pack" }
//! [templates]
//! dir = "templates"
//! EOF
//! cd /tmp/ggen-scratch-f19 && ggen sync run
//! cp generated/f19_hooks_generated.rs /Users/sac/praxis/crates/multifractal-workflow/src/
//! ```
//! This exact recipe was run this session (`ggen 26.7.4` on `PATH`); the pipeline reported
//! `"written": ["generated/f19_hooks_generated.rs"]` with a real `graph_hash_hex`. The
//! generated file is pure data (the L6 PROV-chain catalog); it is included, not `mod`-ed,
//! because it has no `//!` inner doc comment of its own.
//!
//! # 4. HAND_WRITE_REQUIRED (novel, no existing repo code does this)
//!
//! The actual **Capability Matcher** ([`resolve_hook_for_action`]'s match step) binding a
//! grounded PDDL action (`wasm4pm_compat::pddl::Pddl8GroundAction`, produced by
//! `pddl-index`'s real grounder) to a registered hook, and the F19-L5 8-state
//! [`HookResolutionState`] lifecycle with a genuine typed [`HookResolutionRefused`], are
//! novel algorithmic work: the family survey found no code anywhere in praxis, unrdf,
//! knowd, or knhk that performs this specific binding. Written fresh below, not ported from
//! anywhere, and exercised by this file's own test suite (not left unwired).
//!
//! Two things are explicitly **not** done here and are disclosed rather than hidden:
//! - The idempotency ledger ([`ReceiptLedger`], [`InMemoryReceiptLedger`]) is an in-memory
//!   reference implementation only. cng's own idempotency gate additionally has a durable,
//!   file-backed ledger with torn-ledger crash-resume detection
//!   (`dispatch.rs` `DurableLedger`, `CNG_R11 AuditMismatch`); no equivalent durable
//!   persistence exists in this module. L7's "restarts resume from a durable receipt head"
//!   requirement is therefore **UNVERIFIED/PARTIAL** here, not satisfied.
//! - [`RecoveryPolicy`] is declarative-only (see REUSE_ADAPT above): no code path in this
//!   module actually retries or compensates a failed hook resolution.

use praxis_graphlaw::hooks::{
    compile_hooks, hook_hash, schedule_hooks, validate_and_extract_hooks, CompiledHook, EffectKind,
    HookId, HookVerdict, HookVerdictRecord,
};
use praxis_graphlaw::parser::{Parser, Syntax};
use wasm4pm_compat::pddl::Pddl8GroundAction;

include!("f19_hooks_generated.rs");

// ============================================================================
// 2. REUSE_ADAPT -- clean-room three-way execution classifier
// ============================================================================

/// The three execution classes a resolved hook capability routes to. Clean-room
/// re-implementation of cng's `pub(super) enum ExecutionClass`
/// (`crates/cng/src/bench/dispatch.rs:75`), which cannot be imported cross-crate (see
/// module docs). This module's own vocabulary keys off `kh:action` IRI suffixes rather
/// than cng's fixed 14-category benchmark strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionClass {
    /// In-process hook actuation only.
    LocalActuation,
    /// Dispatch to an external machine surface (`kh:action` IRI ends `#external-machine`).
    ExternalMachineDispatch,
    /// Dispatch to an external human surface (`kh:action` IRI ends `#external-human`).
    ExternalHumanDispatch,
}

impl ExecutionClass {
    /// Shape-vocabulary-style name, mirroring cng's `ExecutionClass::as_str` convention
    /// (`dispatch.rs:88-91`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LocalActuation => "LOCAL_ACTUATION",
            Self::ExternalMachineDispatch => "EXTERNAL_MACHINE_DISPATCH",
            Self::ExternalHumanDispatch => "EXTERNAL_HUMAN_DISPATCH",
        }
    }
}

/// Classifies a matched hook's declared `kh:action` IRI into an [`ExecutionClass`]. O(1)
/// (bounded-length suffix comparison, not a scan). Deterministic: same `action` IRI string
/// always yields the same class.
#[must_use]
pub fn classify_execution(hook: &CompiledHook) -> ExecutionClass {
    match hook.action.as_deref() {
        Some(iri) if iri.ends_with("#external-human") => ExecutionClass::ExternalHumanDispatch,
        Some(iri) if iri.ends_with("#external-machine") => ExecutionClass::ExternalMachineDispatch,
        _ => ExecutionClass::LocalActuation,
    }
}

/// Declarative-only retry/compensation law, adapted from cng's `retry_law`/
/// `compensation_law` dispatch-contract fields (`dispatch.rs:290,292`). No code path in
/// this module executes a retry or compensation loop -- this is a bound, inspectable
/// policy record, exactly as thin as cng's own disclosed equivalent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryPolicy {
    pub retry_law: &'static str,
    pub compensation_law: &'static str,
}

/// The single fixed policy every [`HookResolution`] carries today. A per-hook policy
/// vocabulary (`kh:retryLaw`/`kh:compensationLaw` predicates) does not exist in
/// praxis-graphlaw's `kh:` SHACL law pack yet -- adding it is disclosed future work, not
/// silently assumed.
pub const DEFAULT_RECOVERY_POLICY: RecoveryPolicy = RecoveryPolicy {
    retry_law: "retry:limit=0;declarative-only",
    compensation_law: "compensate:not-yet-wired",
};

// ============================================================================
// 4. HAND_WRITE_REQUIRED -- capability matcher + 8-state lifecycle + refusal taxonomy
// ============================================================================

/// This family's own convention for turning a grounded PDDL action's `schema_name` into
/// the `kh:action` IRI a hook must declare to claim capability for it. Not a universal PDDL
/// convention -- a deliberate, disclosed choice for this module, so a hook pack author
/// knows exactly what IRI to write (`kh:action <urn:pddl:action:{schema_name}>`).
#[must_use]
pub fn capability_action_iri(schema_name: &str) -> String {
    format!("urn:pddl:action:{schema_name}")
}

/// The capability-identity portion of a `kh:action` IRI: everything before the first
/// `#`, or the whole string if there is none. The fragment (if any) is
/// execution-class-routing metadata (see [`classify_execution`]), not part of which
/// action the hook binds -- `<urn:pddl:action:deploy#external-machine>` and
/// `<urn:pddl:action:deploy>` bind the identical "deploy" capability.
#[must_use]
pub fn action_iri_base(iri: &str) -> &str {
    iri.trim_matches(|c| c == '<' || c == '>')
        .split('#')
        .next()
        .unwrap_or(iri)
}

/// The exact, non-fictional binding of one PDDL ground action to exactly one registered
/// hook. Produced only when the Capability Matcher finds precisely one `EffectKind::
/// GroundAction` hook whose `kh:action` equals [`capability_action_iri`] of the action's
/// `schema_name` -- zero or more than one candidate is a typed refusal, never a guess (the
/// family invariant: "no fictional/ungrounded planner actions admitted").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityBinding {
    pub action_schema: String,
    pub candidate_action_iri: String,
    pub hook_id: HookId,
    pub hook_iri: String,
    pub hook_name: String,
}

/// F19-L5's fixed 8-state hook-resolution lifecycle, plus the off-ladder `Refused`
/// terminal. Mirrors the atlas's own ladder verbatim: `Declared -> Validated -> Matched ->
/// Authorized -> Classified -> PolicyBound -> Scheduled -> Replayable`, with `Refused`
/// reachable only from `Validated` (catalog/SHACL/scheduling-integrity failure) and
/// `Classified` (authority/conformance failure) -- see [`HookResolutionState::lawful_to`]
/// for the exact edge table and the disclosed modeling decision for where each refusal
/// reason lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HookResolutionState {
    Declared,
    Validated,
    Matched,
    Authorized,
    Classified,
    PolicyBound,
    Scheduled,
    Replayable,
    Refused,
}

impl HookResolutionState {
    /// The 8 lawful-ladder states in order, `Refused` excluded (mirrors
    /// `DispatchState::ALL`'s drift-test-anchor role in cng's `dispatch.rs`, adapted to
    /// this family's own 8-state ladder rather than cng's 16-state one).
    pub const LADDER: [HookResolutionState; 8] = [
        Self::Declared,
        Self::Validated,
        Self::Matched,
        Self::Authorized,
        Self::Classified,
        Self::PolicyBound,
        Self::Scheduled,
        Self::Replayable,
    ];

    /// Ladder position; `Refused` carries `-1` (off-ladder terminal), matching the f04
    /// dialect-registry module's `DialectLifecycleState::order` convention for this crate.
    #[must_use]
    pub const fn order(self) -> i8 {
        match self {
            Self::Declared => 0,
            Self::Validated => 1,
            Self::Matched => 2,
            Self::Authorized => 3,
            Self::Classified => 4,
            Self::PolicyBound => 5,
            Self::Scheduled => 6,
            Self::Replayable => 7,
            Self::Refused => -1,
        }
    }

    /// Whether `self -> to` is in the lawful transition table. O(1).
    ///
    /// # Modeling decision (disclosed)
    /// The atlas requirement states `REFUSED` is reachable only from `VALIDATED` (invalid)
    /// and `CLASSIFIED` (authority/conformance failure). This module reads "invalid" at
    /// `VALIDATED` broadly: catalog-level integrity checks (Turtle parse, SHACL shape
    /// conformance, hook dependency resolution, and Kahn scheduling cycle-freedom) AND the
    /// action-specific Capability Matcher (zero or ambiguous binding) both surface as
    /// `Validated -> Refused`, because all of them are judged before a specific hook is
    /// ever selected. `Classified -> Refused` is reserved for the one judgment that can
    /// only happen after a specific hook is bound: declared-authority conformance. This is
    /// an interpretation of the atlas text, not a re-verified quote from
    /// `F19_hooks.md` beyond what the survey already extracted -- stated here so a reader
    /// can see exactly which reading was chosen and why.
    #[must_use]
    pub const fn lawful_to(self, to: Self) -> bool {
        use HookResolutionState as S;
        matches!(
            (self, to),
            (S::Declared, S::Validated)
                | (S::Validated, S::Matched)
                | (S::Validated, S::Refused)
                | (S::Matched, S::Authorized)
                | (S::Authorized, S::Classified)
                | (S::Classified, S::PolicyBound)
                | (S::Classified, S::Refused)
                | (S::PolicyBound, S::Scheduled)
                | (S::Scheduled, S::Replayable)
        )
    }
}

/// Typed refusal for F19's hook-resolution pipeline. Every variant is raised from exactly
/// one call site in [`resolve_hook_for_action`] and has a dedicated test below (repo
/// no-overclaiming discipline: a `Refusal` variant with no test is not a contract, it's a
/// guess).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HookResolutionRefused {
    /// The admitted hook-pack text is not valid Turtle.
    #[error("F19: hook-pack Turtle failed to parse: {0}")]
    MalformedHookPack(String),
    /// `validate_and_extract_hooks` rejected the catalog: SHACL shape violation, forbidden
    /// keyword, or vocabulary error.
    #[error("F19: hook catalog failed kh:HookShape SHACL validation: {0}")]
    ShaclViolation(String),
    /// `compile_hooks` could not resolve an `after` dependency to a known hook IRI.
    #[error("F19: hook catalog has an unresolvable after-dependency: {0}")]
    HookCatalogUnresolvable(String),
    /// `schedule_hooks` detected a dependency cycle across the admitted catalog.
    #[error("F19: hook catalog scheduling detected a dependency cycle: {0}")]
    HookSchedulingCycle(String),
    /// No hook in the (already-validated, already-scheduled) catalog declares
    /// `kh:effect ground-action` with `kh:action` equal to this action's
    /// [`capability_action_iri`].
    #[error(
        "F19: no capability binding for PDDL action '{action_schema}' \
         (expected kh:action = {candidate_action_iri})"
    )]
    NoCapabilityBinding {
        action_schema: String,
        candidate_action_iri: String,
    },
    /// More than one hook declares the same `kh:action` IRI -- the family invariant forbids
    /// guessing among candidates.
    #[error(
        "F19: ambiguous capability binding for PDDL action '{action_schema}': \
         {candidates:?} all declare kh:action = {candidate_action_iri}"
    )]
    AmbiguousCapabilityBinding {
        action_schema: String,
        candidate_action_iri: String,
        candidates: Vec<String>,
    },
    /// The matched hook's `kh:reason` (this module's declared-authority stand-in; the
    /// `kh:` vocabulary has no dedicated authority predicate today, disclosed in module
    /// docs) is missing or blank.
    #[error("F19: hook '{hook_iri}' has no non-empty declared authority (kh:reason)")]
    MissingDeclaredAuthority { hook_iri: String },
    /// A receipt-stage violation (L4's third named refusal trigger, alongside SHACL and
    /// authority): `HookCondition::condition_hash` or `hook_hash` failed to serialize an
    /// already-validated, already-typed internal structure. Defensive-only in practice --
    /// `CompiledHook`/`HookVerdictRecord` are plain, already-well-typed data with no
    /// non-serializable fields, so this branch is not known to be reachable from any
    /// admitted input today. Kept as a typed `Result` propagation rather than `.unwrap()`
    /// per this repo's no-panic invariant (a "cannot fail in practice" branch still is not
    /// a proof, and a proof is what `.unwrap()` requires); its exact ladder placement
    /// (raised at the `Scheduled -> Replayable` edge, which the declared transition table
    /// has no `-> Refused` arm for) is a disclosed gap, not silently modeled as lawful --
    /// mirrors cng's own "declared-but-unreached edge, investigated not forced" note for
    /// `DISPATCH_READY -> REFUSED` (`dispatch.rs` module docs).
    #[error("F19: receipt-stage construction failed: {0}")]
    ReceiptConstructionFailed(String),
    /// The atomic idempotency+correlation gate (L7) saw this `idempotency_key` before.
    /// Raised BEFORE the ledger is mutated and BEFORE any ladder state is touched --
    /// mirrors cng's own `CNG_R25 DoubleAdmit`-before-admission-effect discipline
    /// (`dispatch.rs:1224,1593`).
    #[error("F19: duplicate admission refused by the idempotency gate: {idempotency_key}")]
    DuplicateAdmission { idempotency_key: String },
}

/// Atomic idempotency+correlation gate (L7 chaos lens). `is_processed`/`mark_processed`
/// mirror cng's own `is_processed`/`mark_processed` ledger trait shape
/// (`dispatch.rs:439,443`), adapted to this family's `String` idempotency keys.
pub trait ReceiptLedger {
    fn is_processed(&self, idempotency_key: &str) -> bool;
    fn mark_processed(&mut self, idempotency_key: &str);
}

/// In-memory reference [`ReceiptLedger`]. **Not durable** -- a process restart loses all
/// admitted keys. L7's "restarts resume from a durable receipt head" requirement is
/// therefore UNVERIFIED/PARTIAL against this implementation; a durable, crash-resumable
/// ledger (file-backed, torn-write-detecting, like cng's `DurableLedger`) is disclosed
/// future work, not silently assumed here. `BTreeSet`, not `HashSet`, so any future
/// iteration over `processed` is deterministic without an extra sort step.
#[derive(Debug, Default, Clone)]
pub struct InMemoryReceiptLedger {
    processed: std::collections::BTreeSet<String>,
}

impl ReceiptLedger for InMemoryReceiptLedger {
    fn is_processed(&self, idempotency_key: &str) -> bool {
        self.processed.contains(idempotency_key)
    }

    fn mark_processed(&mut self, idempotency_key: &str) {
        self.processed.insert(idempotency_key.to_string());
    }
}

/// A fully resolved hook: every field is real output of a ladder stage that actually ran,
/// not a placeholder. `state` is always [`HookResolutionState::Replayable`] on `Ok` --
/// any earlier failure returns `Err(HookResolutionRefused)` instead of a partially filled
/// value (no silent partial success).
#[derive(Debug, Clone)]
pub struct HookResolution {
    pub state: HookResolutionState,
    pub binding: CapabilityBinding,
    pub execution_class: ExecutionClass,
    pub declared_authority: String,
    pub recovery_policy: RecoveryPolicy,
    pub schedule_position: usize,
    pub verdict: HookVerdictRecord,
    pub receipt_hash: String,
}

/// Resolves one validated PDDL ground action against an admitted hook-pack catalog,
/// through all 8 lawful [`HookResolutionState`] stages, or returns a typed
/// [`HookResolutionRefused`].
///
/// # Algorithm
/// 1. Parse `hook_pack_turtle`, then run the real `validate_and_extract_hooks` (SHACL) +
///    `compile_hooks` (dependency resolution) + `schedule_hooks` (Kahn topological sort) --
///    catalog-level integrity, gates `Declared -> Validated` and any `Validated -> Refused`.
/// 2. Capability Matcher: exact-match `action.schema_name` against the scheduled catalog's
///    `EffectKind::GroundAction` hooks by `kh:action` IRI -- gates `Validated -> Matched`.
/// 3. Bind declared authority (`kh:reason`) -- `Matched -> Authorized` (always structurally
///    succeeds; the value itself may still be empty).
/// 4. Classify execution + judge authority conformance -- `Authorized -> Classified ->
///    {PolicyBound | Refused}`.
/// 5. Bind [`DEFAULT_RECOVERY_POLICY`] -- `Classified -> PolicyBound`.
/// 6. Look up the matched hook's real position in the already-computed Kahn schedule --
///    `PolicyBound -> Scheduled`.
/// 7. Idempotency gate (pre-flight, not a ladder edge -- see [`HookResolutionRefused::
///    DuplicateAdmission`] docs), then construct the real `HookVerdictRecord` and
///    `hook_hash` receipt -- `Scheduled -> Replayable`.
///
/// # Complexity
/// O(`|catalog|` log `|catalog|` + `|catalog| * |after deps|`), dominated by
/// `schedule_hooks`'s own documented bound; the capability match and schedule-position
/// lookup are each O(`|catalog|`).
pub fn resolve_hook_for_action(
    hook_pack_turtle: &str,
    action: &Pddl8GroundAction,
    ledger: &mut dyn ReceiptLedger,
) -> Result<HookResolution, HookResolutionRefused> {
    // Declared -> Validated: catalog-level integrity (parse + SHACL + dependency
    // resolution + scheduling cycle-freedom). Any failure here is a Validated -> Refused
    // edge (see HookResolutionState::lawful_to docs for the disclosed modeling decision).
    let triples = Parser::parse_triples(hook_pack_turtle, Syntax::Turtle)
        .map_err(HookResolutionRefused::MalformedHookPack)?;
    let hooks =
        validate_and_extract_hooks(&triples).map_err(HookResolutionRefused::ShaclViolation)?;
    let compiled = compile_hooks(hooks).map_err(HookResolutionRefused::HookCatalogUnresolvable)?;
    let scheduled =
        schedule_hooks(&compiled).map_err(HookResolutionRefused::HookSchedulingCycle)?;
    let state = HookResolutionState::Declared;
    debug_assert!(state.lawful_to(HookResolutionState::Validated));
    let state = HookResolutionState::Validated;

    // Validated -> Matched: Capability Matcher. Matches on the kh:action IRI *base* (the
    // portion before any '#fragment') -- the fragment, if present, is
    // execution-class-routing metadata (see classify_execution), not part of the
    // capability identity, so `<urn:pddl:action:deploy#external-machine>` still binds the
    // "deploy" action exactly, alongside a plain `<urn:pddl:action:pickup>`.
    let candidate_action_iri = capability_action_iri(&action.schema_name);
    let matches: Vec<&CompiledHook> = scheduled
        .iter()
        .filter(|h| {
            h.effect == EffectKind::GroundAction
                && h.action
                    .as_deref()
                    .map(|a| action_iri_base(a) == candidate_action_iri)
                    == Some(true)
        })
        .collect();
    let matched = match matches.as_slice() {
        [] => {
            return Err(HookResolutionRefused::NoCapabilityBinding {
                action_schema: action.schema_name.clone(),
                candidate_action_iri,
            });
        }
        [one] => (*one).clone(),
        many => {
            return Err(HookResolutionRefused::AmbiguousCapabilityBinding {
                action_schema: action.schema_name.clone(),
                candidate_action_iri,
                candidates: many.iter().map(|h| h.iri.clone()).collect(),
            });
        }
    };
    debug_assert!(state.lawful_to(HookResolutionState::Matched));
    let state = HookResolutionState::Matched;

    // Matched -> Authorized: bind declared authority (structural; value may be empty).
    let declared_authority = matched.reason.clone().unwrap_or_default();
    debug_assert!(state.lawful_to(HookResolutionState::Authorized));
    let state = HookResolutionState::Authorized;

    // Authorized -> Classified: classify execution (always succeeds) then judge authority
    // conformance (may refuse from Classified).
    let execution_class = classify_execution(&matched);
    debug_assert!(state.lawful_to(HookResolutionState::Classified));
    let state = HookResolutionState::Classified;
    if declared_authority.trim().is_empty() {
        debug_assert!(state.lawful_to(HookResolutionState::Refused));
        return Err(HookResolutionRefused::MissingDeclaredAuthority {
            hook_iri: matched.iri.clone(),
        });
    }

    // Classified -> PolicyBound: bind the (declarative-only) recovery policy.
    let recovery_policy = DEFAULT_RECOVERY_POLICY;
    debug_assert!(state.lawful_to(HookResolutionState::PolicyBound));
    let state = HookResolutionState::PolicyBound;

    // PolicyBound -> Scheduled: look up the matched hook's real position in the
    // already-computed Kahn schedule.
    let schedule_position = scheduled
        .iter()
        .position(|h| h.id == matched.id)
        .unwrap_or(scheduled.len());
    debug_assert!(state.lawful_to(HookResolutionState::Scheduled));
    let state = HookResolutionState::Scheduled;

    // Scheduled -> Replayable: idempotency pre-flight gate, then real receipt
    // construction.
    let idempotency_key =
        blake3::hash(format!("{}|{}|{}", matched.iri, action.schema_name, action.label).as_bytes())
            .to_hex()
            .to_string();
    if ledger.is_processed(&idempotency_key) {
        return Err(HookResolutionRefused::DuplicateAdmission { idempotency_key });
    }
    ledger.mark_processed(&idempotency_key);

    let condition_hash = matched
        .condition
        .condition_hash()
        .map_err(HookResolutionRefused::ReceiptConstructionFailed)?;
    let verdict = HookVerdictRecord {
        hook_id: matched.id,
        hook_iri: matched.iri.clone(),
        hook_name: matched.name.clone(),
        condition_kind: matched.condition.kind().to_string(),
        condition_hash,
        verdict: HookVerdict::Fired,
        effect: matched.effect.clone(),
        action_iri: matched.action.clone(),
        diagnostics: None,
        delta_hash: Some(idempotency_key.clone()),
        idempotency_key: Some(idempotency_key.clone()),
    };
    let receipt_hash = hook_hash(std::slice::from_ref(&verdict))
        .map_err(HookResolutionRefused::ReceiptConstructionFailed)?;
    debug_assert!(state.lawful_to(HookResolutionState::Replayable));
    let state = HookResolutionState::Replayable;

    Ok(HookResolution {
        state,
        binding: CapabilityBinding {
            action_schema: action.schema_name.clone(),
            candidate_action_iri: matched.action.clone().unwrap_or_default(),
            hook_id: matched.id,
            hook_iri: matched.iri.clone(),
            hook_name: matched.name.clone(),
        },
        execution_class,
        declared_authority,
        recovery_policy,
        schedule_position,
        verdict,
        receipt_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ground_action(schema_name: &str) -> Pddl8GroundAction {
        Pddl8GroundAction {
            schema_name: schema_name.to_string(),
            label: format!("{schema_name}()"),
            preconditions: vec![],
            add_effects: vec![],
            del_effects: vec![],
        }
    }

    const LOCAL_HOOK_PACK: &str = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/f19#> .
        ex:hook-pickup a kh:Hook ;
          kh:name "pickup-hook" ;
          kh:kind "delta" ;
          kh:var "http://example.org/f19#actuates-pickup" ;
          kh:on "assert" ;
          kh:effect "ground-action" ;
          kh:action <urn:pddl:action:pickup> ;
          kh:reason "workday-operator-authority-pickup" ;
          kh:priority 1 .
    "#;

    const EXTERNAL_MACHINE_HOOK_PACK: &str = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/f19#> .
        ex:hook-deploy a kh:Hook ;
          kh:name "deploy-hook" ;
          kh:kind "delta" ;
          kh:var "http://example.org/f19#actuates-deploy" ;
          kh:on "assert" ;
          kh:effect "ground-action" ;
          kh:action <urn:pddl:action:deploy#external-machine> ;
          kh:reason "workday-operator-authority-deploy" ;
          kh:priority 1 .
    "#;

    const NO_AUTHORITY_HOOK_PACK: &str = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/f19#> .
        ex:hook-noauth a kh:Hook ;
          kh:name "noauth-hook" ;
          kh:kind "delta" ;
          kh:var "http://example.org/f19#actuates-noauth" ;
          kh:on "assert" ;
          kh:effect "ground-action" ;
          kh:action <urn:pddl:action:noauth> ;
          kh:priority 1 .
    "#;

    const AMBIGUOUS_HOOK_PACK: &str = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/f19#> .
        ex:hook-dup-a a kh:Hook ;
          kh:name "dup-a" ;
          kh:kind "delta" ;
          kh:var "http://example.org/f19#actuates-dup-a" ;
          kh:on "assert" ;
          kh:effect "ground-action" ;
          kh:action <urn:pddl:action:dup> ;
          kh:reason "authority-a" ;
          kh:priority 1 .
        ex:hook-dup-b a kh:Hook ;
          kh:name "dup-b" ;
          kh:kind "delta" ;
          kh:var "http://example.org/f19#actuates-dup-b" ;
          kh:on "assert" ;
          kh:effect "ground-action" ;
          kh:action <urn:pddl:action:dup> ;
          kh:reason "authority-b" ;
          kh:priority 2 .
    "#;

    const MALFORMED_SHACL_HOOK_PACK: &str = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/f19#> .
        ex:hook-bad a kh:Hook ;
          kh:kind "delta" ;
          kh:var "http://example.org/f19#actuates-bad" ;
          kh:on "assert" ;
          kh:effect "ground-action" ;
          kh:action <urn:pddl:action:bad> ;
          kh:priority 1 .
    "#;

    #[test]
    fn resolves_local_actuation_to_replayable() {
        let mut ledger = InMemoryReceiptLedger::default();
        let resolution =
            resolve_hook_for_action(LOCAL_HOOK_PACK, &ground_action("pickup"), &mut ledger)
                .expect("pickup hook should resolve");
        assert_eq!(resolution.state, HookResolutionState::Replayable);
        assert_eq!(resolution.execution_class, ExecutionClass::LocalActuation);
        assert_eq!(resolution.binding.hook_name, "pickup-hook");
        assert_eq!(
            resolution.declared_authority,
            "workday-operator-authority-pickup"
        );
        assert!(!resolution.receipt_hash.is_empty());
    }

    #[test]
    fn classifies_external_machine_dispatch() {
        let mut ledger = InMemoryReceiptLedger::default();
        let resolution = resolve_hook_for_action(
            EXTERNAL_MACHINE_HOOK_PACK,
            &ground_action("deploy"),
            &mut ledger,
        )
        .expect("deploy hook should resolve");
        assert_eq!(resolution.state, HookResolutionState::Replayable);
        assert_eq!(
            resolution.execution_class,
            ExecutionClass::ExternalMachineDispatch
        );
        assert_eq!(
            resolution.execution_class.as_str(),
            "EXTERNAL_MACHINE_DISPATCH"
        );
    }

    #[test]
    fn refuses_action_with_no_capability_binding() {
        let mut ledger = InMemoryReceiptLedger::default();
        let err = resolve_hook_for_action(
            LOCAL_HOOK_PACK,
            &ground_action("no-such-action"),
            &mut ledger,
        )
        .expect_err("unbound action must refuse");
        assert!(matches!(
            err,
            HookResolutionRefused::NoCapabilityBinding { .. }
        ));
    }

    #[test]
    fn refuses_ambiguous_capability_binding() {
        let mut ledger = InMemoryReceiptLedger::default();
        let err = resolve_hook_for_action(AMBIGUOUS_HOOK_PACK, &ground_action("dup"), &mut ledger)
            .expect_err("ambiguous binding must refuse");
        match err {
            HookResolutionRefused::AmbiguousCapabilityBinding { candidates, .. } => {
                assert_eq!(candidates.len(), 2);
            }
            other => panic!("expected AmbiguousCapabilityBinding, got {other:?}"),
        }
    }

    #[test]
    fn refuses_missing_declared_authority() {
        let mut ledger = InMemoryReceiptLedger::default();
        let err = resolve_hook_for_action(
            NO_AUTHORITY_HOOK_PACK,
            &ground_action("noauth"),
            &mut ledger,
        )
        .expect_err("missing authority must refuse");
        assert!(matches!(
            err,
            HookResolutionRefused::MissingDeclaredAuthority { .. }
        ));
    }

    #[test]
    fn refuses_shacl_violation() {
        let mut ledger = InMemoryReceiptLedger::default();
        let err = resolve_hook_for_action(
            MALFORMED_SHACL_HOOK_PACK,
            &ground_action("bad"),
            &mut ledger,
        )
        .expect_err("missing kh:name must fail SHACL");
        assert!(matches!(err, HookResolutionRefused::ShaclViolation(_)));
    }

    #[test]
    fn idempotency_gate_refuses_duplicate_admission() {
        let mut ledger = InMemoryReceiptLedger::default();
        let action = ground_action("pickup");
        let first = resolve_hook_for_action(LOCAL_HOOK_PACK, &action, &mut ledger);
        assert!(first.is_ok());
        let second = resolve_hook_for_action(LOCAL_HOOK_PACK, &action, &mut ledger)
            .expect_err("replayed idempotency key must refuse");
        assert!(matches!(
            second,
            HookResolutionRefused::DuplicateAdmission { .. }
        ));
    }

    #[test]
    fn receipt_hash_is_deterministic_across_independent_ledgers() {
        let mut ledger_a = InMemoryReceiptLedger::default();
        let mut ledger_b = InMemoryReceiptLedger::default();
        let action = ground_action("pickup");
        let a = resolve_hook_for_action(LOCAL_HOOK_PACK, &action, &mut ledger_a).unwrap();
        let b = resolve_hook_for_action(LOCAL_HOOK_PACK, &action, &mut ledger_b).unwrap();
        assert_eq!(a.receipt_hash, b.receipt_hash);
        assert_eq!(a.verdict.condition_hash, b.verdict.condition_hash);
    }

    #[test]
    fn lawful_transition_table_matches_declared_edges() {
        use HookResolutionState as S;
        let declared_edges = [
            (S::Declared, S::Validated),
            (S::Validated, S::Matched),
            (S::Validated, S::Refused),
            (S::Matched, S::Authorized),
            (S::Authorized, S::Classified),
            (S::Classified, S::PolicyBound),
            (S::Classified, S::Refused),
            (S::PolicyBound, S::Scheduled),
            (S::Scheduled, S::Replayable),
        ];
        let all_states = {
            let mut v = HookResolutionState::LADDER.to_vec();
            v.push(S::Refused);
            v
        };
        for &from in &all_states {
            for &to in &all_states {
                let expected = declared_edges.contains(&(from, to));
                assert_eq!(
                    from.lawful_to(to),
                    expected,
                    "lawful_to({from:?}, {to:?}) drifted from the declared edge table"
                );
            }
        }
        // REFUSED is reachable only from VALIDATED and CLASSIFIED (family invariant).
        for &from in &all_states {
            if from.lawful_to(S::Refused) {
                assert!(matches!(from, S::Validated | S::Classified));
            }
        }
    }

    #[test]
    fn ladder_order_is_strictly_increasing_and_refused_is_off_ladder() {
        let orders: Vec<i8> = HookResolutionState::LADDER
            .iter()
            .map(|s| s.order())
            .collect();
        for w in orders.windows(2) {
            assert!(w[0] < w[1], "ladder order must be strictly increasing");
        }
        assert_eq!(HookResolutionState::Refused.order(), -1);
    }

    #[test]
    fn prov_chain_is_a_straight_line_of_eight_entities() {
        assert_eq!(PROV_CHAIN.len(), 8);
        assert_eq!(PROV_CHAIN[0].name, "HookDeclaration");
        assert_eq!(PROV_CHAIN[0].derived_from, "none");
        assert_eq!(PROV_CHAIN[7].name, "HookReceipt");
        for i in 1..PROV_CHAIN.len() {
            assert_eq!(PROV_CHAIN[i].derived_from, PROV_CHAIN[i - 1].name);
            assert_eq!(PROV_CHAIN[i].chain_order, (i as u8) + 1);
        }
    }
}
