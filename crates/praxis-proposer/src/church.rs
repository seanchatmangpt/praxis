//! Church-operations domain pack (Genesis Day 6) — Mission Physics beyond
//! revenue.
//!
//! # What this proves
//!
//! This module is the *parallel* of [`crate::domain`] (revenue): same shape,
//! different ontology and objective. It carries **no** proposer, scorer,
//! ranker, or hasher of its own — the church proposer is literally
//! [`crate::engine::Proposer`]`<`[`ChurchDomain`]`>`. Adding church cost only
//! this domain description plus an authored objective (`church_objective.json`).
//! The substrate (enumerate → score → rank → hash → admit → receipt) is
//! reused verbatim. That is the doctrine: the substrate is domain-independent;
//! only the ontology and the authored objective function change.
//!
//! # Grounded in real service, not reduction of the spiritual
//!
//! The stages and evidence flags below are an *operational discretization* of
//! a welcome-team workflow (ZOE Church): a way to make sure no one who came
//! for help gets lost between their first visit and being genuinely known and
//! cared for. `welcomed`, `followed_up`, `in_small_group`, `care_assigned` are
//! observable acts of hospitality, not measures of a person's worth or faith.
//! The objective weights are the ministry's authored judgment about what to
//! attend to first (Non-goal 1: the system never invents them).
//!
//! # Boundary position (AR-9) — inherited, unchanged
//!
//! Every proposal is an observation (O, not O*): a suggestion for a human on
//! the welcome team to weigh, never an instruction and never authority over a
//! person. It must pass admission like any other input.

use serde::{Deserialize, Serialize};

use crate::engine::{self, Domain};
use crate::objective::{ObjectiveError, ObjectiveFunction};

/// Ordered assimilation stage of a person the welcome team is walking with.
///
/// The ordering is load-bearing: candidate goals may only move a person
/// *forward* (strictly greater stage), and evidence gates key off the target
/// stage. Discriminants are explicit so the order survives refactoring. This
/// mirrors `revenue::Stage` exactly in shape — an ordered enum with an
/// evidence-gated tail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    /// Showed up once. We know they came; not yet that they'll return.
    FirstTime = 0,
    /// Came back. A pattern, not yet a relationship.
    Returning = 1,
    /// Genuinely connected: known by name, in relationship (e.g. small group).
    Connected = 2,
    /// Serving others — has moved from being cared for to sharing the care.
    Serving = 3,
    /// Leading: shepherding others through the same journey.
    Leading = 4,
}

impl Stage {
    /// All stages in assimilation order.
    pub const ALL: [Stage; 5] = [
        Stage::FirstTime,
        Stage::Returning,
        Stage::Connected,
        Stage::Serving,
        Stage::Leading,
    ];

    /// Zero-based assimilation index (FirstTime = 0 .. Leading = 4).
    pub fn index(self) -> u8 {
        self as u8
    }

    /// Lower-kebab name used in PDDL goal atoms and `ontology/church.ttl`.
    /// Must stay byte-stable: emitted text is hashed into proposal hashes.
    pub fn pddl_name(self) -> &'static str {
        match self {
            Stage::FirstTime => "first-time",
            Stage::Returning => "returning",
            Stage::Connected => "connected",
            Stage::Serving => "serving",
            Stage::Leading => "leading",
        }
    }
}

/// A single person the welcome team is walking with: the unit the proposer
/// reasons over. Mirrors `revenue::Account`; the evidence flags are the
/// hospitality acts a stage advance is gated on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Person {
    /// Stable identifier (a ministry record id, never sensitive PII in the
    /// snapshot). Appears verbatim in PDDL goal atoms, e.g.
    /// `visitor-7 -> (stage visitor-7 connected)`.
    pub id: String,
    /// Current assimilation stage.
    pub stage: Stage,
    /// Evidence flag: someone from the welcome team actually greeted them.
    pub welcomed: bool,
    /// Evidence flag: a follow-up contact was made after a visit.
    pub followed_up: bool,
    /// Evidence flag: they are in a small group (real relational connection).
    pub in_small_group: bool,
    /// Evidence flag: a care need has been assigned to a care team member.
    pub care_assigned: bool,
    /// Days the person has sat at their current stage (staleness / a proxy
    /// for follow-up timeliness). Numeric fluent source.
    pub days_in_stage: u32,
}

/// Snapshot of everyone the welcome team is tracking. Mirrors
/// `revenue::RevenueState`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChurchState {
    pub people: Vec<Person>,
}

impl ChurchState {
    /// Observe an **admitted** church snapshot (the AR-9-preferred path),
    /// deserialized from the JSON payload of a
    /// `LawObject<serde_json::Value, Admitted, _>`. Because `Admitted` is only
    /// reachable through `Judge`/`Admit`, a snapshot obtained this way has
    /// passed obligation judgment: the proposer observes admitted reality,
    /// never raw input. Exactly mirrors `RevenueState::from_admitted`.
    pub fn from_admitted<Law>(
        admitted: &praxis_core::LawObject<serde_json::Value, praxis_core::lifecycle::Admitted, Law>,
    ) -> Result<ChurchState, serde_json::Error> {
        serde_json::from_value(admitted.payload().clone())
    }
}

/// Evidence gate: may `person` lawfully be *proposed* into `target`?
///
/// This mirrors (does not replace) the admission rules that gate any proposal
/// downstream — the identical mechanism as `revenue::evidence_permits`, only
/// the rules are the ministry's:
///
/// | target     | required evidence                                                    |
/// |------------|----------------------------------------------------------------------|
/// | FirstTime  | (never a forward target)                                             |
/// | Returning  | none — inviting someone back is never gated                          |
/// | Connected  | `welcomed` AND `followed_up`                                          |
/// | Serving    | `welcomed` AND `followed_up` AND `in_small_group`                    |
/// | Leading    | `welcomed` AND `followed_up` AND `in_small_group` AND `care_assigned` |
///
/// In particular: a person who was never `followed_up` can never be proposed
/// past `Returning` — you don't route someone into deeper commitment when the
/// basic act of following up hasn't happened. This is the church analog of
/// "an account missing `legal_approved` can never be proposed past Proposal".
pub fn evidence_permits(person: &Person, target: Stage) -> bool {
    match target {
        Stage::FirstTime | Stage::Returning => true,
        Stage::Connected => person.welcomed && person.followed_up,
        Stage::Serving => person.welcomed && person.followed_up && person.in_small_group,
        Stage::Leading => {
            person.welcomed && person.followed_up && person.in_small_group && person.care_assigned
        }
    }
}

/// Lawful forward targets: every stage strictly ahead whose evidence gate
/// passes. Bounded by construction at `Stage::ALL.len() - 1 == 4`. Identical
/// in shape to `revenue::lawful_targets`.
pub fn lawful_targets(person: &Person) -> Vec<Stage> {
    Stage::ALL
        .iter()
        .copied()
        .filter(|t| t.index() > person.stage.index())
        .filter(|t| evidence_permits(person, *t))
        .collect()
}

/// The fixed fluent vocabulary for the church pack, in canonical evaluation
/// order. These are the *mission variables* — not revenue. Their WEIGHTS are
/// authored in `church_objective.json` (Non-goal 1); this names the fluents
/// the shared algebra understands. Same length/role structure as
/// `revenue::FLUENT_NAMES`.
pub const FLUENT_NAMES: [&str; 4] = [
    "people_connected",
    "care_completion_rate",
    "volunteer_capacity_used",
    "first_time_followup_within_48h",
];

/// Fluent semantics for a candidate move of `person` to `target`:
///
/// | fluent                           | value                                                                   |
/// |----------------------------------|-------------------------------------------------------------------------|
/// | `people_connected`               | assimilation depth reached — `target index` if `target >= Connected`, else `0` |
/// | `care_completion_rate`           | `1.0` if the person has care assigned, else `0.0` (aggregates to a rate) |
/// | `volunteer_capacity_used`        | `1.0` if `target >= Serving` (the move recruits into service), else `0.0` — a *cost* |
/// | `first_time_followup_within_48h` | `1.0` if a first-timer (≤ 2 days) is being invited back promptly, else `0.0` |
///
/// The *signs and magnitudes* attached to these are the ministry's judgment,
/// expressed in the authored objective weights. This function reports facts
/// about the candidate only. Values are returned in [`FLUENT_NAMES`] order.
pub fn compute_fluents(person: &Person, target: Stage) -> [f64; 4] {
    let connected_depth = if target.index() >= Stage::Connected.index() {
        target.index() as f64
    } else {
        0.0
    };
    let care = if person.care_assigned { 1.0 } else { 0.0 };
    let volunteer = if target.index() >= Stage::Serving.index() {
        1.0
    } else {
        0.0
    };
    let prompt_followup = if person.stage == Stage::FirstTime
        && person.days_in_stage <= 2
        && target.index() >= Stage::Returning.index()
    {
        1.0
    } else {
        0.0
    };
    [connected_depth, care, volunteer, prompt_followup]
}

/// Zero-sized marker binding the church ontology to the generic substrate.
/// This is the *entire* wiring — no proposer/scorer/ranker/hasher is written
/// here; [`engine::Proposer`]`<ChurchDomain>` supplies all of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChurchDomain;

impl Domain for ChurchDomain {
    type Stage = Stage;
    type Entity = Person;
    type State = ChurchState;

    fn pack_name() -> &'static str {
        "church"
    }

    fn fluent_names() -> &'static [&'static str] {
        &FLUENT_NAMES
    }

    fn entities(state: &Self::State) -> &[Self::Entity] {
        &state.people
    }

    fn entity_id(entity: &Self::Entity) -> &str {
        &entity.id
    }

    fn entity_stage(entity: &Self::Entity) -> Self::Stage {
        entity.stage
    }

    fn stage_index(stage: Self::Stage) -> u32 {
        stage.index() as u32
    }

    fn stage_pddl_name(stage: Self::Stage) -> &'static str {
        stage.pddl_name()
    }

    fn lawful_targets(entity: &Self::Entity) -> Vec<Self::Stage> {
        lawful_targets(entity)
    }

    fn compute_fluents(entity: &Self::Entity, target: Self::Stage) -> Vec<f64> {
        compute_fluents(entity, target).to_vec()
    }

    fn candidate_description(entity: &Self::Entity, target: Self::Stage) -> String {
        format!(
            "candidate: person {} {} -> {} (days_in_stage={}, care_assigned={})",
            entity.id,
            entity.stage.pddl_name(),
            target.pddl_name(),
            entity.days_in_stage,
            entity.care_assigned
        )
    }
}

/// A ranked church proposal — the generic proposal specialized to the church
/// ontology. No new type: reuse.
pub type ChurchProposal = engine::Proposal<ChurchDomain>;

/// The church proposer — the generic substrate specialized to the church
/// ontology. No new proposer code: this alias *is* the church proposer.
pub type ChurchProposer = engine::Proposer<ChurchDomain>;

/// Load + validate a church objective from JSON text, checking weights
/// against [`FLUENT_NAMES`] (the church vocabulary). Reuses the identical
/// [`ObjectiveFunction`] loader/algebra as revenue — only the allowed fluent
/// vocabulary differs.
pub fn objective_from_json_str(s: &str) -> Result<ObjectiveFunction, ObjectiveError> {
    ObjectiveFunction::from_json_str_for(s, &FLUENT_NAMES)
}

/// Load + validate a church objective from a JSON file on disk.
pub fn objective_from_path(path: &std::path::Path) -> Result<ObjectiveFunction, ObjectiveError> {
    ObjectiveFunction::from_path_for(path, &FLUENT_NAMES)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person(
        id: &str,
        stage: Stage,
        welcomed: bool,
        followed_up: bool,
        in_small_group: bool,
        care_assigned: bool,
        days: u32,
    ) -> Person {
        Person {
            id: id.into(),
            stage,
            welcomed,
            followed_up,
            in_small_group,
            care_assigned,
            days_in_stage: days,
        }
    }

    #[test]
    fn stage_order_is_assimilation_order() {
        assert!(Stage::FirstTime < Stage::Returning);
        assert!(Stage::Connected < Stage::Serving);
        assert!(Stage::Serving < Stage::Leading);
    }

    #[test]
    fn missing_followup_never_passes_returning() {
        // Welcomed but never followed up: cannot be routed past Returning.
        let p = person("p", Stage::FirstTime, true, false, true, true, 3);
        let targets = lawful_targets(&p);
        assert!(targets.iter().all(|t| *t <= Stage::Returning));
        // Returning itself is still lawful (inviting back is never gated).
        assert!(targets.contains(&Stage::Returning));
    }

    #[test]
    fn full_evidence_unlocks_leading() {
        let p = person("p", Stage::Serving, true, true, true, true, 0);
        assert_eq!(lawful_targets(&p), vec![Stage::Leading]);
    }

    #[test]
    fn leading_has_no_forward_targets() {
        let p = person("p", Stage::Leading, true, true, true, true, 0);
        assert!(lawful_targets(&p).is_empty());
    }

    #[test]
    fn serving_requires_small_group() {
        // Welcomed + followed up but not in a small group: Connected OK,
        // Serving/Leading never.
        let p = person("p", Stage::Returning, true, true, false, true, 0);
        let targets = lawful_targets(&p);
        assert!(targets.contains(&Stage::Connected));
        assert!(!targets.contains(&Stage::Serving));
        assert!(!targets.contains(&Stage::Leading));
    }

    #[test]
    fn from_admitted_observes_a_judged_and_admitted_snapshot() {
        use praxis_core::{Admit, DefaultLaw, Judge, LawObject};

        let payload = serde_json::json!({
            "people": [{
                "id": "visitor-1",
                "stage": "first_time",
                "welcomed": true,
                "followed_up": true,
                "in_small_group": false,
                "care_assigned": false,
                "days_in_stage": 1
            }]
        });
        let raw = LawObject::<serde_json::Value, praxis_core::lifecycle::Raw, DefaultLaw>::new(
            payload,
            vec![],
        );
        let Ok(validated) = DefaultLaw::judge(raw) else {
            panic!("no obligations => validated");
        };
        let admitted = DefaultLaw::admit(validated).expect("green andon => admitted");

        let state = ChurchState::from_admitted(&admitted)
            .expect("admitted payload deserializes into ChurchState");
        assert_eq!(state.people.len(), 1);
        assert_eq!(state.people[0].id, "visitor-1");
        assert_eq!(state.people[0].stage, Stage::FirstTime);
    }

    #[test]
    fn from_admitted_rejects_non_church_payload() {
        use praxis_core::{Admit, DefaultLaw, Judge, LawObject};

        let payload = serde_json::json!({"people": [], "vibes": true});
        let raw = LawObject::<serde_json::Value, praxis_core::lifecycle::Raw, DefaultLaw>::new(
            payload,
            vec![],
        );
        let Ok(validated) = DefaultLaw::judge(raw) else {
            panic!("no obligations => validated");
        };
        let admitted = DefaultLaw::admit(validated).expect("green andon => admitted");

        // deny_unknown_fields: admission does not imply schema conformance.
        assert!(ChurchState::from_admitted(&admitted).is_err());
    }
}
