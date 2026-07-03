//! AA livelock soundness model — human livelocks mapped to sound recovery
//! workflow operations, as CODE, not prose.
//!
//! A livelock is a loop that consumes cycles without progressing the graph:
//! rehearsal without release, shame without an amend candidate, self-will
//! without surrender. Each [`LivelockClass`] carries a DETECTION program in
//! the same bounded datalog micro-syntax the hook registry uses
//! (`hooks.rs`), evaluated by the same engine over `t(s, p, o)` facts —
//! detection is a query on the admitted graph, never a judgment call.
//!
//! The twelve [`STEPS`] map the recovery discipline onto workflow-soundness
//! operations one-for-one: detection, external recoverability, control
//! transfer, translation to the graph, witness, readiness, removal request,
//! repair queue, bounded repair, daily re-detection, daily alignment,
//! service output.
//!
//! NO-INFINITE-REHEARSAL: [`rehearsal_exceeded`] is the window-hook law
//! applied to rumination — deltas may touch an open loop only a bounded
//! number of times before the rehearsal itself must park.

use crate::delta::GraphDelta;
use crate::graph::{Object, Triple};
use crate::hooks::eval_datalog;
use crate::Refusal;

/// The recognized human livelock classes. Closed vocabulary — a loop praxis
/// cannot classify is not "detected loosely"; it simply is not a livelock
/// this model speaks about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LivelockClass {
    /// A rehearsal loop no release act points at.
    Resentment,
    /// A harm with no candidate amends act addressing it.
    Shame,
    /// A self-authored plan never surrendered.
    SelfWill,
    /// Provision anxiety with no daily-bread receipt fact.
    Fear,
    /// A temptation risk no guard act covers.
    ReliefSeeking,
    /// An irreversible event neither repaired, learned from, nor released.
    SpilledMilk,
}

/// All classes, in declaration order — 6 of the 8-bound.
pub const ALL_CLASSES: [LivelockClass; 6] = [
    LivelockClass::Resentment,
    LivelockClass::Shame,
    LivelockClass::SelfWill,
    LivelockClass::Fear,
    LivelockClass::ReliefSeeking,
    LivelockClass::SpilledMilk,
];

const RESENTMENT_PROGRAM: &str = "released(?0) :- t(?1, <http://seanchatmangpt.github.io/praxis/life#releases>, ?0). \
     openresentment(?0) :- t(?0, <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>, <http://seanchatmangpt.github.io/praxis/life#ResentmentLoop>), !released(?0).";

const SHAME_PROGRAM: &str = "amendable(?0) :- t(?1, <http://seanchatmangpt.github.io/praxis/life#amendsFor>, ?0). \
     openshame(?0) :- t(?0, <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>, <http://seanchatmangpt.github.io/praxis/life#Harm>), !amendable(?0).";

const SELF_WILL_PROGRAM: &str = "handedoff(?0) :- t(?0, <http://seanchatmangpt.github.io/praxis/life#surrendered>, ?1). \
     openselfwill(?0) :- t(?0, <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>, <http://seanchatmangpt.github.io/praxis/life#SelfPlan>), !handedoff(?0).";

const FEAR_PROGRAM: &str = "receipted(?0) :- t(?0, <http://seanchatmangpt.github.io/praxis/life#hasBreadReceipt>, ?1). \
     openfear(?0) :- t(?0, <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>, <http://seanchatmangpt.github.io/praxis/life#ProvisionAnxiety>), !receipted(?0).";

const RELIEF_SEEKING_PROGRAM: &str = "guarded(?0) :- t(?1, <http://seanchatmangpt.github.io/praxis/life#guards>, ?0). \
     openrisk(?0) :- t(?0, <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>, <http://seanchatmangpt.github.io/praxis/life#TemptationRisk>), !guarded(?0).";

const SPILLED_MILK_PROGRAM: &str = "closedspill(?0) :- t(?1, <http://seanchatmangpt.github.io/praxis/life#repairs>, ?0). \
     closedspill(?0) :- t(?1, <http://seanchatmangpt.github.io/praxis/life#learnsFrom>, ?0). \
     closedspill(?0) :- t(?1, <http://seanchatmangpt.github.io/praxis/life#releases>, ?0). \
     openspill(?0) :- t(?0, <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>, <http://seanchatmangpt.github.io/praxis/life#IrreversibleEvent>), !closedspill(?0).";

/// The detection program for one class: `(program, goal)` in the hook
/// micro-syntax over `t(s, p, o)` facts with `life#` vocabulary. The goal
/// is derivable iff at least one open livelock of the class exists.
#[must_use]
pub fn detection_program(class: LivelockClass) -> (&'static str, &'static str) {
    match class {
        LivelockClass::Resentment => (RESENTMENT_PROGRAM, "openresentment"),
        LivelockClass::Shame => (SHAME_PROGRAM, "openshame"),
        LivelockClass::SelfWill => (SELF_WILL_PROGRAM, "openselfwill"),
        LivelockClass::Fear => (FEAR_PROGRAM, "openfear"),
        LivelockClass::ReliefSeeking => (RELIEF_SEEKING_PROGRAM, "openrisk"),
        LivelockClass::SpilledMilk => (SPILLED_MILK_PROGRAM, "openspill"),
    }
}

/// The twelve steps mapped one-for-one to workflow-soundness operations:
/// `(step, recovery statement, soundness operation)`. The mapping is a code
/// constant so it is testable and content-addressable, not prose.
pub const STEPS: [(u8, &str, &str); 12] = [
    (
        1,
        "admitted the loop is unmanageable from inside",
        "livelock detection: the datalog goal is derivable and no local transition closes it",
    ),
    (
        2,
        "a power greater than the loop can restore sanity",
        "external recoverability: soundness is judged from outside the stuck component",
    ),
    (
        3,
        "decided to turn will and life over",
        "self-will control transfer: the selfPlan gains a surrendered edge; scheduling moves to the solver",
    ),
    (
        4,
        "made a searching and fearless inventory",
        "life-to-graph translation: every loop, debt, and harm becomes a typed triple",
    ),
    (
        5,
        "admitted the exact nature of the wrongs to another",
        "external witness node: the graph is shared with a second verifier, not self-audited",
    ),
    (
        6,
        "entirely ready to have defects removed",
        "defective transition readiness: dead transitions are marked removable, not defended",
    ),
    (
        7,
        "humbly asked to have shortcomings removed",
        "removal request: the retraction delta is proposed through the quarantine door",
    ),
    (
        8,
        "listed all persons harmed, willing to make amends",
        "repair queue: open debts and harms become an ordered amends worklist",
    ),
    (
        9,
        "made direct amends except when it would injure",
        "bounded repair / safe withholding: repair fires within budget; harmful repair is refused",
    ),
    (
        10,
        "continued personal inventory, promptly admitted wrongs",
        "daily livelock detection: the detection programs re-run on every admitted delta",
    ),
    (
        11,
        "sought conscious contact through prayer and meditation",
        "daily external alignment: the kernel re-orients against the reference, not against itself",
    ),
    (
        12,
        "carried the message and practiced the principles",
        "service output: the recovered workflow grounds actions for graphs beyond its own",
    ),
];

/// Detect whether any open livelock of `class` exists in the post-state
/// triples: builds the class's hook-style datalog and evaluates it via the
/// hook engine (`hooks.rs`), so livelock detection and hook firing share
/// one evaluator.
pub fn detect(class: LivelockClass, post_triples: &[Triple]) -> Result<bool, Refusal> {
    let (program, goal) = detection_program(class);
    eval_datalog(program, goal, post_triples, "(livelock-detection)")
}

fn triple_touches(t: &Triple, iri: &str) -> bool {
    t.s == iri || t.p == iri || matches!(&t.o, Object::Iri(o) if o == iri)
}

/// NO-INFINITE-REHEARSAL: true iff at least `bound` deltas in `history`
/// touch `loop_iri` (as subject, predicate, or object, on either side).
/// At the bound the rehearsal must PARK — the window-hook law applied to
/// rumination: revisiting an open loop is lawful only a bounded number of
/// times before the loop is handed off instead of replayed. `bound == 0`
/// is trivially exceeded (no rehearsal budget at all).
#[must_use]
pub fn rehearsal_exceeded(history: &[GraphDelta], loop_iri: &str, bound: u8) -> bool {
    let touches = history
        .iter()
        .filter(|d| {
            d.additions().iter().chain(d.removals().iter()).any(|t| triple_touches(t, loop_iri))
        })
        .count();
    touches >= usize::from(bound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::parse_ttl;

    #[test]
    fn steps_cover_one_through_twelve_in_order() {
        for (i, (n, recovery, soundness)) in STEPS.iter().enumerate() {
            assert_eq!(usize::from(*n), i + 1);
            assert!(!recovery.is_empty() && !soundness.is_empty());
        }
    }

    #[test]
    fn self_will_closes_via_surrendered_edge() {
        let open = parse_ttl(
            "<http://e/plan> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> \
             <http://seanchatmangpt.github.io/praxis/life#SelfPlan> .",
        )
        .expect("parses");
        assert!(detect(LivelockClass::SelfWill, &open).expect("evaluates"));

        let closed = parse_ttl(
            "<http://e/plan> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> \
             <http://seanchatmangpt.github.io/praxis/life#SelfPlan> .\n\
             <http://e/plan> <http://seanchatmangpt.github.io/praxis/life#surrendered> \
             <http://e/higher> .",
        )
        .expect("parses");
        assert!(!detect(LivelockClass::SelfWill, &closed).expect("evaluates"));
    }
}
