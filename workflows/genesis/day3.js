export const meta = {
  name: 'genesis-day-3',
  description: 'Genesis Day 3: adversarial hardening — differential oracles, fuzzing, mutation, benches (the simdjson method)',
  phases: [
    { title: 'Gate', detail: 'wait for Day 2 receipt + quiescence' },
    { title: 'Differential', detail: 'independent implementations as mutual oracles' },
    { title: 'Fuzz+Mutate', detail: 'fuzz admission boundaries, mutate the chain' },
    { title: 'Receipt', detail: 'Day 3 receipt chained to Day 2' },
  ],
}

phase('Gate')
const gate = await agent(
  'Quiescence gate for Genesis Day 3 on /Users/sac/praxis. Poll every 120s (cap 100 min): require docs/genesis/DAY_2_RECEIPT.md exists with a chain hash, no praxis cargo builds running, git stable across two polls. On pass survey: git log --oneline -8, DAY_2_RECEIPT.md contents, cargo test --workspace --all-features 2>&1 | tail -5. Compact report.',
  { label: 'day3-gate', effort: 'low' }
)

phase('Differential')
const differential = await agent(
  'Genesis Day 3, phase 1: differential verification (the simdjson method — correctness via independent implementations agreeing, not comprehension). Repo /Users/sac/praxis; gate: ' + JSON.stringify(gate).slice(0, 2000) +
  ' Build tests/differential.rs exercising FOUR oracle pairs, each with a shared corpus of generated cases: (1) PLANNERS: bcinr-pddl (praxis dep) vs /Users/sac/wasm4pm/crates/wasm4pm-planner (add as dev-dependency path dep if it builds cleanly; if its 26.7.1 manifest conflicts, receipt the blocker and use its parse layer only) — same domain+problem text into both; assert both find a plan or both refuse; when both plan, assert goal-satisfaction of each plan under the OTHER implementation\'s state semantics if feasible, else assert makespan/step-count agreement bounds. Corpus: the lawobject PDDL8 exemplar, the revenue domain, plus 20+ generated small STRIPS domains (deterministic seeded generator you write). (2) CONFORMANCE: praxis\'s PowlReplayVerifier lifecycle replay vs /Users/sac/dteam src/conformance/bitmask_replay.rs NetBitmask64 (add dteam as dev-dep ONLY if its unibit path deps resolve; likely they will not from praxis — in that case extract the comparison as a standalone test binary in a scratch crate under crates/ or receipt the blocker precisely). (3) CHAIN: praxis chain recompute vs an independent from-scratch reimplementation you write in the test itself (30 lines: BLAKE3(prev || frame_bytes)) — byte-for-byte agreement on 100 random records. (4) OBJECTIVE: praxis-proposer scoring vs a naive reimplementation in test code — bit-exact score agreement. Every disagreement found is a BUG to fix (root-cause it — do not paper over by loosening assertions). Report: pairs wired, corpus sizes, disagreements found and fixed, blockers receipted.',
  { label: 'differential', effort: 'high' }
)

phase('Fuzz+Mutate')
const fuzzmutate = await agent(
  'Genesis Day 3, phase 2: fuzzing + mutation testing on /Users/sac/praxis. Prior: ' + JSON.stringify(differential).slice(0, 2000) +
  ' (1) FUZZ every admission boundary with proptest (already a dev-dep; do NOT add cargo-fuzz/libfuzzer — proptest with high case counts is the right fit here): arbitrary bytes/strings into RiceQuarantine::admit, arbitrary JSON into every ops::*_payload fn (never panics, always Err or structured refusal), arbitrary TOML into the PraxisConfig loader, arbitrary PDDL-ish text into plan solve (parser never panics), arbitrary standing strings into promote. Property: total absence of panics + every rejection carries a reason. Add crates/praxis-core/tests/fuzz_boundaries.rs + tests/fuzz_ops.rs with PROPTEST_CASES documented. (2) MUTATION-test the chain: implement affidavit-style mutation operators as test helpers (EventDrop, EventReorder, FieldFlip, HashTruncate, TimestampSkew) applied to receipt-record sequences; assert the validator catches EVERY mutant at the correct stage (drop→linkage, reorder→monotonic or token_replay, flip→chain_recompute, truncate→schema). Kill every survivor or receipt exactly why it survives (a mutant the design genuinely cannot catch, e.g. mutating a field outside the hash preimage, must be documented as a known-uncovered class with the design reason). (3) If cargo-mutants is installed (check ~/.cargo/bin, dteam uses it), run it scoped to crates/praxis-core with a time cap, triage the top surviving mutants, kill the cheap ones. (4) BENCHES: run all criterion benches; record receipt-validation vs the <5ms target and admission-path latency; add an admission-throughput bench (ops::judge_payload calls/sec on the green path). cargo build/test --all-features green. Report: properties added, mutants killed/survived-with-reason, bench numbers vs targets.',
  { label: 'fuzz-mutate', effort: 'high' }
)

phase('Receipt')
const receipt = await agent(
  'Genesis Day 3 closer for /Users/sac/praxis. Reports: ' + JSON.stringify(differential).slice(0, 1200) + ' | ' + JSON.stringify(fuzzmutate).slice(0, 1500) +
  ' Write docs/genesis/DAY_3_RECEIPT.md: oracle pairs + disagreements fixed, fuzz properties + case counts, mutation kill table (caught-at-stage matrix), bench numbers vs stated targets, refusals/blockers with reasons. Chain: prev = manifest_hash from docs/genesis/MANIFEST_DAY_2.json, write MANIFEST_DAY_3.json same algorithm. Update GENESIS.md Day 3 row. Full test sweep tail recorded. Commit ("feat(genesis): day 3 — differential oracles, fuzz, mutation kill") and push. Return receipt + Day 4 inheritance.',
  { label: 'day3-receipt', effort: 'medium' }
)

return { gate, differential, fuzzmutate, receipt }
