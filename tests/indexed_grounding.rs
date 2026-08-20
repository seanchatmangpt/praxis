//! Genesis Day 5, phase 2 — dictionary-encoded grounding (the qlever treatment).
//!
//! Two claims are checked here against two *independent* grounders:
//!
//! 1. **Differential agreement.** On the Day-3 shared corpus (the same seeded
//!    linear-producer-chain family that `tests/differential.rs`'s `gen_model`
//!    emits, rendered here as classical STRIPS `Pddl8Domain`/`Pddl8Problem`
//!    values), `pddl_index`'s lazy join grounder + BFS returns the byte-identical
//!    plan to `bcinr_pddl::GroundProblem::find_plan` — solvable/unsolvable
//!    outcome and, when solvable, the exact `Pddl8Tape`. Correctness via
//!    agreement.
//!
//! 2. **Materialization ratio + time.** On a transport domain with 10³+
//!    candidate groundings, the indexed grounder materializes only the reachable
//!    fraction (<< all), and grounds faster than naive. The ratio and wall-clock
//!    times are recorded to stderr (run with `--nocapture`).

// Recorded lint debt (v26.7.6 verification gate) -- see src/lib.rs and
// docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(clippy::pedantic)]

use std::time::Instant;

use bcinr_pddl::GroundProblem;
use pddl_index::IndexedGroundProblem;
use wasm4pm_compat::pddl::{Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8Problem, Pddl8Tape};

// ── builders ────────────────────────────────────────────────────────────────

fn atom(pred: &str, args: &[&str]) -> Pddl8Atom {
    Pddl8Atom {
        pred: pred.into(),
        args: args.iter().map(|s| (*s).into()).collect(),
    }
}

fn schema(
    name: &str,
    params: &[&str],
    pre: Vec<Pddl8Atom>,
    add: Vec<Pddl8Atom>,
    del: Vec<Pddl8Atom>,
) -> Pddl8ActionSchema {
    Pddl8ActionSchema {
        name: name.into(),
        params: params.iter().map(|s| (*s).into()).collect(),
        preconditions: pre,
        add_effects: add,
        del_effects: del,
        typed_params: Vec::new(),
        condition: None,
        effects: Vec::new(),
        numeric_effects: Vec::new(),
    }
}

fn domain(name: &str, actions: Vec<Pddl8ActionSchema>) -> Pddl8Domain {
    Pddl8Domain {
        name: name.into(),
        predicates: Vec::new(),
        actions,
        types: Vec::new(),
        functions: Vec::new(),
        durative_actions: Vec::new(),
        derived: Vec::new(),
        constraints: Vec::new(),
        processes: Vec::new(),
        events: Vec::new(),
    }
}

fn problem(
    name: &str,
    objects: Vec<String>,
    init: Vec<Pddl8Atom>,
    goal: Vec<Pddl8Atom>,
) -> Pddl8Problem {
    Pddl8Problem {
        name: name.into(),
        domain: name.into(),
        objects,
        init,
        goal,
        object_types: Vec::new(),
        fn_values: Vec::new(),
        timed_inits: Vec::new(),
        preferences: Vec::new(),
        metric: None,
    }
}

// ── Day-3 shared corpus, rendered as classical STRIPS values ──────────────────

/// Same xorshift RNG as `tests/differential.rs` so the corpus is the identical
/// seeded family.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
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

/// A linear producer chain `p0 -> p1 -> … -> p_d` over `m` objects, mirroring
/// `gen_model` in `tests/differential.rs` but built directly as classical
/// STRIPS `Pddl8*` values (single arity-1 object type, add-only effects).
fn gen_classical(seed: u64) -> (Pddl8Domain, Pddl8Problem) {
    let mut r = Rng::new(seed);
    let d = r.range(1, 4) as usize; // chain links
    let m = r.range(2, 5) as usize; // objects
    let objects: Vec<String> = (0..m).map(|i| format!("o{i}")).collect();
    let preds: Vec<String> = (0..=d).map(|k| format!("p{k}")).collect();

    let mut schemas = Vec::new();
    for k in 0..d {
        schemas.push(schema(
            &format!("mk-p{}", k + 1),
            &["?x"],
            vec![atom(&preds[k], &["?x"])],
            vec![atom(&preds[k + 1], &["?x"])],
            vec![],
        ));
    }
    let dom = domain(&format!("gen{seed}"), schemas);

    let mut init: Vec<Pddl8Atom> = Vec::new();
    for o in &objects {
        if r.chance(1, 2) {
            init.push(atom("p0", &[o]));
        }
    }
    if init.is_empty() {
        init.push(atom("p0", &[&objects[0]]));
    }
    let target = objects[r.range(0, (m - 1) as u64) as usize].clone();
    let goal = vec![atom(&format!("p{d}"), &[&target])];
    (dom, problem(&format!("gen{seed}"), objects, init, goal))
}

/// Canonicalize a plan to the ordered sequence of ground-action labels — the
/// comparison key for differential agreement.
fn plan_labels(tape: &Pddl8Tape) -> String {
    serde_json::to_string(tape).expect("tape is serializable")
}

#[test]
fn differential_indexed_matches_naive_on_day3_corpus() {
    let mut solvable = 0usize;
    let mut unsolvable = 0usize;
    let mut disagreements: Vec<String> = Vec::new();
    let mut total_candidate = 0usize;
    let mut total_materialized = 0usize;

    for seed in 1..=40u64 {
        let (dom, prob) = gen_classical(seed);

        let naive = GroundProblem::build(&dom, &prob, None);
        let indexed = IndexedGroundProblem::build(&dom, &prob, None);

        // Grounding must succeed for both here (every instance has ≥1 ground
        // action), and the indexed grounder must never materialize *more* than
        // naive.
        let (naive, indexed) = match (naive, indexed) {
            (Ok(n), Ok(i)) => (n, i),
            (n, i) => {
                disagreements.push(format!(
                    "[gen{seed}] build mismatch: naive_ok={} indexed_ok={}",
                    n.is_ok(),
                    i.is_ok()
                ));
                continue;
            }
        };
        let materialized = indexed.stats().materialized_groundings;
        total_candidate += indexed.stats().candidate_groundings;
        total_materialized += materialized;

        let n_plan = naive.find_plan().into_result();
        let i_plan = indexed.find_plan();

        match (n_plan, i_plan) {
            (Ok(np), Ok(ip)) => {
                solvable += 1;
                let (a, b) = (plan_labels(&np), plan_labels(&ip));
                if a != b {
                    disagreements.push(format!(
                        "[gen{seed}] PLAN DIFF:\n  naive={a}\n  indexed={b}"
                    ));
                }
                assert!(
                    materialized <= naive_action_count(&dom, &prob),
                    "[gen{seed}] indexed materialized more than naive"
                );
            }
            (Err(_), Err(_)) => unsolvable += 1,
            (n, i) => disagreements.push(format!(
                "[gen{seed}] SOLVABILITY DIFF: naive_solved={} indexed_solved={}",
                n.is_ok(),
                i.is_ok()
            )),
        }
    }

    eprintln!(
        "day3 corpus: 40 instances, solvable={solvable} unsolvable={unsolvable}; \
         manufactured ground actions {total_materialized}/{total_candidate} candidate \
         ({:.1}%)",
        100.0 * total_materialized as f64 / total_candidate.max(1) as f64
    );
    assert!(
        disagreements.is_empty(),
        "indexed vs naive disagreements (each is a BUG): {disagreements:#?}"
    );
    // The corpus must exercise both branches, or "agreement" is vacuous.
    assert!(
        solvable >= 3,
        "corpus too easy — expected some solvable, got {solvable}"
    );
    assert!(
        unsolvable >= 3,
        "corpus too hard — expected some unsolvable, got {unsolvable}"
    );
}

/// Count what the naive grounder materializes (its full product), independent
/// of `pddl_index`'s own estimate — an independent oracle for the bound check.
fn naive_action_count(dom: &Pddl8Domain, prob: &Pddl8Problem) -> usize {
    dom.actions
        .iter()
        .map(|s| prob.objects.len().pow(s.params.len() as u32))
        .sum()
}

// ── transport benchmark: 10³+ candidate groundings ───────────────────────────

fn transport(n: usize) -> (Pddl8Domain, Pddl8Problem) {
    // Precondition order [link, at]: the join is driven by the static `link`
    // relation (binding both params), leaving `(at ?from)` as a *closed* atom
    // settled by an XOR-filter-gated membership probe — the filter is
    // load-bearing on this path.
    let mv = schema(
        "move",
        &["?from", "?to"],
        vec![atom("link", &["?from", "?to"]), atom("at", &["?from"])],
        vec![atom("at", &["?to"])],
        vec![atom("at", &["?from"])],
    );
    let dom = domain("transport", vec![mv]);
    let names: Vec<String> = (0..n).map(|i| format!("l{i}")).collect();
    let mut init = vec![atom("at", &["l0"])];
    for i in 0..n - 1 {
        init.push(atom("link", &[&names[i], &names[i + 1]]));
    }
    let goal = vec![atom("at", &[&names[n - 1]])];
    (dom, problem("transport", names, init, goal))
}

#[test]
fn benchmark_indexed_materializes_far_fewer_than_naive() {
    let n = 50; // 50 locations ⇒ 2500 candidate move groundings (> 10³).
    let (dom, prob) = transport(n);
    let candidates = naive_action_count(&dom, &prob);
    assert!(
        candidates >= 1000,
        "benchmark must have 10³+ candidates, got {candidates}"
    );

    // Naive grounding. `GroundProblem::build` materializes the full product;
    // its own action-vec length equals `candidates` (it does no static pruning).
    let t0 = Instant::now();
    let naive = GroundProblem::build(&dom, &prob, None).expect("naive build");
    let naive_ground_ns = t0.elapsed().as_nanos();
    let naive_materialized = candidates;

    // Indexed grounding.
    let t1 = Instant::now();
    let indexed = IndexedGroundProblem::build(&dom, &prob, None).expect("indexed build");
    let indexed_ground_ns = t1.elapsed().as_nanos();
    let stats = indexed.stats();

    // Pruning: naive materializes all 2500, indexed only the 49 real links.
    assert_eq!(stats.candidate_groundings, candidates);
    assert_eq!(
        stats.materialized_groundings,
        n - 1,
        "one move per existing link"
    );
    let ratio = stats.materialization_ratio();
    assert!(ratio < 0.05, "materialization ratio {ratio} not << 1");

    // Same plan (differential agreement at scale).
    let np = naive.find_plan().into_result().expect("naive plan");
    let ip = indexed.find_plan().expect("indexed plan");
    assert_eq!(
        plan_labels(&np),
        plan_labels(&ip),
        "indexed and naive plans differ at scale"
    );
    assert_eq!(ip.len(), n - 1);

    eprintln!(
        "\n=== indexed grounding benchmark (transport, N={n}) ===\n\
         candidate groundings (naive product): {candidates}\n\
         naive materialized:                   {naive_materialized}\n\
         indexed materialized:                 {}\n\
         materialization ratio:                {:.4} ({:.2}%)\n\
         reachable atoms (R):                  {}\n\
         naive ground time:                    {naive_ground_ns} ns\n\
         indexed ground time:                  {indexed_ground_ns} ns\n\
         speedup (naive/indexed):              {:.2}x\n",
        stats.materialized_groundings,
        ratio,
        ratio * 100.0,
        stats.reachable_atoms,
        naive_ground_ns as f64 / indexed_ground_ns.max(1) as f64,
    );
}
