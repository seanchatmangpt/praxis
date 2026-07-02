# Day 1 Receipt — Foundation & Publication

**Date:** 2026-07-01 (closed 2026-07-02 by the Day-1 closer)
**Program:** GENESIS Day 1 — Land the frontier, integrate proposer + propose verbs, walkthrough, PRD reconciliation, publication sweep, genesis manifest.

## Manifest

- File: `docs/genesis/MANIFEST_DAY_1.json`
- Hash algorithm: **blake3** (computed via `b3sum` over canonical JSON — sorted keys, compact separators — with the `manifest_hash` field absent)
- Manifest hash: `f6ec2387af8c0a6493f3f03c7fb918b7d0879434da5b4afb07458da585fa5dba`
- Repos recorded: praxis, wasm4pm-compat, wasm4pm, bcinr, star-toml, cargo-cicd, ggen, stpnt, affidavit, lsp-max, semantic_bit
- Per repo: HEAD commit, branch, dirty-file count, crate versions (root + `crates/*` Cargo.toml packages).

Verification: canonical form is Python `json.dumps(sort_keys=True, separators=(",",":"))` with `manifest_hash` removed, piped to `b3sum`. **Re-verified at close: recomputed hash matches the stored hash byte-for-byte.**

## What Landed

All lanes of the frontier plan (`continue-work-on-the-elegant-wirth`) landed on `main` through `54e6c9b` (integration report), with the working tree handed to Day-2 agents at close:

| Lane | Deliverable | Status |
|---|---|---|
| Phase 0 | hygiene/repoints (`b3778f7`..`099087f`) | committed |
| 8a | ops extraction — `src/ops.rs` single source of truth, verbs thin | committed |
| 4 | signing — `law verify-signature`, fail-closed | committed; **fail-closed re-verified live at close**: `receipt issue` without `PRAXIS_SIGNING_KEY` refuses with "no signing key available" |
| 6 | config noun — `config show/witness/validate` (+ env-prefix fix) | committed; **spot-run at close**: `config show` returns effective `PraxisConfig` JSON |
| 2 | mfg noun — `mfg pddl/facts/validate`, `ontology/lawobject.ttl` | committed; **spot-run at close**: `mfg pddl --ontology ontology/lawobject.ttl` manufactures deterministic PDDL8 domain+problem, `graph_hash f75eeba3…` (note: noun requires the `ggen` feature — see Day-2 inheritance) |
| 1 | plan noun — all 5 verbs + tests | committed; **spot-run at close**: `plan lawobject` self-test returns `admitted: true`, same `graph_hash f75eeba3…` |
| 3 | receipt core (praxis-core) + CLI verbs (`src/verbs/receipt.rs`) | committed; **spot-run at close**: `receipt issue` (signed, chain from 64-zero genesis) then `receipt validate --dir` → all stages Pass; `verifier verify --path … --profile default` → 6/6 stages passed, `accepted: true` |
| 5 | admission enrichment — refusal.rs taxonomy, law_andon, `ReceiptMeta.denial`, prolog8 Kernel proof path | committed; `refusal_categories` verified in live output at integration |
| 7 | verify — `verify.rs` pipeline, verifier verb rewrite, insta snapshots, proptests (+2 fix-forward commits: `c28882b` duration_ms field, `0c07df5`/`54e6c9b` test fixes) | committed; **spot-run at close** (see lane 3 row) |
| 8b | MCP de-stub — ops-backed tools + moka cache (`mcp_lawobject_server`, `src/mcp_cache.rs`) | committed |
| PR-14 | proposer promoted into workspace + `propose` noun (revenue/goal) | committed |

**Law noun spot-run at close:** `law judge --payload '{"id":…,"value":…}' --law default` → `verdict: validated`, `andon: Green`.

### Frontier matrix (`dod matrix`)

Run at close (2026-07-02):

- `pass_rate`: **1.0**
- `total`: 286 cells, `evaluated`: 30, `passing`: 30, `failures`: 0
- `coverage`: 0.1049 (30/286 cells evaluated; unevaluated cells are recorded, not silently passed)
- Full report: `target/frontier-report.json` (written by the verb on every run)

### Final sweep (run at close)

- `cargo build --all-features`: **green** (warnings only).
- `cargo test --workspace --all-features`: **202 passed / 1 failed** across 4 suites (15 + 4 + 57 + 127).
  - The single failure, `ops::tests::receipt_validate_on_clean_ledger_is_ok`, occurred **only while the Day-3 agent's in-place cargo-mutants run had a live mutant applied** to `crates/praxis-core` (marker `/* ~ changed by cargo-mutants ~ */` observed in `receipt_record.rs`, then `receipt_validator.rs`, at each failure). The test failing under a mutated validator is the test *killing the mutant* — expected mutation-testing behavior, not a regression. Three clean-window retries all raced the mutation loop; recorded honestly rather than fabricating a green line. Prior quiescent-tree sweeps (integration close, `54e6c9b`) were fully green.

## Publication Results

Executed 2026-07-02. Credential: `~/.cargo/credentials.toml` (1 token). Every publish preceded by a passing dry-run and a not-already-published check.

### Git pushes

| repo | branch | action | outcome | detail |
|---|---|---|---|---|
| praxis | main | none needed | already-up-to-date | HEAD == origin/main (Day-2 agents' work already pushed) |
| wasm4pm-compat | main | push | PUSHED | 17f4d91..6aead3c |
| wasm4pm | release/v26.7.1 | push | PUSHED | 1770cfc7..f048dc61 |
| bcinr | main | push | PUSHED | cd08dc35..c61648eb (61 commits) |
| ggen | main | push | **PUSH-FAILED (rule)** → branch fallback | see Refusals |
| stpnt | main | push | PUSHED | 41df8ee..9f5a400 (includes the license-field flip commit) |
| lsp-max | master | push | PUSHED | b53d483..571bed5 (25 commits) |
| semantic_bit | main | push | PUSHED | 50b444e..cc6af50 |

No force pushes.

### crates.io publishes (executed dependency order)

`miniml` was promoted ahead of `wasm4pm` after wasm4pm's dry-run failed to resolve `miniml = "^26.7.1"` — actual dependency order overrode the matrix's stated order.

| # | crate | version | dry-run | outcome |
|---|---|---|---|---|
| 1 | wasm4pm-compat | 26.6.29 | PASS | PUBLISHED (API-verified) |
| 2 | prolog8 | 26.7.1 | PASS | PUBLISHED (API-verified) |
| 3 | wasm4pm-cognition | 26.7.1 | PASS | PUBLISHED (API-verified) |
| 4 | miniml | 26.7.1 | PASS | PUBLISHED (API-verified; reordered before wasm4pm) |
| 5 | wasm4pm | 26.7.1 | FAIL then PASS after #4 | PUBLISHED (API-verified) |
| 6 | ocpq | 26.7.1 | PASS | PUBLISHED (API-verified) |
| 7 | tps-metrics | 26.7.1 | PASS | 1st attempt: transient DNS failure (`index.crates.io` unresolvable); verified NOT published (registry still 26.6.25); retried once → PUBLISHED (cargo registry-availability wait confirmed 26.7.1) |
| 8 | wasm4pm-cli | 26.7.1 | PASS | PUBLISHED (API-verified) |

**8/8 publishable crates published.** Verification basis: cargo's post-publish registry-availability wait for all 8, plus direct crates.io API confirmation for 7 of 8 (rate-limiting stopped the eighth API check; cargo's wait covers it).

## Refusals

Every blocked or refused item, with reason and unblock condition. No silent gaps.

1. **ggen `main` push — REFUSED by remote ruleset.** `GH013: Changes must be made through a pull request` (repo ruleset on `refs/heads/main`; all 4/4 local pre-push gates passed). Retried exactly once to capture the full rejection reason. **Salvage:** all 16 commits preserved on remote branch `publication/genesis-day1-metadata` (HEAD `f0cf6758b`). **Unblock:** open PR `publication/genesis-day1-metadata` → `main` and merge.
2. **tps-metrics first publish attempt — failed transiently** (DNS: `Could not resolve host: index.crates.io`). Not a refusal of the crate; receipted because a publish attempt errored. Verified not-published before the single retry, which succeeded. **Resolved within the day.**
3. **stpnt license-field flip — refusal obsoleted.** The publication metadata audit landed the license-field fix on stpnt `main` (pushed in 41df8ee..9f5a400). This **obsoletes the praxis refusal-register cell** that previously recorded stpnt as unpublishable for a missing/invalid license field; the register should be updated when that cell is next touched.
4. **Crates outside the publishable subset — not published by design.** Standing rule 3: dry-run first, refusal-receipt when unpublishable (path deps, missing metadata). The Day-1 matrix admitted exactly the 8 crates above; workspace members not in that subset (e.g. `bench-tools`, `wasm4pm-planner`, the praxis workspace crates at 26.7.2 on a dirty tree) were not attempted. **Unblock:** metadata/path-dep cleanup, then a future publication sweep.
5. **praxis push at publication time — no-op, receipted.** HEAD already equalled origin/main; nothing to push, nothing silently skipped.

## Chain

- `prev_day_hash`: `0000000000000000000000000000000000000000000000000000000000000000` (genesis — no prior day)
- Day 1 `manifest_hash`: `f6ec2387af8c0a6493f3f03c7fb918b7d0879434da5b4afb07458da585fa5dba` (re-verified at close: recomputed from `MANIFEST_DAY_1.json` and matched)
- Day 2's manifest records this hash as its `prev_day_hash` (`MANIFEST_DAY_2.json`, sealed `cb184872…`). Day 7's receipt closes the chain by committing Day 1's hash; the week seal (`GENESIS_SEAL.json`, `9c666317…`) commits the ordered sealed-manifest list beginning with this hash.
