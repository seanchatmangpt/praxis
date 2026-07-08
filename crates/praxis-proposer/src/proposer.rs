//! The proposer: enumerate, score, rank. Output is a ranked list of
//! **observations** for a human or admission gate to judge — never actions.

use crate::domain::{lawful_targets, Account, RevenueState, Stage};
use crate::objective::ObjectiveFunction;

/// Hard cap on the number of proposals a single `propose()` call may return.
///
/// Boundedness contract (mirrors AR-5's bounded-planner stance):
/// - Per account, enumeration is structurally bounded at
///   `Stage::ALL.len() - 1 == 4` candidates (forward stages only).
/// - Globally, the ranked list is truncated to `MAX_PROPOSALS` *after*
///   deterministic sorting, so truncation always drops the lowest-ranked
///   candidates and is itself deterministic.
pub const MAX_PROPOSALS: usize = 64;

/// Version tag mixed into every proposal hash so future format changes
/// can never collide with today's hashes.
pub const PROPOSAL_HASH_DOMAIN: &str = "praxis-proposal-v1";

/// A single ranked candidate goal state.
///
/// A `Proposal` is an **observation (O), not an authority (O\*)**. It asserts
/// nothing about what *will* happen; it records what the authored objective
/// function scored a lawful candidate at, and why.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Proposal {
    /// Human-readable statement of the candidate goal.
    pub goal_description: String,
    /// Account id the goal targets (verbatim in the PDDL atom).
    pub target_account: String,
    /// Stage the goal would move the account into.
    pub target_stage: Stage,
    /// Authored-objective score (higher ranks earlier).
    pub score: f64,
    /// Audit trail: which objective, which fluent values, which weights,
    /// and what each contributed to the score. Every score is fully
    /// explained by these lines.
    pub rationale: Vec<String>,
    /// blake3 hex digest of the canonical proposal encoding (see
    /// [`Proposal::canonical_bytes`]). The eventual admission receipt binds
    /// back to this hash, so "which proposal was admitted" is provable.
    pub proposal_hash: String,
}

impl Proposal {
    /// Canonical byte encoding hashed into `proposal_hash`.
    ///
    /// Format (one field per line, `\n`-separated, UTF-8):
    ///
    /// ```text
    /// praxis-proposal-v1
    /// account=<target_account>
    /// target=<stage pddl name>
    /// score_bits=<f64 to_bits as 16-digit lowercase hex>
    /// goal=<pddl goal atom>
    /// rationale.<i>=<line>   (one per rationale line, in order)
    /// ```
    ///
    /// The score is encoded as its exact IEEE-754 bit pattern, so the hash
    /// is bit-deterministic and never depends on decimal formatting.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut s = String::new();
        s.push_str(PROPOSAL_HASH_DOMAIN);
        s.push('\n');
        s.push_str("account=");
        s.push_str(&self.target_account);
        s.push('\n');
        s.push_str("target=");
        s.push_str(self.target_stage.pddl_name());
        s.push('\n');
        s.push_str(&format!("score_bits={:016x}\n", self.score.to_bits()));
        s.push_str("goal=");
        s.push_str(&self.pddl_goal());
        s.push('\n');
        for (i, line) in self.rationale.iter().enumerate() {
            s.push_str(&format!("rationale.{i}={line}\n"));
        }
        s.into_bytes()
    }

    /// Emit the goal as a PDDL goal-atom text line.
    ///
    /// Format: `(stage <account-id> <stage-name>)` where `<stage-name>` is
    /// the lower-kebab [`Stage::pddl_name`] (e.g. `closed-won`). This matches
    /// the `stage` predicate declared in `ontology/revenue.ttl`
    /// (`rev:stageOf`, PDDL projection `(stage ?a - account ?s - revenue-stage)`).
    ///
    /// INTEGRATED (Genesis Day 1): the root crate's `propose goal` verb
    /// emits this line, and `src/verbs/propose.rs` documents the splice into
    /// a PDDL problem `:goal` block consumable by `plan solve` over
    /// `ontology/revenue.pddl` (see the `propose_goal_feeds_plan_solve`
    /// test there). This crate itself stays planner-free by design.
    pub fn pddl_goal(&self) -> String {
        format!(
            "(stage {} {})",
            self.target_account,
            self.target_stage.pddl_name()
        )
    }
}

/// Ranks candidate goal states for a revenue pipeline under a
/// domain-authored objective function.
///
/// # This proposer has no authority (AR-9)
///
/// Every [`Proposal`] this type emits is an **observation (O), not an
/// authority (O\*)**. The proposer sits *outside* the admission boundary:
/// its output is an untrusted proposal that must pass Rice quarantine and
/// admission — like any other raw input — before it can have any effect.
/// The proposer may be heuristic or model-backed precisely *because* it has
/// no authority; nothing downstream may treat a proposal as permission.
/// [`Proposal::proposal_hash`] exists so the eventual admission receipt can
/// bind back to exactly which proposal was admitted.
///
/// # No value discovery (Non-goal 1)
///
/// The proposer never invents values: the [`ObjectiveFunction`] is
/// domain-authored data loaded from a file. This type supplies enumeration
/// and arithmetic only.
#[derive(Debug, Clone)]
pub struct Proposer {
    objective: ObjectiveFunction,
}

impl Proposer {
    /// Build a proposer around an authored (already-validated) objective.
    pub fn new(objective: ObjectiveFunction) -> Self {
        Proposer { objective }
    }

    /// The objective this proposer scores with (read-only).
    pub fn objective(&self) -> &ObjectiveFunction {
        &self.objective
    }

    /// Enumerate, score, and rank candidate goal states.
    ///
    /// - **Enumeration** is bounded: for each account, only lawful forward
    ///   stages per [`lawful_targets`] (at most 4 per account); the evidence
    ///   pre-filter guarantees no over-reaching proposal is ever emitted
    ///   (e.g. no `legal_approved` => nothing past `Proposal`).
    /// - **Scoring** is [`ObjectiveFunction::score`]: deterministic
    ///   fixed-order weighted sum with a cited rationale.
    /// - **Ranking** sorts by score descending, tie-broken by
    ///   `(account id, target stage index)` ascending so equal scores have
    ///   a stable, documented order.
    /// - **Truncation** to [`MAX_PROPOSALS`] happens after sorting.
    ///
    /// Same `state` + same objective always yields the byte-identical
    /// ranked list (including hashes).
    pub fn propose(&self, state: &RevenueState) -> Vec<Proposal> {
        let mut proposals: Vec<Proposal> = Vec::new();
        for account in &state.accounts {
            for target in lawful_targets(account) {
                proposals.push(self.build_proposal(account, target));
            }
        }
        proposals.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.target_account.cmp(&b.target_account))
                .then_with(|| a.target_stage.index().cmp(&b.target_stage.index()))
        });
        proposals.truncate(MAX_PROPOSALS);
        proposals
    }

    fn build_proposal(&self, account: &Account, target: Stage) -> Proposal {
        let (score, rationale) = self.objective.score(account, target);
        let mut proposal = Proposal {
            goal_description: format!(
                "advance account {} from {} to {}",
                account.id,
                account.stage.pddl_name(),
                target.pddl_name()
            ),
            target_account: account.id.clone(),
            target_stage: target,
            score,
            rationale,
            proposal_hash: String::new(),
        };
        proposal.proposal_hash = blake3::hash(&proposal.canonical_bytes())
            .to_hex()
            .to_string();
        proposal
    }
}
