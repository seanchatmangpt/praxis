//! Shared fixture: the 5-step lawobject capability domain
//! (supply-evidence → clear-obligations → judge → admit → receipt),
//! expressed as *declared capabilities* — no PDDL text anywhere.

use praxis_synthesis::{Atom, Capability, Program, Term};

/// Build the lawobject domain: a program holding `raw(o1)` and the five
/// declared capabilities. Returns `(program, capabilities, goal)`.
#[must_use]
#[allow(dead_code)] // each test binary uses its own subset of this module
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

/// Measurement-discipline helpers (the anti-2025-theatre kit). The 2025
/// lineage's benchmark register shows the failure modes these guard:
/// min-of-samples PASS, aggregate throughput masking per-op tails, and
/// unit drift. Verdicts come from percentiles/worst-case, never minimums.
#[allow(dead_code)]
pub mod stats {
    /// Nearest-rank percentile (`p` in 0..=100) over raw samples.
    /// Sorts in place. Panics on an empty slice — an empty sample set is
    /// not a measurement.
    pub fn percentile(samples: &mut [u128], p: f64) -> u128 {
        assert!(!samples.is_empty(), "no samples is not a measurement");
        assert!((0.0..=100.0).contains(&p), "percentile out of range");
        samples.sort_unstable();
        // Nearest-rank: ceil(p/100 * N), 1-based.
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let rank = ((p / 100.0) * samples.len() as f64).ceil() as usize;
        samples[rank.saturating_sub(1).min(samples.len() - 1)]
    }

    /// Median = the 50th percentile.
    pub fn median(samples: &mut [u128]) -> u128 {
        percentile(samples, 50.0)
    }

    /// Throughput recomputation: items per second from a count and an
    /// elapsed duration in nanoseconds. Every reported `*_per_sec` figure
    /// must equal this recomputation — unit redefinition was 2025's quiet
    /// killer ("8-tick compliant" quoted at 7,000ns).
    #[allow(clippy::cast_precision_loss)]
    pub fn per_second(items: usize, elapsed_ns: u128) -> f64 {
        assert!(elapsed_ns > 0, "zero elapsed time is not a measurement");
        items as f64 / (elapsed_ns as f64 / 1e9)
    }
}
