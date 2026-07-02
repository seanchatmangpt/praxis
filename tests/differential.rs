//! Differential verification — the simdjson method: correctness established by
//! two (or three) *independent* implementations agreeing on a shared corpus,
//! not by any single implementation being trusted or "comprehended".
//!
//! Four oracle pairs, each fed a shared, generated (or exhaustively enumerated)
//! corpus:
//!
//! 1. PLANNERS  — `bcinr_pddl::ground::GroundTemporalProblem` (praxis dep) vs
//!    `wasm4pm_planner::find_temporal_plan` (independent dev-dep). Same durative
//!    domain+problem text into both parsers/planners. A third, from-scratch
//!    monotone-reachability fixpoint decides ground truth for solvability, and
//!    an independent replay validates that each returned plan actually reaches
//!    the goal. Corpus: a seeded generator of durative-STRIPS domains + a
//!    numeric-fluent exemplar (capacity) + a revenue-stage chain. The classical
//!    `:strips`/`:adl` exemplars (revenue.pddl, lawobject-capability.pddl) are
//!    parse-anchored on the bcinr side only — see the SCOPE receipt in
//!    `pair1_scope_classical_exemplars_are_bcinr_only`.
//!
//! 2. CONFORMANCE — praxis's `replay_adapter` (POWL token-passing
//!    `PowlReplayVerifier`) vs an independent Petri-net token-game replay
//!    (the algorithm dteam's `NetBitmask64` implements). dteam itself does not
//!    resolve as a praxis dep — see the BLOCKER receipt in
//!    `pair2_blocker_dteam_dep`.
//!
//! 3. CHAIN — praxis `chain::recompute_chain` vs a 15-line from-scratch
//!    BLAKE3(prev_hex || frame) reimplementation. Byte-for-byte on 100 random
//!    records.
//!
//! 4. OBJECTIVE — `praxis_proposer::objective` scoring vs a naive in-test dot
//!    product. Bit-exact f64 agreement. (Behind `--features proposer`.)
//!
//! Every disagreement is a BUG to be root-caused and fixed, never papered over
//! by loosening an assertion.

use std::collections::BTreeSet;

// ===========================================================================
// Shared deterministic RNG (xorshift64*) — no external rand dependency, so the
// corpus is byte-reproducible across machines and toolchains.
// ===========================================================================

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        // Avoid the zero fixed-point.
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn range(&mut self, lo: u64, hi_incl: u64) -> u64 {
        lo + self.next_u64() % (hi_incl - lo + 1)
    }
    fn chance(&mut self, num: u64, den: u64) -> bool {
        self.next_u64() % den < num
    }
}

// ===========================================================================
// PAIR 1 — PLANNERS
// ===========================================================================

/// Abstract, planner-independent model of a durative-STRIPS instance. All
/// predicates are arity-1 over a single object type. Both the emitted PDDL
/// (fed to the two planners) and the reachability fixpoint (the third oracle)
/// are derived from *this* struct, so the three consumers are genuinely
/// independent downstreams of one shared corpus item.
struct GenModel {
    name: String,
    objects: Vec<String>,
    /// predicate names in declaration order
    preds: Vec<String>,
    /// (action-name, duration, precondition-preds, at-end add-preds)
    schemas: Vec<(String, u32, Vec<String>, Vec<String>)>,
    /// initial ground atoms as (pred, object)
    init: BTreeSet<(String, String)>,
    /// goal ground atoms as (pred, object)
    goal: Vec<(String, String)>,
}

impl GenModel {
    /// Emit PDDL domain text (durative actions, `(at start …)` conditions,
    /// `(at end …)` add effects) — the exact shape both parsers accept.
    fn domain_pddl(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("(define (domain {})\n", self.name));
        s.push_str("  (:requirements :durative-actions :typing)\n");
        s.push_str("  (:predicates");
        for p in &self.preds {
            s.push_str(&format!(" ({p} ?x)"));
        }
        s.push_str(")\n");
        for (aname, dur, pre, add) in &self.schemas {
            s.push_str(&format!("  (:durative-action {aname}\n"));
            s.push_str("    :parameters (?x - obj)\n");
            s.push_str(&format!("    :duration (= ?duration {dur})\n"));
            s.push_str("    :condition (and");
            for p in pre {
                s.push_str(&format!(" (at start ({p} ?x))"));
            }
            s.push_str(")\n");
            s.push_str("    :effect (and");
            for p in add {
                s.push_str(&format!(" (at end ({p} ?x))"));
            }
            s.push_str("))\n");
        }
        s.push_str(")\n");
        s
    }

    fn problem_pddl(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("(define (problem {}-p)\n", self.name));
        s.push_str(&format!("  (:domain {})\n", self.name));
        s.push_str("  (:objects");
        for o in &self.objects {
            s.push_str(&format!(" {o}"));
        }
        s.push_str(" - obj)\n");
        s.push_str("  (:init");
        for (p, o) in &self.init {
            s.push_str(&format!(" ({p} {o})"));
        }
        s.push_str(")\n");
        s.push_str("  (:goal (and");
        for (p, o) in &self.goal {
            s.push_str(&format!(" ({p} {o})"));
        }
        s.push_str(")))\n");
        s
    }

    /// Third oracle: least-fixpoint monotone reachability. For add-only
    /// (monotone) STRIPS this is exact — the goal is reachable iff every goal
    /// atom is in the closure of `init` under the grounded add-effects. Fully
    /// independent of both planners.
    fn goal_reachable(&self) -> bool {
        let mut state = self.init.clone();
        loop {
            let mut changed = false;
            for (_a, _d, pre, add) in &self.schemas {
                for o in &self.objects {
                    let applicable = pre.iter().all(|p| state.contains(&(p.clone(), o.clone())));
                    if applicable {
                        for p in add {
                            if state.insert((p.clone(), o.clone())) {
                                changed = true;
                            }
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        self.goal.iter().all(|g| state.contains(g))
    }

    /// Independent plan validator: replay a returned plan's `(action_name,
    /// args)` steps against the abstract model, applying each fired action's
    /// add-effects, and confirm the goal is produced. Catches the case where
    /// *both* planners agree on a wrong answer.
    fn plan_reaches_goal(&self, steps: &[(String, Vec<String>)]) -> bool {
        let mut state = self.init.clone();
        for (aname, args) in steps {
            let obj = match args.first() {
                Some(o) => o.clone(),
                None => continue,
            };
            if let Some((_, _, _, add)) = self.schemas.iter().find(|(n, ..)| n == aname) {
                for p in add {
                    state.insert((p.clone(), obj.clone()));
                }
            }
        }
        self.goal.iter().all(|g| state.contains(g))
    }
}

/// Seeded generator: a linear producer chain `p0 -> p1 -> … -> p_d` (each link
/// an action requiring `p_{k}` and adding `p_{k+1}`), over `M` objects with a
/// random subset seeded with `p0`, and a goal `(p_d target)`. Randomised chain
/// length, object count, init membership, durations, and target. Naturally
/// yields both solvable and unsolvable instances; the fixpoint oracle decides
/// which, and all three implementations must agree.
fn gen_model(seed: u64) -> GenModel {
    let mut r = Rng::new(seed);
    let d = r.range(1, 4) as usize; // chain links
    let m = r.range(2, 5) as usize; // objects
    let objects: Vec<String> = (0..m).map(|i| format!("o{i}")).collect();
    let preds: Vec<String> = (0..=d).map(|k| format!("p{k}")).collect();
    let mut schemas = Vec::new();
    for k in 0..d {
        let dur = r.range(1, 6) as u32;
        schemas.push((
            format!("mk-p{}", k + 1),
            dur,
            vec![preds[k].clone()],
            vec![preds[k + 1].clone()],
        ));
    }
    // Seed p0 on a random subset of objects (at least one, so :init is nonempty).
    let mut init: BTreeSet<(String, String)> = BTreeSet::new();
    for o in &objects {
        if r.chance(1, 2) {
            init.insert(("p0".to_string(), o.clone()));
        }
    }
    if init.is_empty() {
        init.insert(("p0".to_string(), objects[0].clone()));
    }
    // Goal: the top predicate on a random target object.
    let target = objects[r.range(0, (m - 1) as u64) as usize].clone();
    let goal = vec![(format!("p{d}"), target)];
    GenModel { name: format!("gen{seed}"), objects, preds, schemas, init, goal }
}

/// Run one abstract model through both planners and the fixpoint oracle and
/// assert triple agreement. Returns `(solvable, disagreement_note)`.
fn run_planner_case(m: &GenModel) -> (bool, Option<String>) {
    use bcinr_pddl::ground::GroundTemporalProblem;
    use bcinr_pddl::{domain_from_pddl as b_domain, problem_from_pddl as b_problem};
    use wasm4pm_planner::{
        domain_from_pddl as w_domain, find_temporal_plan, ground_domain,
        problem_from_pddl as w_problem,
    };

    let dtext = m.domain_pddl();
    let ptext = m.problem_pddl();

    // Independent ground truth.
    let truth = m.goal_reachable();

    // wasm4pm oracle.
    let wd = w_domain(&dtext).expect("wasm4pm domain parse");
    let wp = w_problem(&ptext).expect("wasm4pm problem parse");
    let wg = ground_domain(&wd, &wp).expect("wasm4pm grounding");
    let w_plan = find_temporal_plan(&wg, &wp);

    // bcinr oracle.
    let bd = b_domain(&dtext).expect("bcinr domain parse");
    let bp = b_problem(&ptext).expect("bcinr problem parse");
    let gtp = GroundTemporalProblem::build(&bd, &bp).expect("bcinr build");
    let b_plan = gtp.find_temporal_plan();

    let mut note = None;

    // (a) solvability: bcinr == wasm4pm == fixpoint truth.
    if w_plan.is_ok() != truth || b_plan.is_ok() != truth {
        note = Some(format!(
            "[{}] SOLVABILITY DISAGREEMENT: fixpoint={truth} wasm4pm_ok={} bcinr_ok={}",
            m.name,
            w_plan.is_ok(),
            b_plan.is_ok()
        ));
    }

    if let (Ok(wpl), Ok(bpl)) = (&w_plan, &b_plan) {
        // (b) same greedy-tick semantics ⇒ identical step count and makespan.
        if wpl.steps.len() != bpl.steps.len() {
            note.get_or_insert_with(|| {
                format!(
                    "[{}] STEP-COUNT DISAGREEMENT: wasm4pm={} bcinr={}",
                    m.name,
                    wpl.steps.len(),
                    bpl.steps.len()
                )
            });
        }
        if (wpl.makespan - bpl.makespan).abs() > 1e-9 {
            note.get_or_insert_with(|| {
                format!(
                    "[{}] MAKESPAN DISAGREEMENT: wasm4pm={} bcinr={}",
                    m.name, wpl.makespan, bpl.makespan
                )
            });
        }
        // (c) each returned plan actually reaches the goal under the
        // independent replay validator.
        let w_steps: Vec<(String, Vec<String>)> =
            wpl.steps.iter().map(|s| (s.action_name.clone(), s.args.clone())).collect();
        let b_steps: Vec<(String, Vec<String>)> =
            bpl.steps.iter().map(|s| (s.action_name.clone(), s.args.clone())).collect();
        if !m.plan_reaches_goal(&w_steps) {
            note.get_or_insert_with(|| format!("[{}] wasm4pm plan does NOT reach goal", m.name));
        }
        if !m.plan_reaches_goal(&b_steps) {
            note.get_or_insert_with(|| format!("[{}] bcinr plan does NOT reach goal", m.name));
        }
    }

    (truth, note)
}

#[test]
fn pair1_planners_generated_corpus_triple_agreement() {
    let mut solvable = 0usize;
    let mut unsolvable = 0usize;
    let mut disagreements: Vec<String> = Vec::new();

    // 30 seeded instances (spec: "20+ generated small STRIPS domains").
    for seed in 1..=30u64 {
        let m = gen_model(seed);
        let (solv, note) = run_planner_case(&m);
        if solv {
            solvable += 1;
        } else {
            unsolvable += 1;
        }
        if let Some(n) = note {
            disagreements.push(n);
        }
    }

    eprintln!(
        "pair1 generated corpus: 30 instances, solvable={solvable} unsolvable={unsolvable}"
    );
    assert!(
        disagreements.is_empty(),
        "planner disagreements (each is a BUG): {disagreements:#?}"
    );
    // Coverage: the corpus must exercise BOTH branches of the solvability oracle.
    assert!(solvable >= 3, "corpus too easy — expected some solvable, got {solvable}");
    assert!(unsolvable >= 3, "corpus too easy — expected some unsolvable, got {unsolvable}");
}

/// Numeric-fluent exemplar (shared-capacity concurrency). Exercises the
/// `increase`/`decrease` fluent path both planners implement — a path the
/// add-only generated corpus does not touch. Ground-truth outcomes are known,
/// so the two planners are checked against each other *and* against the
/// asserted expected step/makespan.
#[test]
fn pair1_planners_capacity_numeric_exemplar() {
    use bcinr_pddl::ground::GroundTemporalProblem;
    use bcinr_pddl::{domain_from_pddl as b_domain, problem_from_pddl as b_problem};
    use wasm4pm_planner::{
        domain_from_pddl as w_domain, find_temporal_plan, ground_domain,
        problem_from_pddl as w_problem,
    };

    const DOMAIN: &str = r#"
(define (domain capacity-demo)
  (:requirements :durative-actions :numeric-fluents :typing)
  (:predicates (idle ?w) (busy ?w) (done ?w))
  (:functions (available-workers))
  (:durative-action assign-worker
    :parameters (?w - worker)
    :duration (= ?duration 5)
    :condition (and (at start (idle ?w)) (at start (>= (available-workers) 1)))
    :effect (and
      (at start (decrease (available-workers) 1))
      (at start (not (idle ?w))) (at start (busy ?w))
      (at end (increase (available-workers) 1))
      (at end (not (busy ?w))) (at end (done ?w)))))
"#;
    let problem = |cap: u32| {
        format!(
            r#"(define (problem assign-two-workers)
  (:domain capacity-demo)
  (:objects w1 w2 - worker)
  (:init (idle w1) (idle w2) (= (available-workers) {cap}))
  (:goal (and (done w1) (done w2))))"#
        )
    };

    // cap=1 forces sequential (makespan 10); cap=2 allows concurrent (makespan 5).
    for (cap, want_makespan) in [(1u32, 10.0f64), (2u32, 5.0f64)] {
        let p = problem(cap);
        let wd = w_domain(DOMAIN).unwrap();
        let wp = w_problem(&p).unwrap();
        let wg = ground_domain(&wd, &wp).unwrap();
        let wpl = find_temporal_plan(&wg, &wp).expect("wasm4pm plan");

        let bd = b_domain(DOMAIN).unwrap();
        let bp = b_problem(&p).unwrap();
        let gtp = GroundTemporalProblem::build(&bd, &bp).unwrap();
        let bpl = gtp.find_temporal_plan().expect("bcinr plan");

        assert_eq!(wpl.steps.len(), 2, "cap={cap} wasm4pm step count");
        assert_eq!(bpl.steps.len(), 2, "cap={cap} bcinr step count");
        assert_eq!(wpl.steps.len(), bpl.steps.len(), "cap={cap} cross step count");
        assert!(
            (wpl.makespan - bpl.makespan).abs() < 1e-9,
            "cap={cap} cross makespan: wasm4pm={} bcinr={}",
            wpl.makespan,
            bpl.makespan
        );
        assert!(
            (wpl.makespan - want_makespan).abs() < 1e-9,
            "cap={cap} makespan want {want_makespan} got {}",
            wpl.makespan
        );
    }
}

/// Revenue-stage chain (honors the "revenue domain" corpus item) rendered as a
/// durative-STRIPS advance chain: lead -> qualified -> proposal -> procurement
/// -> closed-won. Add-monotone, so the fixpoint oracle applies.
#[test]
fn pair1_planners_revenue_stage_chain() {
    let stages = ["lead", "qualified", "proposal", "procurement", "closed-won"];
    let objects = vec!["acct".to_string()];
    let preds: Vec<String> = stages.iter().map(|s| format!("at-{s}")).collect();
    let mut schemas = Vec::new();
    for k in 0..stages.len() - 1 {
        schemas.push((
            format!("advance-{}", stages[k + 1]),
            1u32,
            vec![preds[k].clone()],
            vec![preds[k + 1].clone()],
        ));
    }
    let mut init = BTreeSet::new();
    init.insert(("at-lead".to_string(), "acct".to_string()));
    let goal = vec![("at-closed-won".to_string(), "acct".to_string())];
    let m = GenModel {
        name: "revenue".to_string(),
        objects,
        preds,
        schemas,
        init,
        goal,
    };
    let (solvable, note) = run_planner_case(&m);
    assert!(solvable, "revenue chain must be solvable");
    assert!(note.is_none(), "revenue chain disagreement: {note:?}");
}

/// SCOPE receipt: the two named classical exemplars are `:strips`/`:adl`, which
/// the wasm4pm-planner parser (durative-actions subset only) cannot consume.
/// They are therefore parse-anchored on the bcinr side only — a single-oracle
/// well-formedness check, NOT a cross-planner differential. Documented here so
/// the boundary is explicit rather than silently dropped.
#[test]
fn pair1_scope_classical_exemplars() {
    use bcinr_pddl::domain_from_pddl as b_domain;

    // revenue.pddl is clean classical `:strips` — bcinr's `pddl`-crate parser
    // accepts it (single-oracle well-formedness anchor). wasm4pm-planner is
    // durative-only, so no cross-planner differential is run on it; its
    // durative shape IS exercised as a differential in
    // `pair1_planners_revenue_stage_chain`.
    let revenue =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/ontology/revenue.pddl"));
    if let Ok(text) = revenue {
        let d = b_domain(&text).expect("bcinr parses revenue.pddl (classical :strips)");
        assert!(!d.actions.is_empty(), "revenue.pddl should have classical actions");
    }

    // lawobject-capability.pddl uses `:adl` (forall / implies / when, and
    // `:precondition ((not …))` without an enclosing `and`). This is out of
    // scope for BOTH oracles: wasm4pm-planner is durative-STRIPS-only, and
    // bcinr's strict `pddl`-crate parser also rejects these ADL constructs.
    // Documented here as a scope boundary rather than silently dropped — it is
    // an exemplar of the *capability model doc*, not a planner input.
    let lawobj = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/lawobject-capability.pddl"
    ));
    if let Ok(text) = lawobj {
        let parsed = b_domain(&text);
        // We assert the observed reality (both parsers decline the ADL domain),
        // making the scope boundary an explicit, checked fact.
        assert!(
            parsed.is_err(),
            "lawobject-capability.pddl is expected to be out of scope for bcinr's \
             PDDL8 parser (ADL forall/implies/when); if this now parses, promote \
             it to a real cross-oracle case and update this receipt"
        );
    }
}

// ===========================================================================
// PAIR 2 — CONFORMANCE
// ===========================================================================

/// Independent Petri-net token-game replay — the algorithm dteam's
/// `NetBitmask64` implements (bitmask marking, consume in-set / produce
/// out-set, count missing + remaining). Written from scratch here because the
/// dteam crate does not resolve as a praxis dependency (see
/// `pair2_blocker_dteam_dep`).
///
/// The lifecycle net (mirroring praxis's fixed judge->admit->receipt SEQ):
///   places: START(0) JUDGED(1) ADMITTED(2) DONE(3), initial={START}, final={DONE}
///   transitions: judge (START->JUDGED), admit (JUDGED->ADMITTED),
///                receipt (ADMITTED->DONE)
/// Returns `(missing_during, missing_final, remaining)`:
///   - `missing_during`: input tokens absent when a transition fired (an
///     *illegal fire* — a transition executed out of causal order).
///   - `missing_final`: final-marking tokens absent at end (incomplete run).
///   - `remaining`: leftover non-final tokens at end (over-run / stuck tokens).
/// aalst token-replay is perfectly conformant iff all three are zero.
fn petri_replay(trace: &[u8]) -> (u32, u32, u32) {
    // token bits
    const START: u64 = 1 << 0;
    const JUDGED: u64 = 1 << 1;
    const ADMITTED: u64 = 1 << 2;
    const DONE: u64 = 1 << 3;
    const FINAL: u64 = DONE;

    let mut marking = START;
    let mut missing_during = 0u32;
    for &t in trace {
        let (inb, outb) = match t {
            0 => (START, JUDGED),    // judge
            1 => (JUDGED, ADMITTED), // admit
            2 => (ADMITTED, DONE),   // receipt
            _ => unreachable!(),
        };
        // Consume in-set; any absent in-token is "missing" and force-created.
        let absent = inb & !marking;
        if absent != 0 {
            missing_during += absent.count_ones();
            marking |= absent;
        }
        marking &= !inb;
        marking |= outb;
    }
    let missing_final = (FINAL & !marking).count_ones();
    let remaining = (marking & !FINAL).count_ones();
    (missing_during, missing_final, remaining)
}

#[test]
fn pair2_conformance_powl_vs_petri_agreement() {
    use praxis_core::replay_adapter::{replay_lifecycle, LifecycleStep};

    // Map compact codes to the two representations.
    fn to_step(c: u8) -> (LifecycleStep, u64, Vec<String>) {
        let s = match c {
            0 => LifecycleStep::Judged,
            1 => LifecycleStep::Admitted,
            2 => LifecycleStep::Receipted,
            _ => unreachable!(),
        };
        (s, c as u64 + 1, vec![])
    }

    // Corpus: all length-1..=3 sequences over {judge,admit,receipt} (39 traces),
    // covering the lawful order, every permutation, skips, and duplicates.
    let mut corpus: Vec<Vec<u8>> = Vec::new();
    for a in 0..3u8 {
        corpus.push(vec![a]);
        for b in 0..3u8 {
            corpus.push(vec![a, b]);
            for c in 0..3u8 {
                corpus.push(vec![a, b, c]);
            }
        }
    }

    // The two implementations measure conformance along different axes, and the
    // shared, load-bearing invariant is *illegal-fire detection*: a transition
    // that fires without its causal predecessor's token. POWL's
    // `PowlReplayVerifier` returns `Err(TokenNotEnabled)` on the first such
    // fire; the Petri token-game records `missing_during > 0`. These MUST
    // coincide on every trace — that is the differential contract. (POWL's
    // fitness does not additionally gate on run *completion*; the Petri game's
    // `remaining`/`missing_final` do — that extra axis is not a disagreement,
    // so it is checked separately, not against POWL.)
    let mut illegal_free_count = 0usize; // traces with no illegal fire
    let mut complete_lawful_count = 0usize; // fully conformant AND completing
    let mut disagreements = Vec::new();
    for trace in &corpus {
        let steps: Vec<_> = trace.iter().map(|&c| to_step(c)).collect();
        let powl = replay_lifecycle(&steps);
        let powl_legal = match &powl {
            Ok(m) => {
                // POWL's own contract: an accepted (illegal-fire-free) trace has
                // fitness exactly 1.0 in Q16.16.
                if m.fitness != 0x0001_0000 {
                    disagreements.push(format!(
                        "trace {trace:?}: POWL Ok but fitness={:#x} != 1.0",
                        m.fitness
                    ));
                }
                true
            }
            Err(_) => false,
        };

        let (missing_during, missing_final, remaining) = petri_replay(trace);
        let petri_legal = missing_during == 0;

        // CORE DIFFERENTIAL: both oracles must agree on illegal-fire presence.
        if powl_legal != petri_legal {
            disagreements.push(format!(
                "trace {trace:?}: POWL legal={powl_legal} but Petri illegal-fire-free={petri_legal} (missing_during={missing_during})"
            ));
        }

        if powl_legal {
            illegal_free_count += 1;
        }
        if powl_legal && missing_during == 0 && missing_final == 0 && remaining == 0 {
            complete_lawful_count += 1;
        }
    }

    eprintln!(
        "pair2 conformance: {} traces, {illegal_free_count} illegal-fire-free (both oracles agree), {complete_lawful_count} fully-complete-lawful",
        corpus.len(),
    );
    assert!(disagreements.is_empty(), "conformance disagreements (BUG): {disagreements:#?}");
    // Legal prefixes of the canonical order: [judge], [judge,admit],
    // [judge,admit,receipt]. Both oracles agree these have no illegal fire.
    assert_eq!(illegal_free_count, 3, "expected exactly 3 illegal-fire-free traces");
    // Exactly one trace both fires legally AND completes the net: [J, A, R].
    assert_eq!(complete_lawful_count, 1, "exactly one complete lawful sequence expected");
}

/// BLOCKER receipt (pair 2): dteam's `NetBitmask64` lives in
/// `/Users/sac/dteam/src/conformance/bitmask_replay.rs` and is tightly coupled
/// to `crate::models::petri_net::PetriNet` and `crate::models::{EventLog,
/// Trace, AttributeValue}`. The dteam crate is a large separate workspace
/// pinned to a nightly `rust-toolchain.toml` with `unibit`/HDC path
/// dependencies that do not resolve from praxis's stable graph. Adding it as a
/// path dev-dependency is therefore not feasible without dragging in that
/// whole toolchain. Per the differential-verification plan's fallback, the
/// NetBitmask64 *algorithm* (bitmask marking, in/out sets, missing+remaining,
/// aalst fitness) is reimplemented independently in `petri_replay` above and
/// checked against praxis's POWL `PowlReplayVerifier`. This test documents the
/// blocker; it is not a functional gate.
#[test]
fn pair2_blocker_dteam_dep() {
    let path = "/Users/sac/dteam/src/conformance/bitmask_replay.rs";
    // Sanity: the source we mirrored still exists where we read it.
    assert!(
        std::path::Path::new(path).exists(),
        "dteam NetBitmask64 source not found at {path}; update the blocker receipt"
    );
}

// ===========================================================================
// PAIR 3 — CHAIN
// ===========================================================================

/// Independent, from-scratch reimplementation of the praxis audit chain:
/// genesis = BLAKE3(GENESIS_SEED) as lowercase hex; fold = BLAKE3(prev_hex_bytes
/// || frame_bytes) as lowercase hex. No praxis chain code used except the
/// shared GENESIS_SEED constant (shared config, akin to a shared corpus seed).
fn independent_chain(events: &[Vec<u8>]) -> String {
    use my_conforming_project::chain::GENESIS_SEED;
    let mut acc = blake3::hash(GENESIS_SEED).to_hex().to_string();
    for e in events {
        let mut buf = Vec::with_capacity(acc.len() + e.len());
        buf.extend_from_slice(acc.as_bytes());
        buf.extend_from_slice(e);
        acc = blake3::hash(&buf).to_hex().to_string();
    }
    acc
}

#[test]
fn pair3_chain_recompute_vs_independent_100_records() {
    use my_conforming_project::chain::recompute_chain;

    let mut r = Rng::new(0xC0FFEE);
    let mut disagreements = 0usize;
    for _ in 0..100 {
        // A random record = a random number of random-length frames.
        let n_frames = r.range(1, 8) as usize;
        let mut events: Vec<Vec<u8>> = Vec::with_capacity(n_frames);
        for _ in 0..n_frames {
            let len = r.range(1, 40) as usize;
            let mut frame = Vec::with_capacity(len);
            for _ in 0..len {
                frame.push((r.next_u64() & 0xFF) as u8);
            }
            events.push(frame);
        }
        let praxis = recompute_chain(&events);
        let indep = independent_chain(&events);
        if praxis != indep {
            disagreements += 1;
            eprintln!("CHAIN DISAGREEMENT: praxis={praxis} indep={indep} events={events:?}");
        }
        assert_eq!(praxis.len(), 64, "chain hash must be 64 hex chars");
    }
    assert_eq!(disagreements, 0, "chain disagreements found (BUG)");
}

// ===========================================================================
// PAIR 4 — OBJECTIVE (requires --features proposer)
// ===========================================================================

#[cfg(feature = "proposer")]
#[test]
fn pair4_objective_score_bit_exact() {
    use praxis_proposer::domain::{Account, Stage};
    use praxis_proposer::objective::{ObjectiveFunction, FLUENT_NAMES};
    use std::collections::BTreeMap;

    // Naive, independent reimplementation of the scoring dot product. Mirrors
    // the documented fluent semantics and the fixed FLUENT_NAMES summation
    // order so the f64 result is bit-identical (not merely close).
    fn naive_score(obj: &ObjectiveFunction, a: &Account, target: Stage) -> f64 {
        let amount = a.amount_cents as f64;
        let realized = if target == Stage::ClosedWon { amount } else { 0.0 };
        let at_risk = if (target.index()) < Stage::ClosedWon.index() { amount } else { 0.0 };
        let staleness = a.days_in_stage as f64;
        let advance = (target.index() as f64) - (a.stage.index() as f64);
        let fluents = [realized, at_risk, staleness, advance];
        let mut s = 0.0f64;
        for (i, name) in FLUENT_NAMES.iter().enumerate() {
            s += obj.weights.get(*name).copied().unwrap_or(0.0) * fluents[i];
        }
        s
    }

    let mut r = Rng::new(0xB17E_AC70_u64);
    let stages = Stage::ALL;
    let mut disagreements = Vec::new();
    let mut cases = 0usize;

    for _ in 0..40 {
        // Random authored objective (finite weights, some zero, some negative).
        let mut weights = BTreeMap::new();
        for name in FLUENT_NAMES {
            // weight in [-1000, 1000] with 2 decimal places, occasionally 0.
            let raw = r.range(0, 200_000) as f64 / 100.0 - 1000.0;
            let w = if r.chance(1, 5) { 0.0 } else { raw };
            weights.insert(name.to_string(), w);
        }
        let obj = ObjectiveFunction {
            name: "diff".to_string(),
            version: "1".to_string(),
            weights,
        };
        obj.validate().expect("weights finite");

        for _ in 0..10 {
            let acct = Account {
                id: format!("a{}", r.next_u64() % 1000),
                stage: stages[r.range(0, 4) as usize],
                amount_cents: r.range(0, 10_000_000) as i64,
                security_review_done: r.chance(1, 2),
                legal_approved: r.chance(1, 2),
                exec_sponsor: r.chance(1, 2),
                days_in_stage: r.range(0, 400) as u32,
            };
            for target in stages {
                cases += 1;
                let (praxis_score, _rationale) = obj.score(&acct, target);
                let naive = naive_score(&obj, &acct, target);
                // Bit-exact comparison (via bit pattern) — not epsilon.
                if praxis_score.to_bits() != naive.to_bits() {
                    disagreements.push(format!(
                        "acct.stage={:?} target={:?}: praxis={praxis_score:?} naive={naive:?}",
                        acct.stage, target
                    ));
                }
            }
        }
    }

    eprintln!("pair4 objective: {cases} scoring cases compared bit-exact");
    assert!(disagreements.is_empty(), "objective scoring disagreements (BUG): {disagreements:#?}");
}
