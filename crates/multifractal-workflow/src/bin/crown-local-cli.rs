//! `crown-local-cli` — the first real, non-test entry point for
//! [`multifractal_workflow::crown_local::drive_local_witness_prefix`].
//!
//! Closes a real "reachability ceiling" gap: `drive_local_witness_prefix` is a genuine,
//! non-`#[cfg(test)]` `pub fn` (see its own module doc, `crown_local.rs:1-142`) composing the
//! **entire LOCAL crown witness** — `F02 -> F03 -> F08 -> F09 -> F10 -> F11 -> F18 -> F19 ->
//! F02(re-admit) -> F24 -> F21 -> F25` — but before this binary, nothing outside its own test
//! file (`crown_local_test.rs`) ever called it. This binary drives it from a real file-system
//! input, not a hand-built in-process fixture.
//!
//! # Input contract
//!
//! Point this binary at a real RDF/Turtle file (`crown-local-cli <path>`) representing one
//! F02-admissible planning observation — for example, "an email arrived that needs a response".
//! The file must carry, on one subject:
//!
//! - `<http://www.w3.org/ns/prov#wasDerivedFrom> <https://connectors.example.org/email-connector-1>`
//!   (the CLI's one fixed, pre-registered known principal — F02 gate 2 requires this to match
//!   exactly, so a file asserting any other principal is genuinely refused, not silently
//!   accepted).
//! - `<urn:chatman:engine#pddlDomain> """...raw PDDL8 STRIPS domain text..."""`
//! - `<urn:chatman:engine#pddlProblem> """...raw PDDL8 STRIPS problem text..."""`
//! - `<urn:mfw:f08#hookPack> """...raw F19 hook-pack Turtle catalog text..."""`
//!
//! `examples/email-needs-response.ttl` (in this crate) is a real, checked-in example built to
//! this exact contract.
//!
//! # Why direct-text extraction for the three literal fields, not `Term`'s `Display`
//!
//! [`multifractal_workflow::crown_local`]'s own module doc discloses that praxis-graphlaw
//! literals expose no public lexical-value accessor, so recovering a literal's raw string value
//! by string-munging `Term`'s `Display` form is an escaping-asymmetry risk that module
//! deliberately declines to take. This binary inherits that same constraint but must recover
//! three raw multi-line text values (PDDL domain/problem, hook-pack Turtle) from the input file,
//! so it extracts them directly from the file's own bytes (documented convention: locate the
//! predicate IRI, then the first `"""..."""` block that follows it) rather than round-tripping
//! through the parsed `Term`. The subject IRI and provenance-principal IRI, by contrast, ARE
//! extracted from the real parsed triples (via [`praxis_graphlaw::parser::Parser::parse_triples`],
//! the same parser F02's own Ingress Adapter gate uses) — IRI terms have no escaping ambiguity in
//! their `Display` form (`<...>`, a plain bracket-trim), unlike literals.
//!
//! # Chain driven
//!
//! 1. Read + real-parse the input Turtle file (a local, pre-flight ingress check — F02's own
//!    Ingress Adapter gate re-parses independently downstream; this is not a substitute for it).
//! 2. Extract `subject_iri` / `source_principal_iri` (real IRI terms) and `pddl_domain` /
//!    `pddl_problem` / `hook_pack_turtle` (direct text extraction, see above).
//! 3. Build a real [`AdmissionPolicy`] (one known principal, one actuation principal, closed
//!    vocabulary, real — if vacuous — SHACL shapes) and a real
//!    [`multifractal_workflow::f09_mfw_growth::GrowthOutcome`] input (`Powl`/
//!    `RecursiveSocketClosure`), matching the shape `crown_local_test.rs`'s own fixture uses.
//! 4. Call [`drive_local_witness_prefix`] once, for real, over these inputs.
//! 5. Print every stage's real output: F02 admission receipt, F08 plan, F09/F10 growth +
//!    geometry, the real F18 [`BrokerReceipt`], F19 hook resolution, the F02 re-admission, F24
//!    OCEL construction, F21 closure admission, F25 receipt replay, and the final crown receipt.
//!
//! Exit codes: 0 success: full chain composed and printed. 1: a chain stage genuinely refused
//! (F02/F03/F08/F09/F10/F11/F18/F19/F24/F21/F25 — the refusal is printed verbatim). 2: usage,
//! I/O, or input-file-shape error (before any chain stage ran).
//!
//! # Complexity
//! O(1) CLI/file-I/O glue around [`drive_local_witness_prefix`]'s own documented complexity
//! (see that function's own `# Complexity` doc comment).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;

use powl2_decompose::{ParentChildClosure, Powl, SocketKind, SocketPath, WorkflowSocketId};
use praxis_graphlaw::chatman::closure::{ClosureLaw, RecursiveSocketClosure};
use praxis_graphlaw::parser::{Parser, Syntax};
use praxis_graphlaw::triples::{BodyLiteral, Rule, Term, Triple, VarOrTerm};

use multifractal_workflow::f02_observation_admission::{AdmissionPolicy, PROV_WAS_DERIVED_FROM};
use multifractal_workflow::f05_datalog_closure::RulePack;
use multifractal_workflow::f08_pddl_planning::projector::{
    HOOK_PACK_PREDICATE, PDDL_DOMAIN_PREDICATE, PDDL_PROBLEM_PREDICATE,
};
use multifractal_workflow::f18_broker_law::ActionId;
use multifractal_workflow::{
    crown_local::{drive_local_witness_prefix, LocalWitnessOutcome, LocalWitnessRefused},
    f02_observation_admission::AdmissionLedger,
};

/// The CLI's one pre-registered known principal ("email connector") — F02 gate 2 requires an
/// input file's `prov:wasDerivedFrom` object to match this exactly. A fixed value, not derived
/// from the file itself, so the gate stays real (a file that doesn't know this IRI is genuinely
/// refused, not trivially self-authorized).
const SOURCE_ID: &str = "email-connector-1";
const SOURCE_PRINCIPAL_IRI: &str = "https://connectors.example.org/email-connector-1";

/// F19 -> F02 re-admission principal: the local runtime's own identity, distinct from
/// `SOURCE_ID` (see `crown_local.rs`'s module doc F19->F02 nuance).
const ACTUATION_SOURCE_ID: &str = "crown-local-cli-runtime";
const ACTUATION_PRINCIPAL_IRI: &str = "urn:mfw:crown:local-cli-runtime";

/// SHACL shapes that conform vacuously (target class no admitted node carries), reused as both
/// F02's policy shapes and F03's contraction shapes — the identical pattern
/// `crown_local_test.rs`'s own `VACUOUS_SHAPES` fixture uses.
const VACUOUS_SHAPES: &str = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <urn:mfw:crown-local-cli#> .
ex:PlanningSnapshotShape a sh:NodeShape ;
    sh:targetClass ex:AbsentClass .
"#;

/// Typed, exhaustive failure surface for this binary. No panics, no unwraps on external input.
#[derive(Debug)]
enum CliError {
    /// Bad CLI arguments (exit 2).
    Usage(String),
    /// The input file could not be read (exit 2).
    Io { path: String, reason: String },
    /// The input file's Turtle failed to parse, or did not carry the required shape (exit 2) —
    /// a local pre-flight check, distinct from F02's own downstream Ingress Adapter gate.
    ObservationFileMalformed(String),
    /// The input file's declared `prov:wasDerivedFrom` object did not match this CLI's one
    /// pre-registered known principal (exit 2) — a fast, explicit pre-flight rejection, not a
    /// substitute for F02's own real Provenance Checker gate (F02 re-verifies independently
    /// downstream against the payload this binary reconstructs).
    UnrecognizedPrincipal { found: String, expected: String },
    /// This CLI's own fixed [`AdmissionPolicy`] failed to construct (defensive: the SHACL
    /// literal above is hand-verified, kept as a typed refusal rather than `.expect()` per this
    /// repo's no-panics-on-fallible-code invariant).
    PolicyInvalid(String),
    /// F09's real, open recursive-socket closure over the fixed 2-leaf growth root failed to
    /// declare (defensive, same rationale as `PolicyInvalid`).
    ClosureInvalid(String),
    /// The full LOCAL crown-witness chain genuinely refused at some stage (exit 1).
    Chain(LocalWitnessRefused),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(m) => write!(f, "usage error: {m}"),
            Self::Io { path, reason } => write!(f, "could not read {path}: {reason}"),
            Self::ObservationFileMalformed(reason) => {
                write!(f, "input observation file is malformed: {reason}")
            }
            Self::UnrecognizedPrincipal { found, expected } => write!(
                f,
                "input file's prov:wasDerivedFrom principal {found} does not match this CLI's \
                 one known principal {expected}; refusing before the chain ever runs"
            ),
            Self::PolicyInvalid(reason) => write!(f, "internal admission policy invalid: {reason}"),
            Self::ClosureInvalid(reason) => {
                write!(f, "internal growth-root closure invalid: {reason}")
            }
            Self::Chain(refusal) => write!(f, "crown-local chain refused: {refusal}"),
        }
    }
}

impl From<LocalWitnessRefused> for CliError {
    fn from(e: LocalWitnessRefused) -> Self {
        Self::Chain(e)
    }
}

impl CliError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Chain(_) => 1,
            _ => 2,
        }
    }
}

/// Extracts a bare (unbracketed) IRI string from an RDF term, or `None` if the term is not
/// `Term::Iri` (a literal, blank node, or unresolved variable never has "an IRI"). Only inspects
/// the variant tag and its own `Display` form — the same pattern `crown_local.rs`'s private
/// `bare_iri` helper and F02's own `bare_iri` helper both use; IRI `Display` is a plain
/// `<...>` bracket-trim with no escaping ambiguity (unlike literals).
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

/// Scans real parsed triples for the first `?s prov:wasDerivedFrom ?o` triple where both `?s`
/// and `?o` are IRI terms, returning `(subject_iri, principal_iri)` — the same structural check
/// F02's own Identity Resolver (gate 1) and Provenance Checker (gate 2) perform.
///
/// # Complexity
/// O(T) over the parsed triples, T = triple count.
fn find_subject_and_principal(triples: &[Triple]) -> Option<(String, String)> {
    let prov_pred = VarOrTerm::convert(PROV_WAS_DERIVED_FROM.to_string());
    triples.iter().find_map(|t| {
        if t.p != prov_pred {
            return None;
        }
        let subject = bare_iri(&t.s)?;
        let principal = bare_iri(&t.o)?;
        Some((subject, principal))
    })
}

/// Extracts the first `"""..."""` Turtle long-string literal that follows `<predicate>` in the
/// raw file text (this binary's own documented input-file convention; see the module doc's
/// "Why direct-text extraction" section for why this is used instead of round-tripping through
/// `Term`'s `Display`).
///
/// # Complexity
/// O(len(text)) for the three substring searches.
fn extract_triple_quoted_field(text: &str, predicate: &'static str) -> Result<String, CliError> {
    let marker = format!("<{predicate}>");
    let after_predicate = text
        .find(&marker)
        .map(|i| &text[i + marker.len()..])
        .ok_or_else(|| {
            CliError::ObservationFileMalformed(format!(
                "no <{predicate}> predicate found in the input file"
            ))
        })?;
    let quote_start = after_predicate.find("\"\"\"").ok_or_else(|| {
        CliError::ObservationFileMalformed(format!(
            "<{predicate}> has no following \"\"\"...\"\"\" long-string literal"
        ))
    })?;
    let rest = &after_predicate[quote_start + 3..];
    let quote_end = rest.find("\"\"\"").ok_or_else(|| {
        CliError::ObservationFileMalformed(format!(
            "<{predicate}>'s \"\"\"...\"\"\" long-string literal is never closed"
        ))
    })?;
    Ok(rest[..quote_end].to_string())
}

/// This CLI's one fixed [`AdmissionPolicy`]: two known principals (the external email connector
/// and the local runtime's own actuation identity), a closed authorized-predicate set per
/// principal, a closed vocabulary-prefix allowlist, and the vacuous-but-real SHACL shapes above.
/// Mirrors `crown_local_test.rs`'s own `crown_policy()` fixture, adapted to this CLI's own
/// principal names.
fn build_policy() -> Result<AdmissionPolicy, CliError> {
    let mut known_principals = BTreeMap::new();
    known_principals.insert(SOURCE_ID.to_string(), SOURCE_PRINCIPAL_IRI.to_string());
    known_principals.insert(
        ACTUATION_SOURCE_ID.to_string(),
        ACTUATION_PRINCIPAL_IRI.to_string(),
    );

    let mut planning_authorized = BTreeSet::new();
    planning_authorized.insert(PDDL_DOMAIN_PREDICATE.to_string());
    planning_authorized.insert(PDDL_PROBLEM_PREDICATE.to_string());
    planning_authorized.insert(HOOK_PACK_PREDICATE.to_string());
    let mut authorized_predicates = BTreeMap::new();
    authorized_predicates.insert(SOURCE_ID.to_string(), planning_authorized);

    let mut actuation_authorized = BTreeSet::new();
    actuation_authorized.insert("urn:mfw:f19#actuatedHookName".to_string());
    actuation_authorized.insert("urn:mfw:f19#actuationReceiptHash".to_string());
    actuation_authorized.insert("urn:mfw:f18#brokerReceiptHash".to_string());
    authorized_predicates.insert(ACTUATION_SOURCE_ID.to_string(), actuation_authorized);

    AdmissionPolicy::new(
        known_principals,
        authorized_predicates,
        vec![
            "urn:chatman:engine#".to_string(),
            "urn:mfw:f08#".to_string(),
            "urn:mfw:f19#".to_string(),
            "urn:mfw:f18#".to_string(),
        ],
        vec!["urn:".to_string(), "https://".to_string()],
        VACUOUS_SHAPES,
    )
    .map_err(CliError::PolicyInvalid)
}

/// A real, open recursive-socket closure over a 2-leaf `PartialOrder` root (nothing admitted
/// yet, so not already closed). Identical shape to `crown_local_test.rs`'s own
/// `open_growth_root_and_closure()` fixture — F09's growth operator needs a real root workflow
/// to graft the manufactured "respond to email" child into; the input file names the PDDL/hook
/// content, not the surrounding workflow topology (which this codebase has no RDF
/// serialization for — a disclosed input-format boundary, not a fabrication).
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

/// A real, stratifiable, single-rule Datalog pack that fires only on `crown-local-cli#Widget`
/// typed nodes (of which the admitted planning graph has none) — F03's stratifier reports a
/// spurious cycle for a zero-rule ruleset, so a harmless non-firing rule is required rather than
/// an empty pack. Identical shape to `crown_local_test.rs`'s own `harmless_rule_pack()` fixture.
fn harmless_rule_pack() -> RulePack {
    let rule = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/crown-local-cli#derivedFlag".to_string(),
            "\"yes\"".to_string(),
        ),
        body: vec![BodyLiteral {
            pattern: Triple::from(
                "?x".to_string(),
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                "http://example.org/crown-local-cli#Widget".to_string(),
            ),
            negated: false,
        }],
    };
    RulePack::new("crown-local-cli-widget-pack", vec![rule])
}

/// Everything this binary needs, extracted from one real input Turtle file.
struct ExtractedObservation {
    subject_iri: String,
    pddl_domain: String,
    pddl_problem: String,
    hook_pack_turtle: String,
}

/// Reads + real-parses `path`, then extracts the fields [`drive_local_witness_prefix`] needs.
///
/// # Complexity
/// O(len(file)) for the read plus the parse plus the three literal-field extractions.
fn load_observation(path: &str) -> Result<ExtractedObservation, CliError> {
    let text = fs::read_to_string(path).map_err(|e| CliError::Io {
        path: path.to_string(),
        reason: e.to_string(),
    })?;

    // Local pre-flight ingress check: confirm this is really parseable Turtle (the same parser
    // F02's own Ingress Adapter gate uses downstream) before doing any further work.
    let triples = Parser::parse_triples(&text, Syntax::Turtle)
        .map_err(|e| CliError::ObservationFileMalformed(format!("Turtle parse error: {e}")))?;
    if triples.is_empty() {
        return Err(CliError::ObservationFileMalformed(
            "file contains zero triples".to_string(),
        ));
    }

    let (subject_iri, principal_iri) = find_subject_and_principal(&triples).ok_or_else(|| {
        CliError::ObservationFileMalformed(
            "no <subject> prov:wasDerivedFrom <principal> triple (both IRI terms) found"
                .to_string(),
        )
    })?;
    if principal_iri != SOURCE_PRINCIPAL_IRI {
        return Err(CliError::UnrecognizedPrincipal {
            found: principal_iri,
            expected: SOURCE_PRINCIPAL_IRI.to_string(),
        });
    }

    let pddl_domain = extract_triple_quoted_field(&text, PDDL_DOMAIN_PREDICATE)?;
    let pddl_problem = extract_triple_quoted_field(&text, PDDL_PROBLEM_PREDICATE)?;
    let hook_pack_turtle = extract_triple_quoted_field(&text, HOOK_PACK_PREDICATE)?;

    Ok(ExtractedObservation {
        subject_iri,
        pddl_domain,
        pddl_problem,
        hook_pack_turtle,
    })
}

/// Prints every real stage output in [`LocalWitnessOutcome`] — the proof this chain actually
/// ran, not merely that it returned `Ok`.
fn print_outcome(outcome: &LocalWitnessOutcome) {
    println!(
        "[F02 admission]      state={:?} correlation_id={} subject={} receipt_hash={}",
        outcome.admission.state,
        outcome.admission.correlation_id,
        outcome.admission.subject_iri,
        outcome.admission.receipt_hash
    );
    println!(
        "[F03 contraction]    state={:?} receipt_head={}",
        outcome.planning_state.state,
        outcome.planning_state.receipt_head.to_hex()
    );
    println!(
        "[F08 plan]           goal_reached={} ops={} capability_map_iri={}",
        outcome.plan.receipt.goal_reached,
        outcome.plan.tape.ops.len(),
        outcome.plan.capability_map.iri
    );
    println!(
        "[F09/F10 growth]     geometry_leaves={} geometry_bindings={} geometry_turtle_len={}",
        outcome.growth.geometry_shape.leaves,
        outcome.growth.geometry_shape.child_bindings,
        outcome.growth.geometry_turtle.len()
    );
    println!(
        "[F11->F18 broker]    correlation_id={} authority_token_hex={} consequence_hash_hex={} \
         receipt_hash_hex={}",
        outcome.broker_receipt.correlation_id,
        outcome.broker_receipt.authority_token_hex,
        outcome.broker_receipt.consequence_hash_hex,
        outcome.broker_receipt.receipt_hash_hex
    );
    println!(
        "[F18->F19 hook]      state={:?} hook_name={} declared_authority={} receipt_hash={}",
        outcome.hook_resolution.state,
        outcome.hook_resolution.binding.hook_name,
        outcome.hook_resolution.declared_authority,
        outcome.hook_resolution.receipt_hash
    );
    println!(
        "[F19->F02 re-admit]  state={:?} correlation_id={} receipt_hash={}",
        outcome.actuation_admission.state,
        outcome.actuation_admission.correlation_id,
        outcome.actuation_admission.receipt_hash
    );
    println!(
        "[F24 OCEL construct] profile={:?} ocel_quads={} receipt_quads={} receipt_head={}",
        outcome.ocel_outcome.profile,
        outcome.ocel_outcome.ocel_quads.len(),
        outcome.ocel_outcome.receipt_quads.len(),
        outcome.ocel_outcome.receipt_head
    );
    println!(
        "[F24->F21 closure]   parent_closed={}",
        outcome.parent_closed
    );
    println!(
        "[F21->F25 replay]    matched_kinds={} receipt_root_matched={} receipt_root={}",
        outcome.replay_outcome.report.matched_kinds.len(),
        outcome.replay_outcome.report.receipt_root_matched,
        outcome.replay_outcome.receipt.receipt_root.as_str()
    );
    println!("CROWN_RECEIPT={}", outcome.crown_receipt);
}

fn run(path: &str, correlation_id: &str) -> Result<(), CliError> {
    let observation = load_observation(path)?;
    println!(
        "[input]              file={path} subject={} pddl_domain_bytes={} pddl_problem_bytes={} \
         hook_pack_bytes={}",
        observation.subject_iri,
        observation.pddl_domain.len(),
        observation.pddl_problem.len(),
        observation.hook_pack_turtle.len()
    );

    let policy = build_policy()?;
    let ledger = AdmissionLedger::new();
    let (growth_root, growth_closure) = open_growth_root_and_closure()?;

    let run = multifractal_workflow::crown_local::LocalWitnessRun {
        policy: &policy,
        ledger: &ledger,
        correlation_id: correlation_id.to_string(),
        source_id: SOURCE_ID.to_string(),
        subject_iri: observation.subject_iri.clone(),
        source_principal_iri: SOURCE_PRINCIPAL_IRI.to_string(),
        pddl_domain: observation.pddl_domain,
        pddl_problem: observation.pddl_problem,
        hook_pack_turtle: observation.hook_pack_turtle,
        datalog_rule_pack: harmless_rule_pack(),
        f03_shacl_shapes: VACUOUS_SHAPES.to_string(),
        growth_root,
        growth_closure,
        socket_blocked: true,
        descent_budget: 4,
        closure_law: ClosureLaw::AllRequired,
        broker_secret: [7u8; 32],
        action: ActionId::new(
            observation.subject_iri.clone(),
            "respond-to-email",
            correlation_id,
        ),
        actor: "crown-local-cli-actor".to_string(),
        has_standing: true,
        standing_reason: "crown-local-cli: CLI-caller-asserted standing for the email-response \
                           demo"
            .to_string(),
        local_run_id: [9u8; 32],
        local_max_ticks: 16,
        actuation_source_id: ACTUATION_SOURCE_ID.to_string(),
        actuation_principal_iri: ACTUATION_PRINCIPAL_IRI.to_string(),
    };

    let outcome = drive_local_witness_prefix(run)?;
    print_outcome(&outcome);
    Ok(())
}

fn parse_args(argv: &[String]) -> Result<(String, String), CliError> {
    let mut path: Option<String> = None;
    let mut correlation_id = "crown-local-cli-run-1".to_string();
    let mut iter = argv.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!(
                    "crown-local-cli: drive the entire LOCAL crown witness \
                     (F02->F03->F08->F09->F10->F11->F18->F19->F02(re-admit)->F24->F21->F25) \
                     over a real RDF/Turtle observation file"
                );
                println!("usage: crown-local-cli <observation.ttl> [--correlation-id ID]");
                std::process::exit(0);
            }
            "--correlation-id" => {
                let value = iter.next().ok_or_else(|| {
                    CliError::Usage("--correlation-id requires a value".to_string())
                })?;
                correlation_id = value.clone();
            }
            other if path.is_none() => path = Some(other.to_string()),
            other => {
                return Err(CliError::Usage(format!(
                    "unexpected extra argument: {other}"
                )))
            }
        }
    }
    let path = path.ok_or_else(|| {
        CliError::Usage("missing required <observation.ttl> file path argument".to_string())
    })?;
    Ok((path, correlation_id))
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let outcome = parse_args(&argv).and_then(|(path, correlation_id)| run(&path, &correlation_id));
    match outcome {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            eprintln!("crown-local-cli: {err}");
            std::process::exit(err.exit_code());
        }
    }
}
