//! P4 scale evidence: the join engine measured on the shapes where naive
//! engines die — cyclic patterns (triangles) and recursive closure — with the
//! numbers written to a receipt, not a narrative.
//!
//! The default-run tests prove correctness at moderate size; the `#[ignore]`d
//! measurement runs the real sizes and writes
//! `target/synthesis-scale-receipt.json`.

use praxis_synthesis::{Atom, DlRule, Program, Term};

/// Deterministic xorshift64* — no `rand` dep, no ambient entropy.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// Random directed graph: `m` distinct edges over `n` nodes (seeded).
fn random_edges(seed: u64, n: u64, m: usize) -> Vec<(u64, u64)> {
    let mut rng = Rng(seed);
    let mut seen = std::collections::HashSet::with_capacity(m);
    let mut edges = Vec::with_capacity(m);
    while edges.len() < m {
        let a = rng.below(n);
        let b = rng.below(n);
        if a != b && seen.insert((a, b)) {
            edges.push((a, b));
        }
    }
    edges
}

/// Build the triangle-listing program: `tri(X,Y,Z) :- e(X,Y), e(Y,Z), e(X,Z)`.
/// The cyclic pattern: the third atom is fully bound when reached — one
/// membership probe — which is exactly where prefix-probe joins beat
/// nested-loop scans.
fn triangle_program(
    edges: &[(u64, u64)],
) -> (Program, praxis_synthesis::datalog::SaturationReceipt) {
    let mut p = Program::new();
    let e = p.intern("e");
    let tri = p.intern("tri");
    let mut ids = std::collections::HashMap::new();
    for &(a, b) in edges {
        let ia = *ids
            .entry(a)
            .or_insert_with(|| p.dict.intern(&format!("n{a}")));
        let ib = *ids
            .entry(b)
            .or_insert_with(|| p.dict.intern(&format!("n{b}")));
        p.add_fact(e, &[ia, ib]).expect("edge");
    }
    p.add_rule(DlRule {
        head: Atom::new(tri, vec![Term::Var(0), Term::Var(1), Term::Var(2)]),
        body: vec![
            Atom::new(e, vec![Term::Var(0), Term::Var(1)]),
            Atom::new(e, vec![Term::Var(1), Term::Var(2)]),
            Atom::new(e, vec![Term::Var(0), Term::Var(2)]),
        ],
        negative: vec![],
    })
    .expect("rule");
    let receipt = p.saturate().expect("saturation");
    (p, receipt)
}

/// Reference triangle count by brute hash-join (independent oracle).
fn triangle_oracle(edges: &[(u64, u64)]) -> usize {
    let set: std::collections::HashSet<(u64, u64)> = edges.iter().copied().collect();
    let mut out: std::collections::HashMap<u64, Vec<u64>> = std::collections::HashMap::new();
    for &(a, b) in edges {
        out.entry(a).or_default().push(b);
    }
    let mut count = 0;
    for &(x, y) in edges {
        if let Some(zs) = out.get(&y) {
            for &z in zs {
                if set.contains(&(x, z)) {
                    count += 1;
                }
            }
        }
    }
    count
}

#[test]
fn triangles_agree_with_the_independent_oracle() {
    let edges = random_edges(42, 300, 3_000);
    let (p, receipt) = triangle_program(&edges);
    let tri = p.dict.get("tri").expect("interned");
    assert_eq!(
        p.count_for(tri),
        triangle_oracle(&edges),
        "join engine and hash-join oracle must agree on the cyclic pattern"
    );
    assert!(receipt.derived_count > 0, "the seed graph has triangles");
}

#[test]
fn tree_closure_agrees_with_arithmetic() {
    // Complete binary tree, 1023 nodes: TC size = sum of depths = 8194.
    let mut p = Program::new();
    let parent = p.intern("parent");
    let anc = p.intern("anc");
    let ids: Vec<_> = (0..1023u32).map(|i| p.intern(&format!("v{i}"))).collect();
    for i in 1..1023usize {
        p.add_fact(parent, &[ids[(i - 1) / 2], ids[i]])
            .expect("edge");
    }
    p.add_rule(DlRule {
        head: Atom::new(anc, vec![Term::Var(0), Term::Var(1)]),
        body: vec![Atom::new(parent, vec![Term::Var(0), Term::Var(1)])],
        negative: vec![],
    })
    .expect("rule");
    p.add_rule(DlRule {
        head: Atom::new(anc, vec![Term::Var(0), Term::Var(2)]),
        body: vec![
            Atom::new(anc, vec![Term::Var(0), Term::Var(1)]),
            Atom::new(parent, vec![Term::Var(1), Term::Var(2)]),
        ],
        negative: vec![],
    })
    .expect("rule");
    p.saturate().expect("saturation");
    // Depth of node i (0-based, complete binary tree) = floor(log2(i+1));
    // TC pairs = sum of depths.
    let expected: usize = (1..1024usize)
        .map(|i| (usize::BITS - i.leading_zeros() - 1) as usize)
        .sum();
    assert_eq!(p.count_for(anc), expected);
}

/// The measurement run. Sizes chosen so the pathological shape (triangles on
/// a dense-ish random graph) and recursive closure both run at 10^5–10^6
/// EDB facts. Writes `target/synthesis-scale-receipt.json` with tuples/s.
#[test]
#[ignore = "measurement run; execute with --ignored --release to regenerate the scale receipt"]
fn scale_receipt() {
    let mut points = Vec::new();

    // Triangle listing at rising edge counts (cyclic pattern).
    for &(n, m) in &[
        (3_000u64, 100_000usize),
        (10_000, 400_000),
        (30_000, 1_000_000),
    ] {
        let edges = random_edges(7, n, m);
        let start = std::time::Instant::now();
        let (p, receipt) = triangle_program(&edges);
        let elapsed = start.elapsed();
        points.push(serde_json::json!({
            "shape": "triangle-listing (cyclic 3-join)",
            "nodes": n,
            "edb_facts": m,
            "derived": receipt.derived_count,
            "total_tuples": p.len(),
            "iterations": receipt.iterations,
            "elapsed_ms": elapsed.as_millis(),
            "edb_tuples_per_sec": (m as f64 / elapsed.as_secs_f64()) as u64,
        }));
    }

    // Recursive closure on a wide shallow forest (bounded output).
    for &n in &[100_000u32, 1_000_000] {
        let mut p = Program::new();
        let parent = p.intern("parent");
        let anc = p.intern("anc");
        let ids: Vec<_> = (0..n).map(|i| p.intern(&format!("v{i}"))).collect();
        for i in 1..n as usize {
            // 64-ary forest: depth ~ log64(n) — closure stays ~2n..3n.
            p.add_fact(parent, &[ids[(i - 1) / 64], ids[i]])
                .expect("edge");
        }
        p.add_rule(DlRule {
            head: Atom::new(anc, vec![Term::Var(0), Term::Var(1)]),
            body: vec![Atom::new(parent, vec![Term::Var(0), Term::Var(1)])],
            negative: vec![],
        })
        .expect("rule");
        p.add_rule(DlRule {
            head: Atom::new(anc, vec![Term::Var(0), Term::Var(2)]),
            body: vec![
                Atom::new(anc, vec![Term::Var(0), Term::Var(1)]),
                Atom::new(parent, vec![Term::Var(1), Term::Var(2)]),
            ],
            negative: vec![],
        })
        .expect("rule");
        let start = std::time::Instant::now();
        let receipt = p.saturate().expect("saturation");
        let elapsed = start.elapsed();
        points.push(serde_json::json!({
            "shape": "recursive closure (64-ary forest)",
            "edb_facts": n - 1,
            "derived": receipt.derived_count,
            "total_tuples": p.len(),
            "iterations": receipt.iterations,
            "elapsed_ms": elapsed.as_millis(),
            "derived_tuples_per_sec":
                (receipt.derived_count as f64 / elapsed.as_secs_f64()) as u64,
        }));
    }

    let receipt = serde_json::json!({
        "what": "saturation scale receipt — pathological shapes, measured honestly",
        "engine": "semi-naive over columnar sorted relations, greedy bound-prefix join order",
        "identity": "structural [u32;8] — no packed-key collisions",
        "points": points,
    });
    std::fs::write(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../target/synthesis-scale-receipt.json"
        ),
        serde_json::to_string_pretty(&receipt).expect("serialize"),
    )
    .expect("write receipt");
}
