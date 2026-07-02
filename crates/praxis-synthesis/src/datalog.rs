//! Layer 1 — semi-naive Datalog saturation over columnar sorted relations
//! (the Nemo lesson; P4 wires the join engine).
//!
//! prolog8 is a bounded *backward-chaining* Horn kernel (SLD, depth ≤ 32): it
//! answers point queries with proofs but cannot close large recursive
//! relations. This module runs *forward* stratified semi-naive saturation:
//! each round joins only against the previous round's delta, so cost tracks
//! novelty, not database size.
//!
//! ## The join engine (P4)
//!
//! Storage is [`crate::rel`]: per-predicate lexicographically sorted
//! `Vec<[u32; 8]>` with structural atom identity (no packed-key birthday
//! problem — arity is capped at 8, so the tuple *is* the identity).
//! Joins are planned per firing in the bounded variable-elimination style
//! the 8-caps make trivial:
//!
//! - the delta-restricted atom (if any) is evaluated **first** (semi-naive
//!   discipline);
//! - remaining atoms are ordered greedily by how many of their positions are
//!   bound under the bindings accumulated so far (most-bound-first — the
//!   worst-case-optimal intuition at ≤ 8 atoms / ≤ 8 vars);
//! - each atom probe uses the sorted relation's **bound-prefix range**
//!   (binary search), post-filtering non-prefix bound positions.
//!
//! On a fully-bound atom the probe degenerates to one `contains` binary
//! search — which is exactly what closes triangle-style cyclic patterns
//! without intermediate blowup.

use pddl_index::{Dict, SymId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::rel::{pack, RelStore, Tuple, ARITY_CAP};
use crate::Refusal;

/// Hard cap on total stored tuples (EDB + derived). Refused, not truncated.
/// Raised to 10^8 behind the P4 join engine; the scale receipt records what
/// this machine actually sustains.
pub const MAX_TUPLES: u64 = 100_000_000;
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

/// One semi-naive delta: (pred, tuple) pairs first derived last round.
type Delta = BTreeSet<(u32, Tuple)>;
/// Restriction for one semi-naive join: `(body position, delta tuples)`.
type DeltaRestrict<'a> = (usize, &'a Delta);

/// Domain seed for the fixpoint-hash chain across increments.
pub const FIXPOINT_CHAIN_DOMAIN: &str = "praxis-synthesis/fixpoint/v1";

/// A stratified Datalog program over an interned ID space.
#[derive(Debug)]
pub struct Program {
    /// String interner shared with Layer 2.
    pub dict: Dict,
    rels: RelStore,
    rules: Vec<DlRule>,
    derived: u64,
    /// Rolling chain over successive fixpoint hashes — the living ledger:
    /// every saturation (batch or incremental) folds its fixpoint hash here.
    chain: String,
}

impl Default for Program {
    fn default() -> Self {
        Self {
            dict: Dict::new(),
            rels: RelStore::with_cap(MAX_TUPLES),
            rules: Vec::new(),
            derived: 0,
            chain: chatman_common::provenance::genesis_seed(FIXPOINT_CHAIN_DOMAIN),
        }
    }
}

/// Receipt for one incremental assertion (P5): the delta that arrived, what
/// it caused, and the new fixpoint hash **chained to the previous one** —
/// a receipt structure batch reasoners do not ship.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncrementReceipt {
    /// EDB facts newly asserted (duplicates excluded).
    pub asserted: u64,
    /// Tuples derived as a consequence of this increment alone.
    pub derived: u64,
    /// Semi-naive rounds this increment needed.
    pub iterations: u64,
    /// Fixpoint hash before the increment.
    pub prev_fixpoint_hash: String,
    /// Fixpoint hash after the increment.
    pub fixpoint_hash: String,
    /// Chain value after folding the new fixpoint hash: the ledger link.
    pub chain: String,
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
    /// BLAKE3 over the sorted (pred, tuple) bytes of the full saturated
    /// database — a content address for the *reasoned state*. Structural:
    /// no packed-key collisions to account for.
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

    /// Insert a ground EDB fact. Refuses past [`MAX_TUPLES`] or arity cap.
    pub fn add_fact(&mut self, pred: SymId, args: &[SymId]) -> Result<bool, Refusal> {
        let raw: Vec<u32> = args.iter().map(|s| s.0).collect();
        let t = pack(&raw).ok_or_else(|| Refusal::InvalidInput {
            detail: format!("fact arity {} exceeds ARITY_CAP ({ARITY_CAP})", args.len()),
        })?;
        #[allow(clippy::cast_possible_truncation)]
        self.rels.insert(pred.0, args.len() as u8, t)
    }

    /// Add a rule, enforcing the safety invariant and [`MAX_VARS`].
    pub fn add_rule(&mut self, rule: DlRule) -> Result<(), Refusal> {
        let bound: BTreeSet<u8> = rule.body.iter().flat_map(Atom::vars).collect();
        if bound.iter().any(|v| usize::from(*v) >= MAX_VARS) {
            return Err(Refusal::InvalidInput {
                detail: format!("rule uses variable index >= MAX_VARS ({MAX_VARS})"),
            });
        }
        if rule.head.args.len() > ARITY_CAP
            || rule.body.iter().chain(rule.negative.iter()).any(|a| a.args.len() > ARITY_CAP)
        {
            return Err(Refusal::InvalidInput {
                detail: format!("rule atom arity exceeds ARITY_CAP ({ARITY_CAP})"),
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
        self.rules.push(rule);
        Ok(())
    }

    /// Exact membership test against the current database.
    #[must_use]
    pub fn contains(&self, pred: SymId, args: &[SymId]) -> bool {
        let raw: Vec<u32> = args.iter().map(|s| s.0).collect();
        pack(&raw).is_some_and(|t| {
            self.rels
                .rel(pred.0)
                .is_some_and(|r| usize::from(r.arity()) == args.len() && r.contains(&t))
        })
    }

    /// Number of tuples stored for `pred`.
    #[must_use]
    pub fn count_for(&self, pred: SymId) -> usize {
        self.rels.rel(pred.0).map_or(0, crate::rel::Rel::len)
    }

    /// Total tuples in the database (EDB + derived).
    #[must_use]
    pub fn len(&self) -> usize {
        self.rels.len()
    }

    /// Whether the database is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rels.is_empty()
    }

    /// All predicates present in the database, sorted.
    #[must_use]
    pub fn predicates(&self) -> Vec<SymId> {
        self.rels.preds().map(SymId).collect()
    }

    /// Iterate `pred`'s tuples as `(arity, tuple)` pairs.
    pub fn tuples_of(&self, pred: SymId) -> impl Iterator<Item = (u8, &Tuple)> + '_ {
        self.rels
            .rel(pred.0)
            .into_iter()
            .flat_map(|r| r.iter().map(move |t| (r.arity(), t)))
    }

    /// Compute predicate strata: `stratum(head) >= stratum(pos body)` and
    /// `stratum(head) > stratum(neg body)`. Refuses on negation cycles.
    fn stratify(&self) -> Result<BTreeMap<u32, usize>, Refusal> {
        let mut preds: BTreeSet<u32> = self.rels.preds().collect();
        for r in &self.rules {
            preds.insert(r.head.pred.0);
            for a in r.body.iter().chain(r.negative.iter()) {
                preds.insert(a.pred.0);
            }
        }
        let mut stratum: BTreeMap<u32, usize> = preds.iter().map(|p| (*p, 0)).collect();
        let passes = preds.len().saturating_mul(MAX_STRATA).max(1);
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

    /// Greedy join order: `first` (the delta atom) leads if given; then
    /// repeatedly the atom with the most positions bound under variables
    /// accumulated so far (ties: lowest index). ≤ 8 atoms, ≤ 8 vars — the
    /// bounded variable-elimination order the doctrine makes precomputable.
    fn join_order(body: &[Atom], first: Option<usize>) -> Vec<usize> {
        let mut order = Vec::with_capacity(body.len());
        let mut used = vec![false; body.len()];
        let mut bound_vars: BTreeSet<u8> = BTreeSet::new();
        let take = |i: usize, used: &mut Vec<bool>, bound: &mut BTreeSet<u8>| {
            used[i] = true;
            for v in body[i].vars() {
                bound.insert(v);
            }
        };
        if let Some(f) = first {
            order.push(f);
            take(f, &mut used, &mut bound_vars);
        }
        while order.len() < body.len() {
            let mut best = usize::MAX;
            let mut best_score = -1i32;
            for (i, atom) in body.iter().enumerate() {
                if used[i] {
                    continue;
                }
                #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
                let score = atom
                    .args
                    .iter()
                    .filter(|t| match t {
                        Term::Const(_) => true,
                        Term::Var(v) => bound_vars.contains(v),
                    })
                    .count() as i32;
                if score > best_score {
                    best_score = score;
                    best = i;
                }
            }
            order.push(best);
            take(best, &mut used, &mut bound_vars);
        }
        order
    }

    /// Enumerate all bindings satisfying `body` against the store, with the
    /// atom at `delta_restrict.0` (if given) drawn from the delta instead.
    /// Probes use bound-prefix binary-search ranges; non-prefix bound
    /// positions post-filter. Deterministic order.
    fn join(
        rels: &RelStore,
        body: &[Atom],
        delta_restrict: Option<DeltaRestrict<'_>>,
        out: &mut Vec<[Option<u32>; MAX_VARS]>,
    ) {
        if body.is_empty() {
            out.push([None; MAX_VARS]);
            return;
        }
        let order = Self::join_order(body, delta_restrict.map(|(i, _)| i));

        fn descend(
            rels: &RelStore,
            body: &[Atom],
            order: &[usize],
            depth: usize,
            delta_restrict: Option<DeltaRestrict<'_>>,
            binding: &mut [Option<u32>; MAX_VARS],
            out: &mut Vec<[Option<u32>; MAX_VARS]>,
        ) {
            let Some(&ai) = order.get(depth) else {
                out.push(*binding);
                return;
            };
            let atom = &body[ai];
            // Resolve each position: Some(v) = must equal v, None = free.
            let mut want: [Option<u32>; ARITY_CAP] = [None; ARITY_CAP];
            for (i, term) in atom.args.iter().enumerate() {
                want[i] = match term {
                    Term::Const(c) => Some(c.0),
                    Term::Var(v) => binding[usize::from(*v)],
                };
            }
            let arity = atom.args.len();
            // Bound prefix length for the sorted-range probe.
            let k = want[..arity].iter().take_while(|w| w.is_some()).count();
            let mut prefix: Tuple = [0; ARITY_CAP];
            for i in 0..k {
                prefix[i] = want[i].expect("prefix positions are bound");
            }

            let try_tuple =
                |tuple: &Tuple,
                 binding: &mut [Option<u32>; MAX_VARS],
                 out: &mut Vec<[Option<u32>; MAX_VARS]>| {
                    let mut newly: [u8; ARITY_CAP] = [u8::MAX; ARITY_CAP];
                    let mut n_new = 0;
                    let mut ok = true;
                    for (i, term) in atom.args.iter().enumerate() {
                        match want[i] {
                            Some(w) => {
                                if tuple[i] != w {
                                    ok = false;
                                    break;
                                }
                            }
                            None => {
                                let Term::Var(v) = term else { unreachable!() };
                                match binding[usize::from(*v)] {
                                    Some(b) if b != tuple[i] => {
                                        ok = false;
                                        break;
                                    }
                                    Some(_) => {}
                                    None => {
                                        binding[usize::from(*v)] = Some(tuple[i]);
                                        newly[n_new] = *v;
                                        n_new += 1;
                                    }
                                }
                            }
                        }
                    }
                    if ok {
                        descend(rels, body, order, depth + 1, delta_restrict, binding, out);
                    }
                    for &v in &newly[..n_new] {
                        binding[usize::from(v)] = None;
                    }
                };

            match delta_restrict {
                Some((at, delta)) if at == ai => {
                    for (pred, tuple) in delta {
                        if *pred == atom.pred.0 {
                            try_tuple(tuple, binding, out);
                        }
                    }
                }
                _ => {
                    let Some(rel) = rels.rel(atom.pred.0) else { return };
                    if usize::from(rel.arity()) != arity {
                        return;
                    }
                    if k == arity {
                        // Fully bound: one membership probe.
                        if rel.contains(&prefix) {
                            try_tuple(&prefix.clone(), binding, out);
                        }
                    } else {
                        let tuples: Vec<Tuple> =
                            rel.prefix_range(&prefix, k).copied().collect();
                        for tuple in &tuples {
                            try_tuple(tuple, binding, out);
                        }
                    }
                }
            }
        }
        let mut binding = [None; MAX_VARS];
        descend(rels, body, &order, 0, delta_restrict, &mut binding, out);
    }

    fn instantiate(atom: &Atom, binding: &[Option<u32>; MAX_VARS]) -> Tuple {
        let mut t: Tuple = [0; ARITY_CAP];
        for (i, term) in atom.args.iter().enumerate() {
            t[i] = match term {
                Term::Const(c) => c.0,
                // Safety invariant guarantees the slot is bound.
                Term::Var(v) => binding[usize::from(*v)].unwrap_or(u32::MAX),
            };
        }
        t
    }

    /// Run stratified semi-naive saturation to fixpoint.
    pub fn saturate(&mut self) -> Result<SaturationReceipt, Refusal> {
        let strata = self.stratify()?;
        let max_stratum = strata.values().copied().max().unwrap_or(0);
        let mut iterations: u64 = 0;
        let rules = self.rules.clone();

        for s in 0..=max_stratum {
            let layer: Vec<&DlRule> =
                rules.iter().filter(|r| strata[&r.head.pred.0] == s).collect();
            if layer.is_empty() {
                continue;
            }
            // Naive first round for this stratum.
            let mut delta: Delta = BTreeSet::new();
            for rule in &layer {
                let mut bindings = Vec::new();
                Self::join(&self.rels, &rule.body, None, &mut bindings);
                for b in bindings {
                    self.fire(rule, &b, &mut delta)?;
                }
            }
            self.rels.merge_all();
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
                            self.rels.len()
                        ),
                    });
                }
                let prev = std::mem::take(&mut delta);
                for rule in &layer {
                    for at in 0..rule.body.len() {
                        let mut bindings = Vec::new();
                        Self::join(&self.rels, &rule.body, Some((at, &prev)), &mut bindings);
                        for b in bindings {
                            self.fire(rule, &b, &mut delta)?;
                        }
                    }
                }
                self.rels.merge_all();
                iterations += 1;
            }
        }

        let fixpoint_hash = self.fixpoint_hash();
        self.chain =
            chatman_common::provenance::fold_event(&self.chain, fixpoint_hash.as_bytes());
        Ok(SaturationReceipt {
            derived_count: self.derived,
            iterations,
            strata: max_stratum + 1,
            fixpoint_hash,
        })
    }

    /// The rolling fixpoint-hash chain (the living ledger's current link).
    #[must_use]
    pub fn chain(&self) -> &str {
        &self.chain
    }

    /// Incrementally assert new EDB facts and resume saturation **from the
    /// delta only** — no re-derivation of the existing closure. This is
    /// semi-naive evaluation exposed as a lifecycle: facts arrive, the
    /// closure grows by exactly their consequences, and the new fixpoint
    /// hash is chained to the old one.
    ///
    /// Refused when the program has negative rules: retracting nothing but
    /// *adding* facts can invalidate earlier negation-as-failure conclusions,
    /// so incremental assertion under stratified negation is nonmonotonic.
    /// Salvage: full re-saturation from a fresh program remains sound.
    pub fn assert_facts(
        &mut self,
        facts: &[(SymId, Vec<SymId>)],
    ) -> Result<IncrementReceipt, Refusal> {
        if self.rules.iter().any(|r| !r.negative.is_empty()) {
            return Err(Refusal::InvalidInput {
                detail: "incremental assertion over stratified negation is nonmonotonic \
                         (new facts can invalidate earlier NAF conclusions); refused — \
                         salvage: rebuild and fully re-saturate"
                    .into(),
            });
        }
        let prev_fixpoint_hash = self.fixpoint_hash();
        let derived_before = self.derived;
        let mut delta: Delta = BTreeSet::new();
        let mut asserted = 0u64;
        for (pred, args) in facts {
            let raw: Vec<u32> = args.iter().map(|s| s.0).collect();
            let t = pack(&raw).ok_or_else(|| Refusal::InvalidInput {
                detail: format!("fact arity {} exceeds ARITY_CAP ({ARITY_CAP})", args.len()),
            })?;
            #[allow(clippy::cast_possible_truncation)]
            if self.rels.insert(pred.0, args.len() as u8, t)? {
                asserted += 1;
                delta.insert((pred.0, t));
            }
        }
        self.rels.merge_all();
        // Resume semi-naive rounds seeded with exactly the arrived delta.
        let rules = self.rules.clone();
        let mut iterations = 0u64;
        while !delta.is_empty() {
            if iterations >= MAX_ITERATIONS {
                return Err(Refusal::BudgetExceeded {
                    what: "increment_iterations".into(),
                    budget: MAX_ITERATIONS,
                    spent: iterations,
                    salvage: format!(
                        "database holds {} tuples at cutoff; increment not closed",
                        self.rels.len()
                    ),
                });
            }
            let prev = std::mem::take(&mut delta);
            for rule in &rules {
                for at in 0..rule.body.len() {
                    let mut bindings = Vec::new();
                    Self::join(&self.rels, &rule.body, Some((at, &prev)), &mut bindings);
                    for b in bindings {
                        self.fire(rule, &b, &mut delta)?;
                    }
                }
            }
            self.rels.merge_all();
            iterations += 1;
        }
        let fixpoint_hash = self.fixpoint_hash();
        self.chain =
            chatman_common::provenance::fold_event(&self.chain, fixpoint_hash.as_bytes());
        Ok(IncrementReceipt {
            asserted,
            derived: self.derived - derived_before,
            iterations,
            prev_fixpoint_hash,
            fixpoint_hash,
            chain: self.chain.clone(),
        })
    }

    /// Fire one rule under one binding: check negation, insert head.
    fn fire(
        &mut self,
        rule: &DlRule,
        binding: &[Option<u32>; MAX_VARS],
        delta: &mut Delta,
    ) -> Result<(), Refusal> {
        // Negated atoms are fully ground here (safety invariant) and their
        // predicates live in strictly lower, already-saturated strata.
        for neg in &rule.negative {
            let t = Self::instantiate(neg, binding);
            if self.rels.contains(neg.pred.0, &t) {
                return Ok(());
            }
        }
        let head = Self::instantiate(&rule.head, binding);
        #[allow(clippy::cast_possible_truncation)]
        let added = self.rels.insert(rule.head.pred.0, rule.head.args.len() as u8, head)?;
        if added {
            self.derived += 1;
            delta.insert((rule.head.pred.0, head));
        }
        Ok(())
    }

    /// Content address of the full database: BLAKE3 over sorted
    /// (pred, arity, tuple) byte strings. Structural — nothing to collide.
    #[must_use]
    pub fn fixpoint_hash(&self) -> String {
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(self.rels.len());
        for pred in self.rels.preds() {
            let Some(rel) = self.rels.rel(pred) else { continue };
            let arity = usize::from(rel.arity());
            for tuple in rel.iter() {
                let mut b = Vec::with_capacity(4 + 4 * arity);
                b.extend_from_slice(&pred.to_le_bytes());
                for v in &tuple[..arity] {
                    b.extend_from_slice(&v.to_le_bytes());
                }
                keys.push(b);
            }
        }
        keys.sort_unstable();
        let mut bytes = Vec::with_capacity(keys.iter().map(Vec::len).sum());
        for k in keys {
            bytes.extend_from_slice(&k);
        }
        chatman_common::provenance::content_address(&bytes)
    }

    /// Run one extra saturation round and report whether anything new derived
    /// — the [`crate::verify`] `FixpointClosed` refinement.
    pub fn is_closed(&mut self) -> Result<bool, Refusal> {
        let before = self.rels.len();
        self.saturate()?;
        Ok(self.rels.len() == before)
    }
}
