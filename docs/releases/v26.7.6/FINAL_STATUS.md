# Final Status — Praxis v26.7.6 "After Neon"

## OCEL + wasm4pm Final Standing

Date: 2026-07-06 (closing phase). Earned state:

**Praxis v26.7.6 After Neon is ALIVE_WITH_OCEL_AND_WASM4PM_EVIDENCE.**

Every claim in `CLAIM_PROMOTION_TABLE.md` is promoted with cited evidence or
typed to its resolution status; every row of the `NO_TERMINAL_BLOCKERS.md`
ledger is terminal (no `TEMP_BLOCKED`); the OCEL v2 evidence log passes
wasm4pm integrity validation (0 errors) and POWL conformance
(`fitness: 1.0`); the closing-phase `just verify-all` rerun is green — exit
0, check + test (153 binaries, 1566 passed, 0 failed) + clippy + doctor,
log sha256 `5e87e7bb…90bab7` — after one receipted fix-forward repair
(ledger row "Closing-phase `just verify-all` red"). The only remaining actions are external operator
side effects (credentials-holding human), typed non-blocking:

- crates.io publish → **ALIVE_EXCEPT_EXTERNAL_PUBLISH** for that lane.
  Operator checklist: `cargo login`; optionally bump `praxis-graphlaw`
  version 26.7.5 → 26.7.6 (one-line `crates/praxis-graphlaw/Cargo.toml`
  change); `cargo publish -p praxis-graphlaw`. Local packaging
  fresh-verified this pass: `cargo publish --dry-run --allow-dirty
  -p praxis-graphlaw` → exit 0, 2026-07-06T19:44:59Z.
- arXiv submission → **ALIVE_EXCEPT_EXTERNAL_SUBMISSION** for that lane.
  Operator checklist: make the artifact bundle / repository public
  (ARXIV_READINESS.md Sec. 11 blocker 2); upload
  `docs/releases/v26.7.6/arxiv-package/arxiv-submission.tar.gz` at
  https://arxiv.org/submit (cs.SE primary, cs.LO cross-list).

### Evidence pointers (all paths relative to repo root)

| Evidence | Path |
|---|---|
| OCEL v2 log (the ONE final log) | `docs/releases/v26.7.6/ocel/playwright-wasm4pm-validation.ocel.json` (sha256 `628807e0…f89e61`, blake3 `4c0d8584…afca40`; 50 events / 36 objects) |
| OCEL evidence report (narrative) | `docs/releases/v26.7.6/OCEL_V2_WASM4PM_REPORT.md` |
| UTC window | `docs/releases/v26.7.6/ocel/utc-window.json` (run `ocel-evidence-2026-07-06T19-10-42-924Z`) |
| Playwright test | `clients/autonomic-platform/tests/playwright/ocel-wasm4pm-validation.spec.ts` (driver: `clients/autonomic-platform/tests/run-evidence-pass.mjs`) |
| Screenshots | `docs/releases/v26.7.6/ocel/raw/screenshot-command.png`, `screenshot-ops.png`, `screenshot-dod.png`, `screenshot-optimus.png` |
| Trace | `clients/autonomic-platform/test-results/ocel-wasm4pm-validation-OC-5467e-idation-over-real-artifacts-chromium/trace.zip` |
| Benchmark artifacts (this pass) | `docs/releases/v26.7.6/ocel/raw/bench-root.txt`, `bench-ggen.txt`, `bench-graphlaw.txt`; medians in `BLUE_RIVER_DAM_BENCHMARKS.md` revalidation note |
| Receipt verification | `docs/releases/v26.7.6/RECEIPT_VERIFY_OCEL.md` (+ `docs/releases/v26.7.6/ocel/ledger-export.ocel.json`, `ocel/raw/receipt-validate.txt`, `ggen-receipt-verify.txt`, `ggen-receipt-history.txt`) |
| GraphLaw evidence | `docs/releases/v26.7.6/ocel/raw/graphlaw-e2e.txt` (5 passed 0 failed), `ggen-law-derive.txt` (55 derived) |
| Planner loop evidence | `docs/releases/v26.7.6/ocel/raw/full-loop.txt`, `full-loop-2.txt` (`powl_chain_hash blake3:1f97313c…c677e9bb` equal across runs) |
| wasm4pm validation | `docs/releases/v26.7.6/ocel/wasm4pm-process-validation.json` + `docs/releases/v26.7.6/WASM4PM_PROCESS_VALIDATION.md` |
| Claim promotion table | `docs/releases/v26.7.6/CLAIM_PROMOTION_TABLE.md` |
| No-terminal-blockers ledger | `docs/releases/v26.7.6/NO_TERMINAL_BLOCKERS.md` |
| Fresh publish dry-run | `docs/releases/v26.7.6/ocel/raw/cargo-publish-dry-run.txt` (sha256 `f562ff28…97474241`) |

### Exact next operator commands

```sh
# crates.io lane
cargo login                       # operator credentials
# optional: bump crates/praxis-graphlaw/Cargo.toml version 26.7.5 -> 26.7.6
cargo publish -p praxis-graphlaw

# arXiv lane (after making the artifact bundle public)
#   upload docs/releases/v26.7.6/arxiv-package/arxiv-submission.tar.gz
#   at https://arxiv.org/submit  (cs.SE primary, cs.LO cross-list)
```

---

Date: 2026-07-06. Verdict: **ALIVE**. All seven exit criteria
(RELEASE_CONTROL.md Sec. 5) are met with recorded evidence; the two
publication lanes (crates.io, arXiv) are dry-run-verified and await only the
operator's go. No `VerifierUnavailable` or other external blocker exists.

## 1. Exit criteria — each with evidence

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | `just verify-all` green | **MET** | Round 6 exit 0: check + test (152 binaries, 1555 passed, 0 failed) + clippy (`-D warnings`, all targets/features) + doctor (exit 0; single WARN: optional `cicd-evidence-gen` absent). Log SHA-256 `bdb7063d…92cea806`; six-round repair history in TEST_REPORT.md Sec. 1. |
| 2 | graphlaw live in ggen with e2e proof | **MET** | `crates/ggen/tests/graphlaw_e2e.rs` green inside the round-6 run (5 tests: SHACL refusal by focus node, denial refusal, N3-materialization guard, engine agreement, …). |
| 3 | One-command full-loop demo, deterministic across 2 runs | **MET** | `plan run` over `examples/v26_7_6_after_neon/goal.ttl`: identical `powl_chain_hash blake3:1f97313c…c677e9bb` across runs; with identical paths the full output JSON and the receipt ledger are **byte-identical** (same SHA-256 `ebab9f63…9d194d8e`; `diff -r` clean). Also pinned by `tests/plan_run_e2e.rs::two_runs_identical_chain_hashes`. |
| 4 | Breeds/algorithms admitted with a generated artifact | **MET** | `crates/ggen/tests/wasm4pm_facts_e2e.rs` green in round 6 (registry report lists all breeds/algorithms; standing rule derives evidence bounds; sync over the breeds graph deterministic); registry doc `docs/releases/v26.7.6/BREED_ALGORITHM_REGISTRY.md`; mapping doc `docs/v26.7.3/COGNITIVE_BREED_MAPPING.md`. |
| 5 | Full command surface typed-refusal-complete | **MET** (tested surface) | New `tests/command_surface.rs` iterates every noun/verb the root binary itself documents (enumerated from `--help`, ~90 probes) asserting typed-refusal-or-behavior, no panics, refusal-by-name for unknown commands — green in round 6. `ggen`: `crates/ggen/tests/cli_boundary.rs` (pre-existing). `praxis-l4`: covered by its own tests; not noun/verb-enumerated — noted as residual scope in Sec. 4. |
| 6 | 15 release docs in `docs/releases/v26.7.6/` | **MET** | 20 markdown docs on disk (this doc, TEST_REPORT.md, RECEIPTS.md, AXIOM_CENSUS.md, arxiv-package/MANIFEST.md joined the prior 15). |
| 7 | Receipt chain verifies | **MET** | `receipt validate` over the demo ledger: ok true, all five stages Pass (schema, chain_recompute, chain_linkage, monotonic, token_replay); `ggen receipt verify`/`history`: valid true, 8 records, head `35bc4ab0…ab04765a`. Details in RECEIPTS.md. |

## 2. What works / is proven / generated / verified / receipted / documented

- **Works**: the one-command loop `goal.ttl → PDDL plan → POWL tape →
  receipted bcinr execution → artifact (domain/problem/plan) → ledger
  receipt` (`plan run`); the GraphLaw engine inside the ggen factory; the
  full CLI surface (no panics observed across ~90 verb probes).
- **Proven (deterministic)**: byte-identical artifacts, output JSON, and
  receipt ledger across independent runs; `ts_ns = 0` and genesis-anchored
  `prev = 00…00` observed in the actual ledger record (invariants 2 and 3).
- **Generated**: after-neon plan artifacts; the arXiv source package;
  AXIOM_CENSUS.md (71 axiom declarations across 28 of 188 Lean files, zero
  sorry/admit).
- **Verified**: 1555 tests green; clippy clean under `-D warnings`; the
  Mathlib-linked Lean corpus kernel-checks end to end (`lake build`, 826
  jobs, exit 0).
- **Receipted**: demo ledger (chain recomputed, not asserted), `.ggen` sync
  ledger (8 records valid), Lean per-label receipts (202 records, commit
  `1ea2385`).
- **Documented**: 20 release docs; six-round gate history with root causes
  in TEST_REPORT.md; recorded lint debt in RELEASE_CONTROL.md Sec. 9.

## 3. Remaining blockers

None hard. Residual items, all typed and internal:

1. arXiv submission still requires a **public artifact bundle**
   (ARXIV_READINESS.md Sec. 11 blocker 2 — the repository is private);
   blockers 1 and 3 are closed by this gate and AXIOM_CENSUS.md.
2. `praxis-graphlaw` version says 26.7.5; decide whether to bump to 26.7.6
   before the real publish (one-line Cargo.toml change).
3. Doctor WARN: optional `cicd-evidence-gen` tool absent (`just evidence`
   lane only).
4. Recorded lint debt: pedantic/missing-docs allows at crate roots
   (TEST_REPORT.md Sec. 2) — burn down at leisure; correctness lints stay
   active.

## 4. Out of scope for this gate

- Phase 3b (Blue River Dam Divan benchmarks + sales report), Phase 6
  (Autonomic PI client surface) — separate tracked tasks.
- `praxis-l4` noun/verb-enumerated refusal probing (its gate behavior is
  tested, its CLI surface enumeration is not).
- Burning down the recorded lint debt.
- Publishing anything: **nothing was published or submitted**.

## 5. Operator commands (when you decide to go)

Preconditions: commit/clean tree (cargo publish refuses dirty), logged-in
cargo (`cargo login`), and the version decision from Sec. 3 item 2.

```sh
# crates.io — real publish (order matters only if you also publish the
# other two; praxis-graphlaw is self-contained):
cargo publish -p praxis-graphlaw
# optional, also self-contained:
cargo publish -p chatman-common
cargo publish -p powl2-decompose

# arXiv — upload the dry-run package after the public artifact bundle exists:
#   file: docs/releases/v26.7.6/arxiv-package/arxiv-submission.tar.gz
#   at:   https://arxiv.org/submit   (cs.SE primary, cs.LO cross-list,
#         per PUBLICATION_PLAN.md)

# Re-verify at any time:
just verify-all
cargo run --features ggen --bin my-conforming-project -- plan run \
  --goal examples/v26_7_6_after_neon/goal.ttl \
  --out-dir target/plan_run/after_neon --receipts-dir target/plan_run/after_neon_receipts
target/debug/my-conforming-project receipt validate --dir target/plan_run/after_neon_receipts
PATH="$HOME/.elan/bin:$PATH" lake build   # in tools/paper-factory/lean-lake/
```
