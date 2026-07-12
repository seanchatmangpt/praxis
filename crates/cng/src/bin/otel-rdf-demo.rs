//! Rail F/G reachability demo: wires the OTLP->RDF->OCEL->receipt chain into
//! ONE real, non-test entry point.
//!
//! `docs/jira/v26.7.11/PATH_TO_100.md` §5.2(a) named this the worst
//! reachability gap in the milestone: `otel_rdf::admit` ->
//! `project_admitted_spans` -> `admitted_spans_to_trig`,
//! `otel_ocel::project_otel_to_ocel`, and `otel_receipt::receipt_otel_to_ocel`
//! all had zero callers outside their own test files — "a closed loop", never
//! reached from `main.rs`/`pipeline.rs`/`runner.rs`/`otel-live.rs`. This
//! binary is that entry point. It adds no new logic: every step below calls
//! an already-tested `pub fn` verbatim.
//!
//! # Chain
//!
//! 1. [`otel_rdf::admit`] — re-validates one fixture `OtlpSpan` against the
//!    `event.praxis.activity_executed` contract.
//! 2. [`otel_rdf::project_admitted_spans`] — projects the admitted span into
//!    `G_OTEL` (`urn:graph:otel`) quads; [`otel_rdf::admitted_spans_to_trig`]
//!    additionally serializes that graph alone as TriG, printed first.
//! 3. [`otel_ocel::project_otel_to_ocel`] — SPARQL CONSTRUCT derives `G_OCEL`
//!    (`urn:graph:ocel`) from `G_OTEL`.
//! 4. [`otel_receipt::receipt_otel_to_ocel`] — computes the PROV-O ancestry +
//!    digest-chain receipt into `G_RECEIPT` (`urn:graph:receipts`), and
//!    [`otel_receipt::verify_receipt_otel_to_ocel`] independently replays the
//!    claimed `cngr:receiptHead` against the store to prove it is computed,
//!    never merely asserted.
//!
//! All three named graphs are inserted into one in-memory [`Store`] and
//! dumped as a single TriG document at the end (`--dump-only` skips step-by-step
//! prose and prints just that document, for piping to a file).
//!
//! Exit codes: 0 success, 1 any chain stage refused, 2 usage error.

use std::fmt;

use oxigraph::io::RdfFormat;
use oxigraph::model::{NamedNode, Term};
use oxigraph::store::Store;

use cng::otel_ocel::{self, insert_quads};
use cng::otel_rdf::{self, OtlpSpan, SpanStatus, SpanStatusCode};
use cng::otel_receipt;
use cng::powl::CngRefusal;
use cng::telemetry_gen;

/// Typed failure surface for this binary — no panics, no unwraps.
#[derive(Debug)]
enum DemoError {
    /// Bad CLI arguments (exit 2).
    Usage(String),
    /// Any chain stage (admit / project / receipt / store I/O) refused (exit 1).
    Chain(CngRefusal),
}

impl fmt::Display for DemoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(m) => write!(f, "usage error: {m}"),
            Self::Chain(e) => write!(f, "chain refused: {e}"),
        }
    }
}

impl From<CngRefusal> for DemoError {
    fn from(e: CngRefusal) -> Self {
        Self::Chain(e)
    }
}

impl DemoError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Chain(_) => 1,
        }
    }
}

/// The `cngr:receiptHead` predicate IRI, matching `otel_receipt.rs`'s own
/// `CNGR_NS` binding (`https://truex.io/ontology/cng-receipt#`) — that
/// constant is private to `otel_receipt`, so this binary names the same IRI
/// independently rather than reaching into the module's internals; drift
/// would be caught immediately (zero matches below) rather than silently.
const RECEIPT_HEAD_PRED_IRI: &str = "https://truex.io/ontology/cng-receipt#receiptHead";

/// One real, hand-constructed admissible span: all five required
/// `event.praxis.activity_executed` attributes present, `process.outcome`
/// in the closed vocabulary. Deliberately the same shape (not the identical
/// literal fixture — this binary does not import test-only code) as
/// `otel_rdf_test.rs::admissible_span()`, so the demo exercises the exact
/// contract the test suite already proves `admit` enforces.
fn fixture_span() -> OtlpSpan {
    OtlpSpan {
        trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
        span_id: "00f067aa0ba902b7".to_string(),
        parent_span_id: None,
        name: telemetry_gen::REGISTRY_GROUP_ID.to_string(),
        start_time_unix_nano: 1_700_000_000_000_000_000,
        end_time_unix_nano: 1_700_000_000_500_000_000,
        attributes: vec![
            (
                telemetry_gen::ATTR_WORKFLOW_ID.to_string(),
                "wf-demo-1".to_string(),
            ),
            (
                telemetry_gen::ATTR_OBJECT_ID.to_string(),
                "order-demo-1".to_string(),
            ),
            (
                telemetry_gen::ATTR_OBJECT_TYPE.to_string(),
                "Order".to_string(),
            ),
            (
                telemetry_gen::ATTR_ACTIVITY_IRI.to_string(),
                "urn:praxis:activity:ship-order".to_string(),
            ),
            (
                telemetry_gen::ATTR_OUTCOME.to_string(),
                "completed".to_string(),
            ),
        ],
        status: SpanStatus {
            code: SpanStatusCode::Ok,
            message: None,
        },
    }
}

/// Pulls the `cngr:receiptHead` literal out of the sealed receipt quads
/// `otel_receipt::receipt_otel_to_ocel` returned.
///
/// # Errors
/// `CngRefusal::IoRefused` if no such quad exists (would mean
/// `receipt_otel_to_ocel`'s own documented output shape changed underneath
/// this caller).
fn extract_receipt_head(receipt_quads: &[oxigraph::model::Quad]) -> Result<String, CngRefusal> {
    let pred = NamedNode::new(RECEIPT_HEAD_PRED_IRI).map_err(|e| {
        CngRefusal::IoRefused(format!(
            "receipt head predicate IRI construction failed: {e}"
        ))
    })?;
    receipt_quads
        .iter()
        .find(|q| q.predicate == pred)
        .and_then(|q| match &q.object {
            Term::Literal(lit) => Some(lit.value().to_string()),
            _ => None,
        })
        .ok_or_else(|| {
            CngRefusal::IoRefused("no cngr:receiptHead literal in receipt quads".to_string())
        })
}

fn run(dump_only: bool) -> Result<(), DemoError> {
    let span = fixture_span();

    // 1. admit: in-process re-validation of the same contract Weaver's live
    //    check enforces.
    otel_rdf::admit(&span)?;
    if !dump_only {
        println!(
            "[1/4] admit: span {}:{} ADMITTED",
            span.trace_id, span.span_id
        );
    }

    // `admitted_spans_to_trig` closes PATH_TO_100.md's named zero-caller
    // finding for this exact function: it internally re-runs `admit` +
    // `project_admitted_spans` and serializes G_OTEL alone as TriG.
    let otel_only_trig = otel_rdf::admitted_spans_to_trig(std::slice::from_ref(&span))?;
    if !dump_only {
        println!("[2/4] project_admitted_spans -> G_OTEL (urn:graph:otel), TriG:");
        println!("{otel_only_trig}");
    }

    // Materialize G_OTEL into a store shared by every later stage — mirrors
    // otel_ocel_test.rs::store_with_admitted_span's pattern, over the real
    // (non-test) `project_admitted_spans` entry point.
    let store = Store::new().map_err(|e| CngRefusal::IoRefused(format!("store: {e}")))?;
    let otel_quads = otel_rdf::project_admitted_spans(&[span])?;
    insert_quads(&store, &otel_quads)?;

    // 3. project_otel_to_ocel: SPARQL CONSTRUCT derives G_OCEL from G_OTEL.
    let ocel_quads = otel_ocel::project_otel_to_ocel(&store)?;
    insert_quads(&store, &ocel_quads)?;
    if !dump_only {
        println!(
            "[3/4] project_otel_to_ocel -> G_OCEL (urn:graph:ocel): {} quads",
            ocel_quads.len()
        );
    }

    // 4. receipt_otel_to_ocel: PROV-O ancestry + digest-chain receipt over
    //    the store's current G_OTEL/G_OCEL content.
    let receipt_quads = otel_receipt::receipt_otel_to_ocel(&store)?;
    insert_quads(&store, &receipt_quads)?;
    let receipt_head = extract_receipt_head(&receipt_quads)?;

    // Independently replay the claimed head against the store — proves the
    // printed value is computed and verifiable, not merely asserted.
    otel_receipt::verify_receipt_otel_to_ocel(&store, &receipt_head)?;

    if !dump_only {
        println!(
            "[4/4] receipt_otel_to_ocel -> G_RECEIPT (urn:graph:receipts): {} quads, verified",
            receipt_quads.len()
        );
        println!("RECEIPT_HEAD={receipt_head}");
        println!("--- full store TriG dump (G_OTEL + G_OCEL + G_RECEIPT) ---");
    }

    let bytes = store
        .dump_to_writer(RdfFormat::TriG, Vec::new())
        .map_err(|e| CngRefusal::IoRefused(format!("TriG serialization failed: {e}")))?;
    let trig = String::from_utf8(bytes)
        .map_err(|e| CngRefusal::IoRefused(format!("TriG output was not valid UTF-8: {e}")))?;
    println!("{trig}");

    if dump_only {
        eprintln!("RECEIPT_HEAD={receipt_head}");
    }

    Ok(())
}

fn parse_args(argv: &[String]) -> Result<bool, DemoError> {
    let mut dump_only = false;
    for arg in argv {
        match arg.as_str() {
            "--dump-only" => dump_only = true,
            "--help" | "-h" => {
                println!(
                    "otel-rdf-demo: admit -> project_otel_to_ocel -> receipt_otel_to_ocel over one fixture span"
                );
                println!("usage: otel-rdf-demo [--dump-only]");
                std::process::exit(0);
            }
            other => {
                return Err(DemoError::Usage(format!(
                    "unknown flag {other}; supported: --dump-only"
                )))
            }
        }
    }
    Ok(dump_only)
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let outcome = parse_args(&argv).and_then(run);
    match outcome {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            eprintln!("otel-rdf-demo: {err}");
            std::process::exit(err.exit_code());
        }
    }
}
