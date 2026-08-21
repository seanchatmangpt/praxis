//! Increment 2 approval-seam backend for the `plan present` / `plan check` /
//! `plan step` CLI verbs (`crates/cng/src/main.rs`). Persists an append-only
//! JSONL ledger under `<ledger_dir>/plan-ledger.jsonl`: one `Presented`
//! event per `plan present` call and one `StepExecuted` event per
//! successful `plan step --approved` call, chain-hashed the same shape
//! `bench::dispatch::FileLedgerSink` chains its own Turtle ledger entries
//! (`blake3(prev | ... | seq)`), independently reimplemented here (that
//! type is `pub(super)` to `cng::bench`, unreachable from this sibling
//! module, and this ledger's payload is plain JSON, never Turtle, so
//! invariant #2's "no inline Turtle" rule never engages here).
//!
//! ## Approval boundary (a disclosed design decision, not a silent gap)
//!
//! cng has no identity-authenticated human-approval channel. Within cng's
//! own structural boundary, "approved" means exactly "a plan digest that
//! was disclosed via `plan present` before any action was proposed against
//! it" — one ledger status (Presented == Approved), not a Presented ->
//! Approved transition needing a fourth verb. The true human-in-the-loop
//! gate is external to cng (a human or orchestrator reads `plan present`'s
//! output and only then decides whether to ever call `check`/`step
//! --approved`); cng's enforceable contribution is refusing anything not
//! disclosed and refusing anything out of order.
//!
//! ## Plan digest: a disclosed deviation from the original design
//!
//! The original design for this module called for reusing chatman's own
//! canonical tape digest, `praxis_graphlaw::chatman::engine::tape_digest`
//! (receipt digest #7's algorithm), after widening its visibility to
//! `pub`. That prerequisite edit was explicitly out of this session's
//! scope (a `crates/praxis-graphlaw/src/chatman/` file, `chatman-rust`
//! agent territory per this repo's CLAUDE.md) — but even with that edit
//! made, the call would not compile today: `praxis-graphlaw`'s
//! `Cargo.toml` depends on `bcinr-pddl` via a hardcoded local path
//! (`{ path = "/Users/sac/bcinr/crates/bcinr-pddl" }`), while `cng`
//! depends on the crates.io-published `bcinr-pddl = "26.6.26"`; the
//! workspace root's `[patch.crates-io]` table unifies several of
//! `bcinr-pddl`'s own path dependencies (`wasm4pm-compat`, `bcinr-logic`)
//! but does not itself patch `bcinr-pddl` — `Cargo.lock` carries two
//! separate `bcinr-pddl` package entries as a result. (`Pddl8Tape` itself
//! turns out to be a plain `pub use wasm4pm_compat::pddl::Pddl8Tape`
//! re-export in both copies, and `wasm4pm-compat` IS patch-unified, so the
//! *type* would in fact be identical across the boundary — but
//! `tape_digest` is not `pub`, and widening it is out of scope this
//! session regardless.) So this module computes its own plan digest,
//! `compute_plan_digest` below: the identical *formula*
//! `chatman::engine::tape_digest`/`atoms_key` use (same fields, same
//! length-prefixed combine-then-hash construction), reimplemented locally
//! against cng's own already-hard `blake3` dependency — never a call into
//! `praxis-graphlaw`, and never `pipeline::plan_id` (that hashes only op
//! labels, not preconditions/effects/schema — not injective enough to
//! carry this seam's "exactly one lawful next action" guarantee).
//! Consequence: `plan_approval` needs no new dependency and no `bench`
//! feature gate — it, and the three CLI verbs that call it
//! (`crates/cng/src/main.rs`), ship in cng's default, unconditional build
//! surface, unlike `plan decompose` and the benchmark verbs.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use bcinr_pddl::{Pddl8GroundAtom, Pddl8Tape};
use serde::{Deserialize, Serialize};

use crate::pipeline;
use crate::powl::CngRefusal;

const PLAN_DIGEST_TAG: &str = "cng/plan_approval/plan_digest/v1";
const PLAN_LEDGER_FILE: &str = "plan-ledger.jsonl";

/// Joins ground atoms as `pred(a,b);pred2(c)` — mirrors
/// `chatman::engine::atoms_key`'s canonical join (atom order inside an
/// action is grounding-deterministic, hence canonical); reimplemented here
/// rather than imported because the source function is private to
/// `praxis-graphlaw::chatman::engine` (see module doc).
///
/// # Complexity
/// O(total atom bytes).
fn atoms_key(atoms: &[Pddl8GroundAtom]) -> String {
    let mut out = String::new();
    // O(a) over atoms.
    for (i, atom) in atoms.iter().enumerate() {
        if i > 0 {
            out.push(';');
        }
        out.push_str(&atom.pred);
        out.push('(');
        out.push_str(&atom.args.join(","));
        out.push(')');
    }
    out
}

/// Length-prefix-then-concatenate-then-hash combine, identical in shape to
/// `wasm4pm_compat::hash::blake3_combined` (16 hex chars of byte length,
/// `:`, then the part) — reimplemented locally against cng's own `blake3`
/// dependency rather than imported (see module doc); the length prefix is
/// what makes the combine injective across a split boundary.
///
/// # Complexity
/// O(total part bytes).
fn blake3_combined_local(parts: &[&str]) -> String {
    let mut combined = String::new();
    // O(parts) with O(1) work per part beyond its own byte length.
    for part in parts {
        combined.push_str(&format!("{:016x}:", part.len()));
        combined.push_str(part);
    }
    blake3::hash(combined.as_bytes()).to_hex().to_string()
}

/// Canonical, injective plan digest over a tape's full semantic content
/// (index, label, pred_mask, schema name, and precondition/add/del atom
/// keys per op) — the same fields `chatman::engine::tape_digest` hashes,
/// under an independent tag (`PLAN_DIGEST_TAG`) so this is never mistaken
/// for chatman receipt digest #7 (no sealed `StageSeal`/
/// `EngineProcessReceipt` was ever involved in producing it). Deliberately
/// stronger than `pipeline::plan_id` (labels only) — the approval seam's
/// "exactly one lawful next action" check needs the digest to uniquely
/// identify the plan's full content, not just its step names.
///
/// # Complexity
/// O(tape bytes).
fn compute_plan_digest(tape: &Pddl8Tape) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(tape.ops.len() * 7 + 1);
    parts.push(PLAN_DIGEST_TAG.to_string());
    // O(n) over ops: index order is canonical for a plan.
    for op in &tape.ops {
        parts.push(op.index.to_string());
        parts.push(op.label.clone());
        parts.push(op.pred_mask.to_string());
        parts.push(op.action.schema_name.clone());
        parts.push(atoms_key(&op.action.preconditions));
        parts.push(atoms_key(&op.action.add_effects));
        parts.push(atoms_key(&op.action.del_effects));
    }
    let refs: Vec<&str> = parts.iter().map(String::as_str).collect();
    format!("blake3:{}", blake3_combined_local(&refs))
}

/// Folds one ledger link: `blake3(prev | plan_digest | step_label |
/// step_index | logical_seq)`, hex — mirrors the shape of
/// `bench::dispatch::ledger_chain_hash` (private to that sibling module,
/// reimplemented here, not imported; see module doc).
///
/// # Complexity
/// O(1).
fn ledger_chain_hash(
    prev: &str,
    plan_digest: &str,
    step_label: &str,
    step_index: u64,
    logical_seq: u64,
) -> String {
    let mut h = blake3::Hasher::new();
    h.update(prev.as_bytes());
    h.update(plan_digest.as_bytes());
    h.update(step_label.as_bytes());
    h.update(step_index.to_string().as_bytes());
    h.update(logical_seq.to_string().as_bytes());
    h.finalize().to_hex().to_string()
}

/// One ledger line. `#[serde(tag = "event")]` so the JSONL file is
/// self-describing per line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
enum PlanLedgerEvent {
    Presented {
        plan_digest: String,
        steps: Vec<String>,
        source_dir: String,
        /// Logical monotonic counter — never wall clock.
        logical_seq: u64,
    },
    StepExecuted {
        plan_digest: String,
        step_index: usize,
        step_label: String,
        prev_chain: String,
        chain_hash: String,
        /// Logical monotonic counter — never wall clock.
        logical_seq: u64,
    },
}

/// In-memory state for one presented plan, reconstructed by replaying the
/// ledger.
#[derive(Debug, Clone)]
struct PlanRecord {
    steps: Vec<String>,
    next_step_index: usize,
    chain_head: String,
}

/// Append-only JSONL ledger at `<ledger_dir>/plan-ledger.jsonl`. Reload
/// reconstructs full in-memory state by replaying every line (mirrors
/// `bench::dispatch::FileLedgerSink`'s resume/tamper-detection shape); a
/// line that fails to parse, references an unknown `plan_digest`, or whose
/// recomputed chain hash does not match the recorded one refuses the
/// existing `CNG_R11 AuditMismatch` (no new variant needed — invariant #1).
struct PlanLedger {
    path: PathBuf,
    records: BTreeMap<String, PlanRecord>,
    /// `1 + max(logical_seq)` seen on reload — a logical monotonic counter,
    /// never wall clock.
    next_seq: u64,
}

impl PlanLedger {
    /// Opens (creating `ledger_dir` if absent) and reloads the ledger.
    ///
    /// # Errors
    /// `CNG_R10 IoRefused` for unreadable/uncreatable paths; `CNG_R11
    /// AuditMismatch` for a torn/tampered ledger.
    ///
    /// # Complexity
    /// O(ledger bytes).
    fn open(ledger_dir: &Path) -> Result<Self, CngRefusal> {
        fs::create_dir_all(ledger_dir)
            .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", ledger_dir.display())))?;
        let mut ledger = PlanLedger {
            path: ledger_dir.join(PLAN_LEDGER_FILE),
            records: BTreeMap::new(),
            next_seq: 0,
        };
        ledger.reload()?;
        Ok(ledger)
    }

    /// Replays every JSONL line into `records`/`next_seq`.
    ///
    /// # Complexity
    /// O(ledger bytes) parse + O(events) fold.
    fn reload(&mut self) -> Result<(), CngRefusal> {
        self.records.clear();
        self.next_seq = 0;
        if !self.path.exists() {
            return Ok(());
        }
        let text = fs::read_to_string(&self.path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", self.path.display())))?;
        // O(lines): each line is one ledger event, folded in file order.
        for (i, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let event: PlanLedgerEvent = serde_json::from_str(line).map_err(|e| {
                CngRefusal::AuditMismatch(format!(
                    "plan ledger {} line {}: failed to parse as a ledger event: {e}",
                    self.path.display(),
                    i + 1
                ))
            })?;
            self.apply_event(event)?;
        }
        Ok(())
    }

    /// Folds one event into `records`/`next_seq`, verifying order and chain
    /// hash for `StepExecuted`. `CNG_R11 AuditMismatch` on any violation.
    ///
    /// # Complexity
    /// O(1) amortized per event.
    fn apply_event(&mut self, event: PlanLedgerEvent) -> Result<(), CngRefusal> {
        match event {
            PlanLedgerEvent::Presented {
                plan_digest,
                steps,
                logical_seq,
                ..
            } => {
                if logical_seq >= self.next_seq {
                    self.next_seq = logical_seq + 1;
                }
                // First Presented for a digest wins; a replayed duplicate is
                // idempotent (present_plan only ever appends one).
                self.records.entry(plan_digest).or_insert(PlanRecord {
                    steps,
                    next_step_index: 0,
                    chain_head: String::new(),
                });
            }
            PlanLedgerEvent::StepExecuted {
                plan_digest,
                step_index,
                step_label,
                prev_chain,
                chain_hash,
                logical_seq,
            } => {
                if logical_seq >= self.next_seq {
                    self.next_seq = logical_seq + 1;
                }
                let path_display = self.path.display().to_string();
                let record = self.records.get_mut(&plan_digest).ok_or_else(|| {
                    CngRefusal::AuditMismatch(format!(
                        "plan ledger {path_display}: step_executed references unknown \
                         plan_digest {plan_digest} (never presented)"
                    ))
                })?;
                if step_index != record.next_step_index {
                    return Err(CngRefusal::AuditMismatch(format!(
                        "plan ledger {path_display}: step_executed out of order for plan \
                         {plan_digest} (expected step {}, recorded step {step_index})",
                        record.next_step_index
                    )));
                }
                if record.steps.get(step_index) != Some(&step_label) {
                    return Err(CngRefusal::AuditMismatch(format!(
                        "plan ledger {path_display}: step_executed label mismatch for plan \
                         {plan_digest} step {step_index}"
                    )));
                }
                if prev_chain != record.chain_head {
                    return Err(CngRefusal::AuditMismatch(format!(
                        "plan ledger {path_display}: step_executed prev_chain does not match \
                         the recorded chain head for plan {plan_digest} step {step_index}"
                    )));
                }
                let recomputed = ledger_chain_hash(
                    &prev_chain,
                    &plan_digest,
                    &step_label,
                    step_index as u64,
                    logical_seq,
                );
                if recomputed != chain_hash {
                    return Err(CngRefusal::AuditMismatch(format!(
                        "plan ledger {path_display}: recomputed chain hash does not match the \
                         recorded chain_hash for plan {plan_digest} step {step_index}"
                    )));
                }
                record.chain_head = chain_hash;
                record.next_step_index += 1;
            }
        }
        Ok(())
    }

    /// Appends one JSON line, then folds it into in-memory state.
    ///
    /// # Complexity
    /// O(|event| serialized bytes).
    fn append(&mut self, event: PlanLedgerEvent) -> Result<(), CngRefusal> {
        let line = serde_json::to_string(&event)
            .map_err(|e| CngRefusal::IoRefused(format!("serialize ledger event: {e}")))?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| CngRefusal::IoRefused(format!("open {}: {e}", self.path.display())))?;
        writeln!(file, "{line}")
            .map_err(|e| CngRefusal::IoRefused(format!("append {}: {e}", self.path.display())))?;
        self.apply_event(event)
    }
}

/// `plan present`'s result: the disclosed plan digest and its step labels
/// in tape order.
#[derive(Debug, Clone)]
pub struct PresentedPlan {
    pub imported_pddl_ttl_paths: Vec<String>,
    pub plan_digest: String,
    pub steps: Vec<String>,
    pub ledger_dir: String,
    /// worker id → Datalog-derived role, real role-inference over
    /// `pipeline::import_roster`'s roster facts for this directory (see
    /// `derive_roster_roles`). Empty when the artifact set carries no
    /// roster triples — an arbitrary PDDL-only planning artifact has none,
    /// and that is reported honestly, not fabricated.
    #[cfg(feature = "role-inference")]
    pub roster_roles: BTreeMap<String, String>,
    /// worker id → Datalog-derived `:obligation` atom, parallel to
    /// `roster_roles`.
    #[cfg(feature = "role-inference")]
    pub roster_obligations: BTreeMap<String, String>,
}

/// Real, non-bench-fixture role inference for the live plan-admit path:
/// scans `dir` for roster triples (`pipeline::import_roster`) and, if any
/// exist, runs the SAME praxis-graphlaw Datalog engine bench uses
/// (`crate::roles::derive_roles_datalog`) over the on-disk
/// `crates/cng/rules/bench-roles.dl` rule set. Returns `Ok(None)` — not an
/// error — when the artifact set carries no roster facts: this is the
/// disclosed generalization boundary (see `crate::roles`'s module doc),
/// not a silent failure.
///
/// # Errors
/// See `pipeline::import_roster`; `CNG_R05 UnsupportedConstruct` /
/// `CNG_R09 HardcodingSuspicion` from `derive_roles_datalog` itself.
#[cfg(feature = "role-inference")]
pub fn derive_roster_roles(dir: &Path) -> Result<Option<crate::roles::DatalogRoles>, CngRefusal> {
    const RULES_TEXT: &str = include_str!("../rules/bench-roles.dl");
    let workers = pipeline::import_roster(dir)?;
    if workers.is_empty() {
        return Ok(None);
    }
    Ok(Some(crate::roles::derive_roles_datalog(
        &workers,
        RULES_TEXT,
    )?))
}

/// Imports + plans `dir` once, computes its digest, and — if this exact
/// digest has never been presented against `ledger_dir` before — appends
/// one `Presented` ledger event. Never executes anything: this call HALTs
/// for external approval (a human or orchestrator reading its return
/// value) before any `check`/`step` call is lawful against this digest.
///
/// # Errors
/// See `pipeline::import_artifacts`/`generate_plan`; `CNG_R10 IoRefused` /
/// `CNG_R11 AuditMismatch` from the ledger.
///
/// # Complexity
/// O(pipeline cost) + O(ledger bytes) reload.
pub fn present_plan(dir: &Path, ledger_dir: &Path) -> Result<PresentedPlan, CngRefusal> {
    let artifacts = pipeline::import_artifacts(dir)?;
    let (tape, _surface) = pipeline::generate_plan(&artifacts)?;
    let plan_digest = compute_plan_digest(&tape);
    let steps: Vec<String> = tape.ops.iter().map(|op| op.label.clone()).collect();
    let mut ledger = PlanLedger::open(ledger_dir)?;
    if !ledger.records.contains_key(&plan_digest) {
        ledger.append(PlanLedgerEvent::Presented {
            plan_digest: plan_digest.clone(),
            steps: steps.clone(),
            source_dir: dir.display().to_string(),
            logical_seq: ledger.next_seq,
        })?;
    }
    #[cfg(feature = "role-inference")]
    let (roster_roles, roster_obligations) = match derive_roster_roles(dir)? {
        Some(roles) => (roles.derived, roles.obligations),
        None => (BTreeMap::new(), BTreeMap::new()),
    };
    Ok(PresentedPlan {
        imported_pddl_ttl_paths: artifacts
            .iter()
            .map(|a| a.path.display().to_string())
            .collect(),
        plan_digest,
        steps,
        ledger_dir: ledger_dir.display().to_string(),
        #[cfg(feature = "role-inference")]
        roster_roles,
        #[cfg(feature = "role-inference")]
        roster_obligations,
    })
}

/// `plan check`'s admitted outcome: the index of the step that was
/// literal-matched.
#[derive(Debug, Clone)]
pub struct CheckOutcome {
    pub step_index: usize,
}

/// Read-only admission check: `action` is admitted iff it is byte-exact
/// equal to the single next unexecuted step of the plan identified by
/// `plan_digest`. Never mutates the ledger — safe to call on every
/// PreToolUse invocation.
///
/// # Errors
/// `CNG_R30 PlanNotPresented` if `plan_digest` was never presented;
/// `CNG_R31 ActionNotNextApprovedStep` if `action` is not the single
/// lawful next step (or the plan is exhausted); `CNG_R11 AuditMismatch`
/// from the ledger reload.
///
/// # Complexity
/// O(ledger bytes) reload + O(1) lookup.
pub fn check_action(
    ledger_dir: &Path,
    plan_digest: &str,
    action: &str,
) -> Result<CheckOutcome, CngRefusal> {
    let ledger = PlanLedger::open(ledger_dir)?;
    let record = ledger
        .records
        .get(plan_digest)
        .ok_or_else(|| CngRefusal::PlanNotPresented {
            plan_digest: plan_digest.to_string(),
        })?;
    if record.next_step_index >= record.steps.len() {
        return Err(CngRefusal::ActionNotNextApprovedStep {
            plan_digest: plan_digest.to_string(),
            proposed_action: action.to_string(),
            expected_next: None,
        });
    }
    let expected = &record.steps[record.next_step_index];
    if expected == action {
        Ok(CheckOutcome {
            step_index: record.next_step_index,
        })
    } else {
        Err(CngRefusal::ActionNotNextApprovedStep {
            plan_digest: plan_digest.to_string(),
            proposed_action: action.to_string(),
            expected_next: Some(expected.clone()),
        })
    }
}

/// `plan step`'s receipt: which step executed and the resulting chain
/// hash.
#[derive(Debug, Clone)]
pub struct StepReceipt {
    pub plan_digest: String,
    pub step_index: usize,
    pub step_label: String,
    pub chain_hash: String,
}

/// Executes exactly one step — always the literal next unexecuted one of
/// the plan identified by `plan_digest` — and durably chain-hash-records
/// that it was authorized to proceed. `approved` is a default-deny gate
/// checked BEFORE any ledger I/O: its absence alone refuses, regardless of
/// ledger state.
///
/// "Executes" here means "admits and receipts", not "runs a POWL engine" —
/// Increment 2's scope is the approval gate and its receipt chain,
/// decoupled from whatever the external tool call actually does.
///
/// # Errors
/// `CNG_R32 StepNotApproved` if `approved` is false; `CNG_R30
/// PlanNotPresented` if `plan_digest` was never presented; `CNG_R31
/// ActionNotNextApprovedStep` (`expected_next: None`) if the plan is
/// exhausted; `CNG_R11 AuditMismatch` from the ledger reload.
///
/// # Complexity
/// O(ledger bytes) reload + O(1) fold.
pub fn execute_approved_step(
    ledger_dir: &Path,
    plan_digest: &str,
    approved: bool,
) -> Result<StepReceipt, CngRefusal> {
    if !approved {
        return Err(CngRefusal::StepNotApproved {
            plan_digest: plan_digest.to_string(),
        });
    }
    let mut ledger = PlanLedger::open(ledger_dir)?;
    let (step_index, step_label, chain_head) = {
        let record =
            ledger
                .records
                .get(plan_digest)
                .ok_or_else(|| CngRefusal::PlanNotPresented {
                    plan_digest: plan_digest.to_string(),
                })?;
        if record.next_step_index >= record.steps.len() {
            return Err(CngRefusal::ActionNotNextApprovedStep {
                plan_digest: plan_digest.to_string(),
                proposed_action: "<plan step takes no action argument; plan already exhausted>"
                    .to_string(),
                expected_next: None,
            });
        }
        (
            record.next_step_index,
            record.steps[record.next_step_index].clone(),
            record.chain_head.clone(),
        )
    };
    let logical_seq = ledger.next_seq;
    let chain_hash = ledger_chain_hash(
        &chain_head,
        plan_digest,
        &step_label,
        step_index as u64,
        logical_seq,
    );
    ledger.append(PlanLedgerEvent::StepExecuted {
        plan_digest: plan_digest.to_string(),
        step_index,
        step_label: step_label.clone(),
        prev_chain: chain_head,
        chain_hash: chain_hash.clone(),
        logical_seq,
    })?;
    Ok(StepReceipt {
        plan_digest: plan_digest.to_string(),
        step_index,
        step_label,
        chain_hash,
    })
}

#[cfg(test)]
#[path = "plan_approval_test.rs"]
mod plan_approval_test;
