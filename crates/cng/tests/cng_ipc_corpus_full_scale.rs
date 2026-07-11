//! PROJ-711-followup — full-scale corpus law: seeds `0..IPC_CORPUS_SEEDS`
//! (20) across all 5 [`IPC_DOMAINS`], 100 `decompose()` calls total.
//!
//! `tests/cng_ipc_corpus.rs`'s
//! `ipc_corpus_seeds_plan_decompose_and_regenerate_byte_identically` covers
//! seeds `0..3` per domain (12 calls) by design — see that file's own doc
//! comment: "the full 20-seed corpus is the benchmark run, not this test."
//! This file IS that benchmark run, exercised as a real `#[test]` so its
//! pass/fail and wall-clock are directly citable (not just asserted in
//! prose). It asserts the exact same three properties per `(domain, seed)`
//! pair as the existing test:
//!   1. `generate_solvable` finds a plan at some gated size ≤ `max_size`.
//!   2. Same-seed regeneration (`generate`) is byte-identical
//!      (`domain_pddl`, `problem_pddl`, `meta`).
//!   3. `decompose` returns one of the three TYPED outcomes (never a
//!      refusal, never a silent fallback) with `candidate_receipts[0]` ==
//!      `"0-single"`.
//!
//! No generator files are touched by this file — it only calls the existing
//! `cng::bench::ipc` / `cng::bench::decomp` public surface.

#![cfg(feature = "bench")]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use chicago_tdd_tools::prelude::*;

use cng::bench::decomp::decompose;
use cng::bench::ipc::{generate, generate_solvable, max_size, parse_surface, plan, IPC_DOMAINS};

fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/ipc-corpus-full-scale")
        .join(test_name);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

// ---------------------------------------------------------------------------
// PROJ-711-followup — full 20-seed x 5-domain corpus law (100 decompose()
// calls), the scale PROJ-711 declared but the unit test never exercised.
// ---------------------------------------------------------------------------

test!(
    ipc_corpus_full_20_seeds_plan_decompose_and_regenerate_byte_identically,
    {
        // Arrange: the full declared corpus width, all 5 domains.
        let seeds: u64 = cng::bench::ipc::IPC_CORPUS_SEEDS;
        let overall_start = Instant::now();
        let mut per_domain_secs: Vec<(&str, f64)> = Vec::new();

        for domain_name in IPC_DOMAINS {
            let domain_start = Instant::now();
            for seed in 0..seeds {
                // Act: size-backoff solvability gate + plan.
                let (problem, gated_size) =
                    generate_solvable(domain_name, seed, max_size(domain_name)?)?;
                let tape = plan(&problem)?;

                // Assert: a real plan exists at the gated size.
                assert!(
                    !tape.ops.is_empty(),
                    "{domain_name} seed {seed}: empty plan"
                );
                assert_eq!(problem.meta.size, gated_size);
                assert_eq!(problem.meta.domain, domain_name);
                assert_eq!(problem.meta.seed, seed);

                // Assert: same-seed regeneration is byte-identical.
                let again = generate(domain_name, seed, gated_size)?;
                assert_eq!(problem.domain_pddl, again.domain_pddl);
                assert_eq!(problem.problem_pddl, again.problem_pddl);
                assert_eq!(problem.meta, again.meta);

                // Assert: decompose returns one of the three TYPED outcomes
                // (never a refusal, never a silent fallback) for a solvable
                // corpus problem.
                let (parsed_domain, parsed_problem) = parse_surface(&problem)?;
                let out = scratch_dir(&format!("corpus-{domain_name}-{seed}"));
                let result = decompose(
                    &parsed_domain,
                    &parsed_problem,
                    &out,
                    &format!("urn:cng:test:ipc:full:{domain_name}:{seed}"),
                )?;
                assert!(
                    [
                        "Selected",
                        "NoAdmissibleDecomposition",
                        "NoBeneficialDecomposition"
                    ]
                    .contains(&result.outcome.as_str()),
                    "{domain_name} seed {seed}: unexpected outcome {:?}",
                    result.outcome
                );
                assert_eq!(result.candidate_receipts[0].candidate_id, "0-single");
            }
            let domain_secs = domain_start.elapsed().as_secs_f64();
            per_domain_secs.push((domain_name, domain_secs));
            eprintln!(
                "PROJ-711-followup: {domain_name} {seeds} seeds in {domain_secs:.3}s \
                 ({:.3}s/seed)",
                domain_secs / seeds as f64
            );
        }

        let total_secs = overall_start.elapsed().as_secs_f64();
        eprintln!(
            "PROJ-711-followup: TOTAL {} domain*seed pairs in {total_secs:.3}s; per-domain: \
             {per_domain_secs:?}",
            IPC_DOMAINS.len() as u64 * seeds
        );
    }
);
