//! Shared fixture: the 5-step lawobject capability domain
//! (supply-evidence → clear-obligations → judge → admit → receipt),
//! expressed as *declared capabilities* — no PDDL text anywhere.

use praxis_synthesis::{Atom, Capability, Program, Term};

/// Build the lawobject domain: a program holding `raw(o1)` and the five
/// declared capabilities. Returns `(program, capabilities, goal)`.
#[must_use]
pub fn lawobject_domain() -> (Program, Vec<Capability>, Vec<Atom>) {
    let mut p = Program::new();
    let raw = p.intern("raw");
    let evidence = p.intern("evidence");
    let clear = p.intern("clear");
    let validated = p.intern("validated");
    let admitted = p.intern("admitted");
    let receipted = p.intern("receipted");
    let o1 = p.intern("o1");
    p.add_fact(raw, &[o1]).expect("fact");

    let v0 = Term::Var(0);
    let step = |name: &str, pre: praxis_synthesis::datalog::Atom, add: Atom| Capability {
        name: name.into(),
        params: 1,
        pre: vec![pre],
        add: vec![add],
        del: vec![],
        cost: 1,
    };
    let caps = vec![
        step("supply-evidence", Atom::new(raw, vec![v0]), Atom::new(evidence, vec![v0])),
        step("clear-obligations", Atom::new(evidence, vec![v0]), Atom::new(clear, vec![v0])),
        step("judge", Atom::new(clear, vec![v0]), Atom::new(validated, vec![v0])),
        step("admit", Atom::new(validated, vec![v0]), Atom::new(admitted, vec![v0])),
        step("receipt", Atom::new(admitted, vec![v0]), Atom::new(receipted, vec![v0])),
    ];
    let goal = vec![Atom::new(receipted, vec![Term::Const(o1)])];
    (p, caps, goal)
}
