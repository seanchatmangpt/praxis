//! Domain-independent proposer substrate (Genesis Day 6).
//!
//! # The doctrine this module proves
//!
//! The proposer substrate — *enumerate lawful candidates, score them under a
//! domain-authored objective, rank deterministically, commit a blake3
//! proposal hash* — is **domain-independent**. Only two things change between
//! domains: the **ontology** (what states/evidence/stages exist, and which
//! moves are lawful) and the **authored objective** (the weights). The
//! algebra, the lawfulness-gate mechanism, the ranking rule, the rationale
//! format, and the hashing are all authored **once, here**, and reused
//! verbatim by every domain pack.
//!
//! Revenue (PR-14) predates this module and keeps its bespoke concrete
//! [`crate::Proposer`]/[`crate::Proposal`] for API stability, but it also
//! implements [`Domain`] ([`crate::domain::RevenueDomain`]) so the generic
//! substrate can reproduce its ranking — see the equivalence test. Church
//! operations ([`crate::church`]) run **entirely** on this generic substrate
//! with zero new engine code: the church proposer is literally
//! `engine::Proposer<ChurchDomain>`. That reuse *is* the proof.
//!
//! # Boundary position (AR-9) — unchanged across domains
//!
//! Everything this module emits is an untrusted **observation (O, not O*)**:
//! a ranked candidate with a rationale and a proposal hash, sitting *outside*
//! the admission boundary. A proposal grants nothing; it must pass Rice
//! quarantine and admission like any other raw input before it has effect.
//! The admission receipt binds back to [`Proposal::proposal_hash`] so "which
//! proposal was admitted" stays provable — in every domain.
//!
//! # No value discovery (Non-goal 1) — unchanged across domains
//!
//! The objective ([`crate::objective::ObjectiveFunction`]) is authored data
//! loaded from a file; this module contributes only enumeration and algebra.

use std::marker::PhantomData;

use crate::objective::ObjectiveFunction;

/// Hard cap on proposals returned by one `propose()` call (mirrors AR-5's
/// bounded-planner stance). Enumeration is structurally bounded per entity by
/// the number of forward stages; the global list is truncated *after*
/// deterministic sorting, so truncation always drops the lowest-ranked
/// candidates and is itself deterministic.
pub const MAX_PROPOSALS: usize = 64;

/// Version tag mixed into every proposal hash, in every domain, so future
/// format changes can never collide with today's hashes.
pub const PROPOSAL_HASH_DOMAIN: &str = "praxis-proposal-v1";

/// A planning domain, described as **data the substrate consumes**.
///
/// Implementing this trait is the *entire* cost of adding a domain pack: name
/// the ordered stages, the per-entity evidence gate ([`Domain::lawful_targets`]
/// — the lawfulness mechanism reused across domains), and the scoring fluents
/// ([`Domain::compute_fluents`], evaluated in [`Domain::fluent_names`] order).
/// The generic [`Proposer`] does everything else identically for every domain.
pub trait Domain: Sized {
    /// Ordered stage type (an enum). Ordering is load-bearing: candidate
    /// goals move an entity strictly *forward*, and evidence gates key off
    /// the target stage. `'static` because stages are plain value enums —
    /// this lets callers hold `&'static [Self::Stage]` stage tables (e.g.
    /// the mission substrate's reverse PDDL-name lookup).
    type Stage: Copy + Eq + std::fmt::Debug + 'static;
    /// Per-entity observation (revenue `Account`, church `Person`).
    type Entity;
    /// Whole-domain snapshot the proposer observes.
    type State;

    /// Domain-pack identifier, e.g. `"revenue"` / `"church"`.
    fn pack_name() -> &'static str;

    /// The fixed fluent vocabulary, in canonical evaluation order. Scoring
    /// iterates this slice (never a map) so f64 summation order is
    /// deterministic. [`Domain::compute_fluents`] must return values in
    /// exactly this order and length.
    fn fluent_names() -> &'static [&'static str];

    /// PDDL goal predicate name emitted into goal atoms. Both shipped packs
    /// use `"stage"`: the goal atom is `(stage <id> <stage-name>)`.
    fn goal_predicate() -> &'static str {
        "stage"
    }

    /// The entities to enumerate over.
    fn entities(state: &Self::State) -> &[Self::Entity];
    /// Stable identifier of an entity; appears verbatim in the goal atom.
    fn entity_id(entity: &Self::Entity) -> &str;
    /// Current stage of an entity.
    fn entity_stage(entity: &Self::Entity) -> Self::Stage;
    /// Zero-based pipeline index of a stage (for tie-breaking + fluents).
    fn stage_index(stage: Self::Stage) -> u32;
    /// Lower-kebab PDDL/ontology name for a stage. Byte-stable: hashed into
    /// proposal hashes.
    fn stage_pddl_name(stage: Self::Stage) -> &'static str;

    /// **The lawfulness gate.** Forward stages an entity may lawfully be
    /// *proposed* into, given its evidence. This mirrors — does not replace —
    /// the admission rules that gate any proposal downstream (AR-9); the
    /// proposer pre-filters so it never emits a proposal admission would
    /// trivially refuse. Bounded by construction (at most #stages − 1).
    fn lawful_targets(entity: &Self::Entity) -> Vec<Self::Stage>;

    /// Fluent values for the candidate move `entity -> target`, in
    /// [`Domain::fluent_names`] order (same length).
    fn compute_fluents(entity: &Self::Entity, target: Self::Stage) -> Vec<f64>;

    /// The per-candidate description line cited in the rationale (identifies
    /// the entity, its move, and any salient numbers). Domain-authored prose;
    /// the scoring lines below it are generated identically for every domain.
    fn candidate_description(entity: &Self::Entity, target: Self::Stage) -> String;
}

/// A single ranked candidate goal state — an **observation (O), not an
/// authority (O\*)**. Identical shape for every domain; `target_stage` is the
/// domain's own stage type.
pub struct Proposal<D: Domain> {
    /// Human-readable statement of the candidate goal.
    pub goal_description: String,
    /// Entity id the goal targets (verbatim in the PDDL atom).
    pub target_id: String,
    /// Stage the goal would move the entity into.
    pub target_stage: D::Stage,
    /// Authored-objective score (higher ranks earlier).
    pub score: f64,
    /// Audit trail: objective identity, every fluent value × weight, and the
    /// total. Every score is fully explained by these lines.
    pub rationale: Vec<String>,
    /// blake3 hex digest of the canonical encoding ([`Proposal::canonical_bytes`]).
    pub proposal_hash: String,
}

// Manual trait impls: derive would demand `D: Clone`/`Debug`/`PartialEq`
// bounds on the marker type; the fields only involve `D::Stage` (which is
// `Copy + Eq + Debug`), so hand-written impls stay bound-free.
impl<D: Domain> Clone for Proposal<D> {
    fn clone(&self) -> Self {
        Proposal {
            goal_description: self.goal_description.clone(),
            target_id: self.target_id.clone(),
            target_stage: self.target_stage,
            score: self.score,
            rationale: self.rationale.clone(),
            proposal_hash: self.proposal_hash.clone(),
        }
    }
}

impl<D: Domain> std::fmt::Debug for Proposal<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Proposal")
            .field("goal_description", &self.goal_description)
            .field("target_id", &self.target_id)
            .field("target_stage", &self.target_stage)
            .field("score", &self.score)
            .field("rationale", &self.rationale)
            .field("proposal_hash", &self.proposal_hash)
            .finish()
    }
}

impl<D: Domain> PartialEq for Proposal<D> {
    fn eq(&self, other: &Self) -> bool {
        self.goal_description == other.goal_description
            && self.target_id == other.target_id
            && self.target_stage == other.target_stage
            && self.score.to_bits() == other.score.to_bits()
            && self.rationale == other.rationale
            && self.proposal_hash == other.proposal_hash
    }
}

impl<D: Domain> Proposal<D> {
    /// Emit the goal as a PDDL goal-atom text line:
    /// `(<goal_predicate> <target_id> <target-stage-name>)`, e.g.
    /// `(stage visitor-1 leading)`. Splices directly into a problem
    /// `(:goal ...)` block for `plan solve`.
    pub fn pddl_goal(&self) -> String {
        format!(
            "({} {} {})",
            D::goal_predicate(),
            self.target_id,
            D::stage_pddl_name(self.target_stage)
        )
    }

    /// Canonical byte encoding hashed into `proposal_hash`. Domain-neutral
    /// (one field per line, `\n`-separated, UTF-8):
    ///
    /// ```text
    /// praxis-proposal-v1
    /// pack=<pack_name>
    /// target-id=<target_id>
    /// target-stage=<stage pddl name>
    /// score_bits=<f64 to_bits as 16-digit lowercase hex>
    /// goal=<pddl goal atom>
    /// rationale.<i>=<line>   (one per rationale line, in order)
    /// ```
    ///
    /// The score is its exact IEEE-754 bit pattern, so the hash never depends
    /// on decimal formatting.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut s = String::new();
        s.push_str(PROPOSAL_HASH_DOMAIN);
        s.push('\n');
        s.push_str("pack=");
        s.push_str(D::pack_name());
        s.push('\n');
        s.push_str("target-id=");
        s.push_str(&self.target_id);
        s.push('\n');
        s.push_str("target-stage=");
        s.push_str(D::stage_pddl_name(self.target_stage));
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
}

/// The shared scorer: a weighted linear sum over the domain's fluents, summed
/// in `fluent_names` order (deterministic f64), with a fully-cited rationale.
///
/// This is the *one* scoring implementation in the workspace — revenue and
/// church both produce their rationales through it. Zero-weight fluents are
/// explicitly marked ignored; every nonzero weight's contribution is shown.
pub fn score(
    objective: &ObjectiveFunction,
    fluent_names: &[&str],
    candidate_desc: &str,
    fluents: &[f64],
) -> (f64, Vec<String>) {
    debug_assert_eq!(
        fluent_names.len(),
        fluents.len(),
        "compute_fluents must return one value per fluent name"
    );
    let mut rationale = Vec::with_capacity(fluent_names.len() + 3);
    rationale.push(format!(
        "objective '{}' v{} (domain-authored weights; system supplies algebra only)",
        objective.name, objective.version
    ));
    rationale.push(candidate_desc.to_string());
    let mut total = 0.0f64;
    for (i, name) in fluent_names.iter().enumerate() {
        let w = objective.weight(name);
        let v = fluents[i];
        let contribution = w * v;
        total += contribution;
        if w != 0.0 {
            rationale.push(format!("fluent {name} = {v} x weight {w} = {contribution}"));
        } else {
            rationale.push(format!("fluent {name} = {v} (weight 0, ignored)"));
        }
    }
    rationale.push(format!("total score = {total}"));
    (total, rationale)
}

/// Ranks candidate goal states for **any** [`Domain`] under a domain-authored
/// objective. See the module docs for the AR-9 / Non-goal-1 boundary.
///
/// The church pack instantiates this directly (`Proposer<ChurchDomain>`); the
/// substrate needs no per-domain code.
pub struct Proposer<D: Domain> {
    objective: ObjectiveFunction,
    _marker: PhantomData<fn() -> D>,
}

impl<D: Domain> Proposer<D> {
    /// Build a proposer around an authored (already-validated) objective.
    pub fn new(objective: ObjectiveFunction) -> Self {
        Proposer {
            objective,
            _marker: PhantomData,
        }
    }

    /// The objective this proposer scores with (read-only).
    pub fn objective(&self) -> &ObjectiveFunction {
        &self.objective
    }

    /// Enumerate lawful forward candidates, score each, and rank.
    ///
    /// - **Enumeration** is bounded to [`Domain::lawful_targets`] per entity;
    ///   the evidence pre-filter guarantees no over-reaching proposal is ever
    ///   emitted.
    /// - **Ranking** sorts by score descending, tie-broken by
    ///   `(target_id, target stage index)` ascending — a stable, documented
    ///   order for equal scores.
    /// - **Truncation** to [`MAX_PROPOSALS`] happens after sorting.
    ///
    /// Same `state` + same objective always yields the byte-identical ranked
    /// list (including hashes).
    pub fn propose(&self, state: &D::State) -> Vec<Proposal<D>> {
        let mut proposals: Vec<Proposal<D>> = Vec::new();
        for entity in D::entities(state) {
            for target in D::lawful_targets(entity) {
                proposals.push(self.build_proposal(entity, target));
            }
        }
        proposals.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.target_id.cmp(&b.target_id))
                .then_with(|| D::stage_index(a.target_stage).cmp(&D::stage_index(b.target_stage)))
        });
        proposals.truncate(MAX_PROPOSALS);
        proposals
    }

    fn build_proposal(&self, entity: &D::Entity, target: D::Stage) -> Proposal<D> {
        let names = D::fluent_names();
        let fluents = D::compute_fluents(entity, target);
        let desc = D::candidate_description(entity, target);
        let (score, rationale) = score(&self.objective, names, &desc, &fluents);
        let goal_description = format!(
            "advance {} from {} to {}",
            D::entity_id(entity),
            D::stage_pddl_name(D::entity_stage(entity)),
            D::stage_pddl_name(target)
        );
        let mut proposal = Proposal {
            goal_description,
            target_id: D::entity_id(entity).to_string(),
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
