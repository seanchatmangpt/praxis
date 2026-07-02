//! `synth/v1` — the wire format and payload ops for the synthesis pipeline.
//!
//! One page of schema is the whole onboarding cost for a foreign agent
//! (see `docs/SYNTH_V1.md`). Terms are strings: `"?N"` is variable `N`
//! (N < 8), anything else is an interned constant. Atoms are
//! `["pred", ["t1", "t2", ...]]` (arity ≤ 8).
//!
//! ```json
//! {
//!   "synth": "v1",
//!   "facts":        [["raw", ["o1"]]],
//!   "rules":        [{"head": ["d", ["?0"]], "body": [["raw", ["?0"]]], "neg": []}],
//!   "capabilities": [{"name": "work", "params": 1,
//!                     "pre": [["d", ["?0"]]], "add": [["done", ["?0"]]],
//!                     "del": [], "cost": 1}],
//!   "goal":         [["done", ["o1"]]],
//!   "horizon":      4,
//!   "constraints":  [{"kind": "Before", "a": "x", "b": "y"},
//!                    {"kind": "NotLater", "a": "x", "k": 2},
//!                    {"kind": "Budget", "max": 10}]
//! }
//! ```
//!
//! Both payload fns are the single implementation shared by the `synth` CLI
//! verbs and the MCP membrane tools (zero drift, like every other noun).

use praxis_synthesis::datalog::Atom;
use praxis_synthesis::{
    BoundedCsp, Capability, Constraint, DlRule, HashRunner, MemoCache, Program, Solver, Solver8,
    Synthesis, Term,
};
use serde::Deserialize;
use serde_json::{json, Value};

type WireAtom = (String, Vec<String>);

#[derive(Deserialize)]
struct WireRule {
    head: WireAtom,
    body: Vec<WireAtom>,
    #[serde(default)]
    neg: Vec<WireAtom>,
}

#[derive(Deserialize)]
struct WireCapability {
    name: String,
    params: u8,
    pre: Vec<WireAtom>,
    #[serde(default)]
    add: Vec<WireAtom>,
    #[serde(default)]
    del: Vec<WireAtom>,
    #[serde(default)]
    cost: u32,
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum WireConstraint {
    Before { a: String, b: String },
    After { a: String, b: String },
    NotLater { a: String, k: u8 },
    NotEarlier { a: String, k: u8 },
    Excludes { a: String, b: String },
    Requires { a: String, b: String },
    AtMost { a: String, n: u8 },
    Budget { max: u32 },
}

#[derive(Deserialize)]
struct WireProblem {
    synth: String,
    #[serde(default)]
    facts: Vec<WireAtom>,
    #[serde(default)]
    rules: Vec<WireRule>,
    capabilities: Vec<WireCapability>,
    goal: Vec<WireAtom>,
    horizon: usize,
    #[serde(default)]
    constraints: Vec<WireConstraint>,
    /// "solver8" (default) or "brute" (the differential oracle).
    #[serde(default)]
    solver: Option<String>,
}

fn term(p: &mut Program, s: &str) -> Result<Term, String> {
    if let Some(v) = s.strip_prefix('?') {
        let idx: u8 = v.parse().map_err(|_| format!("bad variable '{s}'"))?;
        if idx >= 8 {
            return Err(format!("variable '{s}' >= MAX_VARS (8)"));
        }
        Ok(Term::Var(idx))
    } else {
        Ok(Term::Const(p.intern(s)))
    }
}

fn atom(p: &mut Program, (pred, args): &WireAtom) -> Result<Atom, String> {
    if args.len() > 8 {
        return Err(format!("atom '{pred}' arity {} > 8", args.len()));
    }
    let pred_id = p.intern(pred);
    let terms = args.iter().map(|a| term(p, a)).collect::<Result<Vec<_>, _>>()?;
    Ok(Atom::new(pred_id, terms))
}

fn constraint(w: WireConstraint) -> Constraint {
    match w {
        WireConstraint::Before { a, b } => Constraint::Before { a, b },
        WireConstraint::After { a, b } => Constraint::After { a, b },
        WireConstraint::NotLater { a, k } => Constraint::NotLater { a, k },
        WireConstraint::NotEarlier { a, k } => Constraint::NotEarlier { a, k },
        WireConstraint::Excludes { a, b } => Constraint::Excludes { a, b },
        WireConstraint::Requires { a, b } => Constraint::Requires { a, b },
        WireConstraint::AtMost { a, n } => Constraint::AtMost { a, n },
        WireConstraint::Budget { max } => Constraint::Budget { max },
    }
}

#[allow(clippy::type_complexity)]
fn build(
    payload: &str,
) -> Result<(Program, Vec<Capability>, Vec<Atom>, usize, Vec<Constraint>, String), String> {
    let wire: WireProblem =
        serde_json::from_str(payload).map_err(|e| format!("synth/v1 parse error: {e}"))?;
    if wire.synth != "v1" {
        return Err(format!("unsupported synth version '{}' (this is synth/v1)", wire.synth));
    }
    let mut p = Program::new();
    for f in &wire.facts {
        let a = atom(&mut p, f)?;
        let consts = a
            .args
            .iter()
            .map(|t| match t {
                Term::Const(c) => Ok(*c),
                Term::Var(_) => Err("facts must be ground (no '?N' variables)".to_string()),
            })
            .collect::<Result<Vec<_>, _>>()?;
        p.add_fact(a.pred, &consts).map_err(|e| e.to_string())?;
    }
    for r in &wire.rules {
        let head = atom(&mut p, &r.head)?;
        let body = r.body.iter().map(|a| atom(&mut p, a)).collect::<Result<Vec<_>, _>>()?;
        let negative =
            r.neg.iter().map(|a| atom(&mut p, a)).collect::<Result<Vec<_>, _>>()?;
        p.add_rule(DlRule { head, body, negative }).map_err(|e| e.to_string())?;
    }
    let mut caps = Vec::with_capacity(wire.capabilities.len());
    for c in &wire.capabilities {
        caps.push(Capability {
            name: c.name.clone(),
            params: c.params,
            pre: c.pre.iter().map(|a| atom(&mut p, a)).collect::<Result<Vec<_>, _>>()?,
            add: c.add.iter().map(|a| atom(&mut p, a)).collect::<Result<Vec<_>, _>>()?,
            del: c.del.iter().map(|a| atom(&mut p, a)).collect::<Result<Vec<_>, _>>()?,
            cost: c.cost,
        });
    }
    let goal = wire.goal.iter().map(|a| atom(&mut p, a)).collect::<Result<Vec<_>, _>>()?;
    let constraints = wire.constraints.into_iter().map(constraint).collect();
    let solver = wire.solver.unwrap_or_else(|| "solver8".into());
    Ok((p, caps, goal, wire.horizon, constraints, solver))
}

fn pick_solver(name: &str) -> Result<Box<dyn Solver>, String> {
    match name {
        "solver8" => Ok(Box::new(Solver8)),
        "brute" => Ok(Box::new(BoundedCsp)),
        other => Err(format!("unknown solver '{other}' (solver8 | brute)")),
    }
}

/// Full pipeline: saturate → sequence → content-addressed DAG → verify →
/// one `SynthesisReceipt`. Refusals (including certified unsat proofs with
/// named culprits) are returned as `{"status":"refused", ...}` — domain
/// denials are results, not errors.
pub fn synth_run_payload(payload: &str) -> Result<Value, String> {
    let (mut p, caps, goal, horizon, constraints, solver_name) = build(payload)?;
    let solver = pick_solver(&solver_name)?;
    // The pipeline runs sequencing internally without constraints; when the
    // payload carries constraints, sequence explicitly first.
    if constraints.is_empty() {
        match Synthesis::run(
            &mut p,
            caps,
            goal,
            horizon,
            solver.as_ref(),
            &mut HashRunner,
            &mut MemoCache::new(),
        ) {
            Ok(receipt) => Ok(json!({"status": "admitted", "receipt": receipt})),
            Err(refusal) => {
                Ok(json!({"status": "refused", "refusal": refusal, "rendered": refusal.to_string()}))
            }
        }
    } else {
        // Constraint-bearing runs: solve + replay-verify (the DAG/verify
        // composition over constrained plans lands with the pipeline's
        // constraint passthrough; receipted as partial surface).
        let saturation = p.saturate().map_err(|e| e.to_string())?;
        let problem = praxis_synthesis::SequenceProblem::with_constraints(
            &p, caps, goal, horizon, constraints,
        )
        .map_err(|e| e.to_string())?;
        match solver.solve(&problem) {
            Ok(plan) => {
                let replayed = problem.replay_reaches_goal(&plan);
                Ok(json!({
                    "status": if replayed { "admitted" } else { "refused" },
                    "saturation": saturation,
                    "plan": plan,
                    "replay_reaches_goal": replayed,
                }))
            }
            Err(refusal) => {
                Ok(json!({"status": "refused", "refusal": refusal, "rendered": refusal.to_string()}))
            }
        }
    }
}

/// Sequencing only: saturate → solve. Returns the plan (with its receipt)
/// or the refusal — which, under Solver8, may be a certified unsat proof a
/// second agent can verify without searching.
pub fn synth_solve_payload(payload: &str) -> Result<Value, String> {
    let (mut p, caps, goal, horizon, constraints, solver_name) = build(payload)?;
    let solver = pick_solver(&solver_name)?;
    let saturation = p.saturate().map_err(|e| e.to_string())?;
    let problem =
        praxis_synthesis::SequenceProblem::with_constraints(&p, caps, goal, horizon, constraints)
            .map_err(|e| e.to_string())?;
    match solver.solve(&problem) {
        Ok(plan) => Ok(json!({"status": "solved", "saturation": saturation, "plan": plan})),
        Err(refusal) => {
            Ok(json!({"status": "refused", "refusal": refusal, "rendered": refusal.to_string()}))
        }
    }
}
