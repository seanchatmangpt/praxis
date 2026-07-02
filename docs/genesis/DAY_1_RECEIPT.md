# Day 1 Receipt — Foundation & Publication

**Date:** 2026-07-01
**Program:** GENESIS Day 1 — Land the frontier, integrate proposer + propose verbs, walkthrough, PRD reconciliation, publication sweep, genesis manifest.

## Manifest

- File: `docs/genesis/MANIFEST_DAY_1.json`
- Hash algorithm: **blake3** (computed via `b3sum` over canonical JSON — sorted keys, compact separators — with the `manifest_hash` field absent)
- Manifest hash: `f6ec2387af8c0a6493f3f03c7fb918b7d0879434da5b4afb07458da585fa5dba`
- Repos recorded: praxis, wasm4pm-compat, wasm4pm, bcinr, star-toml, cargo-cicd, ggen, stpnt, affidavit, lsp-max, semantic_bit
- Per repo: HEAD commit, branch, dirty-file count, crate versions (root + `crates/*` Cargo.toml packages).

Verification: `jq 'del(.manifest_hash)' -cS MANIFEST_DAY_1.json` piped to `b3sum` must reproduce the hash above. (Note: canonical form is Python `json.dumps(sort_keys=True, separators=(",",":"))`.)

## What Landed

<!-- FINAL-PHASE: fill with lane deliverables, frontier pass_rate, walkthrough + PRD reconciliation results -->
_Pending final phase._

## Publication Results

<!-- FINAL-PHASE: fill with per-crate dry-run/publish outcomes and pushed remotes -->
_Pending final phase._

## Refusals

<!-- FINAL-PHASE: every refusal receipted with reason and salvage; silent gaps forbidden -->
_Pending final phase._

## Chain

- `prev_day_hash`: `0000000000000000000000000000000000000000000000000000000000000000` (genesis — no prior day)
- Day 1 `manifest_hash`: `f6ec2387af8c0a6493f3f03c7fb918b7d0879434da5b4afb07458da585fa5dba`
- Day 2's manifest must record this hash as its `prev_day_hash`. Day 7's receipt closes the chain by committing Day 1's hash.
