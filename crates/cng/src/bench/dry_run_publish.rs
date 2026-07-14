//! Dry-run publish gate structural validator — Kestrel Toolkit release
//! candidate case study (v26.7.13).
//!
//! Drives the ENTIRE 6-gate dry-run publish Definition-of-Done lifecycle
//! (Release Scope & Identity through Receipt & Replay) through the real cng
//! manufacture chain: the 6 phase fixtures under
//! `tests/fixtures/dry-run-publish/` are domain FRAGMENTS of one shared
//! `kestrel-dry-run-publish-cycle` STRIPS domain (each contributing that
//! phase's 3 actions, chained by phase-completion predicates), plus one
//! problem fragment carrying the single init atom (`scope-engaged kestrel`)
//! and the single terminal goal atom (`dry-run-verified kestrel`).
//! `pipeline::import_artifacts` admits them, `generate_plan` grounds + plans
//! the full 18-step cycle, and `pipeline::hierarchical_projection` groups the
//! plan back into 6 gate children by artifact provenance — the dry-run
//! publish gate cycle as a hierarchical POWL. This mirrors
//! `crate::bench::soc2` and `crate::bench::togaf`'s structure exactly (same
//! pipeline, same `verify_eight_constraint_split` law, reused verbatim from
//! `togaf` rather than reimplemented here — it is already generic over
//! `AdmittedSurface`, not TOGAF-specific in its signature).
//!
//! # HONEST SCOPE (non-negotiable — no overclaiming)
//!
//! This module is a STRUCTURAL validator of the pack's PDDL MODEL: it parses
//! the checked-in dry-run-publish fixtures and mechanically asserts the
//! 6-phase completion chain, the DRY-RUN-OVERCLAIM FENCE, and the init/goal
//! atoms. It does NOT execute a real `cargo publish --dry-run`, does NOT
//! inspect the real workspace, and does NOT move the v26.7.13 Dry-Run Publish
//! Definition of Done off REFUSED (see
//! `docs/releases/v26.7.13/DRY_RUN_PUBLISH_VERDICT.md` — Gate 1's
//! clean-worktree/doc-sync and the B1–B7 blockers are unrelated to this
//! structural harness). What this closes is the narrow "bench harness absent /
//! 0 collected tests" gap: the dry-run-publish pack now has a structural
//! validator exactly like `soc2`/`togaf` do.
//!
//! # DRY-RUN-OVERCLAIM FENCE (non-negotiable, structural)
//!
//! A "dry run" publish check is a set of LOCAL, REVERSIBLE verification steps
//! (packaging, `cargo publish --dry-run`, clean-room unpack/build/test,
//! receipt replay) — it is NOT a crates.io upload, and NOTHING this module or
//! the fixtures it drives can ever BE a live release. Enforced structurally,
//! not just in prose: the merged PDDL domain has NO action whose effect names
//! an external mutation (`published`, `crates-io-uploaded`,
//! `release-complete`, or any synonym) — the only terminal goal atom is
//! `dry-run-verified`. [`verify_no_external_mutation_effects`] greps the
//! parsed, merged domain's action effects mechanically for those forbidden
//! substrings and refuses on any match; `dry_run_publish_test.rs` exercises it
//! on the real fixture set (must be clean) and adversarial mutants (must
//! refuse typed). This is the same line SOC2 draws between an evidence bundle
//! and an auditor's opinion (`soc2::verify_no_compliance_or_opinion_effects`):
//! this pipeline's terminal deliverable is evidence for a human release
//! manager's go/no-go decision, never the decision itself, and never a claim
//! that `cargo publish` (without `--dry-run`) ran.
//!
//! Fixture vocabulary is PUBLIC-ontology-first (blank nodes + `skos:notation`
//! handles; skos / prov / dcterms) — see
//! `packs/dry-run-publish-pack/ontology.ttl` for the full disclosure. The
//! only private predicates are the pipeline's own ABI
//! (`urn:chatman:engine#pddlDomain`/`#pddlProblem`, the PDDL text carrier,
//! `pipeline.rs`) and the `powl2:` output vocabulary (`powl.rs`); neither is
//! minted here.

use std::path::PathBuf;

use crate::pipeline::AdmittedSurface;
use crate::powl::CngRefusal;

use super::togaf::verify_eight_constraint_split;

/// The declared dry-run publish gate-phase order: (fixture file, SKOS
/// notation), in v26.7.13 Definition-of-Done order. Like `soc2::SOC2_PHASES`,
/// this is a declared reference table for tests (phase count, fixture
/// enumeration); the actual plan-step order is determined by the STRIPS
/// planner satisfying the precondition chain, not by this array's order or by
/// filesystem iteration order.
pub(super) const DRY_RUN_PHASES: [(&str, &str); 6] = [
    ("dry-run-scope.ttl", "DRY-RUN-SCOPE"),
    ("dry-run-generate.ttl", "DRY-RUN-GENERATE"),
    ("dry-run-verify.ttl", "DRY-RUN-VERIFY"),
    ("dry-run-manufacture.ttl", "DRY-RUN-MANUFACTURE"),
    ("dry-run-cleanroom.ttl", "DRY-RUN-CLEANROOM"),
    ("dry-run-receipt.ttl", "DRY-RUN-RECEIPT"),
];

/// The ordered gate-atom chain that carries the 6-phase cycle across fragment
/// boundaries: the problem's sole init atom, then each phase's terminal
/// completion atom in DoD order, ending at the terminal goal atom. Every
/// interior atom (index 1..=5) must be PRODUCED (add-effect) by exactly one
/// action and CONSUMED (precondition) by exactly one action — the bridge
/// between two adjacent phases; the init atom (index 0) is consumed but never
/// produced (it comes from the problem's `:init`); the terminal atom (index 6)
/// is produced but never consumed (nothing follows it). [`verify_gate_completion_chain`]
/// enforces exactly this, so a fragment that breaks the chain refuses loudly
/// instead of silently planning a shorter cycle.
pub(super) const DRY_RUN_GATE_CHAIN: [&str; 7] = [
    "scope-engaged",        // problem init atom (phase 1 entry)
    "scope-complete",       // phase 1 (Release Scope & Identity)  -> phase 2
    "generate-complete",    // phase 2 (Deterministic Generation)  -> phase 3
    "verify-complete",      // phase 3 (Verification Ladder)        -> phase 4
    "manufacture-complete", // phase 4 (Package Manufacture)        -> phase 5
    "cleanroom-complete",   // phase 5 (Clean-Room Verification)    -> phase 6
    "dry-run-verified",     // phase 6 (Receipt & Replay) terminal = merged goal atom
];

/// The forbidden external-mutation effect substrings the DRY-RUN-OVERCLAIM
/// FENCE bans from every action effect (case-insensitive). None of the
/// domain's real effect atoms (`*-step1-done`, `*-step2-done`, `*-complete`,
/// `dry-run-verified`) contains any of these — the middle Phase 4 action is
/// named `cargo-publish-dry-run-locked` but its EFFECT is
/// `manufacture-step2-done`, never `published`, so the fence greps effects,
/// not action names.
pub(super) const FORBIDDEN_EFFECT_SUBSTRINGS: [&str; 3] =
    ["published", "crates-io-uploaded", "release-complete"];

/// The sole init-atom predicate of the merged problem.
pub(super) const INIT_ATOM: &str = "scope-engaged";
/// The sole terminal/goal-atom predicate of the merged problem.
pub(super) const GOAL_ATOM: &str = "dry-run-verified";
/// The single release-candidate object the whole cycle plans over.
pub(super) const RELEASE_CANDIDATE: &str = "kestrel";

/// Directory holding the generated dry-run publish gate fixtures.
pub(super) fn dry_run_publish_fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dry-run-publish")
}

/// Structurally greps the merged, parsed `AdmittedSurface`'s action effects
/// for the three forbidden substrings the DRY-RUN-OVERCLAIM FENCE bans:
/// `published`, `crates-io-uploaded`, `release-complete` (case-insensitive).
/// This is the mechanical half of the fence — the doc-comment half states the
/// rule, this function (and `dry_run_publish_test.rs`'s use of it) makes a
/// violation impossible to miss. Mirrors
/// `soc2::verify_no_compliance_or_opinion_effects` exactly.
///
/// # Errors
/// `CNG_R05 UnsupportedConstruct` naming the first action/effect that violates
/// the fence.
///
/// # Complexity
/// O(actions × effects × forbidden) over the merged surface (forbidden is a
/// fixed length-3 constant, so this is O(actions × effects)).
pub fn verify_no_external_mutation_effects(surface: &AdmittedSurface) -> Result<(), CngRefusal> {
    for action in &surface.domain.actions {
        for effect in action.add_effects.iter().chain(action.del_effects.iter()) {
            let lowered = effect.pred.to_ascii_lowercase();
            for forbidden in FORBIDDEN_EFFECT_SUBSTRINGS {
                if lowered.contains(forbidden) {
                    return Err(CngRefusal::UnsupportedConstruct(format!(
                        "dry-run-overclaim fence violated: action `{}` has an effect atom `{}` \
                         naming an external mutation (`{forbidden}`); a dry run is LOCAL and \
                         REVERSIBLE — the only lawful terminal atom is `{GOAL_ATOM}`",
                        action.name, effect.pred
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Counts, over every action in the merged domain, how many have `pred` as a
/// precondition atom.
///
/// # Complexity
/// O(actions × preconditions).
fn precondition_consumers(surface: &AdmittedSurface, pred: &str) -> usize {
    surface
        .domain
        .actions
        .iter()
        .filter(|a| a.preconditions.iter().any(|p| p.pred == pred))
        .count()
}

/// Counts, over every action in the merged domain, how many have `pred` as an
/// add-effect atom.
///
/// # Complexity
/// O(actions × add_effects).
fn add_effect_producers(surface: &AdmittedSurface, pred: &str) -> usize {
    surface
        .domain
        .actions
        .iter()
        .filter(|a| a.add_effects.iter().any(|e| e.pred == pred))
        .count()
}

/// Verifies the 6-phase gate-completion chain ([`DRY_RUN_GATE_CHAIN`]) holds
/// structurally over the merged domain: the init atom is consumed once and
/// never produced; every interior completion atom is produced by exactly one
/// action and consumed by exactly one action (the bridge between two adjacent
/// phases); the terminal atom is produced once and never consumed. A broken
/// chain (a rewired or dropped bridge atom) refuses typed.
///
/// # Errors
/// `CNG_R05 UnsupportedConstruct` naming the first gate atom whose producer or
/// consumer count violates the chain law.
///
/// # Complexity
/// O(chain × actions × conjuncts) — chain is a fixed length-7 constant.
pub fn verify_gate_completion_chain(surface: &AdmittedSurface) -> Result<(), CngRefusal> {
    let last = DRY_RUN_GATE_CHAIN.len() - 1;
    for (idx, atom) in DRY_RUN_GATE_CHAIN.iter().enumerate() {
        let producers = add_effect_producers(surface, atom);
        let consumers = precondition_consumers(surface, atom);
        if idx == 0 {
            // Init atom: consumed by exactly the first action, produced by none.
            if producers != 0 {
                return Err(CngRefusal::UnsupportedConstruct(format!(
                    "gate chain broken: init atom `{atom}` must be produced by no action \
                     (it comes from the problem `:init`), found {producers} producer(s)"
                )));
            }
            if consumers != 1 {
                return Err(CngRefusal::UnsupportedConstruct(format!(
                    "gate chain broken: init atom `{atom}` must be a precondition of exactly \
                     one action (the first gate action), found {consumers}"
                )));
            }
        } else if idx == last {
            // Terminal atom: produced by exactly one action, consumed by none.
            if producers != 1 {
                return Err(CngRefusal::UnsupportedConstruct(format!(
                    "gate chain broken: terminal atom `{atom}` must be an add-effect of exactly \
                     one action, found {producers}"
                )));
            }
            if consumers != 0 {
                return Err(CngRefusal::UnsupportedConstruct(format!(
                    "gate chain broken: terminal atom `{atom}` must be a precondition of no \
                     action (nothing follows the goal), found {consumers}"
                )));
            }
        } else {
            // Interior bridge atom: produced once (end of phase i), consumed
            // once (start of phase i+1).
            if producers != 1 {
                return Err(CngRefusal::UnsupportedConstruct(format!(
                    "gate chain broken: completion atom `{atom}` must be an add-effect of \
                     exactly one action, found {producers}"
                )));
            }
            if consumers != 1 {
                return Err(CngRefusal::UnsupportedConstruct(format!(
                    "gate chain broken: completion atom `{atom}` must be a precondition of \
                     exactly one action (the next gate's entry), found {consumers}"
                )));
            }
        }
    }
    Ok(())
}

/// Verifies the merged problem's init and goal are exactly the single atoms
/// the cycle requires: `:init` is `(scope-engaged kestrel)` and `:goal` is
/// `(dry-run-verified kestrel)`, both single-conjunct, single-object. Dropping
/// the goal atom, changing the release-candidate object, or adding extra init
/// atoms refuses typed.
///
/// # Errors
/// `CNG_R05 UnsupportedConstruct` describing the init/goal shape that
/// violates the law.
///
/// # Complexity
/// O(1) over the fixed-size init/goal vectors.
pub fn verify_init_and_goal(surface: &AdmittedSurface) -> Result<(), CngRefusal> {
    let init = &surface.problem.init;
    if init.len() != 1 {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "init law broken: the merged problem must hold exactly one init atom \
             (`{INIT_ATOM} {RELEASE_CANDIDATE}`), found {}",
            init.len()
        )));
    }
    if init[0].pred != INIT_ATOM || init[0].args.len() != 1 || init[0].args[0] != RELEASE_CANDIDATE
    {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "init law broken: the sole init atom must be `({INIT_ATOM} {RELEASE_CANDIDATE})`, \
             found `({} {})`",
            init[0].pred,
            init[0].args.join(" ")
        )));
    }
    let goal = &surface.problem.goal;
    if goal.len() != 1 {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "goal law broken: the merged problem must hold exactly one goal atom \
             (`{GOAL_ATOM} {RELEASE_CANDIDATE}`), found {}",
            goal.len()
        )));
    }
    if goal[0].pred != GOAL_ATOM || goal[0].args.len() != 1 || goal[0].args[0] != RELEASE_CANDIDATE
    {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "goal law broken: the sole goal atom must be `({GOAL_ATOM} {RELEASE_CANDIDATE})`, \
             found `({} {})`",
            goal[0].pred,
            goal[0].args.join(" ")
        )));
    }
    Ok(())
}

/// Structural summary of a validated dry-run publish gate model. Mirrors
/// `soc2::Soc2EvidenceMetrics`'s report shape: a serializable struct
/// summarizing the gate/phase counts a caller can persist as a JSON artifact.
///
/// # HONEST SCOPE
/// Every field describes the parsed PDDL MODEL's structure — the count of
/// gate phases, actions, chain atoms — never the result of a real
/// `cargo publish --dry-run`. See this module's top doc comment.
#[derive(Debug, serde::Serialize)]
pub struct DryRunPublishReport {
    pub validation_class: &'static str,
    /// Number of dry-run gate phases (6 DoD categories).
    pub gate_phases: usize,
    /// Total action schemas in the merged domain (3 per phase × 6 phases).
    pub actions_total: usize,
    /// Actions contributed per phase fragment (`actions_total / gate_phases`).
    pub actions_per_phase: usize,
    /// Length of the verified gate-atom chain (init + 5 bridges + terminal).
    pub gate_chain_length: usize,
    /// The sole init atom predicate (`scope-engaged`).
    pub init_atom: &'static str,
    /// The sole terminal/goal atom predicate (`dry-run-verified`).
    pub goal_atom: &'static str,
    /// The single release-candidate object (`kestrel`).
    pub release_candidate: String,
    /// Count of action effects naming an external mutation. Always `0` on a
    /// validated model — the DRY-RUN-OVERCLAIM FENCE refuses before this
    /// report is built if any exist, so this field is a machine-readable
    /// witness that the fence held, not a discovered nonzero value.
    pub external_mutation_effects: usize,
}

impl DryRunPublishReport {
    /// Machine-readable class tag: this report validates the pack's PDDL
    /// MODEL structure, never a live publish (see [HONEST SCOPE](self)).
    pub const VALIDATION_CLASS: &'static str = "DRY_RUN_PUBLISH_MODEL_STRUCTURE";
}

/// Validates the full merged dry-run publish surface (8-constraint split,
/// external-mutation fence, 6-phase gate chain, init/goal atoms) and returns
/// the structural summary report. Any violated law refuses typed before the
/// report is built.
///
/// # Errors
/// The first `CngRefusal` any of the composed verifiers returns
/// (`verify_eight_constraint_split`, [`verify_no_external_mutation_effects`],
/// [`verify_gate_completion_chain`], [`verify_init_and_goal`]).
///
/// # Complexity
/// O(actions × conjuncts) over the merged surface.
pub fn validate_dry_run_publish_domain(
    surface: &AdmittedSurface,
) -> Result<DryRunPublishReport, CngRefusal> {
    verify_eight_constraint_split(surface)?;
    verify_no_external_mutation_effects(surface)?;
    verify_gate_completion_chain(surface)?;
    verify_init_and_goal(surface)?;

    let gate_phases = DRY_RUN_PHASES.len();
    let actions_total = surface.domain.actions.len();
    // `gate_phases` is the compile-time-constant `DRY_RUN_PHASES.len()` (6),
    // never zero; `checked_div` keeps the divide-by-zero branch honest without
    // a panic path, and the `unwrap_or(0)` fallback is unreachable here (0 is
    // the documented, deliberate value if the phase table were ever emptied).
    let actions_per_phase = actions_total.checked_div(gate_phases).unwrap_or(0);

    Ok(DryRunPublishReport {
        validation_class: DryRunPublishReport::VALIDATION_CLASS,
        gate_phases,
        actions_total,
        actions_per_phase,
        gate_chain_length: DRY_RUN_GATE_CHAIN.len(),
        init_atom: INIT_ATOM,
        goal_atom: GOAL_ATOM,
        release_candidate: RELEASE_CANDIDATE.to_string(),
        external_mutation_effects: 0,
    })
}

#[cfg(test)]
#[path = "dry_run_publish_test.rs"]
mod dry_run_publish_test;
