//! F31 -- "Org Merge" (not one of the v26.7.12 atlas's 30 families; new work).
//!
//! Closes the FIRST real, working instance of
//! `docs/standing/BOOTSTRAP_COLD_START_LIMITATIONS.md` item 14 ("Multi-tenant /
//! multi-org first-merge bootstrap"). That item names three unsolved questions when
//! two independently-governed RDF graphs meet for the first time: "whose shape wins
//! where the two locally-valid shapes conflict, whose permission authority is
//! superior, how the two receipt chains relate." This module answers all three,
//! disclosed as specific, narrow design choices -- not as the only possible answers
//! (see each function's doc comment for the rationale, and this module's "What this
//! does and does not close" section below for the honest scope boundary).
//!
//! # Built on two already-real foundations (reused, not reimplemented)
//!
//! - [`crate::f02_observation_admission::admit_observation`] -- each organization's
//!   graph is admitted independently through the real 6-gate F02 pipeline, with its
//!   own [`crate::f02_observation_admission::AdmissionPolicy`] and its own
//!   [`crate::f02_observation_admission::AdmissionReceipt`]. This module's
//!   [`admit_org_graph`] is a thin wrapper that also keeps the admitted triples and
//!   shape profile around for the later merge step; it adds no new admission gate
//!   and reuses F02's gates unchanged.
//! - `praxis_graphlaw::shacl::{ShapesGraph, Validator}` -- the same real SHACL Core
//!   engine F02's own Gate 4 already uses (confirmed by reading
//!   `f02_observation_admission.rs`'s own module doc), reused here for the
//!   merged-graph re-validation step.
//!
//! # The three questions, answered
//!
//! 1. **Whose shape wins?** Neither. [`union_of_shapes`] builds the merged graph's
//!    governing shape set by literal Turtle-text concatenation of both
//!    organizations' own shape files, parsed once. This is UNION-AS-CONJUNCTION: a
//!    merged individual must satisfy BOTH organizations' shapes simultaneously, not
//!    whichever org's shape happens to be checked first. See that function's doc
//!    comment for the full rationale and its disclosed limit (two shapes that are
//!    directly, irreconcilably contradictory for the same target class have no
//!    resolution here).
//! 2. **Whose authority is superior?** Neither, by construction: [`merge_org_graphs`]
//!    never re-runs or weakens either organization's own F02 authority/semantic
//!    checks (those already happened, independently, before this module is ever
//!    called), and the merge step itself has no separate authority gate that could
//!    rank one organization's principal above the other's. The only new gate this
//!    module adds is [`detect_identifier_collisions`] (identity, not authority) and
//!    shape re-validation (structure, not authority) -- see "What this does and does
//!    not close" below for what a real cross-org authority hierarchy would still
//!    need that this module does not build.
//! 3. **How do the two receipt chains relate?** [`MergeReceipt`] references
//!    (`acquirer_receipt_hash`, `target_receipt_hash`) both organizations' own F02
//!    admission receipt hashes as its own provenance, and is computed by hashing
//!    those two hash strings together with the merged graph's canonical triples
//!    (`merge_receipt_hash`). Neither original [`crate::f02_observation_admission::
//!    AdmissionReceipt`] is mutated, recomputed, or replaced -- this is an APPEND (a
//!    new receipt citing two prior ones), never an edit, matching
//!    `docs/releases/v26.7.13/THESIS.md` Chapter 13.3's "a chain proves tamper
//!    evidence, not event truth."
//!
//! # Identifier collisions (item 14's own phrase: "a locally-scoped ID scheme that
//! happens to collide")
//!
//! [`detect_identifier_collisions`] is a dedicated, standalone check that runs
//! BEFORE the triple union and BEFORE SHACL re-validation -- not merely relying on
//! SHACL's `sh:maxCount` constraints to incidentally catch a collision (some
//! predicates in these fixtures' shapes have no cardinality constraint at all, and a
//! shape-based catch would not name which two organizations disagree or on what). A
//! collision is a ground-IRI subject asserted by BOTH organizations with, for at
//! least one shared predicate, two DIFFERENT object values -- a genuine factual
//! contradiction. A subject shared between both graphs that asserts IDENTICAL facts
//! is not a collision (RDF set union already dedupes it for free); only divergence
//! is refused. Blank-node subjects are never compared (RDF blank-node identifiers
//! are document-scoped by definition).
//!
//! # What this does and does not close (BOOTSTRAP_COLD_START_LIMITATIONS.md item 14)
//!
//! **Closes**: a real, working, adversarially-tested mechanism for (a) two
//! independent F02 admissions each producing their own receipt chain head, (b)
//! deterministic identifier-collision detection between the two graphs that
//! refuses rather than silently overwrites, (c) one disclosed, justified
//! shape-governance rule for the merged graph (union-as-conjunction), (d) real
//! SHACL re-validation of the fused graph that genuinely passes or genuinely
//! refuses, and (e) a new receipt that provably descends from both prior chain
//! heads without rewriting either.
//!
//! **Does not close**: (i) the harder shape-conflict case where the two
//! organizations' shapes genuinely, irreconcilably disagree for the same target
//! class (this module's fixture pair uses byte-identical shape profiles per
//! `packs/ma-case-study-pack/fixtures/acquirer-org-shapes.ttl`'s own header --
//! union-as-conjunction is proven for that case, not for real contradiction); (ii) a
//! genuine cross-org PERMISSION AUTHORITY hierarchy (e.g. "Acquirer's compliance
//! officer may override Target's own shape after signing") -- neither organization
//! is ranked above the other here, which sidesteps the authority-superiority
//! question rather than answering it with a real ranking mechanism; (iii) n-way
//! merges (n > 2) -- every function here is binary; (iv) merge-time SHACL
//! `sh:closed` interaction across the two orgs' own closed shapes for classes only
//! ONE org's shape file declares (not exercised by this fixture pair, since both
//! declare the same four classes); (v) durable, cross-restart persistence of either
//! `AdmissionLedger` or the merge receipt -- both are in-process only, the same
//! disclosed limit F02 itself already carries.

use std::collections::{BTreeMap, BTreeSet};

use praxis_graphlaw::parser::{Parser, Syntax};
use praxis_graphlaw::shacl::{ShapesGraph, Validator};
use praxis_graphlaw::tripleindex::TripleIndex;
use praxis_graphlaw::triples::Triple;

use crate::f02_observation_admission::{
    admit_observation, bare_iri, term_display, AdmissionLedger, AdmissionPolicy, AdmissionReceipt,
    ObservationAdmissionRefused, RawObservation,
};

/// One organization's independently-admitted graph, plus everything
/// [`merge_org_graphs`] needs to re-derive its content and re-validate it without
/// re-running any F02 admission gate.
#[derive(Debug, Clone)]
pub struct AdmittedOrgGraph {
    /// Disambiguation label only (e.g. `"acquirer"`, `"target"`) -- never part of
    /// the admitted RDF, never hashed into [`MergeReceipt`]; used purely to name
    /// which organization a refusal or collision report is about.
    pub org_label: String,
    /// This org's own F02 admission receipt -- the chain head [`MergeReceipt`]
    /// references but never edits.
    pub receipt: AdmissionReceipt,
    /// The admitted payload's parsed triples, re-derived by re-parsing the same
    /// `payload_turtle` F02's Gate 0 already parsed successfully once (Turtle
    /// parsing is a pure function of its input string, so this reconstructs
    /// identical triples without re-running any gate).
    pub triples: Vec<Triple>,
    /// This org's own SHACL shape profile (Turtle text) -- the shape set F02's
    /// Gate 4 validated `triples` against. Kept so [`union_of_shapes`] can build
    /// the merged graph's governing shape set.
    pub shapes_turtle: String,
}

/// Typed refusal taxonomy for the merge step. Every variant has >= 1 end-to-end
/// test in `crates/multifractal-workflow/tests/ma_org_merge.rs`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OrgMergeRefused {
    #[error(
        "identifier collision: subject {subject} predicate {predicate} -- {acquirer_org} \
         asserts {acquirer_value}, {target_org} asserts {target_value}; refusing rather than \
         silently overwriting either fact"
    )]
    IdentifierCollision {
        subject: String,
        predicate: String,
        acquirer_org: String,
        acquirer_value: String,
        target_org: String,
        target_value: String,
    },
    #[error("merge produced zero triples -- refusing an empty merged graph")]
    EmptyMergedGraph,
    #[error("the union of both organizations' own shape files failed to parse: {reason}")]
    MalformedMergedShapes { reason: String },
    #[error(
        "merged graph is non-conformant against the union of both organizations' shapes: \
         {violation_count} SHACL violation(s)"
    )]
    MergedShapeNonConformant { violation_count: usize },
}

/// Admits one organization's [`RawObservation`] through the real F02 pipeline
/// (unchanged, no new gate) and packages the result for a later
/// [`merge_org_graphs`] call. Does not itself compare or merge anything -- callers
/// admit both organizations independently (item 14's own phrase: "each graph was
/// validated under its own shapes... its own receipt chain") before calling
/// [`merge_org_graphs`].
///
/// # Errors
/// Propagates [`ObservationAdmissionRefused`] unchanged from
/// [`admit_observation`]. The second, post-admission `Parser::parse_triples` call
/// below re-parses the exact same `payload_turtle` string F02's own Gate 0 already
/// parsed successfully inside `admit_observation` (Turtle parsing is a pure,
/// deterministic function of its input string); its `Err` arm exists only so this
/// function is total under `?`, not because it is expected to run for any input
/// that already passed `admit_observation`.
///
/// # Complexity
/// O(T) for the re-parse (T = payload triple count) plus whatever
/// [`admit_observation`] itself costs (documented on that function).
pub fn admit_org_graph(
    policy: &AdmissionPolicy,
    ledger: &AdmissionLedger,
    obs: RawObservation,
    shapes_turtle: String,
    org_label: impl Into<String>,
) -> Result<AdmittedOrgGraph, ObservationAdmissionRefused> {
    let payload = obs.payload_turtle.clone();
    let correlation_id = obs.correlation_id.clone();
    let receipt = admit_observation(policy, ledger, obs)?;
    let triples = Parser::parse_triples(&payload, Syntax::Turtle).map_err(|e| {
        ObservationAdmissionRefused::MalformedPayload {
            correlation_id,
            reason: format!("post-admission re-parse failed unexpectedly: {e}"),
        }
    })?;
    Ok(AdmittedOrgGraph {
        org_label: org_label.into(),
        receipt,
        triples,
        shapes_turtle,
    })
}

/// `subject IRI -> predicate IRI -> sorted set of object display strings`, built
/// only from ground-IRI subjects and predicates (blank-node/variable subjects and
/// non-IRI predicates are skipped -- mirrors
/// [`crate::f02_observation_admission::bare_iri`]'s own "only `Term::Iri` counts"
/// contract; a malformed non-IRI predicate is F02's concern at admission time, not
/// this module's).
///
/// # Complexity
/// O(n log n), n = `triples.len()` (one `BTreeMap`/`BTreeSet` insert per triple).
fn subject_predicate_object_index(
    triples: &[Triple],
) -> BTreeMap<String, BTreeMap<String, BTreeSet<String>>> {
    let mut index: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
    for t in triples {
        let Some(subject) = bare_iri(&t.s) else {
            continue;
        };
        let Some(predicate) = bare_iri(&t.p) else {
            continue;
        };
        let object = term_display(&t.o);
        index
            .entry(subject)
            .or_default()
            .entry(predicate)
            .or_default()
            .insert(object);
    }
    index
}

/// Detects locally-scoped identifier collisions between two organizations' graphs
/// (see this module's doc comment for the full definition). Deterministic:
/// `BTreeMap`/`BTreeSet` keys are lexically sorted, so the first divergence this
/// function reports is stable across runs (invariant #5: no `HashMap`-order
/// dependence in any observable output).
///
/// # Errors
/// [`OrgMergeRefused::IdentifierCollision`] on the first (in sorted
/// subject-then-predicate order) shared ground-IRI subject+predicate pair whose
/// asserted object values diverge between `acquirer` and `target`.
///
/// # Complexity
/// O(A + T) to build each org's subject/predicate/object index (A = acquirer
/// triple count, T = target triple count), then O(min(|subjects_a|, |subjects_t|))
/// to walk the smaller subject set against the larger one's index.
// `OrgMergeRefused::IdentifierCollision` carries 5 `String` fields (subject,
// predicate, both orgs' labels, both orgs' diverging values) by design -- per
// "Error Types Are Specifications" (`.claude/rules/rust-agi-core-team.md` #5:
// "Include all context needed to debug"), naming exactly which two
// organizations disagree, on what, with what two values, is the entire point
// of this refusal. Same accepted pattern as this crate's own
// `ExternalF18Refused`/`LocalWitnessRefused` (`crown_external.rs`,
// `crown_local.rs`) and `praxis-core/src/law.rs:145`; boxing the variant here
// would touch this module's and `tests/ma_org_merge.rs`'s pattern matches for
// a stack-size lint, not a correctness one.
#[allow(clippy::result_large_err)]
fn detect_identifier_collisions(
    acquirer: &AdmittedOrgGraph,
    target: &AdmittedOrgGraph,
) -> Result<(), OrgMergeRefused> {
    let acquirer_index = subject_predicate_object_index(&acquirer.triples);
    let target_index = subject_predicate_object_index(&target.triples);

    for (subject, acquirer_preds) in &acquirer_index {
        let Some(target_preds) = target_index.get(subject) else {
            continue;
        };
        for (predicate, acquirer_objects) in acquirer_preds {
            let Some(target_objects) = target_preds.get(predicate) else {
                continue;
            };
            if acquirer_objects != target_objects {
                let acquirer_value = acquirer_objects
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                let target_value = target_objects
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(OrgMergeRefused::IdentifierCollision {
                    subject: subject.clone(),
                    predicate: predicate.clone(),
                    acquirer_org: acquirer.org_label.clone(),
                    acquirer_value,
                    target_org: target.org_label.clone(),
                    target_value,
                });
            }
        }
    }
    Ok(())
}

/// Canonical (sorted-field) N-Triples-shaped display of one triple, for both
/// dedup-keying ([`union_triples`]) and receipt hashing ([`merge_receipt_hash`]).
/// Same line format as
/// `crate::f02_observation_admission`'s own private `receipt_hash` helper (kept as
/// a small, independent function here rather than exported, since the two modules'
/// hashing needs differ in what else gets mixed into the hasher).
fn canonical_triple_string(t: &Triple) -> String {
    format!(
        "{} {} {} .",
        term_display(&t.s),
        term_display(&t.p),
        term_display(&t.o)
    )
}

/// Unions two organizations' triples under ordinary RDF set semantics: a triple
/// asserted verbatim by both organizations contributes exactly one triple to the
/// merge, not two. Must be called only after [`detect_identifier_collisions`] has
/// already ruled out any genuine contradiction -- this function does not itself
/// check for divergence, it only dedupes exact matches.
///
/// # Complexity
/// O((A + T) log (A + T)) -- one `BTreeSet` insert (keyed by canonical string) per
/// triple across both graphs, A = acquirer triple count, T = target triple count.
fn union_triples(acquirer: &[Triple], target: &[Triple]) -> Vec<Triple> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut merged = Vec::with_capacity(acquirer.len() + target.len());
    for t in acquirer.iter().chain(target.iter()) {
        let key = canonical_triple_string(t);
        if seen.insert(key) {
            merged.push(t.clone());
        }
    }
    merged
}

/// Builds the merged graph's governing shape set by literal Turtle-text
/// concatenation of both organizations' own shape profiles, parsed once via the
/// real `ShapesGraph::parse`.
///
/// # Disclosed design choice: union-as-conjunction (item 14's "whose shape wins")
/// Neither organization's shapes are senior to the other's, and neither is
/// discarded. Because a Turtle/RDF graph is a SET of triples, shape triples that
/// are literally identical across both files (e.g. both organizations having
/// independently adopted the same public shape profile, as
/// `packs/ma-case-study-pack/fixtures/acquirer-org-shapes.ttl` /
/// `target-org-shapes.ttl` do) collapse for free under set union -- no
/// special-casing needed for that case. Where the two organizations' shapes
/// genuinely differ for the same target class (this module's fixture pair does
/// NOT exercise that harder case -- see this module's own "What this does and does
/// not close" section), the union is logically the AND of both constraint sets: a
/// merged individual must satisfy BOTH shape profiles simultaneously. A fact that
/// was valid under one organization's own (looser) shape but violates the other's
/// (stricter) shape becomes a real SHACL violation of the union and is refused,
/// never silently accepted because it happened to pass one side. This is the safe
/// default (fails closed) at the cost of being unable to merge two organizations
/// whose shapes are directly, irreconcilably contradictory for the same path on the
/// same class (e.g. one requires `sh:maxCount 1` where the other requires
/// `sh:minCount 2` on the identical property) -- that harder case has no answer
/// here and is disclosed, not solved, by this module.
///
/// # Complexity
/// O(A_s + T_s) to concatenate the two shape strings (A_s/T_s = their byte
/// lengths) plus whatever `ShapesGraph::parse` itself costs (documented on that
/// function).
// See `detect_identifier_collisions`'s `#[allow(clippy::result_large_err)]`
// comment for the rationale (`OrgMergeRefused::IdentifierCollision`'s 5
// `String` fields are a debuggability choice, not an oversight).
#[allow(clippy::result_large_err)]
fn union_of_shapes(
    acquirer_shapes: &str,
    target_shapes: &str,
) -> Result<ShapesGraph, OrgMergeRefused> {
    let combined = format!("{acquirer_shapes}\n\n{target_shapes}\n");
    ShapesGraph::parse(&combined)
        .map_err(|reason| OrgMergeRefused::MalformedMergedShapes { reason })
}

/// Terminal receipt of a successful merge. Neither `acquirer`'s nor `target`'s own
/// [`AdmissionReceipt`] is mutated, recomputed, or replaced by this struct's
/// construction -- this is an APPEND (a new receipt citing two prior chain heads),
/// never an edit (`docs/releases/v26.7.13/THESIS.md` Chapter 13.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeReceipt {
    /// The acquirer's own F02 [`AdmissionReceipt::receipt_hash`], cited verbatim.
    pub acquirer_receipt_hash: String,
    /// The target's own F02 [`AdmissionReceipt::receipt_hash`], cited verbatim.
    pub target_receipt_hash: String,
    /// Triple count of the merged (deduplicated) graph.
    pub merged_triple_count: usize,
    /// BLAKE3-hex over a fixed domain tag, both organizations' receipt hashes (in
    /// that fixed acquirer-then-target order -- not sorted, since which
    /// organization is the acquirer vs. target is itself a meaningful, ordered
    /// fact this receipt should not erase), and the merged graph's canonical
    /// (sorted) triple lines. No wall clock; deterministic given the same two
    /// inputs.
    pub merge_receipt_hash: String,
}

/// BLAKE3-hex receipt hash over both organizations' own receipt hashes plus the
/// merged graph's canonical (sorted) triple lines. Same `\x00`-delimited,
/// sort-before-hash pattern as
/// `crate::f02_observation_admission`'s own `receipt_hash` (invariant #5/#6: no
/// `HashMap`-order dependence, BLAKE3-only).
fn merge_receipt_hash(acquirer_hash: &str, target_hash: &str, merged: &[Triple]) -> String {
    let mut lines: Vec<String> = merged.iter().map(canonical_triple_string).collect();
    lines.sort();

    let mut hasher = blake3::Hasher::new();
    hasher.update(b"org-merge-v1\x00");
    hasher.update(acquirer_hash.as_bytes());
    hasher.update(b"\x00");
    hasher.update(target_hash.as_bytes());
    hasher.update(b"\x00");
    for line in &lines {
        hasher.update(line.as_bytes());
        hasher.update(b"\x00");
    }
    hex::encode(hasher.finalize().as_bytes())
}

/// Fuses two independently F02-admitted organization graphs into one re-validated
/// merged graph -- the first real, working instance of
/// `docs/standing/BOOTSTRAP_COLD_START_LIMITATIONS.md` item 14. Runs, in order:
///
/// 1. [`detect_identifier_collisions`] -- refuses on any genuine value
///    contradiction for a shared ground-IRI subject.
/// 2. [`union_triples`] -- RDF-set-semantics union (safe: (1) already ruled out
///    contradiction, so any remaining shared subject asserts only identical facts).
/// 3. [`union_of_shapes`] -- the merged graph must conform to BOTH organizations'
///    own shapes simultaneously (see that function's doc for the full rationale).
/// 4. Real SHACL re-validation of the merged graph (`Validator::validate`) against
///    the unioned shapes -- genuinely passes or genuinely, typedly refuses.
/// 5. [`MergeReceipt`] construction, referencing (never rewriting) both
///    organizations' own F02 admission receipt hashes.
///
/// # Errors
/// - [`OrgMergeRefused::IdentifierCollision`] -- see (1) above.
/// - [`OrgMergeRefused::EmptyMergedGraph`] -- degenerate input; should not occur
///   for any F02-admitted graph (F02's own Gate 0 already refuses zero-triple
///   payloads), checked here too rather than assumed.
/// - [`OrgMergeRefused::MalformedMergedShapes`] -- the concatenated shape Turtle
///   failed to parse (a setup-time defect in one organization's own shape file,
///   not a per-merge data problem).
/// - [`OrgMergeRefused::MergedShapeNonConformant`] -- the merged graph violates
///   the unioned shape set.
///
/// # Complexity
/// O((A + T) log (A + T) + S) -- collision detection and triple union are both
/// O(n log n) in the combined triple count (A = acquirer, T = target triple
/// count); shape re-validation is delegated to `Validator::validate`, whose own
/// complexity is documented in `praxis_graphlaw::shacl::validate`; S = the number
/// of shape-checked constraints over the merged graph.
// See `detect_identifier_collisions`'s `#[allow(clippy::result_large_err)]`
// comment for the rationale (`OrgMergeRefused::IdentifierCollision`'s 5
// `String` fields are a debuggability choice, not an oversight).
#[allow(clippy::result_large_err)]
pub fn merge_org_graphs(
    acquirer: &AdmittedOrgGraph,
    target: &AdmittedOrgGraph,
) -> Result<MergeReceipt, OrgMergeRefused> {
    detect_identifier_collisions(acquirer, target)?;

    let merged = union_triples(&acquirer.triples, &target.triples);
    if merged.is_empty() {
        return Err(OrgMergeRefused::EmptyMergedGraph);
    }

    let shapes = union_of_shapes(&acquirer.shapes_turtle, &target.shapes_turtle)?;

    let mut index = TripleIndex::new();
    for t in merged.iter().cloned() {
        index.add(t);
    }
    let report = Validator::validate(&index, &shapes);
    if !report.conforms {
        return Err(OrgMergeRefused::MergedShapeNonConformant {
            violation_count: report.results.len(),
        });
    }

    Ok(MergeReceipt {
        acquirer_receipt_hash: acquirer.receipt.receipt_hash.clone(),
        target_receipt_hash: target.receipt.receipt_hash.clone(),
        merged_triple_count: merged.len(),
        merge_receipt_hash: merge_receipt_hash(
            &acquirer.receipt.receipt_hash,
            &target.receipt.receipt_hash,
            &merged,
        ),
    })
}

#[cfg(test)]
mod tests {
    //! Focused unit tests over small inline fixtures (the full two-named-entity
    //! adversarial proof, using the real
    //! `packs/ma-case-study-pack/fixtures/{acquirer-org,target-org,
    //! target-org-colliding}.ttl` fixtures, lives in
    //! `crates/multifractal-workflow/tests/ma_org_merge.rs` per this repo's
    //! convention of small unit tests in-module + a larger fixture-driven
    //! integration test in `tests/`).

    use super::*;

    const SOURCE: &str = "test-source";
    const PRINCIPAL: &str = "https://source.example.org/test-source";
    const PASSING_SHAPES: &str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <https://example.org/vocab#> .
        ex:OrgShape a sh:NodeShape ;
            sh:targetClass ex:Org ;
            sh:property [ sh:path ex:name ; sh:minCount 1 ] .
    "#;

    fn policy() -> AdmissionPolicy {
        let mut known_principals = BTreeMap::new();
        known_principals.insert(SOURCE.to_string(), PRINCIPAL.to_string());
        let mut authorized = BTreeSet::new();
        authorized.insert("http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string());
        authorized.insert("https://example.org/vocab#name".to_string());
        let mut authorized_predicates = BTreeMap::new();
        authorized_predicates.insert(SOURCE.to_string(), authorized);
        AdmissionPolicy::new(
            known_principals,
            authorized_predicates,
            vec![
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
                "https://example.org/vocab#".to_string(),
            ],
            vec!["https://".to_string()],
            PASSING_SHAPES,
        )
        .expect("valid SHACL shapes in test fixture")
    }

    fn graph(correlation_id: &str, subject: &str, name: &str, label: &str) -> AdmittedOrgGraph {
        let obs = RawObservation {
            correlation_id: correlation_id.to_string(),
            source_id: SOURCE.to_string(),
            declared_subject: subject.to_string(),
            payload_turtle: format!(
                r#"
                @prefix ex: <https://example.org/vocab#> .
                @prefix prov: <http://www.w3.org/ns/prov#> .
                @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
                <{subject}> rdf:type ex:Org ;
                    prov:wasDerivedFrom <{PRINCIPAL}> ;
                    ex:name "{name}" .
                "#
            ),
        };
        admit_org_graph(
            &policy(),
            &AdmissionLedger::new(),
            obs,
            PASSING_SHAPES.to_string(),
            label,
        )
        .expect("gates should pass")
    }

    #[test]
    fn merge_succeeds_with_no_shared_subjects() {
        let acquirer = graph(
            "corr-a",
            "https://acquirer.example.org/entity/a",
            "Acquirer Co",
            "acquirer",
        );
        let target = graph(
            "corr-t",
            "https://target.example.org/entity/t",
            "Target Co",
            "target",
        );

        let receipt = merge_org_graphs(&acquirer, &target).expect("no collision, must merge");
        assert_eq!(receipt.merged_triple_count, 6);
        assert_eq!(receipt.acquirer_receipt_hash, acquirer.receipt.receipt_hash);
        assert_eq!(receipt.target_receipt_hash, target.receipt.receipt_hash);
        assert!(!receipt.merge_receipt_hash.is_empty());
    }

    #[test]
    fn merge_deterministic_across_repeated_calls() {
        let acquirer = graph(
            "corr-a2",
            "https://acquirer.example.org/entity/a2",
            "Acquirer Co",
            "acquirer",
        );
        let target = graph(
            "corr-t2",
            "https://target.example.org/entity/t2",
            "Target Co",
            "target",
        );

        let first = merge_org_graphs(&acquirer, &target).expect("merge 1");
        let second = merge_org_graphs(&acquirer, &target).expect("merge 2");
        assert_eq!(first, second);
    }

    #[test]
    fn merge_refuses_on_shared_subject_with_conflicting_object() {
        let shared_subject = "https://shared.example.org/entity/same-id";
        let acquirer = graph(
            "corr-collide-a",
            shared_subject,
            "Acquirer Value",
            "acquirer",
        );
        let target = graph("corr-collide-t", shared_subject, "Target Value", "target");

        let err = merge_org_graphs(&acquirer, &target)
            .expect_err("colliding subject with differing ex:name must refuse");
        assert!(matches!(
            err,
            OrgMergeRefused::IdentifierCollision { ref subject, .. }
            if subject == shared_subject
        ));
    }

    #[test]
    fn merge_does_not_refuse_on_shared_subject_with_identical_facts() {
        // Same subject, same asserted name -- a harmless overlap (e.g. both orgs
        // correctly citing the same shared reference), not a collision.
        let shared_subject = "https://shared.example.org/entity/agreed-id";
        let acquirer = graph("corr-agree-a", shared_subject, "Same Value", "acquirer");
        let target = graph("corr-agree-t", shared_subject, "Same Value", "target");

        let receipt =
            merge_org_graphs(&acquirer, &target).expect("identical shared facts must not refuse");
        // 3 acquirer triples + 3 target triples, all pairwise identical -> 3 after
        // dedup, not 6.
        assert_eq!(receipt.merged_triple_count, 3);
    }

    #[test]
    fn merge_refuses_shape_non_conformant_merged_graph() {
        // A merged shape set that requires ex:extra (present in neither org's own
        // payload) proves the union-of-shapes step is genuinely re-validated, not
        // rubber-stamped.
        let strict_shapes = r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <https://example.org/vocab#> .
            ex:OrgShape a sh:NodeShape ;
                sh:targetClass ex:Org ;
                sh:property [ sh:path ex:extra ; sh:minCount 1 ] .
        "#;
        let obs_a = RawObservation {
            correlation_id: "corr-strict-a".to_string(),
            source_id: SOURCE.to_string(),
            declared_subject: "https://acquirer.example.org/entity/strict-a".to_string(),
            payload_turtle: format!(
                r#"
                @prefix ex: <https://example.org/vocab#> .
                @prefix prov: <http://www.w3.org/ns/prov#> .
                @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
                <https://acquirer.example.org/entity/strict-a> rdf:type ex:Org ;
                    prov:wasDerivedFrom <{PRINCIPAL}> ;
                    ex:name "Acquirer" .
                "#
            ),
        };
        let acquirer = admit_org_graph(
            &policy(),
            &AdmissionLedger::new(),
            obs_a,
            PASSING_SHAPES.to_string(),
            "acquirer",
        )
        .expect("acquirer own-shape admission should pass (own shape has no ex:extra requirement)");
        let target = graph(
            "corr-strict-t",
            "https://target.example.org/entity/strict-t",
            "Target",
            "target",
        );

        // Union acquirer's PASSING shapes with target's STRICT shapes: the merged
        // graph now must satisfy target's ex:extra requirement too, which neither
        // org's own payload provides.
        let mut strict_target = target;
        strict_target.shapes_turtle = strict_shapes.to_string();

        let err = merge_org_graphs(&acquirer, &strict_target)
            .expect_err("merged graph missing ex:extra under the unioned shapes must refuse");
        assert!(matches!(
            err,
            OrgMergeRefused::MergedShapeNonConformant { violation_count } if violation_count >= 1
        ));
    }

    #[test]
    fn union_of_shapes_rejects_malformed_turtle() {
        // `ShapesGraph` (the `Ok` type) does not implement `Debug`, so
        // `.expect_err()`/`.unwrap_err()` (both require `T: Debug`) cannot be used
        // here -- match explicitly instead.
        match union_of_shapes(PASSING_SHAPES, "this is not { valid Turtle <<<") {
            Err(err) => assert!(matches!(err, OrgMergeRefused::MalformedMergedShapes { .. })),
            Ok(_) => panic!("malformed shape Turtle must refuse, not silently ignore the bad half"),
        }
    }
}
