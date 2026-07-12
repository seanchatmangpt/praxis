//! Frontier receipt — Lane 10 (Phase 4 of the CPhy frontier plan,
//! `~/.claude/plans/continue-work-on-the-elegant-wirth.md` and its lane-10
//! synthesis plan).
//!
//! Builds the [`wasm4pm_compat::dfcm::DfCmMatrix`] cataloguing every lawful
//! combination of `capability_source` (the sibling repos/crates explored
//! during the frontier planning session — see [`CAPABILITY_SOURCES`]) ×
//! `praxis_socket` (the concrete verbs/nouns, `praxis-core` modules, MCP
//! tools, config, and feature gates those sources were actually wired into,
//! or refused from — see [`PRAXIS_SOCKETS`]).
//!
//! # Method — reality, not the plan
//!
//! Every cell this module marks `Admitted`/`Executed` was verified by
//! actually running the integration during this session (a CLI invocation,
//! an integration test, or a `cargo test` suite) — see each cell's
//! `fixture` string for exactly what was run and its observed result.
//! Every cell marked impossible (`expected == Impossible`, `actual ==
//! Refused`) corresponds one-for-one to a row in the frontier plan's
//! refusal register; its `manufacture_witness` records what was salvaged
//! instead, so the refusal is a first-class receipt rather than a silent
//! gap.
//!
//! Cells for combinations that were never a candidate integration in this
//! session (e.g. `stpnt` × `plan-noun`) are deliberately left at their
//! default `Standing::Unknown` (unevaluated) — [`DfCmMatrix::coverage`]
//! measures exactly the fraction of the full Cartesian product that this
//! session actually touched (as an attempted integration *or* a stated
//! refusal), not a fabricated 100%. See `tests/frontier_matrix.rs` for the
//! coverage threshold and its justification.
//!
//! Single source of truth: [`build_frontier_matrix`] is shared by the
//! `frontier matrix` verb (`src/verbs/frontier.rs`) and the integration
//! test (`tests/frontier_matrix.rs`).

use std::{io, path::Path};

use serde::{Deserialize, Serialize};
use wasm4pm_compat::dfcm::{DfCmAxis, DfCmMatrix, DfCmReport, Standing};

/// `capability_source` axis variants — the sibling repos/crates explored
/// while mapping the frontier (see the master plan's Context section).
pub const CAPABILITY_SOURCES: &[&str] = &[
    "bcinr-pddl",
    "wasm4pm-prolog8-cognition",
    "wasm4pm-compat",
    "ggen-core-ggen-graph",
    "star-toml",
    "chatman-common",
    "lsp-max-andon",
    "mcpp-core",
    "stpnt",
    "clnrm-core",
    "open-ontologies",
    "affidavit",
    "ggen-mcp",
    "wasm4pm-planner",
    "ggen-core-v2",
    // Sources surveyed after the matrix was first built (Genesis Day 7
    // release sweep): each refused with reason + salvage below.
    "unibit",
    "dteam",
    "bytestar",
    "unrdf",
    "agent8",
    "powl2-decompose",
    "pddl-index",
];

/// `praxis_socket` axis variants — the concrete integration points inside
/// praxis: verbs/nouns, `praxis-core` modules, MCP tools, config, and
/// feature gates.
pub const PRAXIS_SOCKETS: &[&str] = &[
    "plan-noun",
    "mfg-noun",
    "receipt-noun",
    "law-noun",
    "signing",
    "admission",
    "config-noun",
    "verifier",
    "mcp-membrane",
    "hygiene",
    "testbed-noun",
    "diff-oracle",
    "frontier-noun",
];

/// Set a cell to `Admitted`/`Executed` with a fixture string describing the
/// concrete check that was run and its observed result.
fn admit(matrix: &mut DfCmMatrix, source: &str, socket: &str, executed: bool, fixture: &str) -> crate::error::Result<()> {
    let cell = matrix
        .find_cell_mut(&[source, socket])
        .ok_or_else(|| crate::error::AppError::Other(format!("cell ({source}, {socket}) not in expanded matrix")))?;
    cell.expected_standing = if executed {
        Standing::Executed
    } else {
        Standing::Admitted
    };
    cell.actual_standing = cell.expected_standing;
    cell.fixture = Some(fixture.to_string());
    Ok(())
}

/// Mark a cell as a refused ("impossible") combination, citing the refusal
/// reason and what was salvaged instead — matching the frontier plan's
/// refusal register exactly, one row per cell.
fn refuse(matrix: &mut DfCmMatrix, source: &str, socket: &str, reason: &str, salvage: &str) -> crate::error::Result<()> {
    let cell = matrix
        .find_cell_mut(&[source, socket])
        .ok_or_else(|| crate::error::AppError::Other(format!("cell ({source}, {socket}) not in expanded matrix")))?;
    cell.expected_standing = Standing::Impossible;
    cell.actual_standing = Standing::Refused;
    cell.is_impossible = true;
    cell.refusal_reason = Some(reason.to_string());
    cell.manufacture_witness = Some(salvage.to_string());
    Ok(())
}

/// Build the frontier `DfCmMatrix`, fully populated with the Cartesian
/// product of [`CAPABILITY_SOURCES`] × [`PRAXIS_SOCKETS`], with the cells
/// this session actually evaluated (admitted integrations and refusals)
/// filled in. All other cells remain `Standing::Unknown` (unevaluated —
/// never a candidate integration this session).
#[must_use]
pub fn build_frontier_matrix() -> crate::error::Result<DfCmMatrix> {
    let axes = vec![
        DfCmAxis {
            name: "capability_source".to_string(),
            description: Some(
                "Sibling repos/crates explored during the frontier planning session".to_string(),
            ),
            variants: CAPABILITY_SOURCES.iter().map(|s| s.to_string()).collect(),
        },
        DfCmAxis {
            name: "praxis_socket".to_string(),
            description: Some(
                "Verbs/nouns, praxis-core modules, MCP tools, config, and feature gates"
                    .to_string(),
            ),
            variants: PRAXIS_SOCKETS.iter().map(|s| s.to_string()).collect(),
        },
    ];

    let mut matrix = DfCmMatrix::new("cphy-frontier", axes);
    matrix.expand_cartesian();

    // ── Admitted / executed cells (verified by actually running the
    // integration in this session — see fixture strings) ──────────────────

    admit(
        &mut matrix,
        "bcinr-pddl",
        "plan-noun",
        true,
        "cargo run -- plan lawobject: manufactures ontology/lawobject.ttl, grounds+solves via \
         bcinr-pddl, returns the golden 5-step plan [supply-evidence, clear-obligations, judge, \
         admit, receipt]; observed admitted=true, plan_len=5",
    )?;
    admit(
        &mut matrix,
        "bcinr-pddl",
        "mfg-noun",
        true,
        "cargo test --features ggen --test mfg_golden: 4/4 passed (golden_roundtrip_and_solve, \
         determinism_byte_identical_across_runs, out_of_bounds_predicate_arity_rejected_before_emission, \
         facts_json_row_shape_matches_ggen_core_expectations) — manufactured PDDL8 text round-trips \
         through bcinr_pddl::domain_from_pddl/problem_from_pddl and solves",
    )?;
    admit(
        &mut matrix,
        "wasm4pm-prolog8-cognition",
        "law-noun",
        true,
        "cargo test --all-features --test snapshots_verbs: law_judge_prolog8_admitted passed \
         (src/ops.rs judge_payload builds a real prolog8::Kernel + wasm4pm_cognition::BreedStanding)",
    )?;
    admit(
        &mut matrix,
        "wasm4pm-prolog8-cognition",
        "admission",
        true,
        "cargo build --features andon; cargo test -p praxis-core --all-features: refusal.rs \
         totality tests + ops.rs run_kernel_query proof-carrying path (Kernel::query -> \
         RefusalScenario::Kernel*) all green",
    )?;
    admit(
        &mut matrix,
        "wasm4pm-compat",
        "receipt-noun",
        true,
        "cargo run -- receipt issue && receipt export-ocel: produced a real \
         wasm4pm_compat::ocel::OCEL JSON document with RFC3339 event timestamps; \
         cargo test -p praxis-core --all-features --test receipt_lane: 10/10 passed",
    )?;
    admit(
        &mut matrix,
        "wasm4pm-compat",
        "law-noun",
        true,
        "crates/praxis-core/src/law.rs: TryFrom<Obligation> for wasm4pm_compat::pddl::Precondition; \
         covered by the full praxis-core test suite (all green under --all-features)",
    )?;
    admit(
        &mut matrix,
        "wasm4pm-compat",
        "hygiene",
        true,
        "git -C ../wasm4pm-compat status --porcelain: clean tree at v26.6.29, matching praxis's \
         [patch.crates-io]/Cargo.toml pin; cargo build --all-features clean at HEAD",
    )?;
    admit(
        &mut matrix,
        "wasm4pm-compat",
        "frontier-noun",
        true,
        "this module: build_frontier_matrix() and tests/frontier_matrix.rs consume \
         wasm4pm_compat::dfcm::{DfCmMatrix, DfCmAxis, DfCmCell, Standing, DfCmReport} directly",
    )?;
    admit(
        &mut matrix,
        "ggen-core-ggen-graph",
        "mfg-noun",
        true,
        "src/mfg.rs uses ggen_graph::prelude::{parse_turtle, DeterministicGraph} directly; \
         mfg_golden's facts_json_row_shape_matches_ggen_core_expectations test passed",
    )?;
    admit(
        &mut matrix,
        "ggen-core-ggen-graph",
        "testbed-noun",
        true,
        "cargo test -p rust-fable-testbed: 4/5 walking_skeleton tests passed (1 ignored, requires \
         live ANTHROPIC_API_KEY) — crates/rust-fable-testbed uses ggen_core::prompt_mfg::PromptCompiler \
         end to end",
    )?;
    admit(
        &mut matrix,
        "star-toml",
        "config-noun",
        true,
        "cargo run -- config show && config witness: layered TrustedLoader admission succeeded, \
         returned an admitted PraxisConfig + BLAKE3 witness hash",
    )?;
    admit(
        &mut matrix,
        "chatman-common",
        "signing",
        true,
        "cargo run --features law-signed -- law receipt (PRAXIS_SIGNING_KEY set): produced a real \
         ed25519 signature via chatman_common::signed_receipt; law verify-signature returned \
         {\"status\":\"valid\"}",
    )?;
    admit(
        &mut matrix,
        "lsp-max-andon",
        "admission",
        true,
        "cargo run --features andon -- law judge (andon_ring:true): AndonRing::evaluate ran a real \
         lsp_max::andon probe against the payload and produced a live AndonEvent — confirms runtime \
         wiring, not a registry-health no-op",
    )?;
    admit(
        &mut matrix,
        "wasm4pm-planner",
        "diff-oracle",
        true,
        "cargo test --all-features --test differential: 8/8 passed (pair1_planners_* x4, \
         pair2_conformance_powl_vs_petri_agreement, pair2_blocker_dteam_dep, \
         pair3_chain_recompute_vs_independent_100_records, pair4_objective_score_bit_exact) — \
         bcinr-pddl and wasm4pm-planner agree on shared durative-STRIPS corpora across a \
         generated-case suite. Dev-dependency only (see the wasm4pm-planner x plan-noun refusal \
         cell: never wired into a production verb)",
    )?;

    // ── Impossible cells — the refusal register, one row per cell ──────────
    // (see ~/.claude/plans/continue-work-on-the-elegant-wirth.md and its
    // lane-10 synthesis plan for the canonical table this mirrors)

    refuse(
        &mut matrix,
        "stpnt",
        "admission",
        "dependency refused: stpnt's Cargo.toml has no `license` field (confirmed: `grep license \
         Cargo.toml` empty) — cannot depend on an unlicensed crate",
        "the 8-bucket RefusalCategory + RefusalScenario taxonomy shape was design-ported into \
         crates/praxis-core/src/refusal.rs, reimplemented from scratch against praxis's own \
         Obligation/DenialPolarity types, with prior-art citation in the module docs",
    )?;
    refuse(
        &mut matrix,
        "mcpp-core",
        "mcp-membrane",
        "vendoring refused: mcpp-core's manifest is workspace-coupled with a wasm4pm path \
         dependency it cannot be extracted from cleanly",
        "the sealed-verdict pattern is already mirrored by praxis-core's AdmittedConfig/Receipt \
         seals; mcp_lawobject_server is praxis's own rmcp-based server rather than a vendor of \
         mcpp-core's",
    )?;
    refuse(
        &mut matrix,
        "clnrm-core",
        "verifier",
        "dependency refused: ~49-transitive-dep footprint (found at /Users/sac/clnrm/crates/clnrm-core) \
         for what would be a single verification helper",
        "its EquivalenceViolation/ResourceContract shapes are noted as reference designs in \
         verify.rs's module docs only; not vendored",
    )?;
    refuse(
        &mut matrix,
        "open-ontologies",
        "receipt-noun",
        "dependency refused: fat deps (oxigraph+arrow+parquet+rmcp); its certify_action needs a \
         live StateDb/GraphStore this crate does not run",
        "its autoreceipt Receipt/builder design is noted as a reference only; its .ttl ontologies \
         are reused as mfg-noun inputs instead (see ontology/lawobject.ttl)",
    )?;
    refuse(
        &mut matrix,
        "affidavit",
        "receipt-noun",
        "dependency refused: chain rule incompatible with bcinr's (hex-prev + JSON mixing vs \
         raw-bytes + 99-byte little-endian encoding)",
        "the sealed-Receipt design was ported as praxis-core's ReceiptRecord shape (receipt_record.rs)",
    )?;
    refuse(
        &mut matrix,
        "affidavit",
        "verifier",
        "dependency refused: same chain-rule incompatibility carries into its verification pipeline",
        "the 7-stage verify pipeline + CheckOutcome/Verdict types were design-ported into \
         praxis-core/verify.rs",
    )?;
    refuse(
        &mut matrix,
        "ggen-mcp",
        "mcp-membrane",
        "dependency refused: sync_ggen is AppState-coupled and its SafeRenderer lacks register_all",
        "ggen-core's prompt_mfg and ggen-graph are used directly instead (mfg-noun, testbed-noun); \
         praxis built its own rmcp server rather than reusing ggen-mcp's",
    )?;
    refuse(
        &mut matrix,
        "wasm4pm-planner",
        "plan-noun",
        "production dependency refused: duplicate of bcinr-pddl as the planning substrate; praxis \
         already depends on bcinr-pddl for plan-noun",
        "kept as a dev-dependency-only oracle for cross-validation testing (see the \
         wasm4pm-planner x diff-oracle admitted cell) — never wired into a production verb",
    )?;
    refuse(
        &mut matrix,
        "ggen-core-v2",
        "mfg-noun",
        "dependency refused: ~1.3k lines vs ggen-core's ~143k; missing the SPARQL/Tera surface \
         mfg-noun requires",
        "tracked as a future migration candidate only; ggen-core v1 (via ggen-graph) is used for \
         mfg-noun today",
    )?;

    // ── Genesis Day 7 release sweep: capability sources surveyed after the
    // matrix was first built. Each is a first-class refusal (reason + salvage),
    // not a silent gap — closing the frontier report over the whole survey.
    refuse(
        &mut matrix,
        "unibit",
        "admission",
        "dependency refused: unibit's working tree is dirty (154 uncommitted files) — no \
         reproducible source to pin",
        "the harvest/admit4 admission semantics were design-ported, not vendored as a dependency",
    )?;
    refuse(
        &mut matrix,
        "dteam",
        "diff-oracle",
        "dependency refused: INSA-coupled — dteam cannot be extracted from its INSA workspace cleanly",
        "its bitmask_replay technique is reused differential-only (see the diff-oracle lane), not vendored",
    )?;
    refuse(
        &mut matrix,
        "bytestar",
        "admission",
        "dependency refused: C stubs / dormant — not a buildable Rust crate",
        "its design was ported; bytestar stands as the doctrine's C-era prehistory, cited not depended on",
    )?;
    refuse(
        &mut matrix,
        "unrdf",
        "receipt-noun",
        "dependency refused: unrdf is a Node.js runtime, not a Rust crate",
        "its knowledge-hooks / mu-reactive semantics were reimplemented in Rust; its .ttl receipt \
         shapes are referenced by src/receipt_shacl.rs",
    )?;
    refuse(
        &mut matrix,
        "agent8",
        "mcp-membrane",
        "dependency refused: no locatable artifact — agent8 is not present as a repo or crate in \
         the constellation; surveyed as a concept only",
        "none taken — deferred; recorded here so the survey has no silent omission",
    )?;
    refuse(
        &mut matrix,
        "powl2-decompose",
        "plan-noun",
        "not yet a landed socket: the Kourani WF-net -> POWL 2.0 decomposition is scoped as \
         crates/powl2-decompose (in-flight) but does not exist as an admitted dependency today",
        "POWL is already available via bcinr-powl / wasm4pm-compat for the parts that landed; the \
         decomposition itself is scoped design, tracked not claimed",
    )?;
    refuse(
        &mut matrix,
        "pddl-index",
        "plan-noun",
        "refused as a separate socket: the PDDL capability index is not a standalone dependency — \
         it is realized inside the already-admitted bcinr-pddl planner",
        "captured as docs/PDDL_CAPABILITY_MODEL.md and exercised through the admitted \
         bcinr-pddl x plan-noun cell",
    )?;

    Ok(matrix)
}

/// The number of cells this session actually evaluated (Admitted/Executed
/// integrations plus stated refusals) — i.e. `DfCmMatrix::evaluated()`.
#[must_use]
pub fn evaluated_count() -> crate::error::Result<usize> {
    Ok(build_frontier_matrix()?.evaluated())
}

/// The number of refused (Impossible) cells in the frontier.
#[must_use]
pub fn refused_count() -> crate::error::Result<usize> {
    Ok(build_frontier_matrix()?
        .cells
        .iter()
        .filter(|c| c.is_impossible)
        .count())
}

/// Summarise the frontier matrix into a serialisable [`DfCmReport`].
#[must_use]
pub fn frontier_report() -> crate::error::Result<DfCmReport> {
    Ok(DfCmReport::from_matrix(&build_frontier_matrix()?))
}

/// The full serialisable frontier artifact: the summary, plus the complete
/// matrix so every source×socket cell's disposition and salvage is
/// visible.
///
/// Serializing the whole matrix (not just the failure summary) keeps the
/// report honest: an unevaluated (Unknown) cell is visible as data, not
/// silently absent from the summary counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontierReport {
    /// Provenance: the function that produced this artifact. `String` (not
    /// `&'static str`) so `FrontierReport` can round-trip through
    /// `Deserialize` — a borrowed `'static` field cannot be deserialized
    /// from owned JSON input.
    pub generated_by: String,
    /// Summary counts (total / evaluated / passing / coverage / pass_rate /
    /// failures). Flattened into the top level of the serialized JSON (so
    /// e.g. `.pass_rate` reads directly, matching `scripts/walkthrough.sh`'s
    /// Release Criterion 3 probe and `just evidence-check`'s consumers)
    /// while remaining a normal named field (`report.summary.pass_rate`)
    /// on the Rust side.
    #[serde(flatten)]
    pub summary: DfCmReport,
    /// Every cell with its full disposition.
    pub matrix: DfCmMatrix,
}

/// Assemble the full [`FrontierReport`].
#[must_use]
pub fn full_report() -> crate::error::Result<FrontierReport> {
    let matrix = build_frontier_matrix()?;
    Ok(FrontierReport {
        generated_by: "my_conforming_project::frontier::build_frontier_matrix".to_string(),
        summary: DfCmReport::from_matrix(&matrix),
        matrix,
    })
}

/// Serialise the full frontier report to `path` (creating parent
/// directories) and return it. Pretty-printed and deterministic (no
/// timestamps in the report body itself).
///
/// # Errors
/// Returns any I/O error from creating the parent directory or writing the
/// file, or a serialization error surfaced as [`io::ErrorKind::Other`].
pub fn write_report(path: &Path) -> crate::error::Result<FrontierReport> {
    let report = full_report()?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(path, json + "\n")?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_expands_to_full_cartesian_product() {
        let matrix = build_frontier_matrix().unwrap();
        assert_eq!(
            matrix.total(),
            CAPABILITY_SOURCES.len() * PRAXIS_SOCKETS.len()
        );
    }

    #[test]
    fn matrix_validates_clean() {
        let matrix = build_frontier_matrix().unwrap();
        assert!(matrix.validate().is_empty(), "{:?}", matrix.validate());
    }

    #[test]
    fn every_evaluated_cell_passes() {
        let matrix = build_frontier_matrix().unwrap();
        let failing: Vec<_> = matrix
            .cells
            .iter()
            .filter(|c| c.actual_standing != Standing::Unknown && !c.passes())
            .map(|c| c.coords.clone())
            .collect();
        assert!(failing.is_empty(), "failing cells: {failing:?}");
    }

    #[test]
    fn impossible_cells_match_the_refusal_register_count() {
        let matrix = build_frontier_matrix().unwrap();
        let impossible_count = matrix.cells.iter().filter(|c| c.is_impossible).count();
        // One cell per refusal-register row wired up above (9 original +
        // 7 from the Genesis Day 7 release sweep).
        assert_eq!(impossible_count, 16);
    }

    #[test]
    fn write_report_round_trips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("frontier-report.json");
        let report = write_report(&path).expect("write_report");
        let contents = std::fs::read_to_string(&path).expect("read");
        let read_back: FrontierReport = serde_json::from_str(&contents).expect("parse");
        assert_eq!(read_back.summary.total, report.summary.total);
    }
}
