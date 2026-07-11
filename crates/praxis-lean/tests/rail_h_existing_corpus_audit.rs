//! One-off, real verification for PRD v26.7.11 PROJ-768 (Rail H toolchain
//! bridge, docs/jira/v26.7.11/tickets/index.md): does the existing Lean
//! corpus at tools/paper-factory/lean-lake actually build, and does this
//! crate's own no-sorry audit find anything real in it? Not a claim about
//! any of the 9 formalization targets at PRD.md:1035-1043 -- those are
//! PROJ-769, deliberately not attempted here (writing a real theorem for
//! any of them needs real domain modeling, not a rushed one-liner).
//!
//! Real finding this session: under `AuditPolicy::default()` (forbid_axiom
//! = true, zero allowed prefixes), this pre-existing corpus (predates
//! v26.7.11, from an earlier thesis effort per this session's PRD
//! reconciliation) has 71 `axiom` declarations -- not `sorry`/`admit`, but
//! unauthorized `axiom`s under this crate's own strict default. Some of
//! these look like standard, legitimate cryptographic assumptions (e.g.
//! `axiom chainH_cr : ...` collision-resistance) that formal crypto proofs
//! routinely axiomatize rather than derive -- that is a real category
//! distinct from an axiom used to shortcut a proof that should have been
//! done. Distinguishing the two per-axiom is real epistemic work (PROJ-770
//! scope: building the `allowed_axiom_prefixes` allowlist with a stated
//! justification per entry), not something to wave through here. This test
//! is a regression guard on the CURRENT count (71), not a claim that 71 is
//! correct or acceptable -- an increase means new unaudited axioms landed;
//! a decrease means real reconciliation happened (update the constant).
//! `Praxis/Mathlib` was excluded going forward with a name suggesting
//! mathlib-style axiomatization is expected there; verify that assumption
//! before relying on it (not verified this session).

use camino::Utf8Path;

const KNOWN_UNAUDITED_AXIOM_COUNT: usize = 71;

#[test]
fn existing_lean_lake_corpus_builds() {
    // Real signal this session: `lake build` in
    // tools/paper-factory/lean-lake succeeded (826 jobs, exit 0). This test
    // only checks the directory exists as a precondition for the audit
    // below; it does not re-run `lake build` (slow, and already verified
    // directly this session) to keep `cargo test` fast.
    let root =
        Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/paper-factory/lean-lake");
    assert!(
        root.exists(),
        "expected the pre-existing lean-lake package at {root}"
    );
    assert!(root.join("lakefile.lean").exists());
}

#[test]
fn existing_lean_lake_corpus_axiom_count_has_not_regressed() {
    let root = Utf8Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/paper-factory/lean-lake/Praxis");
    assert!(
        root.exists(),
        "expected the pre-existing lean-lake corpus at {root}"
    );

    let result = praxis_lean::cli::no_sorry(&root).expect("no_sorry audit itself must not error");

    let findings = result
        .get("findings")
        .and_then(|f| f.as_array())
        .cloned()
        .unwrap_or_default();
    let sorries_and_admits = findings
        .iter()
        .filter(|f| f.get("kind").and_then(|k| k.as_str()) != Some("axiom"))
        .count();
    assert_eq!(
        sorries_and_admits, 0,
        "found a real sorry/admit (not axiom) in the existing corpus -- this is always a real \
         regression, unlike the axiom count below: {findings:?}"
    );

    let axiom_count = findings.len() - sorries_and_admits;
    assert_eq!(
        axiom_count, KNOWN_UNAUDITED_AXIOM_COUNT,
        "unaudited-axiom count changed from the known baseline of {KNOWN_UNAUDITED_AXIOM_COUNT} \
         to {axiom_count} -- if this went up, new axioms landed without justification; if it \
         went down, real reconciliation happened and KNOWN_UNAUDITED_AXIOM_COUNT should be \
         updated to match, not silently ignored"
    );
}
