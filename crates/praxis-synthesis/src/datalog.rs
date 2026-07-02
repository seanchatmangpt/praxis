//! Layer 1 — semi-naive Datalog saturation (the Nemo lesson).
//!
//! prolog8 is a bounded *backward-chaining* Horn kernel (SLD, depth ≤ 32): it
//! answers point queries with proofs but cannot close large recursive
//! relations. `pddl-index` already runs a *forward* least fixpoint, but only
//! for delete-relaxed action reachability. This module generalizes that
//! fixpoint to arbitrary stratified Horn rules with **semi-naive** evaluation:
//! each iteration joins only against the tuples derived in the previous round
//! (the delta), so saturation cost tracks new facts, not the whole database.
//!
//! Storage rides on `pddl_index::{Dict, SymId, FactStore}` — the same interned
//! u32 ID space the grounder uses, so a saturated database feeds Layer 2's
//! binding enumeration with zero translation.

use pddl_index::facts::atom_key;
use pddl_index::{Dict, FactStore, SymId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::Refusal;

/// Hard cap on total stored tuples (EDB + derived). Refused, not truncated.
pub const MAX_TUPLES: u64 = 1_000_000;
/// Hard cap on saturation iterations per stratum.
pub const MAX_ITERATIONS: u64 = 10_000;
/// Hard cap on negation strata.
pub const MAX_STRATA: usize = 8;
/// Hard cap on distinct variables per rule (prolog8's byte-governor mirror).
pub const MAX_VARS: usize = 8;

/// A term in a rule atom: a variable slot or an interned constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Term {
    /// Variable, identified by a small index (< [`MAX_VARS`]).
    Var(u8),
    /// Interned constant.
    Const(SymId),
}

// `SymId` has no serde derives upstream; serialize terms via a tagged u32
// wire form: {"v": idx} | {"c": id}.
impl Serialize for Term {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut m = s.serialize_map(Some(1))?;
        match self {
            Term::Var(v) => m.serialize_entry("v", v)?,
            Term::Const(c) => m.serialize_entry("c", &c.0)?,
        }
        m.end()
    }
}

impl<'de> Deserialize<'de> for Term {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let map = std::collections::BTreeMap::<String, u32>::deserialize(d)?;
        if let Some(v) = map.get("v") {
            let v = u8::try_from(*v).map_err(serde::de::Error::custom)?;
            return Ok(Term::Var(v));
        }
        if let Some(c) = map.get("c") {
            return Ok(Term::Const(SymId(*c)));
        }
        Err(serde::de::Error::custom("expected {\"v\": _} or {\"c\": _}"))
    }
}

/// A predicate applied to terms — the building block of rules and patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Atom {
    /// Interned predicate symbol.
    pub pred: SymId,
    /// Argument terms.
    pub args: Vec<Term>,
}

impl Serialize for Atom {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("Atom", 2)?;
        st.serialize_field("pred", &self.pred.0)?;
        st.serialize_field("args", &self.args)?;
        st.end()
    }
}

impl<'de> Deserialize<'de> for Atom {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            pred: u32,
            args: Vec<Term>,
        }
        let w = Wire::deserialize(d)?;
        Ok(Atom { pred: SymId(w.pred), args: w.args })
    }
}

impl Atom {
    /// Construct an atom.
    #[must_use]
    pub fn new(pred: SymId, args: Vec<Term>) -> Self {
        Self { pred, args }
    }

    fn vars(&self) -> impl Iterator<Item = u8> + '_ {
        self.args.iter().filter_map(|t| match t {
            Term::Var(v) => Some(*v),
            Term::Const(_) => None,
        })
    }
}

/// A Horn rule with optional stratified negation.
///
/// Safety invariant (enforced at [`Program::add_rule`]): every variable in the
/// head and in every negated atom must be bound by some positive body atom —
/// the same non-binding-negation rule prolog8's admission enforces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DlRule {
    /// Derived atom.
    pub head: Atom,
    /// Positive body atoms (joined).
    pub body: Vec<Atom>,
    /// Negated body atoms (checked against lower strata after binding).
    pub negative: Vec<Atom>,
}

/// Restriction for one semi-naive join: `(body position, delta tuples)`.
type DeltaRestrict<'a> = (usize, &'a BTreeSet<(u32, Vec<u32>)>);

/// A stratified Datalog program over an interned ID space.
#[derive(Debug, Default)]
pub struct Program {
    /// String interner shared with Layer 2.
    pub dict: Dict,
    facts: FactStore,
    preds: BTreeSet<u32>,
    rules: Vec<DlRule>,
    /// Delta from the most recent iteration: (pred, tuple) pairs.
    delta: Vec<(SymId, Vec<u32>)>,
    derived: u64,
}

/// Receipt for one saturation run: what was derived, how, and a content
/// address for the resulting fixpoint state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaturationReceipt {
    /// Tuples derived by rules (excludes the EDB).
    pub derived_count: u64,
    /// Total semi-naive iterations across all strata.
    pub iterations: u64,
    /// Number of strata evaluated.
    pub strata: usize,
    /// BLAKE3 over the sorted atom keys of the full saturated database — a
    /// content address for the *reasoned state*.
    pub fixpoint_hash: String,
}

impl Program {
    /// Empty program.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a predicate or constant symbol.
    pub fn intern(&mut self, s: &str) -> SymId {
        self.dict.intern(s)
    }

    /// Insert a ground EDB fact. Returns `Refusal::TupleCapExceeded` past the cap.
    pub fn add_fact(&mut self, pred: SymId, args: &[SymId]) -> Result<bool, Refusal> {
        if self.facts.len() as u64 >= MAX_TUPLES {
            return Err(Refusal::TupleCapExceeded {
                derived: self.facts.len() as u64,
                cap: MAX_TUPLES,
                iteration: 0,
            });
        }
        self.preds.insert(pred.0);
        Ok(self.facts.insert(pred, args))
    }

    /// Add a rule, enforcing the safety invariant and [`MAX_VARS`].
    pub fn add_rule(&mut self, rule: DlRule) -> Result<(), Refusal> {
        let bound: BTreeSet<u8> = rule.body.iter().flat_map(Atom::vars).collect();
        if bound.iter().any(|v| usize::from(*v) >= MAX_VARS) {
            return Err(Refusal::InvalidInput {
                detail: format!("rule uses variable index >= MAX_VARS ({MAX_VARS})"),
            });
        }
        for v in rule.head.vars() {
            if !bound.contains(&v) {
                return Err(Refusal::InvalidInput {
                    detail: format!("head variable ?{v} not bound by any positive body atom"),
                });
            }
        }
        for neg in &rule.negative {
            for v in neg.vars() {
                if !bound.contains(&v) {
                    return Err(Refusal::InvalidInput {
                        detail: format!(
                            "negated atom variable ?{v} not bound by any positive body atom"
                        ),
                    });
                }
            }
        }
        self.preds.insert(rule.head.pred.0);
        for a in rule.body.iter().chain(rule.negative.iter()) {
            self.preds.insert(a.pred.0);
        }
        self.rules.push(rule);
        Ok(())
    }

    /// Exact membership test against the current database.
    #[must_use]
    pub fn contains(&self, pred: SymId, args: &[SymId]) -> bool {
        self.facts.contains(pred, args)
    }

    /// Number of tuples stored for `pred`.
    #[must_use]
    pub fn count_for(&self, pred: SymId) -> usize {
        self.facts.arity_count(pred)
    }

    /// Total tuples in the database (EDB + derived).
    #[must_use]
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// Whether the database is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Read access to the underlying fact store (Layer 2 joins against this).
    #[must_use]
    pub fn facts(&self) -> &FactStore {
        &self.facts
    }

    /// All predicates mentioned by facts or rules, sorted.
    #[must_use]
    pub fn predicates(&self) -> Vec<SymId> {
        self.preds.iter().map(|p| SymId(*p)).collect()
    }

    /// Compute predicate strata: `stratum(head) >= stratum(pos body)` and
    /// `stratum(head) > stratum(neg body)`. Refuses on negation cycles.
    fn stratify(&self) -> Result<BTreeMap<u32, usize>, Refusal> {
        let mut stratum: BTreeMap<u32, usize> = self.preds.iter().map(|p| (*p, 0)).collect();
        // Relaxation: at most |preds| * MAX_STRATA passes before declaring a cycle.
        let passes = self.preds.len().saturating_mul(MAX_STRATA).max(1);
        for _ in 0..passes {
            let mut changed = false;
            for rule in &self.rules {
                let head = rule.head.pred.0;
                let mut need = stratum[&head];
                for pos in &rule.body {
                    need = need.max(stratum[&pos.pred.0]);
                }
                for neg in &rule.negative {
                    need = need.max(stratum[&neg.pred.0] + 1);
                }
                if need > stratum[&head] {
                    if need >= MAX_STRATA {
                        return Err(Refusal::Unstratifiable {
                            detail: format!(
                                "predicate {} exceeds MAX_STRATA ({MAX_STRATA}) — negation cycle",
                                self.dict.resolve(SymId(head))
                            ),
                        });
                    }
                    stratum.insert(head, need);
                    changed = true;
                }
            }
            if !changed {
                return Ok(stratum);
            }
        }
        Err(Refusal::Unstratifiable { detail: "stratification did not converge".into() })
    }

    /// Enumerate all bindings satisfying `patterns` (positive conjunction)
    /// against `db`, restricted so atom `delta_at` (if given) matches only the
    /// tuples in `delta`. Deterministic order.
    fn join(
        db: &FactStore,
        patterns: &[Atom],
        delta_restrict: Option<DeltaRestrict<'_>>,
        out: &mut Vec<[Option<u32>; MAX_VARS]>,
    ) {
        fn descend(
            db: &FactStore,
            patterns: &[Atom],
            idx: usize,
            delta_restrict: Option<DeltaRestrict<'_>>,
            binding: &mut [Option<u32>; MAX_VARS],
            out: &mut Vec<[Option<u32>; MAX_VARS]>,
        ) {
            let Some(atom) = patterns.get(idx) else {
                out.push(*binding);
                return;
            };
            let try_tuple = |tuple: &[u32],
                             binding: &mut [Option<u32>; MAX_VARS],
                             out: &mut Vec<[Option<u32>; MAX_VARS]>,
                             db: &FactStore| {
                if tuple.len() != atom.args.len() {
                    return;
                }
                let mut newly: Vec<u8> = Vec::new();
                let mut ok = true;
                for (term, val) in atom.args.iter().zip(tuple.iter()) {
                    match term {
                        Term::Const(c) => {
                            if c.0 != *val {
                                ok = false;
                                break;
                            }
                        }
                        Term::Var(v) => match binding[usize::from(*v)] {
                            Some(b) if b != *val => {
                                ok = false;
                                break;
                            }
                            Some(_) => {}
                            None => {
                                binding[usize::from(*v)] = Some(*val);
                                newly.push(*v);
                            }
                        },
                    }
                }
                if ok {
                    descend(db, patterns, idx + 1, delta_restrict, binding, out);
                }
                for v in newly {
                    binding[usize::from(v)] = None;
                }
            };
            match delta_restrict {
                Some((at, delta)) if at == idx => {
                    for (pred, tuple) in delta {
                        if *pred == atom.pred.0 {
                            try_tuple(tuple, binding, out, db);
                        }
                    }
                }
                _ => {
                    let tuples: Vec<Vec<u32>> = db.tuples_for(atom.pred).cloned().collect();
                    for tuple in &tuples {
                        try_tuple(tuple, binding, out, db);
                    }
                }
            }
        }
        let mut binding = [None; MAX_VARS];
        descend(db, patterns, 0, delta_restrict, &mut binding, out);
    }

    fn instantiate(atom: &Atom, binding: &[Option<u32>; MAX_VARS]) -> Vec<SymId> {
        atom.args
            .iter()
            .map(|t| match t {
                Term::Const(c) => *c,
                // Safety invariant guarantees the slot is bound.
                Term::Var(v) => SymId(binding[usize::from(*v)].unwrap_or(u32::MAX)),
            })
            .collect()
    }

    /// Run stratified semi-naive saturation to fixpoint.
    pub fn saturate(&mut self) -> Result<SaturationReceipt, Refusal> {
        let strata = self.stratify()?;
        let max_stratum = strata.values().copied().max().unwrap_or(0);
        let mut iterations: u64 = 0;
        let rules = self.rules.clone();

        for s in 0..=max_stratum {
            // Rules whose head lives in stratum s.
            let layer: Vec<&DlRule> =
                rules.iter().filter(|r| strata[&r.head.pred.0] == s).collect();
            if layer.is_empty() {
                continue;
            }
            // Naive first round for this stratum (delta = everything known).
            let mut delta: BTreeSet<(u32, Vec<u32>)> = BTreeSet::new();
            for rule in &layer {
                let mut bindings = Vec::new();
                Self::join(&self.facts, &rule.body, None, &mut bindings);
                for b in bindings {
                    self.fire(rule, &b, &mut delta)?;
                }
            }
            iterations += 1;
            // Semi-naive rounds: join each rule once per positive-body
            // position, restricting that position to the previous delta.
            while !delta.is_empty() {
                if iterations >= MAX_ITERATIONS {
                    return Err(Refusal::BudgetExceeded {
                        what: "saturation_iterations".into(),
                        budget: MAX_ITERATIONS,
                        spent: iterations,
                        salvage: format!(
                            "database holds {} tuples at cutoff; fixpoint not reached",
                            self.facts.len()
                        ),
                    });
                }
                let prev = std::mem::take(&mut delta);
                for rule in &layer {
                    for at in 0..rule.body.len() {
                        let mut bindings = Vec::new();
                        Self::join(&self.facts, &rule.body, Some((at, &prev)), &mut bindings);
                        for b in bindings {
                            self.fire(rule, &b, &mut delta)?;
                        }
                    }
                }
                iterations += 1;
            }
        }

        self.delta.clear();
        Ok(SaturationReceipt {
            derived_count: self.derived,
            iterations,
            strata: max_stratum + 1,
            fixpoint_hash: self.fixpoint_hash(),
        })
    }

    /// Fire one rule under one binding: check negation, insert head.
    fn fire(
        &mut self,
        rule: &DlRule,
        binding: &[Option<u32>; MAX_VARS],
        delta: &mut BTreeSet<(u32, Vec<u32>)>,
    ) -> Result<(), Refusal> {
        // Negated atoms are fully ground here (safety invariant) and their
        // predicates live in strictly lower, already-saturated strata.
        for neg in &rule.negative {
            let args = Self::instantiate(neg, binding);
            if self.facts.contains(neg.pred, &args) {
                return Ok(());
            }
        }
        let head_args = Self::instantiate(&rule.head, binding);
        if self.facts.len() as u64 >= MAX_TUPLES {
            return Err(Refusal::TupleCapExceeded {
                derived: self.derived,
                cap: MAX_TUPLES,
                iteration: 0,
            });
        }
        if self.facts.insert(rule.head.pred, &head_args) {
            self.derived += 1;
            delta.insert((rule.head.pred.0, head_args.iter().map(|s| s.0).collect()));
        }
        Ok(())
    }

    /// Content address of the full database: BLAKE3 over sorted atom keys.
    #[must_use]
    pub fn fixpoint_hash(&self) -> String {
        let mut keys: Vec<u64> = Vec::with_capacity(self.facts.len());
        for p in &self.preds {
            for tuple in self.facts.tuples_for(SymId(*p)) {
                keys.push(atom_key(*p, tuple));
            }
        }
        keys.sort_unstable();
        let mut bytes = Vec::with_capacity(keys.len() * 8);
        for k in keys {
            bytes.extend_from_slice(&k.to_le_bytes());
        }
        chatman_common::provenance::content_address(&bytes)
    }

    /// Run one extra saturation round and report whether anything new derived
    /// — the [`crate::verify`] `FixpointClosed` refinement.
    pub fn is_closed(&mut self) -> Result<bool, Refusal> {
        let before = self.facts.len();
        self.saturate()?;
        Ok(self.facts.len() == before)
    }
}
