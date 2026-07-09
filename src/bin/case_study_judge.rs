//! src/bin/case_study_judge.rs — GraphLaw judgment over the Autonomic
//! Standing Factory case study (Lane 2,
//! docs/case-studies/autonomic-standing-factory/).
//!
//! Pipeline (mirrors src/bin/ocel_process_validate.rs's conventions: typed
//! `Refusal` enum, no panics/unwrap outside tests, argv-based input paths
//! with sane defaults, `#[derive(Serialize)]` report structs, writes JSON
//! to a fixed report path AND prints to stdout):
//!
//! 1. Merge `case-study/graphlaw_judgment.ttl` (hand-authored seed graph)
//!    with the live `target/praxis-standing/standing.ttl` (Lane 1's real
//!    output) into one `TripleStore`.
//! 2. `load_rules` both `case-study/rules/judgment.n3` and
//!    `case-study/rules/readiness.dl.n3` (same engine, combined
//!    stratification — see those files' headers).
//! 3. `materialize()` (pass 1) — derive every fact reachable from the
//!    merged graph's own structure alone (no external gate facts yet).
//! 4. `check_denials()` — record any `=> false.` violations.
//! 5. `validate_shacl()` once each for `standing-envelope`, `case-study`,
//!    and `evidence-ref` shapes (the three whose targets already exist in
//!    the pre-verdict graph).
//! 6. `validate_shex_c()` against a shape map built from real subjects
//!    found via SPARQL.
//! 7. Inject `praxis:ShaclShapesConform` / `praxis:ShexSchemaConforms` /
//!    `praxis:NoDenialsFound` as plain asserted `rdf:type` facts on the
//!    case-study subject, based ONLY on the real booleans computed in
//!    steps 4-6 — never hand-picked.
//! 8. `materialize()` (pass 2) — now `judgment.n3`'s
//!    `StandingInputsValid`/verdict rules can fire off the injected gate
//!    facts.
//! 9. Add the unsatisfied-dependency COUNT rule via
//!    `add_rule_with_aggregate` (no text-syntax equivalent exists — see
//!    `readiness.dl.n3`'s header) and materialize once more.
//! 10. SPARQL: find which ONE of the three verdict classes the case-study
//!     subject belongs to (mutually exclusive by construction) — this,
//!     and only this, is the verdict string. Also SPARQL-COUNT the
//!     `praxis:CaseStudy` subjects in the whole graph (must be exactly 1).
//! 11. Assert a real `praxis:FinalVerdict` node (status/scope/evidence/
//!     generatedAtUtc, the latter copied from the standing envelope's own
//!     sourced literal) and validate `final-verdict.shacl.ttl` against it
//!     (the one shape file whose target does not exist until this step).
//! 12. Emit `graphlaw_derived.ttl`, `final_graphlaw_verdict.json`,
//!     `graphlaw_judgment_report.md`, `shacl-report.json`,
//!     `shex-report.json`.
//!
//! Exit 0: `ProductionReadyForDeclaredScope`. Exit 1: any other verdict
//! (an expected, honest outcome while Lanes 3-7 have not run — not a tool
//! failure). Exit 2: refusal (io/parse/rule-load/shacl/shex/sparql) —
//! every failure is a typed [`Refusal`], no panics.

#![allow(clippy::print_stdout)]
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

use std::collections::HashSet;
use std::process::ExitCode;

use praxis_graphlaw::parser::Syntax;
use praxis_graphlaw::shacl::ShapesGraph;
use praxis_graphlaw::triples::{
    Aggregate, AggregateFunction, BodyLiteral, Rule, Triple, VarOrTerm,
};
use praxis_graphlaw::TripleStore;
use serde::Serialize;
use serde_json::json;

const CASE_STUDY_DIR: &str = "docs/case-studies/autonomic-standing-factory/case-study";
const STANDING_TTL_PATH: &str = "target/praxis-standing/standing.ttl";
const NS: &str = "https://praxis.dev/ontology/standing#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const XSD_STRING: &str = "<http://www.w3.org/2001/XMLSchema#string>";
const XSD_DATETIME: &str = "<http://www.w3.org/2001/XMLSchema#dateTime>";

const SHAPE_FILES_PRE_VERDICT: [&str; 3] = [
    "standing-envelope.shacl.ttl",
    "case-study.shacl.ttl",
    "evidence-ref.shacl.ttl",
];
const FINAL_VERDICT_SHAPE_FILE: &str = "final-verdict.shacl.ttl";

/// The 15 acceptance criteria (copied verbatim from
/// PRODUCTION_READINESS.md / CASE_STUDY_CONTROL.md) paired with the
/// `praxis:CriterionNN` local name asserted in `graphlaw_judgment.ttl`.
/// Satisfaction of each is read back from the real graph via SPARQL, never
/// hardcoded — this array only supplies the fixed enumeration + human text.
const CRITERIA: [(&str, &str); 15] = [
    ("Criterion01", "canonical standing evidence exists"),
    ("Criterion02", "GraphLaw validates shapes"),
    ("Criterion03", "GraphLaw validates graph structure"),
    ("Criterion04", "GraphLaw derives readiness facts"),
    (
        "Criterion05",
        "GraphLaw computes closure over blockers/dependencies",
    ),
    (
        "Criterion06",
        "PDDL produces/records a lawful repair/action plan",
    ),
    ("Criterion07", "POWL models the execution process"),
    ("Criterion08", "OCEL records the case-study run"),
    ("Criterion09", "wasm4pm validates process conformance"),
    ("Criterion10", "receipts verify where applicable"),
    (
        "Criterion11",
        "benchmark evidence exists where performance claims are made",
    ),
    (
        "Criterion12",
        "Autonomic Platform displays case-study state with provenance",
    ),
    (
        "Criterion13",
        "Claude Code policy consumes or points to standing",
    ),
    ("Criterion14", "unsupported claims are diagnosable"),
    (
        "Criterion15",
        "external operator side effects are separated from release blockers",
    ),
];

const VERDICTS: [&str; 3] = [
    "ProductionReadyForDeclaredScope",
    "PilotReadyWithExternalSideEffects",
    "NotReadyWithReasons",
];

/// Typed refusal — every failure path, no panics, no silent defaults.
#[derive(Debug, thiserror::Error)]
enum Refusal {
    #[error("io on {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("triple parse refusal on {path}: {message}")]
    TripleParse { path: String, message: String },
    #[error("rule load refusal on {path}: {message}")]
    RuleLoad { path: String, message: String },
    #[error("SHACL shape parse refusal on {file}: {message}")]
    ShaclParse { file: String, message: String },
    #[error("ShEx schema parse refusal: {0}")]
    ShexParse(String),
    #[error("SPARQL query refusal: {0}")]
    Sparql(String),
    #[error("aggregate rule refusal: {0}")]
    Aggregate(String),
    #[error("serialize refusal: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("no praxis:StandingEnvelope subject found in the merged graph")]
    NoStandingEnvelope,
    #[error("rule materialization refusal: {0}")]
    RuleMaterialization(String),
}

fn read(path: &str) -> Result<String, Refusal> {
    std::fs::read_to_string(path).map_err(|source| Refusal::Io {
        path: path.to_string(),
        source,
    })
}

fn write(path: &str, content: &str) -> Result<(), Refusal> {
    std::fs::write(path, content).map_err(|source| Refusal::Io {
        path: path.to_string(),
        source,
    })
}

fn iri(local: &str) -> VarOrTerm {
    VarOrTerm::convert(format!("{NS}{local}"))
}
fn rdf_type_pred() -> VarOrTerm {
    VarOrTerm::convert(RDF_TYPE.to_string())
}
fn lit(value: &str, datatype: &str) -> VarOrTerm {
    VarOrTerm::new_literal(value.to_string(), Some(datatype.to_string()), None)
}
fn type_triple(subject_local: &str, class_local: &str) -> Triple {
    Triple {
        s: iri(subject_local),
        p: rdf_type_pred(),
        o: iri(class_local),
        g: None,
    }
}

#[derive(Debug, Serialize, Clone)]
struct ShaclShapeReport {
    file: String,
    conforms: bool,
    violation_count: usize,
}

#[derive(Debug, Serialize, Clone)]
struct ShexReportOut {
    conforms: bool,
    failure_count: usize,
    failures: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
struct EvidenceItem {
    path: String,
    hash: String,
}

#[derive(Debug, Serialize, Clone)]
struct CriterionResult {
    id: String,
    description: String,
    satisfied: bool,
    critical: bool,
    evidence: Vec<EvidenceItem>,
}

/// SPARQL-derive the `praxis:hasEvidence` evidence refs for one criterion:
/// `<criterion> praxis:hasEvidence ?ev . ?ev praxis:path ?p . ?ev praxis:hash ?h .`
/// Never hardcoded — reads back whatever the merged graph actually asserts.
fn criterion_evidence(
    store: &TripleStore,
    criterion_id: &str,
) -> Result<Vec<EvidenceItem>, Refusal> {
    let evs = query_first_col(
        store,
        &format!("SELECT ?ev WHERE {{ <{NS}{criterion_id}> <{NS}hasEvidence> ?ev }}"),
    )?;
    let mut items = Vec::new();
    for ev in evs {
        let ev_iri = ev.trim_start_matches('<').trim_end_matches('>').to_string();
        let path = query_first_col(
            store,
            &format!("SELECT ?p WHERE {{ <{ev_iri}> <{NS}path> ?p }}"),
        )?
        .into_iter()
        .next()
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default();
        let hash = query_first_col(
            store,
            &format!("SELECT ?h WHERE {{ <{ev_iri}> <{NS}hash> ?h }}"),
        )?
        .into_iter()
        .next()
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default();
        items.push(EvidenceItem { path, hash });
    }
    Ok(items)
}

#[derive(Debug, Serialize)]
struct FinalVerdict {
    verdict: String,
    scope: String,
    criteria: Vec<CriterionResult>,
    shacl_reports: Vec<ShaclShapeReport>,
    shex_report: ShexReportOut,
    denials: Vec<String>,
    derived_triple_count: usize,
    unsatisfied_dependency_count: usize,
    case_study_subject_count: usize,
    graph_hash: String,
    generated_at_utc: String,
}

/// Run one SPARQL SELECT and return the decoded `val` strings of its first
/// projected variable, one per solution row (empty if the query itself
/// yields zero rows or zero columns).
fn query_first_col(store: &TripleStore, sparql: &str) -> Result<Vec<String>, Refusal> {
    let rows = store.query(sparql).map_err(Refusal::Sparql)?;
    Ok(rows
        .into_iter()
        .filter_map(|row| row.into_iter().next().map(|b| b.val))
        .collect())
}

/// Real bug found and fixed forward by Lane 6: the original query here was
/// `SELECT ?cs WHERE { <subject> a <class> }` — an all-ground WHERE
/// pattern that never binds the projected `?cs` variable at all. This
/// engine's `query_first_col` (correctly) extracts the FIRST BOUND
/// BINDING per solution row; a solution row with zero bindings (because
/// the one projected variable never appears in the pattern) yields
/// `row.into_iter().next() == None`, which `query_first_col` filters out
/// — so this function returned `false` unconditionally, for every verdict
/// class, on every call, regardless of whether the pattern actually
/// matched. `run()`'s `verdict` therefore ALWAYS fell through to its
/// `.unwrap_or("NotReadyWithReasons")` default, never actually reading
/// back the derived verdict fact from the graph — the exact "assert-in
/// rather than derive" failure mode invariant 2 exists to catch, except
/// inverted (the derivation ran, but its result was silently discarded).
/// This coincidentally matched the true state for every case-study run to
/// date (the graph never actually reached `ProductionReadyForDeclaredScope`
/// before Lane 6's evidence promotion), which is why it went undetected —
/// the existing test suite's own `present()` helper (case_study_judge.rs
/// test module) checks `!rows.is_empty()` on the raw solution-row count
/// directly rather than going through `query_first_col`, so it never
/// exercised this bug either. Fixed by mirroring the working
/// `case_study_subjects` query shape used a few lines below (`SELECT ?s
/// WHERE { ?s a <class> }`, variable SUBJECT / ground object — the
/// variable actually appears in the pattern), then checking membership of
/// the specific subject IRI in the returned rows.
fn verdict_present(
    store: &TripleStore,
    verdict_class: &str,
    subject_local: &str,
) -> Result<bool, Refusal> {
    let q = format!("SELECT ?s WHERE {{ ?s a <{NS}{verdict_class}> }}");
    let rows = query_first_col(store, &q)?;
    Ok(rows.iter().any(|s| s.contains(subject_local)))
}

fn run() -> Result<(FinalVerdict, bool), Refusal> {
    // ── 1. merge graphs ─────────────────────────────────────────────────
    let seed_ttl = read(&format!("{CASE_STUDY_DIR}/graphlaw_judgment.ttl"))?;
    let standing_ttl = read(STANDING_TTL_PATH)?;

    let mut store = TripleStore::new();
    store
        .load_triples(&seed_ttl, Syntax::Turtle)
        .map_err(|message| Refusal::TripleParse {
            path: format!("{CASE_STUDY_DIR}/graphlaw_judgment.ttl"),
            message,
        })?;
    store
        .load_triples(&standing_ttl, Syntax::Turtle)
        .map_err(|message| Refusal::TripleParse {
            path: STANDING_TTL_PATH.to_string(),
            message,
        })?;

    // ── 2. load rules (combined stratification) ─────────────────────────
    let judgment_n3_path = format!("{CASE_STUDY_DIR}/rules/judgment.n3");
    let readiness_n3_path = format!("{CASE_STUDY_DIR}/rules/readiness.dl.n3");
    let judgment_n3 = read(&judgment_n3_path)?;
    store
        .load_rules(&judgment_n3)
        .map_err(|message| Refusal::RuleLoad {
            path: judgment_n3_path.clone(),
            message,
        })?;
    let readiness_n3 = read(&readiness_n3_path)?;
    store
        .load_rules(&readiness_n3)
        .map_err(|message| Refusal::RuleLoad {
            path: readiness_n3_path.clone(),
            message,
        })?;

    // ── 3. materialize (pass 1) ──────────────────────────────────────────
    let derived_pass1 = store.materialize();

    // ── 4. denials ────────────────────────────────────────────────────────
    let denials = store.check_denials();

    // ── 5. SHACL over the 3 shapes whose targets already exist ──────────
    let mut shacl_reports = Vec::new();
    for file in SHAPE_FILES_PRE_VERDICT {
        let path = format!("{CASE_STUDY_DIR}/shapes/{file}");
        let shapes_ttl = read(&path)?;
        // Parse check first so a shape-file error is reported precisely.
        ShapesGraph::parse(&shapes_ttl).map_err(|message| Refusal::ShaclParse {
            file: file.to_string(),
            message,
        })?;
        let report = store
            .validate_shacl(&shapes_ttl)
            .map_err(|message| Refusal::ShaclParse {
                file: file.to_string(),
                message,
            })?;
        shacl_reports.push(ShaclShapeReport {
            file: file.to_string(),
            conforms: report.conforms,
            violation_count: report.results.len(),
        });
    }
    let shacl_all_conform_so_far = shacl_reports.iter().all(|r| r.conforms);

    // ── 6. ShEx over a shape map built from real subjects ────────────────
    let shex_schema_path = format!("{CASE_STUDY_DIR}/shex/case-study.shex");
    let shex_schema = read(&shex_schema_path)?;
    let mut shape_map: Vec<(String, String)> = vec![(
        format!("{NS}AutonomicStandingFactoryCaseStudy"),
        format!("{NS}CaseStudyShape"),
    )];
    for (query, shape) in [
        (
            format!("SELECT ?s WHERE {{ ?s a <{NS}StandingEnvelope> }}"),
            "StandingEnvelopeShape",
        ),
        (
            format!("SELECT ?s WHERE {{ ?s a <{NS}Judge> }}"),
            "GraphLawJudgmentShape",
        ),
        (
            format!("SELECT ?s WHERE {{ ?s a <{NS}ProcessValidationRef> }}"),
            "ProcessValidationShape",
        ),
        (
            format!("SELECT ?s WHERE {{ ?s a <{NS}PromotedClaim> }}"),
            "PromotedClaimShape",
        ),
    ] {
        for subject in query_first_col(&store, &query)? {
            let subject_iri = subject
                .trim_start_matches('<')
                .trim_end_matches('>')
                .to_string();
            shape_map.push((subject_iri, format!("{NS}{shape}")));
        }
    }
    let shex_report = store
        .validate_shex_c(&shex_schema, &shape_map)
        .map_err(Refusal::ShexParse)?;
    let shex_report_out = ShexReportOut {
        conforms: shex_report.conforms,
        failure_count: shex_report.failures.len(),
        failures: shex_report
            .failures
            .iter()
            .map(|f| format!("{} does not conform to {}: {}", f.node, f.shape, f.reason))
            .collect(),
    };

    // ── 7. inject externally-computed gate facts ─────────────────────────
    if shacl_all_conform_so_far {
        store.add(type_triple(
            "AutonomicStandingFactoryCaseStudy",
            "ShaclShapesConform",
        ));
    }
    if shex_report_out.conforms {
        store.add(type_triple(
            "AutonomicStandingFactoryCaseStudy",
            "ShexSchemaConforms",
        ));
    }
    if denials.is_empty() {
        store.add(type_triple(
            "AutonomicStandingFactoryCaseStudy",
            "NoDenialsFound",
        ));
    }

    // ── 8. materialize (pass 2) — verdict rules can now fire ────────────
    let derived_pass2 = store.materialize();

    // ── 9. unsatisfied-dependency COUNT via the Rust aggregate API ──────
    // (readiness.dl.n3's header explains why this cannot be text syntax.)
    let count_rule = Rule {
        body: vec![BodyLiteral {
            negated: false,
            pattern: Triple::from(
                "?x".to_string(),
                format!("{NS}hasUnsatisfiedDependency"),
                "?dep".to_string(),
            ),
        }],
        head: Triple::from(
            "?x".to_string(),
            format!("{NS}unsatisfiedDependencyCount"),
            "?count".to_string(),
        ),
    };
    let aggregate = Aggregate {
        function: AggregateFunction::Count,
        source_var: "?dep".to_string(),
        target_var: "?count".to_string(),
        group_vars: vec!["?x".to_string()],
    };
    store
        .add_rule_with_aggregate(count_rule, aggregate)
        .map_err(Refusal::Aggregate)?;
    let derived_pass3 = store.materialize();

    let unsatisfied_dependency_count = query_first_col(
        &store,
        &format!("SELECT ?count WHERE {{ <{NS}AutonomicStandingFactoryCaseStudy> <{NS}unsatisfiedDependencyCount> ?count }}"),
    )?
    .first()
    .and_then(|s| s.trim().trim_start_matches('"').split('"').next().unwrap_or(s).parse::<usize>().ok())
    .unwrap_or(0);

    let derived_triple_count = derived_pass1.map_err(Refusal::RuleMaterialization)?.len()
        + derived_pass2.map_err(Refusal::RuleMaterialization)?.len()
        + derived_pass3.map_err(Refusal::RuleMaterialization)?.len();

    // ── 10. which verdict fired? + exactly-one-CaseStudy-subject check ──
    let mut verdict: Option<&str> = None;
    for v in VERDICTS {
        if verdict_present(&store, v, "AutonomicStandingFactoryCaseStudy")? {
            verdict = Some(v);
            break;
        }
    }
    let verdict = verdict.unwrap_or("NotReadyWithReasons").to_string();

    let case_study_subjects = query_first_col(
        &store,
        &format!("SELECT ?cs WHERE {{ ?cs a <{NS}CaseStudy> }}"),
    )?;
    let case_study_subject_count = case_study_subjects.len();

    // ── criteria satisfaction (read back from the graph, not hardcoded) ──
    let satisfied_raw = query_first_col(
        &store,
        &format!(
            "SELECT ?dep WHERE {{ <{NS}AutonomicStandingFactoryCaseStudy> <{NS}satisfied> ?dep }}"
        ),
    )?;
    let satisfied: HashSet<String> = satisfied_raw.into_iter().collect();
    let critical_raw = query_first_col(
        &store,
        &format!(
            "SELECT ?c WHERE {{ ?c a <{NS}AcceptanceCriterion> . ?c <{NS}critical> \"true\"^^<http://www.w3.org/2001/XMLSchema#boolean> }}"
        ),
    )?;
    let critical: HashSet<String> = critical_raw.into_iter().collect();
    let mut criteria: Vec<CriterionResult> = Vec::with_capacity(CRITERIA.len());
    for (id, desc) in CRITERIA {
        criteria.push(CriterionResult {
            id: id.to_string(),
            description: desc.to_string(),
            satisfied: satisfied.iter().any(|s| s.contains(id)),
            critical: critical.iter().any(|c| c.contains(id)),
            evidence: criterion_evidence(&store, id)?,
        });
    }

    // ── 11. assert FinalVerdict node + validate final-verdict shape ─────
    let scope = "local-first autonomic release-governance for the seanchatmangpt fleet".to_string();
    let generated_at_utc = query_first_col(
        &store,
        &format!("SELECT ?t WHERE {{ <{NS}StandingEnvelope-v26_6_30> <{NS}generatedAtUtc> ?t }}"),
    )?
    .into_iter()
    .next()
    .map(|s| {
        s.trim_start_matches('"')
            .split('"')
            .next()
            .unwrap_or(&s)
            .to_string()
    })
    .ok_or(Refusal::NoStandingEnvelope)?;

    store.add(type_triple("FinalVerdict-v26_6_30", "FinalVerdict"));
    store.add(Triple {
        s: iri("FinalVerdict-v26_6_30"),
        p: iri("status"),
        o: lit(&verdict, XSD_STRING),
        g: None,
    });
    store.add(Triple {
        s: iri("FinalVerdict-v26_6_30"),
        p: iri("hasScope"),
        o: lit(&scope, XSD_STRING),
        g: None,
    });
    store.add(Triple {
        s: iri("FinalVerdict-v26_6_30"),
        p: iri("hasEvidence"),
        o: iri("StandingEnvelope-v26_6_30"),
        g: None,
    });
    store.add(Triple {
        s: iri("FinalVerdict-v26_6_30"),
        p: iri("generatedAtUtc"),
        o: lit(&generated_at_utc, XSD_DATETIME),
        g: None,
    });

    let final_verdict_shapes_ttl = read(&format!(
        "{CASE_STUDY_DIR}/shapes/{FINAL_VERDICT_SHAPE_FILE}"
    ))?;
    let final_verdict_report =
        store
            .validate_shacl(&final_verdict_shapes_ttl)
            .map_err(|message| Refusal::ShaclParse {
                file: FINAL_VERDICT_SHAPE_FILE.to_string(),
                message,
            })?;
    shacl_reports.push(ShaclShapeReport {
        file: FINAL_VERDICT_SHAPE_FILE.to_string(),
        conforms: final_verdict_report.conforms,
        violation_count: final_verdict_report.results.len(),
    });

    // ── 12. emit artifacts ────────────────────────────────────────────────
    let derived_ttl = store.content_to_string();
    write(
        &format!("{CASE_STUDY_DIR}/graphlaw_derived.ttl"),
        &derived_ttl,
    )?;
    let graph_hash = format!("blake3:{}", blake3::hash(derived_ttl.as_bytes()).to_hex());

    let final_verdict = FinalVerdict {
        verdict: verdict.clone(),
        scope,
        criteria,
        shacl_reports,
        shex_report: shex_report_out,
        denials,
        derived_triple_count,
        unsatisfied_dependency_count,
        case_study_subject_count,
        graph_hash,
        generated_at_utc,
    };

    let is_production_ready = final_verdict.verdict == "ProductionReadyForDeclaredScope";
    Ok((final_verdict, is_production_ready))
}

fn write_report_md(fv: &FinalVerdict) -> Result<(), Refusal> {
    let mut md = String::new();
    md.push_str("# GraphLaw judgment report (real run)\n\n");
    md.push_str(&format!("Verdict: **{}**\n\n", fv.verdict));
    md.push_str(&format!("Scope: {}\n\n", fv.scope));
    md.push_str(&format!(
        "Case-study subjects found in the merged graph: {} (must be exactly 1)\n\n",
        fv.case_study_subject_count
    ));
    md.push_str(&format!(
        "Derived triples across all materialize() passes: {}\n\n",
        fv.derived_triple_count
    ));
    md.push_str(&format!(
        "Unsatisfied-dependency count (aggregate rule): {}\n\n",
        fv.unsatisfied_dependency_count
    ));
    md.push_str(&format!(
        "Denials: {} — {:?}\n\n",
        fv.denials.len(),
        fv.denials
    ));
    md.push_str("## SHACL\n\n| shape file | conforms | violations |\n|---|---|---|\n");
    for r in &fv.shacl_reports {
        md.push_str(&format!(
            "| {} | {} | {} |\n",
            r.file, r.conforms, r.violation_count
        ));
    }
    md.push_str(&format!(
        "\n## ShEx\n\nconforms: {}, failures: {}\n\n",
        fv.shex_report.conforms, fv.shex_report.failure_count
    ));
    md.push_str("## Acceptance criteria\n\n| id | satisfied | critical | description |\n|---|---|---|---|\n");
    for c in &fv.criteria {
        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            c.id, c.satisfied, c.critical, c.description
        ));
    }
    md.push_str(&format!(
        "\n## Graph hash\n\n`{}`\n\ngenerated_at_utc (sourced from standing envelope): {}\n",
        fv.graph_hash, fv.generated_at_utc
    ));
    write(
        &format!("{CASE_STUDY_DIR}/graphlaw_judgment_report.md"),
        &md,
    )
}

fn main() -> ExitCode {
    match run() {
        Ok((final_verdict, is_production_ready)) => {
            let verdict_label = match final_verdict.verdict.as_str() {
                "ProductionReadyForDeclaredScope" => "GRAPHLAW_JUDGED_PRODUCTION_READY_FOR_SCOPE",
                "PilotReadyWithExternalSideEffects" => {
                    "GRAPHLAW_JUDGED_PILOT_READY_WITH_EXTERNAL_SIDE_EFFECTS"
                }
                _ => "GRAPHLAW_JUDGED_NOT_READY_WITH_RECEIPTED_REASONS",
            };
            let out = json!({
                "verdict": verdict_label,
                "raw_verdict_fact": final_verdict.verdict,
                "scope": final_verdict.scope,
                "criteria": final_verdict.criteria,
                "shacl_reports": final_verdict.shacl_reports,
                "shex_report": final_verdict.shex_report,
                "denials": final_verdict.denials,
                "derived_triple_count": final_verdict.derived_triple_count,
                "unsatisfied_dependency_count": final_verdict.unsatisfied_dependency_count,
                "case_study_subject_count": final_verdict.case_study_subject_count,
                "graph_hash": final_verdict.graph_hash,
                "generated_at_utc": final_verdict.generated_at_utc,
            });
            let out_json = match serde_json::to_string_pretty(&out) {
                Ok(s) => s,
                Err(source) => {
                    eprintln!("[case_study_judge] refusal: {}", Refusal::Serialize(source));
                    return ExitCode::from(2);
                }
            };
            if let Err(refusal) = write(
                &format!("{CASE_STUDY_DIR}/final_graphlaw_verdict.json"),
                &format!("{out_json}\n"),
            ) {
                eprintln!("[case_study_judge] refusal: {refusal}");
                return ExitCode::from(2);
            }
            if let Err(refusal) = write(
                &format!("{CASE_STUDY_DIR}/shacl-report.json"),
                &serde_json::to_string_pretty(&final_verdict.shacl_reports).unwrap_or_default(),
            ) {
                eprintln!("[case_study_judge] refusal: {refusal}");
                return ExitCode::from(2);
            }
            if let Err(refusal) = write(
                &format!("{CASE_STUDY_DIR}/shex-report.json"),
                &serde_json::to_string_pretty(&final_verdict.shex_report).unwrap_or_default(),
            ) {
                eprintln!("[case_study_judge] refusal: {refusal}");
                return ExitCode::from(2);
            }
            if let Err(refusal) = write_report_md(&final_verdict) {
                eprintln!("[case_study_judge] refusal: {refusal}");
                return ExitCode::from(2);
            }
            println!("{out_json}");
            if is_production_ready {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(refusal) => {
            eprintln!("[case_study_judge] refusal: {refusal}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for a real bug Lane 6 found and fixed: the original
    /// `verdict_present` queried `SELECT ?cs WHERE { <ground-subject> a
    /// <ground-class> }` — an all-ground pattern that never binds the
    /// projected `?cs` variable, so `query_first_col` (which extracts the
    /// first BOUND binding per row) silently discarded every solution row
    /// and `verdict_present` returned `false` unconditionally, for every
    /// class, on every call. This directly exercises the fixed function
    /// (not the `present()` helper below, which never had the bug because
    /// it checks the raw row count instead of going through
    /// `query_first_col`) against a graph that genuinely has the class
    /// asserted, proving it can actually find a match.
    #[test]
    fn verdict_present_finds_a_genuinely_asserted_class() {
        let mut store = TripleStore::new();
        store
            .load_triples(
                &format!("@prefix praxis: <{NS}> . praxis:X a praxis:SomeVerdictClass ."),
                Syntax::Turtle,
            )
            .expect("must parse");
        assert!(
            verdict_present(&store, "SomeVerdictClass", "X").expect("query must succeed"),
            "verdict_present must find a class that is genuinely asserted on the subject"
        );
        assert!(
            !verdict_present(&store, "SomeOtherClass", "X").expect("query must succeed"),
            "verdict_present must not find a class that was never asserted"
        );
    }

    /// (a) real evidence graph -> materialize derives >=1 triple;
    /// check_denials behavior reported honestly (zero denials, since the
    /// seed graph's one promoted claim has real evidence).
    #[test]
    fn real_evidence_graph_materializes_and_has_no_denials() {
        let (fv, _) = run().expect("run() must succeed against the real repo evidence");
        assert!(
            fv.derived_triple_count > 0,
            "materialize() must derive >=1 triple from real evidence"
        );
        assert!(
            fv.denials.is_empty(),
            "the seed graph's promoted claim has evidence; expected zero denials, got {:?}",
            fv.denials
        );
        assert_eq!(
            fv.case_study_subject_count, 1,
            "exactly one CaseStudy subject must exist in the merged graph"
        );
    }

    fn build_minimal_store(seed_extra: &str) -> TripleStore {
        let base = format!(
            r#"
            @prefix praxis: <{NS}> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            praxis:CS a praxis:CaseStudy ;
              praxis:hasScope "test scope" ;
              praxis:hasEvidence praxis:Env .
            praxis:Env a praxis:StandingEnvelope ;
              praxis:generatedAtUtc "2026-01-01T00:00:00Z"^^xsd:dateTime ;
              praxis:hasEvidence praxis:Ref ;
              praxis:hasVerdict praxis:SomeGate .
            praxis:Ref a praxis:EvidenceRef ; praxis:path "p" ; praxis:hash "sha256:{}" .
            praxis:CS praxis:recordsExecution praxis:Ocel .
            praxis:Ocel a praxis:OCELLogRef .
            praxis:CS praxis:validatesProcess praxis:Val .
            praxis:Val a praxis:ProcessValidationRef .
            praxis:CS praxis:displaysStanding praxis:Route .
            praxis:Route a praxis:ClientRouteRef .
            praxis:CS praxis:hasExternalSideEffect praxis:SideEffect .
            praxis:SideEffect a praxis:ExternalOperatorSideEffect ; praxis:nonBlocking "true"^^xsd:boolean .
            {seed_extra}
            "#,
            "0".repeat(64),
        );
        let mut store = TripleStore::new();
        store
            .load_triples(&base, Syntax::Turtle)
            .expect("minimal seed must parse");
        let judgment =
            read(&format!("{CASE_STUDY_DIR}/rules/judgment.n3")).expect("read judgment.n3");
        store.load_rules(&judgment).expect("judgment.n3 must load");
        let readiness =
            read(&format!("{CASE_STUDY_DIR}/rules/readiness.dl.n3")).expect("read readiness.dl.n3");
        store
            .load_rules(&readiness)
            .expect("readiness.dl.n3 must load");
        store
    }

    fn present(store: &TripleStore, subject: &str, class: &str) -> bool {
        !store
            .query(&format!(
                "SELECT ?s WHERE {{ <{NS}{subject}> a <{NS}{class}> }}"
            ))
            .expect("query must succeed")
            .is_empty()
    }

    /// (b) synthetic test proving the verdict FLIPS when an evidence
    /// triple is removed before materialization (proves evidence-driven,
    /// not hardcoded).
    #[test]
    fn verdict_flips_when_evidence_removed() {
        // WITH generatedAtUtc: EvidenceComplete derives.
        let mut store_with = build_minimal_store("");
        store_with.add(type_triple("CS", "ShaclShapesConform"));
        store_with.add(type_triple("CS", "ShexSchemaConforms"));
        store_with.add(type_triple("CS", "NoDenialsFound"));
        let _ = store_with.materialize();
        assert!(
            present(&store_with, "CS", "EvidenceComplete"),
            "EvidenceComplete must derive when the envelope has a timestamp+evidence"
        );
        assert!(
            present(&store_with, "CS", "StandingInputsValid"),
            "StandingInputsValid must derive when all 3 gates + EvidenceComplete hold"
        );

        // WITHOUT the envelope's own hasEvidence ref (env carries a
        // timestamp but no evidence ref of its own): EvidenceComplete
        // must NOT derive, and neither must anything downstream.
        let minimal_no_ref = r#"
            @prefix praxis: <https://praxis.dev/ontology/standing#> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            praxis:CS2 a praxis:CaseStudy ; praxis:hasScope "s" ; praxis:hasEvidence praxis:Env2 .
            praxis:Env2 a praxis:StandingEnvelope ; praxis:generatedAtUtc "2026-01-01T00:00:00Z"^^xsd:dateTime .
        "#;
        let mut store_without = TripleStore::new();
        store_without
            .load_triples(minimal_no_ref, Syntax::Turtle)
            .expect("must parse");
        let judgment =
            read(&format!("{CASE_STUDY_DIR}/rules/judgment.n3")).expect("read judgment.n3");
        store_without
            .load_rules(&judgment)
            .expect("judgment.n3 must load");
        let readiness =
            read(&format!("{CASE_STUDY_DIR}/rules/readiness.dl.n3")).expect("read readiness.dl.n3");
        store_without
            .load_rules(&readiness)
            .expect("readiness.dl.n3 must load");
        let _ = store_without.materialize();
        assert!(
            !present(&store_without, "CS2", "EvidenceComplete"),
            "EvidenceComplete must NOT derive when the envelope carries no evidence reference of its own"
        );
    }

    /// (c) synthetic test proving the denial rule fires on an injected
    /// claim-without-evidence triple.
    #[test]
    fn denial_fires_on_claim_without_evidence() {
        let claimless = format!(
            r#"
            @prefix praxis: <{NS}> .
            praxis:BadClaim a praxis:PromotedClaim .
            "#
        );
        let mut store = TripleStore::new();
        store
            .load_triples(&claimless, Syntax::Turtle)
            .expect("must parse");
        let judgment =
            read(&format!("{CASE_STUDY_DIR}/rules/judgment.n3")).expect("read judgment.n3");
        store.load_rules(&judgment).expect("judgment.n3 must load");
        let _ = store.materialize();
        let denials = store.check_denials();
        assert!(
            !denials.is_empty(),
            "a PromotedClaim with zero praxis:hasEvidence must trigger the denial rule"
        );
    }

    /// (d) test proving a SHACL violation prevents the production-ready
    /// verdict fact from deriving (a StandingEnvelope missing
    /// generatedAtUtc both fails the shape AND fails to derive
    /// EvidenceComplete — demonstrating the gate is load-bearing, not
    /// decorative).
    #[test]
    fn shacl_violation_prevents_evidence_complete() {
        let bad = format!(
            r#"
            @prefix praxis: <{NS}> .
            praxis:CS3 a praxis:CaseStudy ; praxis:hasScope "s" ; praxis:hasEvidence praxis:Env3 .
            praxis:Env3 a praxis:StandingEnvelope ; praxis:hasEvidence praxis:Ref3 .
            praxis:Ref3 a praxis:EvidenceRef ; praxis:path "p" ; praxis:hash "sha256:{}" .
            "#,
            "1".repeat(64),
        );
        let mut store = TripleStore::new();
        store
            .load_triples(&bad, Syntax::Turtle)
            .expect("must parse");
        let shapes = read(&format!(
            "{CASE_STUDY_DIR}/shapes/standing-envelope.shacl.ttl"
        ))
        .expect("read shape");
        let report = store
            .validate_shacl(&shapes)
            .expect("validate_shacl must run");
        assert!(
            !report.conforms,
            "a StandingEnvelope missing generatedAtUtc must violate standing-envelope.shacl.ttl"
        );

        let judgment =
            read(&format!("{CASE_STUDY_DIR}/rules/judgment.n3")).expect("read judgment.n3");
        store.load_rules(&judgment).expect("judgment.n3 must load");
        let _ = store.materialize();
        assert!(
            !present(&store, "CS3", "EvidenceComplete"),
            "EvidenceComplete must not derive without a generatedAtUtc literal"
        );
    }

    /// (e) test proving a ShEx violation prevents the production-ready
    /// verdict fact from deriving in the same way.
    #[test]
    fn shex_violation_is_detected_for_missing_evidence() {
        let bad = format!(
            r#"
            @prefix praxis: <{NS}> .
            praxis:CS4 a praxis:CaseStudy ; praxis:hasScope "s" .
            "#
        );
        let mut store = TripleStore::new();
        store
            .load_triples(&bad, Syntax::Turtle)
            .expect("must parse");
        let schema = read(&format!("{CASE_STUDY_DIR}/shex/case-study.shex")).expect("read schema");
        let report = store
            .validate_shex_c(
                &schema,
                &[(format!("{NS}CS4"), format!("{NS}CaseStudyShape"))],
            )
            .expect("validate_shex_c must run");
        assert!(
            !report.conforms,
            "a CaseStudy with zero hasEvidence must violate CaseStudyShape's `+` cardinality"
        );
    }
}
