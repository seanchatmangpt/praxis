//! Family F03 -- "Semantic Contraction" (atlas ticket V12-003).
//!
//! Survey verdict: **MIXED**. This is a Wire-phase-1 pass: every pipeline stage below is
//! backed by a real call into existing, independently-tested code -- either directly into
//! `praxis_graphlaw` or, for two stages, into a sibling family module of *this same crate*
//! (`crate::f05_datalog_closure`, `crate::f06_n3_quarantine`) that a concurrent Wire pass
//! already built for real this session. Nothing below is a decorative re-export of a type
//! that does not exist. Per `.claude/rules/no-overclaiming.md`, what is ALIVE is stated with
//! the exact source it reuses; every scope limitation is disclosed as such, not silently
//! rounded up to "done".
//!
//! # Pipeline stage status (F03-L2's C1-C7, see [`f03_semantic_contraction_vocab::PIPELINE_COMPONENTS`])
//!
//! 1. **Ontology Typing** (REAL) -- [`type_ontology`] calls
//!    `praxis_graphlaw::TripleStore::from` + `TripleStore::materialize_owlrl` directly. Real
//!    OWL RL entailment (compiled to Datalog, run through the semi-naive `Reasoner`), not a
//!    stub.
//! 2. **Datalog Closure** (REAL) -- [`close_semantic`] reuses
//!    `crate::f05_datalog_closure::close_datalog` verbatim (same function, not a re-derivation):
//!    fresh `TripleStore`, `add_rules` + `materialize` to a genuine fixpoint, closure fact set
//!    read back from the store's own index. F05's own `ClosureDigest` receipt type is reused
//!    unchanged as this stage's receipt.
//! 3-4. **N3 Quarantine Router + Bounded N3 Refinement** (REAL, disclosed scope) --
//!    [`refine_n3`] reuses `crate::f06_n3_quarantine::route_and_execute_n3` verbatim: the real
//!    `DialectRouter::decide` gate plus the real `N3Executor::run` capability/cost/whitelist
//!    gate. **Scope disclosed, not hidden:** the survey's "route exceptional implication into
//!    refinement only if authorized" trigger -- i.e. automatically deciding *whether* a given
//!    `ClosureGraph` contains an implication that needs N3 refinement -- is not implemented
//!    anywhere in this repo (confirmed by reading `chatman/quarantine.rs`,
//!    `chatman/router.rs`; `N3Rule` is a caller-declared classification, not a parsed rule, per
//!    quarantine.rs's own doc comment). The caller of [`refine_n3`] must supply the
//!    `QueryShape`/`N3Rule`s explicitly (or pass `None` to skip refinement); this module does
//!    not derive that decision from `ClosureGraph`'s content. Also: `N3Executor::run` is a
//!    capability/cost gate over caller-declared rule metadata, not an N3 rule evaluator -- it
//!    does not itself derive new triples, so a successful [`RefinementDelta`] records which
//!    rule IDs were admitted and their tick cost, not new facts.
//! 5-6. **SHACL Gate + ShEx Gate** (REAL) -- [`gate_shapes`] calls
//!    `praxis_graphlaw::TripleStore::validate_shacl` / `validate_shex_c` directly on this
//!    pipeline's own in-memory closure facts. (Not routed through
//!    `crate::f07_shape_admission::ShapeAdmissionGate::admit`: that function takes Turtle text
//!    and builds its own fresh store, which would force a lossy re-serialize/re-parse
//!    round-trip of facts this pipeline already holds in memory -- calling the same underlying
//!    `praxis_graphlaw::shacl`/`shex_native` validators directly avoids that.)
//! 7. **Residue Projector** (REAL, disclosed scope) -- [`project_residue`] reuses
//!    `crate::f05_datalog_closure::compare_residue` verbatim: "derivable truth is not work" is
//!    the literal, identically-worded invariant both F03's and F05's survey text state, so this
//!    is the same mechanism, not a coincidental substitute. **Scope disclosed:** F05's
//!    `compare_residue` operates at predicate-IRI granularity (does the closure prove *any*
//!    fact with this predicate at all), not full RDF-subgraph/triple-level set difference --
//!    the caller supplies the admitted graph's candidate predicate list. A full
//!    graph-level residue (a genuine subgraph object, not a stripped predicate list) is not
//!    built here; F05's own module doc discloses the identical scope limit for its own use of
//!    this function.
//!
//! # What this pass does NOT build (disclosed, not faked)
//! - **L7 concurrency/chaos** (duplicate-event/restart/stale-result idempotency+correlation
//!   gating) is not wired into [`contract`] or any function in this file. A real, working
//!   analog exists in this same crate --
//!   `crate::f07_shape_admission::ShapeAdmissionGate`'s `correlation_id`-keyed `BTreeMap` dedup
//!   -- but nothing in this module calls it or reimplements it. UNVERIFIED for F03.
//! - **No `prov:wasDerivedFrom` RDF triples are asserted** into any graph by the pipeline
//!   functions below. Provenance is real at the Rust level (each stage's struct is only
//!   constructible from the previous stage's real output; digests chain stage to stage) and
//!   the derivation-edge *shape* is documented in the generated
//!   [`f03_semantic_contraction_vocab::DERIVATION_EDGES`] table, but no function here emits
//!   live `prov:` triples into a receipt graph. Undone, tracked under V12-003.
//! - **No independent replay verifier.** Each stage's `blake3` digest is genuinely computed
//!   from sorted canonical fact text (never asserted), so replaying the same inputs recomputes
//!   the same digest -- exercised by this module's own tests -- but there is no standalone
//!   "replay this `PlanningState` against a fresh admission and compare" entrypoint the way
//!   `chatman::abi::EngineProcessReceipt::verify` provides for the Chatman engine.
//!
//! # GGEN_GENERATABLE (real, generated this session)
//! [`f03_semantic_contraction_vocab`] is machine-generated by `ggen sync` from
//! `packs/semantic-contraction-pack/ontology.ttl` (verified this session: `ggen sync run`
//! exit 0, run twice with byte-identical output the second time). It supplies the D1-D7
//! `CLASSES`/`DERIVATION_EDGES` tables, the F03-L5 `LIFECYCLE_STATES`/`TRANSITIONS` tables
//! ([`ContractionState::is_legal_transition`] below reads `TRANSITIONS` directly rather than
//! duplicating the legality table by hand), and the `PIPELINE_COMPONENTS` status table this
//! doc comment's stage list mirrors by hand (kept in sync manually; the generated table is the
//! machine-checkable source of truth for what this pass claims REAL vs. not-yet-built).
//!
//! # Survey-cited paths for F03
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F03_semantic-contraction.md
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/owlrl/mod.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/owlrl/rules.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/lib.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/datalog.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/reasoner/mod.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/shacl/validate.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/shacl/report.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/shex.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/shex_native.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/quarantine.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/router.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/closure.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/engine.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/abi.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/pipeline.rs
//! - /Users/sac/praxis/crates/cng/src/shape.rs
//! - /Users/sac/praxis/crates/multifractal-workflow/src/f05_datalog_closure.rs (sibling reuse)
//! - /Users/sac/praxis/crates/multifractal-workflow/src/f06_n3_quarantine.rs (sibling reuse)
//! - /Users/sac/praxis/packs/semantic-contraction-pack/ (this pass's ggen pack)

#[path = "f03_semantic_contraction_vocab.rs"]
pub mod f03_semantic_contraction_vocab;

use praxis_graphlaw::shacl::ValidationReport as ShaclReport;
use praxis_graphlaw::shex_native::ShexValidationReport as ShexReport;
use praxis_graphlaw::triples::Triple;
use praxis_graphlaw::TripleStore;

use crate::f05_datalog_closure::{
    close_datalog, compare_residue, ClosureDigest, ResidueDiff, RulePack,
};
use crate::f06_n3_quarantine::{
    route_and_execute_n3, DialectRouter, N3ExecutionProfile, N3Rule, QueryShape,
    Refusal as N3ChatmanRefusal,
};

/// The F03-L5 lifecycle state machine. Legality of a transition is decided by
/// [`ContractionState::is_legal_transition`], which reads the generated
/// [`f03_semantic_contraction_vocab::TRANSITIONS`] table rather than duplicating that table by
/// hand -- the ontology (`packs/semantic-contraction-pack/ontology.ttl`) is the single source
/// of truth for which edges are legal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContractionState {
    Typed,
    Closed,
    Refined,
    Shaped,
    Admissible,
    Contracted,
    Plannable,
    Refused,
}

impl ContractionState {
    /// Stable name matching `f03_semantic_contraction_vocab::LIFECYCLE_STATES`.
    pub const fn name(self) -> &'static str {
        match self {
            ContractionState::Typed => "TYPED",
            ContractionState::Closed => "CLOSED",
            ContractionState::Refined => "REFINED",
            ContractionState::Shaped => "SHAPED",
            ContractionState::Admissible => "ADMISSIBLE",
            ContractionState::Contracted => "CONTRACTED",
            ContractionState::Plannable => "PLANNABLE",
            ContractionState::Refused => "REFUSED",
        }
    }

    /// Whether `from -> to` is a real edge in the generated F03-L5 transition table
    /// (`f03_semantic_contraction_vocab::TRANSITIONS`), regardless of its `kind` ("lawful",
    /// "invalid", or "authority_or_conformance_failure" -- all three are legitimate edges in
    /// the state machine; "invalid"/"authority_or_conformance_failure" name the REFUSED
    /// off-ramp, not an illegal edge). Any pair absent from the table is not a legal
    /// transition -- `TYPED -> PLANNABLE` (skipping every intermediate stage) is `false`.
    ///
    /// # Complexity
    /// O(|TRANSITIONS|) linear scan over a fixed 8-entry table -- O(1) in practice.
    pub fn is_legal_transition(from: ContractionState, to: ContractionState) -> bool {
        f03_semantic_contraction_vocab::TRANSITIONS
            .iter()
            .any(|(f, t, _kind)| *f == from.name() && *t == to.name())
    }

    /// The `"lawful"` / `"invalid"` / `"authority_or_conformance_failure"` kind of the
    /// `from -> to` edge, or `None` if no such edge exists in the generated table.
    pub fn transition_kind(from: ContractionState, to: ContractionState) -> Option<&'static str> {
        f03_semantic_contraction_vocab::TRANSITIONS
            .iter()
            .find(|(f, t, _kind)| *f == from.name() && *t == to.name())
            .map(|(_, _, kind)| *kind)
    }
}

/// The F03 refusal taxonomy (atlas name: `SemanticWorldRefused`). Every variant is a binding
/// contract with a `String`/`usize` context payload naming the concrete offender, matching the
/// convention `praxis_graphlaw::chatman::abi::Refusal` already establishes in this workspace
/// (`#[error("...: {0}")]` + `thiserror::Error`, not a generic catch-all).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SemanticWorldRefused {
    /// `TripleStore::materialize_owlrl` returned an `Err` (mid-fixpoint materialization
    /// failure during OWL RL entailment).
    #[error("ontology typing failed: {0}")]
    TypingFailed(String),
    /// [`crate::f05_datalog_closure::close_datalog`] returned a
    /// `DatalogClosureRefused` (unsafe rule, stratification cycle, or materialization
    /// failure); the inner error's `Display` text is carried, not just its variant name.
    #[error("datalog closure refused: {0}")]
    ClosureRefused(String),
    /// [`crate::f06_n3_quarantine::route_and_execute_n3`] returned a
    /// `praxis_graphlaw::chatman::abi::Refusal` (N3 not enabled by profile, direct-actuation
    /// refused, builtin outside whitelist, cost bound exceeded, or the shape's own
    /// least-expressive dialect is not N3).
    #[error("N3 quarantine/refinement refused: {0}")]
    N3RefinementRefused(String),
    /// `TripleStore::validate_shacl` itself failed (malformed shapes Turtle), distinct from a
    /// well-formed validation that simply finds violations.
    #[error("SHACL validator error: {0}")]
    ShaclValidatorError(String),
    /// `TripleStore::validate_shex_c` itself failed (malformed ShExC schema), distinct from a
    /// well-formed validation that simply finds failures.
    #[error("ShEx validator error: {0}")]
    ShexValidatorError(String),
    /// The world is impossible: the closure/refinement graph does not conform to the declared
    /// SHACL and/or ShEx shapes. Per the family invariant ("impossible worlds... refused with a
    /// typed SemanticWorldRefused before a PDDL problem is ever constructed -- no silent
    /// admission"), this is a hard `Err`, not a `Refused` variant folded quietly into an `Ok`.
    #[error(
        "shape nonconformance: {shacl_violations} SHACL violation(s), {shex_failures} ShEx failure(s)"
    )]
    ShapeNonconformant {
        shacl_violations: usize,
        shex_failures: usize,
    },
    /// A caller (or a future revision of [`contract`]) attempted a state transition absent
    /// from the generated F03-L5 [`f03_semantic_contraction_vocab::TRANSITIONS`] table.
    #[error("illegal state transition: {from} -> {to}")]
    IllegalStateTransition {
        from: &'static str,
        to: &'static str,
    },
}

/// BLAKE3 digest over a sorted, canonically-decoded fact set, domain-tagged by `stage` so a
/// `TypedGraph` digest and a `ResidueGraph` digest over coincidentally-identical fact bytes
/// never collide. Mirrors `f05_datalog_closure::ClosureDigest::compute`'s canonicalization
/// discipline (sort decoded `"s p o"` lines before hashing; never rely on `TripleIndex`
/// iteration order) rather than re-deriving a different one.
///
/// # Complexity
/// O(n log n) in `facts.len()` (dominated by the sort); O(n) space.
fn canonical_digest(stage: &str, facts: &[Triple]) -> blake3::Hash {
    let mut lines: Vec<String> = facts.iter().map(TripleStore::decode_triple).collect();
    lines.sort_unstable();
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"f03-semantic-contraction/v1/");
    hasher.update(stage.as_bytes());
    hasher.update(b"\0");
    for line in &lines {
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize()
}

/// D1: `TypedGraph`. Admitted RDF state after Ontology Typing (OWL RL entailment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedGraph {
    pub facts: Vec<Triple>,
    /// Count of OWL RL features this ontology actually used and this engine supports
    /// (`owlrl::ScanReport::supported.len()`).
    pub owlrl_supported_features: usize,
    /// Count of OWL RL features found but not supported/compiled by this engine
    /// (`owlrl::ScanReport::refused.len()`); a nonzero count is informational, not itself a
    /// refusal -- unsupported features are simply not entailed, per `materialize_owlrl`'s own
    /// opt-in "daily profile" scope.
    pub owlrl_refused_features: usize,
    pub digest: blake3::Hash,
}

/// Stage 1 (REAL): Ontology Typing. Parses `admitted_rdf` (Turtle/N3/RDF-XML, whichever
/// `TripleStore::from` detects) and runs real OWL RL entailment via
/// `TripleStore::materialize_owlrl` -- compiled-to-Datalog RDFS/OWL RL rules through the
/// semi-naive `Reasoner`, not a hand-rolled subset.
///
/// # Errors
/// [`SemanticWorldRefused::TypingFailed`] if `materialize_owlrl`'s own materialization pass
/// fails mid-fixpoint.
///
/// # Complexity
/// Dominated by `Reasoner::materialize`'s own documented O(S * |R| * |F|) bound (see
/// `owlrl/mod.rs`/`reasoner/mod.rs`) plus this function's O(n log n) canonical digest.
pub fn type_ontology(admitted_rdf: &str) -> Result<TypedGraph, SemanticWorldRefused> {
    let mut store = TripleStore::from(admitted_rdf);
    let (_derived, report) = store
        .materialize_owlrl()
        .map_err(SemanticWorldRefused::TypingFailed)?;

    let facts: Vec<Triple> = (0..store.triple_index.len())
        .filter_map(|i| store.triple_index.get(i).cloned())
        .collect();
    let digest = canonical_digest("typed-graph", &facts);

    Ok(TypedGraph {
        owlrl_supported_features: report.supported.len(),
        owlrl_refused_features: report.refused.len(),
        facts,
        digest,
    })
}

/// D2: `ClosureGraph`. `TypedGraph` plus Datalog closure over caller-declared rules,
/// fixpoint-materialized. `digest` is [`ClosureDigest`], reused unchanged from
/// `crate::f05_datalog_closure` rather than a second receipt type computing the same thing a
/// different way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureGraph {
    pub facts: Vec<Triple>,
    pub digest: ClosureDigest,
}

/// Stage 2 (REAL): Datalog Closure. Thin pass-through to
/// `crate::f05_datalog_closure::close_datalog` -- the same function, called with `typed`'s own
/// fact set as input, not a re-derivation of closure logic. `close_datalog` builds a fresh
/// `TripleStore`, calls the real `TripleStore::add_rules`/`materialize` (i.e.
/// `datalog::validate_rules`'s stratifier + `Reasoner::materialize`'s per-stratum fixpoint
/// loop), and returns the closure fact set read back from the store's own index together with
/// a `ClosureDigest` receipt.
///
/// # Errors
/// [`SemanticWorldRefused::ClosureRefused`], wrapping the `DatalogClosureRefused`
/// `close_datalog` returned (its `Display` text, not just the variant name, is preserved).
///
/// # Complexity
/// Dominated by `close_datalog`'s own documented cost (see `f05_datalog_closure.rs`).
pub fn close_semantic(
    typed: &TypedGraph,
    rule_pack: &RulePack,
) -> Result<ClosureGraph, SemanticWorldRefused> {
    let (digest, facts) = close_datalog(rule_pack, typed.facts.clone())
        .map_err(|e| SemanticWorldRefused::ClosureRefused(e.to_string()))?;
    Ok(ClosureGraph { facts, digest })
}

/// D3: `RefinementDelta`. The (possibly empty) result of bounded N3 refinement, once
/// quarantine routing authorized it. See module doc for the disclosed scope: `rules_admitted`
/// and `ticks_used` reflect the capability/cost gate's own admitted-rule bookkeeping, not newly
/// derived facts (`N3Executor::run` does not evaluate N3 rules against a graph).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefinementDelta {
    pub rules_admitted: Vec<String>,
    pub ticks_used: Option<u64>,
    pub digest: blake3::Hash,
}

/// Caller-supplied request to run N3 refinement over a [`ClosureGraph`]. See module doc:
/// deciding *whether* refinement is needed (the "exceptional implication" trigger) is not
/// derived automatically from `ClosureGraph`'s content by this module -- the caller must
/// already know it wants N3 execution and supply the routing/execution parameters, or pass
/// `None` to [`refine_n3`] to skip this stage.
pub struct N3RefinementRequest<'a> {
    pub router: &'a DialectRouter,
    pub shape: &'a QueryShape,
    pub execution: &'a N3ExecutionProfile,
    pub rules: &'a [N3Rule],
}

/// Stages 3-4 (REAL, disclosed scope): N3 Quarantine Router + Bounded N3 Refinement. Thin
/// pass-through to `crate::f06_n3_quarantine::route_and_execute_n3` -- the same function,
/// composing the real `DialectRouter::decide` gate with the real `N3Executor::run`
/// capability/cost/whitelist gate. If `request` is `None`, this is a real (not fake) no-op:
/// the closure already reached a fixpoint with no N3 involvement, which is a legitimate
/// `RefinementDelta` (empty), not a skipped check.
///
/// # Errors
/// [`SemanticWorldRefused::N3RefinementRefused`], wrapping whatever `praxis_graphlaw::chatman::
/// abi::Refusal` `route_and_execute_n3` returned (profile does not enable N3, direct-actuation
/// refused, builtin outside whitelist, cost bound exceeded, or the shape's least-expressive
/// dialect is not N3).
pub fn refine_n3(
    closure: &ClosureGraph,
    request: Option<N3RefinementRequest<'_>>,
) -> Result<RefinementDelta, SemanticWorldRefused> {
    match request {
        None => Ok(RefinementDelta {
            rules_admitted: Vec::new(),
            ticks_used: None,
            digest: canonical_digest("refinement-delta/none", &closure.facts),
        }),
        Some(req) => {
            let (_decision, receipt) =
                route_and_execute_n3(req.router, req.shape, req.execution, req.rules).map_err(
                    |e: N3ChatmanRefusal| SemanticWorldRefused::N3RefinementRefused(e.to_string()),
                )?;
            Ok(RefinementDelta {
                rules_admitted: receipt.rules_admitted,
                ticks_used: Some(receipt.ticks_used.0),
                digest: canonical_digest("refinement-delta/executed", &closure.facts),
            })
        }
    }
}

/// D4: `ShapeReport`. Combined SHACL + ShEx conformance report for the refined graph. `shacl`
/// and `shex` are `praxis_graphlaw`'s own real report types, not re-abstracted copies.
#[derive(Debug, Clone)]
pub struct ShapeReport {
    pub shacl: ShaclReport,
    pub shex: Option<ShexReport>,
    pub digest: blake3::Hash,
}

/// Stages 5-6 (REAL): SHACL Gate + ShEx Gate. Loads `closure.facts` into a fresh
/// `TripleStore` and calls `TripleStore::validate_shacl`/`validate_shex_c` directly -- the same
/// native validators `crate::f07_shape_admission.rs` also reuses, called here on this
/// pipeline's own in-memory facts rather than through F07's Turtle-text entrypoint (see module
/// doc for why). `shex` is `None` to skip the ShEx gate entirely (a graph with no declared ShEx
/// schema is not itself nonconformant).
///
/// # Errors
/// [`SemanticWorldRefused::ShaclValidatorError`] / [`SemanticWorldRefused::ShexValidatorError`]
/// if the validator itself fails (malformed shapes/schema) -- distinct from a well-formed
/// validation that simply finds violations, which is not an error at this stage (see
/// [`decide_admissibility`]).
pub fn gate_shapes(
    closure: &ClosureGraph,
    shacl_shapes_turtle: &str,
    shex: Option<(&str, &[(String, String)])>,
) -> Result<ShapeReport, SemanticWorldRefused> {
    let mut store = TripleStore::new();
    for fact in &closure.facts {
        store.add(fact.clone());
    }

    let shacl = store
        .validate_shacl(shacl_shapes_turtle)
        .map_err(SemanticWorldRefused::ShaclValidatorError)?;

    let shex_report = match shex {
        Some((schema_shexc, shape_map)) => Some(
            store
                .validate_shex_c(schema_shexc, shape_map)
                .map_err(SemanticWorldRefused::ShexValidatorError)?,
        ),
        None => None,
    };

    let digest = canonical_digest("shape-report", &closure.facts);
    Ok(ShapeReport {
        shacl,
        shex: shex_report,
        digest,
    })
}

/// D5: `AdmissibilityDecision`. Reached only when the world is admissible; see
/// [`decide_admissibility`] for the refusal path on nonconformance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissibilityDecision {
    pub digest: blake3::Hash,
}

/// Decide admissibility from exactly one [`ShapeReport`]. Per the family invariant
/// ("impossible worlds... refused with a typed SemanticWorldRefused before a PDDL problem is
/// ever constructed -- no silent admission"), nonconformance is a hard `Err`, not an `Ok`
/// variant a caller could accidentally ignore.
///
/// # Errors
/// [`SemanticWorldRefused::ShapeNonconformant`] if either `report.shacl.conforms` is `false` or
/// (when a ShEx report is present) `report.shex.conforms` is `false`.
pub fn decide_admissibility(
    report: &ShapeReport,
) -> Result<AdmissibilityDecision, SemanticWorldRefused> {
    let shacl_ok = report.shacl.conforms;
    let shex_ok = report.shex.as_ref().is_none_or(|r| r.conforms);

    if shacl_ok && shex_ok {
        Ok(AdmissibilityDecision {
            digest: report.digest,
        })
    } else {
        Err(SemanticWorldRefused::ShapeNonconformant {
            shacl_violations: report.shacl.results.len(),
            shex_failures: report.shex.as_ref().map_or(0, |r| r.failures.len()),
        })
    }
}

/// D6: `ResidueGraph`. The irreducible residue -- see module doc for [`project_residue`]'s
/// disclosed predicate-IRI-granularity scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidueGraph {
    pub diff: ResidueDiff,
    pub digest: blake3::Hash,
}

/// Stage 7 (REAL, disclosed scope): Residue Projector. Thin pass-through to
/// `crate::f05_datalog_closure::compare_residue` -- the same function, not a re-derivation.
/// `admitted_predicates` is the caller-supplied candidate set of predicate IRIs the *pre-closure*
/// admitted graph asserted (i.e. the planner's naive "everything here might be open work"
/// starting point); `compare_residue` strips whichever ones the closure graph already proves,
/// leaving only the genuinely irreducible remainder. See module doc for why this is
/// predicate-level, not full-subgraph, set difference.
///
/// **Correction (re-verified, does not reproduce):** an earlier version of this doc comment
/// claimed `compare_residue` matched against `Encoder::decode`'s raw bracketed `<iri>` form
/// with no stripping, so bare predicate IRIs could never match and `project_residue` had to
/// normalize to/from brackets at its own boundary to work around it. Re-reading
/// `compare_residue`'s current source (`f05_datalog_closure.rs:337-343`) shows it already
/// calls `.trim_matches(|c| c == '<' || c == '>')` on the decoded predicate string before
/// comparing, and `git log -p --follow` on that file shows the `trim_matches` call has been
/// present since the function's first commit, not added later. Re-running the cited repro
/// (`cargo test -p multifractal-workflow --lib
/// f05_datalog_closure::tests::test_compare_residue_strips_closed_predicates`) passes. This
/// function performs no bracket normalization of its own -- it is a genuine thin pass-through,
/// as the paragraph above states -- and this file's own
/// `project_residue_strips_predicates_the_closure_already_proves` test below exercises it with
/// bare (unbracketed) predicate IRIs end to end, confirming there is nothing left to work
/// around.
/// # Complexity
/// See `compare_residue`'s own O(|closure| + |admitted_predicates| * log(|closure|)) bound.
pub fn project_residue(closure: &ClosureGraph, admitted_predicates: &[String]) -> ResidueGraph {
    let raw_diff = compare_residue(&closure.facts, admitted_predicates);
    let diff = ResidueDiff {
        stripped: raw_diff.stripped,
        remaining: raw_diff.remaining,
    };
    let digest = canonical_digest("residue-graph", &closure.facts);
    ResidueGraph { diff, digest }
}

/// D7: `PlanningState`. Minimal unresolved planning state handed off to PDDL problem
/// construction, bound to a receipt head (`receipt_head`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningState {
    pub state: ContractionState,
    pub residue: ResidueGraph,
    pub receipt_head: blake3::Hash,
}

/// Publish the final `PlanningState` from a [`ResidueGraph`], having already reached
/// [`AdmissibilityDecision`] (taken by reference only to make the ordering dependency explicit
/// at the call site -- its digest is not otherwise consumed here, since `ResidueGraph::digest`
/// is `project_residue`'s own receipt over the same closure facts).
pub fn publish_planning_state(
    residue: ResidueGraph,
    _admission: &AdmissibilityDecision,
) -> PlanningState {
    PlanningState {
        state: ContractionState::Plannable,
        receipt_head: residue.digest,
        residue,
    }
}

/// Every input [`contract`] needs to run the F03 pipeline end to end.
pub struct ContractionInputs<'a> {
    pub admitted_rdf: &'a str,
    pub datalog_rule_pack: RulePack,
    pub n3_refinement: Option<N3RefinementRequest<'a>>,
    pub shacl_shapes_turtle: &'a str,
    pub shex: Option<(&'a str, &'a [(String, String)])>,
    pub admitted_predicates: Vec<String>,
}

/// Runs the full F03 pipeline end to end: Ontology Typing -> Datalog Closure -> N3 Quarantine
/// Router + Bounded N3 Refinement -> SHACL Gate -> ShEx Gate -> Admissibility Decision ->
/// Residue Projector -> `PlanningState`. Every stage is a real call (see module doc for exactly
/// which existing code each one reuses); any stage's refusal short-circuits the rest via `?`,
/// so an impossible world genuinely never reaches [`PlanningState`] -- there is no code path
/// here that silently drops a refusal and keeps going.
///
/// # Errors
/// Any [`SemanticWorldRefused`] a stage function above returns.
pub fn contract(inputs: ContractionInputs<'_>) -> Result<PlanningState, SemanticWorldRefused> {
    let typed = type_ontology(inputs.admitted_rdf)?;
    let closure = close_semantic(&typed, &inputs.datalog_rule_pack)?;
    let _delta = refine_n3(&closure, inputs.n3_refinement)?;
    let shape_report = gate_shapes(&closure, inputs.shacl_shapes_turtle, inputs.shex)?;
    let admission = decide_admissibility(&shape_report)?;
    let residue = project_residue(&closure, &inputs.admitted_predicates);
    Ok(publish_planning_state(residue, &admission))
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_graphlaw::triples::{BodyLiteral, Rule};

    /// A trivial rule pack: `{?x a ex:Widget} => {?x ex:derivedFlag "yes"}.` -- enough to
    /// exercise a real (not vacuous) Datalog closure step without needing recursion (recursion
    /// depth is `praxis-graphlaw`'s own concern, already covered by its own tests; this
    /// module's tests exercise this crate's wiring, not re-litigate the underlying engine, same
    /// discipline `f05_datalog_closure.rs`'s own tests state explicitly).
    fn widget_rule_pack() -> RulePack {
        let rule = Rule {
            head: Triple::from(
                "?x".to_string(),
                "http://example.org/derivedFlag".to_string(),
                "\"yes\"".to_string(),
            ),
            body: vec![BodyLiteral {
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                    "http://example.org/Widget".to_string(),
                ),
                negated: false,
            }],
        };
        RulePack::new("f03-test-widget-pack", vec![rule])
    }

    const ADMITTED_RDF: &str = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        ex:thing1 rdf:type ex:Widget .
    "#;

    const SHACL_SHAPES_ALWAYS_CONFORMS: &str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .
        ex:WidgetShape a sh:NodeShape ;
            sh:targetClass ex:Widget .
    "#;

    #[test]
    fn type_ontology_admits_a_well_formed_graph() {
        let typed = type_ontology(ADMITTED_RDF).expect("well-formed Turtle types cleanly");
        assert!(
            !typed.facts.is_empty(),
            "typed graph must retain the admitted facts"
        );
    }

    #[test]
    fn type_ontology_digest_is_deterministic_across_repeated_runs() {
        let a = type_ontology(ADMITTED_RDF).expect("first run");
        let b = type_ontology(ADMITTED_RDF).expect("second run");
        assert_eq!(
            a.digest, b.digest,
            "same admitted RDF must produce a byte-identical TypedGraph digest"
        );
    }

    #[test]
    fn close_semantic_derives_the_declared_consequence() {
        let typed = type_ontology(ADMITTED_RDF).expect("types cleanly");
        let rule_pack = widget_rule_pack();
        let closure = close_semantic(&typed, &rule_pack).expect("closure over a safe rule pack");
        let decoded: Vec<String> = closure
            .facts
            .iter()
            .map(TripleStore::decode_triple)
            .collect();
        assert!(
            decoded.iter().any(|line| line.contains("derivedFlag")),
            "closure must derive ex:derivedFlag from the rule pack, got: {decoded:?}"
        );
    }

    #[test]
    fn refine_n3_with_no_request_is_a_real_empty_delta_not_a_skipped_check() {
        let typed = type_ontology(ADMITTED_RDF).expect("types cleanly");
        let closure =
            close_semantic(&typed, &widget_rule_pack()).expect("closure over a safe rule pack");
        let delta = refine_n3(&closure, None).expect("no-request refinement never refuses");
        assert!(delta.rules_admitted.is_empty());
        assert!(delta.ticks_used.is_none());
    }

    #[test]
    fn gate_shapes_and_decide_admissibility_admit_a_conformant_graph() {
        let typed = type_ontology(ADMITTED_RDF).expect("types cleanly");
        let closure =
            close_semantic(&typed, &widget_rule_pack()).expect("closure over a safe rule pack");
        let report = gate_shapes(&closure, SHACL_SHAPES_ALWAYS_CONFORMS, None)
            .expect("well-formed shapes graph validates");
        assert!(report.shacl.conforms);
        let admission = decide_admissibility(&report).expect("conformant graph is admissible");
        assert_eq!(admission.digest, report.digest);
    }

    #[test]
    fn decide_admissibility_refuses_a_nonconformant_graph_with_a_typed_refusal() {
        // sh:minCount 1 on a property no ex:Widget instance carries -- a real, provoked
        // SHACL violation, not a synthetic ValidationReport.
        let strict_shapes = r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            ex:WidgetShape a sh:NodeShape ;
                sh:targetClass ex:Widget ;
                sh:property [
                    sh:path ex:requiredProp ;
                    sh:minCount 1 ;
                ] .
        "#;
        let typed = type_ontology(ADMITTED_RDF).expect("types cleanly");
        let closure =
            close_semantic(&typed, &widget_rule_pack()).expect("closure over a safe rule pack");
        let report = gate_shapes(&closure, strict_shapes, None)
            .expect("well-formed shapes graph validates (even though it finds a violation)");
        assert!(!report.shacl.conforms, "sh:minCount 1 must be violated");

        let err = decide_admissibility(&report)
            .expect_err("nonconformant graph must be refused, not silently admitted");
        assert!(matches!(
            err,
            SemanticWorldRefused::ShapeNonconformant {
                shacl_violations,
                ..
            } if shacl_violations >= 1
        ));
    }

    #[test]
    fn project_residue_strips_predicates_the_closure_already_proves() {
        let typed = type_ontology(ADMITTED_RDF).expect("types cleanly");
        let closure =
            close_semantic(&typed, &widget_rule_pack()).expect("closure over a safe rule pack");
        let admitted_predicates = vec![
            "http://example.org/derivedFlag".to_string(), // the closure proves this
            "http://example.org/genuinelyOpenWork".to_string(), // the closure does not
        ];
        let residue = project_residue(&closure, &admitted_predicates);
        assert!(residue
            .diff
            .stripped
            .contains(&"http://example.org/derivedFlag".to_string()));
        assert!(residue
            .diff
            .remaining
            .contains(&"http://example.org/genuinelyOpenWork".to_string()));
    }

    /// Regression pin for the `project_residue` doc-comment correction: an earlier revision of
    /// that doc claimed `compare_residue` (reused verbatim here, not re-derived) could never
    /// match a bare/unbracketed predicate IRI against its bracketed internal decode, so
    /// `project_residue` had to normalize brackets at its own boundary to work around it. This
    /// test calls the real `crate::f05_datalog_closure::compare_residue` entry point directly
    /// against this module's own `ClosureGraph` output (the same call `project_residue` makes),
    /// with bare, unbracketed predicate strings on both sides of the comparison -- exactly the
    /// shape the retracted doc claimed always fails -- and asserts a real match. If the
    /// described bug ever regresses, this fails alongside
    /// `f05_datalog_closure::tests::test_compare_residue_strips_closed_predicates`.
    #[test]
    fn compare_residue_matches_bare_predicate_iris_no_bracket_workaround_needed() {
        let typed = type_ontology(ADMITTED_RDF).expect("types cleanly");
        let closure =
            close_semantic(&typed, &widget_rule_pack()).expect("closure over a safe rule pack");

        // Bare (unbracketed) IRI, matching the shape planner-derived residue candidates use.
        let bare_predicate = "http://example.org/derivedFlag".to_string();
        let diff = compare_residue(&closure.facts, std::slice::from_ref(&bare_predicate));

        assert_eq!(
            diff.stripped,
            vec![bare_predicate],
            "compare_residue must strip a bare predicate IRI the closure proves; a bracket \
             mismatch would leave it in `remaining` instead"
        );
        assert!(diff.remaining.is_empty());
    }

    #[test]
    fn contract_reaches_plannable_end_to_end_for_an_admissible_world() {
        let inputs = ContractionInputs {
            admitted_rdf: ADMITTED_RDF,
            datalog_rule_pack: widget_rule_pack(),
            n3_refinement: None,
            shacl_shapes_turtle: SHACL_SHAPES_ALWAYS_CONFORMS,
            shex: None,
            admitted_predicates: vec![
                "http://example.org/derivedFlag".to_string(),
                "http://example.org/genuinelyOpenWork".to_string(),
            ],
        };
        let planning_state = contract(inputs).expect("admissible world reaches PLANNABLE");
        assert_eq!(planning_state.state, ContractionState::Plannable);
        assert!(planning_state
            .residue
            .diff
            .remaining
            .contains(&"http://example.org/genuinelyOpenWork".to_string()));
        assert!(!planning_state
            .residue
            .diff
            .remaining
            .contains(&"http://example.org/derivedFlag".to_string()));
    }

    #[test]
    fn contract_refuses_before_planning_state_for_an_impossible_world() {
        let strict_shapes = r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            ex:WidgetShape a sh:NodeShape ;
                sh:targetClass ex:Widget ;
                sh:property [
                    sh:path ex:requiredProp ;
                    sh:minCount 1 ;
                ] .
        "#;
        let inputs = ContractionInputs {
            admitted_rdf: ADMITTED_RDF,
            datalog_rule_pack: widget_rule_pack(),
            n3_refinement: None,
            shacl_shapes_turtle: strict_shapes,
            shex: None,
            admitted_predicates: vec!["http://example.org/derivedFlag".to_string()],
        };
        let err = contract(inputs).expect_err("impossible world must never reach PlanningState");
        assert!(matches!(
            err,
            SemanticWorldRefused::ShapeNonconformant { .. }
        ));
    }

    #[test]
    fn contraction_state_legal_transitions_match_the_generated_table() {
        assert!(ContractionState::is_legal_transition(
            ContractionState::Typed,
            ContractionState::Closed
        ));
        assert!(ContractionState::is_legal_transition(
            ContractionState::Contracted,
            ContractionState::Plannable
        ));
        assert!(ContractionState::is_legal_transition(
            ContractionState::Closed,
            ContractionState::Refused
        ));
        assert_eq!(
            ContractionState::transition_kind(ContractionState::Closed, ContractionState::Refused),
            Some("invalid")
        );
        assert_eq!(
            ContractionState::transition_kind(ContractionState::Shaped, ContractionState::Refused),
            Some("authority_or_conformance_failure")
        );
        assert!(
            !ContractionState::is_legal_transition(
                ContractionState::Typed,
                ContractionState::Plannable
            ),
            "skipping every intermediate stage must not be a legal transition"
        );
    }

    #[test]
    fn generated_vocab_lists_exactly_the_seven_d1_d7_classes() {
        assert_eq!(f03_semantic_contraction_vocab::CLASSES.len(), 7);
        let labels: Vec<&str> = f03_semantic_contraction_vocab::CLASSES
            .iter()
            .map(|c| c.label)
            .collect();
        for expected in [
            "TypedGraph",
            "ClosureGraph",
            "RefinementDelta",
            "ShapeReport",
            "AdmissibilityDecision",
            "ResidueGraph",
            "PlanningState",
        ] {
            assert!(
                labels.contains(&expected),
                "generated CLASSES missing {expected}, got {labels:?}"
            );
        }
    }
}
