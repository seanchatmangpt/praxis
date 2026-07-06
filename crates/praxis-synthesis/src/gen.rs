//! P6 — the adversarial domain generator.
//!
//! Coverage proves little when the author picked the domains. This module
//! generates sequencing domains as points in an 8-axis space, deterministic
//! from a seed — and the falsifier loop (in `tests/frontier_domains.rs`)
//! mutates specs *toward* solver disagreement, using the Solver8-vs-oracle
//! differential as its fitness function: fuzzing the solver with its own
//! differential, the simdjson method.

use serde::{Deserialize, Serialize};

use crate::datalog::{Atom, DlRule, Program, Term};
use crate::sequence::{Capability, Constraint};

/// A domain as a point in the 8-axis space. All axes are small by the
/// doctrine; the *combinations* are the space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainSpec {
    /// Causal chain depth (1..=8).
    pub depth: u8,
    /// Parallel branches per level (1..=4).
    pub branching: u8,
    /// Objects flowing through the domain (1..=4).
    pub objects: u8,
    /// Dead-end capabilities per level (0..=3): consume state, lead nowhere.
    pub dead_ends: u8,
    /// Interference: dead ends that also DELETE the live token (0..=2).
    pub interference: u8,
    /// Derived-predicate depth: rule chain length feeding the first
    /// precondition (0..=3) — exercises the Datalog→solver stack.
    pub rule_depth: u8,
    /// Constraint load: how many window/order constraints to attach (0..=6).
    pub constraint_load: u8,
    /// Goal width: how many branch-terminals the goal requires (1..=branching).
    pub goal_width: u8,
}

impl DomainSpec {
    /// Deterministic spec from a seed (xorshift64*).
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        let mut x = seed.max(1);
        let mut next = || {
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        };
        let branching = (next() % 3 + 1) as u8;
        Self {
            depth: (next() % 4 + 1) as u8,
            branching,
            objects: (next() % 2 + 1) as u8,
            dead_ends: (next() % 3) as u8,
            interference: (next() % 2) as u8,
            rule_depth: (next() % 3) as u8,
            constraint_load: (next() % 4) as u8,
            goal_width: (next() % u64::from(branching) + 1) as u8,
        }
    }

    /// Deterministically mutate one axis (falsifier coordinate ascent).
    #[must_use]
    pub fn mutate(self, axis: u8, up: bool) -> Self {
        let mut s = self;
        let bump = |v: u8, up: bool, lo: u8, hi: u8| {
            if up {
                (v + 1).min(hi)
            } else {
                v.saturating_sub(1).max(lo)
            }
        };
        match axis % 8 {
            0 => s.depth = bump(s.depth, up, 1, 8),
            1 => {
                s.branching = bump(s.branching, up, 1, 4);
                s.goal_width = s.goal_width.min(s.branching);
            }
            2 => s.objects = bump(s.objects, up, 1, 4),
            3 => s.dead_ends = bump(s.dead_ends, up, 0, 3),
            4 => s.interference = bump(s.interference, up, 0, 2),
            5 => s.rule_depth = bump(s.rule_depth, up, 0, 3),
            6 => s.constraint_load = bump(s.constraint_load, up, 0, 6),
            _ => s.goal_width = bump(s.goal_width, up, 1, s.branching),
        }
        s
    }

    /// Horizon this spec needs (may exceed what's satisfiable — that's the
    /// point of adversarial cells).
    #[must_use]
    pub fn horizon(&self) -> usize {
        usize::from(self.depth) * usize::from(self.goal_width).max(1) + 2
    }
}

/// Materialize the spec into a (saturated-ready) program + declarations.
#[must_use]
#[allow(clippy::missing_panics_doc, clippy::too_many_lines)]
pub fn generate(spec: &DomainSpec) -> (Program, Vec<Capability>, Vec<Atom>, Vec<Constraint>) {
    let mut p = Program::new();
    let v0 = Term::Var(0);

    // Rule chain: base(o) asserted; derived_i(X) :- derived_{i-1}(X); the
    // entry precondition uses the deepest derived predicate.
    let base = p.intern("base");
    let mut entry_pred = base;
    for r in 0..spec.rule_depth {
        let d = p.intern(&format!("derived{r}"));
        p.add_rule(DlRule {
            head: Atom::new(d, vec![v0]),
            body: vec![Atom::new(entry_pred, vec![v0])],
            negative: vec![],
        })
        .expect("chain rule is safe");
        entry_pred = d;
    }
    let objects: Vec<_> = (0..spec.objects)
        .map(|o| p.intern(&format!("obj{o}")))
        .collect();
    for o in &objects {
        p.add_fact(base, &[*o]).expect("base fact");
    }

    // Branch lattice: branch b has stages s0..depth; stage preds per branch.
    let mut caps = Vec::new();
    let mut terminals = Vec::new();
    for b in 0..spec.branching {
        let mut prev = entry_pred;
        for d in 0..spec.depth {
            let stage = p.intern(&format!("b{b}s{d}"));
            caps.push(Capability {
                name: format!("step-b{b}-d{d}"),
                params: 1,
                pre: vec![Atom::new(prev, vec![v0])],
                add: vec![Atom::new(stage, vec![v0])],
                del: vec![],
                cost: 1 + u32::from(b), // asymmetric costs: order matters
            });
            prev = stage;
        }
        terminals.push(prev);
        // Dead ends off this branch (consume, never produce goal-relevant).
        for de in 0..spec.dead_ends {
            let sink = p.intern(&format!("b{b}sink{de}"));
            let interferes = de < spec.interference;
            caps.push(Capability {
                name: format!("dead-b{b}-{de}"),
                params: 1,
                pre: vec![Atom::new(entry_pred, vec![v0])],
                add: vec![Atom::new(sink, vec![v0])],
                // Interfering dead ends DELETE the entry token: taking one
                // early can strand the plan — the trap the solver must avoid.
                del: if interferes {
                    vec![Atom::new(entry_pred, vec![v0])]
                } else {
                    vec![]
                },
                cost: 1,
            });
        }
    }

    // Goal: first goal_width branch terminals, for object 0.
    let goal: Vec<Atom> = terminals
        .iter()
        .take(usize::from(spec.goal_width))
        .map(|t| Atom::new(*t, vec![Term::Const(objects[0])]))
        .collect();

    // Constraint load: alternate Before across branch entries + windows.
    let mut constraints = Vec::new();
    for c in 0..spec.constraint_load {
        let b1 = c % spec.branching;
        let b2 = (c + 1) % spec.branching;
        match c % 3 {
            0 if b1 != b2 => constraints.push(Constraint::Before {
                a: format!("step-b{b1}-d0"),
                b: format!("step-b{b2}-d0"),
            }),
            1 => constraints.push(Constraint::NotEarlier {
                a: format!("step-b{b1}-d0"),
                k: c / 3,
            }),
            _ => constraints.push(Constraint::AtMost {
                a: format!("dead-b{b1}-0"),
                n: 1,
            }),
        }
    }
    // AtMost may reference a dead-end that doesn't exist at dead_ends == 0.
    constraints.retain(|c| match c {
        Constraint::AtMost { a, .. } => caps.iter().any(|cap| cap.name == *a),
        _ => true,
    });

    (p, caps, goal, constraints)
}
