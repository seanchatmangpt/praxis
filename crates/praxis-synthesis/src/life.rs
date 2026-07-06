//! The RDF life graph vocabulary — queryable class constants and pure
//! query helpers over admitted triples.
//!
//! The `prx:`/life namespace names the DEVIATION classes the prayer kernel's
//! hooks watch for: open resentments, unrepaired debts, temptation risks,
//! unbounded threats, day-window load, missing receipts. All helpers are
//! pure functions over `&[Triple]` — no state, no side effects; the graph
//! itself is the only store.

use crate::graph::{Object, Triple};

/// The life vocabulary namespace.
pub const LIFE_NS: &str = "http://seanchatmangpt.github.io/praxis/life#";

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// A resentment rehearsal loop (open until a `releases` act points at it).
pub const RESENTMENT_LOOP: &str = "http://seanchatmangpt.github.io/praxis/life#ResentmentLoop";
/// A debt owed (open until a `repairs` act points at it).
pub const DEBT: &str = "http://seanchatmangpt.github.io/praxis/life#Debt";
/// A harm done or received.
pub const HARM: &str = "http://seanchatmangpt.github.io/praxis/life#Harm";
/// A candidate amends act awaiting scheduling.
pub const AMEND_CANDIDATE: &str = "http://seanchatmangpt.github.io/praxis/life#AmendCandidate";
/// A recognized temptation risk.
pub const TEMPTATION_RISK: &str = "http://seanchatmangpt.github.io/praxis/life#TemptationRisk";
/// Provision anxiety (the daily-bread deviation class).
pub const PROVISION_ANXIETY: &str = "http://seanchatmangpt.github.io/praxis/life#ProvisionAnxiety";
/// A threat that cannot be bounded inside the day window — surrendered,
/// never computed.
pub const UNBOUNDED_THREAT: &str = "http://seanchatmangpt.github.io/praxis/life#UnboundedThreat";
/// A day window (the schedulable bound).
pub const DAY_WINDOW: &str = "http://seanchatmangpt.github.io/praxis/life#DayWindow";
/// An act that completed without a written receipt.
pub const RECEIPT_MISSING: &str = "http://seanchatmangpt.github.io/praxis/life#ReceiptMissing";
/// One entry of the nightly inventory.
pub const INVENTORY_ENTRY: &str = "http://seanchatmangpt.github.io/praxis/life#InventoryEntry";

/// An event that cannot be undone — spilled milk. Closed only through
/// repair, learning, or release; never through rehearsal.
pub const IRREVERSIBLE_EVENT: &str =
    "http://seanchatmangpt.github.io/praxis/life#IrreversibleEvent";
/// A self-authored plan (open self-will until surrendered).
pub const SELF_PLAN: &str = "http://seanchatmangpt.github.io/praxis/life#SelfPlan";

/// Predicate: an act releases a resentment loop.
pub const RELEASES: &str = "http://seanchatmangpt.github.io/praxis/life#releases";
/// Predicate: an act repairs a debt.
pub const REPAIRS: &str = "http://seanchatmangpt.github.io/praxis/life#repairs";
/// Predicate: a task is scheduled inside a day window.
pub const SCHEDULED_IN: &str = "http://seanchatmangpt.github.io/praxis/life#scheduledIn";
/// Predicate: a lesson extracted from an irreversible event.
pub const LEARNS_FROM: &str = "http://seanchatmangpt.github.io/praxis/life#learnsFrom";
/// Predicate: a candidate amends act addresses a harm.
pub const AMENDS_FOR: &str = "http://seanchatmangpt.github.io/praxis/life#amendsFor";
/// Predicate: a self plan has been surrendered (control handed off).
pub const SURRENDERED: &str = "http://seanchatmangpt.github.io/praxis/life#surrendered";
/// Predicate: a guard act covers a temptation risk.
pub const GUARDS: &str = "http://seanchatmangpt.github.io/praxis/life#guards";
/// Predicate: a provision anxiety carries a daily-bread receipt fact.
pub const HAS_BREAD_RECEIPT: &str = "http://seanchatmangpt.github.io/praxis/life#hasBreadReceipt";

/// All subjects typed as `class_iri` via `rdf:type`, byte-sorted, deduped.
#[must_use]
pub fn subjects_of<'a>(triples: &'a [Triple], class_iri: &str) -> Vec<&'a str> {
    let mut out: Vec<&str> = triples
        .iter()
        .filter(|t| t.p == RDF_TYPE && matches!(&t.o, Object::Iri(c) if c == class_iri))
        .map(|t| t.s.as_str())
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Subjects of `class_iri` with NO triple `(_, closer_pred, subject)` — the
/// generic open-loop query: the class instance exists, the closing act does
/// not.
fn open_of<'a>(triples: &'a [Triple], class_iri: &str, closer_pred: &str) -> Vec<&'a str> {
    subjects_of(triples, class_iri)
        .into_iter()
        .filter(|s| {
            !triples
                .iter()
                .any(|t| t.p == closer_pred && matches!(&t.o, Object::Iri(o) if o == s))
        })
        .collect()
}

/// Resentment loops no release act points at — the forgive-debtors
/// deviation set.
#[must_use]
pub fn open_resentments(triples: &[Triple]) -> Vec<&str> {
    open_of(triples, RESENTMENT_LOOP, RELEASES)
}

/// Debts no repair act points at — the amends backlog.
#[must_use]
pub fn open_debts(triples: &[Triple]) -> Vec<&str> {
    open_of(triples, DEBT, REPAIRS)
}

/// Subjects scheduled inside `window_iri` via `scheduledIn`, byte-sorted,
/// deduped — the day-window load set.
#[must_use]
pub fn scheduled_in_window<'a>(triples: &'a [Triple], window_iri: &str) -> Vec<&'a str> {
    let mut out: Vec<&str> = triples
        .iter()
        .filter(|t| t.p == SCHEDULED_IN && matches!(&t.o, Object::Iri(w) if w == window_iri))
        .map(|t| t.s.as_str())
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Acts recorded as missing their receipt.
#[must_use]
pub fn missing_receipts(triples: &[Triple]) -> Vec<&str> {
    subjects_of(triples, RECEIPT_MISSING)
}

/// Threats surrendered as unbounded — never scheduled, never computed.
#[must_use]
pub fn unbounded_threats(triples: &[Triple]) -> Vec<&str> {
    subjects_of(triples, UNBOUNDED_THREAT)
}
