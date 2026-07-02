# Genesis Day 2 Receipt — Revenue Physics end-to-end

**Date:** 2026-07-02 (Day 2 of the seven-day program)
**Principle:** build beyond human reading, within human verification. No claim below exceeds a mechanism you can re-run.

---

## What landed

### 1. The full pipe, live — one command

```
just revenue-demo        # cargo run --features proposer --bin revenue_demo
```

Runs the whole chain in-process over the same `ops::*_payload` functions the `law` verbs wrap. Verified by re-running it while writing this receipt; the transcript below is byte-stable (fixed `ts_ns`):

- **Step 1 — observe → propose.** A 4-account `RevenueState` fixture with mixed evidence flags → `propose revenue` ranked proposals with rationales and a BLAKE3 `proposal_hash`. Top proposal: `(stage acct-apex closed-won)`, `proposal_hash = 81393deaf9b84ced…`.
- **Step 2 — proposal → PDDL goal.** Top proposal compiled to a goal atom.
- **Step 3 — plan solve.** Evidence-gated revenue domain → plan of length 2:
  `advance-gated(acct-apex,proposal,procurement)` → `close(acct-apex,procurement,closed-won)`.
- **Step 4 — law admit.** Every plan action passes `law judge`/`law admit`; both actions `admitted` with their evidence obligations satisfied.
- **Step 5 — signed receipt (AR-9 closure).** `law receipt` binds the `proposal_hash` inside the receipt payload:
  - `chain_hash = 229a4fe9c0ede59fbc4d20640ee5a7a48746f5a91aebf1504c175724ea1863f8`
  - `payload_hash = 28c493990ac45b534725d4740a64aa30240a3082c27050d189a165bee789eac9`
  - `prev_chain_hash = 0000…0000` (genesis)
  - `binds_proposal_hash = 81393deaf9b84ced0ca52d6e27423a05c184395fdb8e53b350d9363ca128461b` (== the top proposal_hash → the receipt chain binds back to *which* proposal was admitted)

The lawfulness pre-filter (proposal) and the admission gate use the **same** evidence flags: an account missing `legal_approved` is both never proposed past Proposal **and** refused by `admit` if forced — asserted in `tests/revenue_pipe.rs`.

### 2. RevTAC v0 — missions, not PDDL

Revenue operators author **missions** in ontology, one level above the substrate; a mission never grants permission — its output still passes judge/admit.

- Verbs: `propose mission`, `propose mrr` (`src/verbs/propose.rs`), compiler in `src/revtac.rs`.
- Format (JSON or TOML): `{ mission, constraints{min_evidence, exclude_accounts}, objective }`. Unknown evidence names are a hard error (never silently ignored); a mission with no objective is a hard error (RevTAC never invents the objective).
- JSON and TOML missions compile to **byte-identical** output (proven by `toml_and_json_missions_compile_identically` in `src/revtac.rs`).
- Documented with two worked examples in `docs/REVTAC.md`.

### 3. Maximum Reachable Revenue — computed, not reported

`maximum_reachable_revenue` (`crates/praxis-proposer/src/mrr.rs`) computes the lawful revenue ceiling. Boundedness argument: the `realized_revenue` fluent of an account depends only on that account, so total realizable revenue is a **sum of per-account maxima** — linear in accounts, no joint-plan enumeration. Numbers for the Day-2 fixture (integer cents; USD in parens):

| Account | Stage | Amount | Max realizable | Why |
|---|---|---|---|---|
| acct-apex | Proposal | 5,000,000 ($50k) | 5,000,000 | full evidence → lawful path to closed-won |
| acct-legal-gap | Qualified | 3,000,000 ($30k) | 0 | missing `legal_approved` |
| acct-fresh | Lead | 1,000,000 ($10k) | 0 | missing all evidence |
| acct-closed | ClosedWon | 500,000 ($5k) | 500,000 | already realized |

- **Maximum Reachable Revenue (MRR):** 5,500,000¢ = **$55,000**
- **Actual closed:** 500,000¢ = **$5,000**
- **Revenue Opportunity (gap):** 5,000,000¢ = **$50,000**
- **Revenue Utilization:** 500,000 / 5,500,000 = **≈ 0.0909 (9.09%)**, confined to `[0,1]`

Tests: MRR invariant to account ordering; removing `legal_approved` from an account lowers MRR by exactly that account's contribution; utilization in `[0,1]` (`0.0` when MRR == 0, documented rather than div-by-zero).

### 4. SHACL validation against the canonical receipt shape

A praxis `ReceiptRecord` is mapped to open-ontologies' `sr:SharedReceiptV1` and validated against `shared-receipt-shapes.ttl` (feature `ggen`, via `ggen_graph::prelude::validate_shacl`), in `src/receipt_shacl.rs`.

**Outcome:** the mapped receipt **conforms** (`mapped_receipt_conforms_to_shared_receipt_v1_shapes`), and a receipt with a required hash dropped is **detected as a violation** (`dropping_a_required_hash_is_detected_as_a_violation`). 4 tests total in the module (plus ISO-8601 and UUID-shape checks).

Every mapped field is tagged **[native]**, **[derived]**, or **[synthesized]** in the module docs. The mapping is honest about what praxis lacks (see refusals below).

---

## Refusals / gaps (receipted, not papered over)

1. **Manifest schema was reconciled across a concurrent workflow (no silent gap).** When this closer started, `docs/genesis/MANIFEST_DAY_1.json` did not yet exist, so an interim reconstruction was drafted. During the run, a concurrent Genesis workflow published the **authoritative** `DAY_1_RECEIPT.md` + `MANIFEST_DAY_1.json` — a full 11-repo constellation manifest (praxis, wasm4pm-compat, wasm4pm, bcinr, star-toml, cargo-cicd, ggen, stpnt, affidavit, lsp-max, semantic_bit; per-repo HEAD/branch/dirty-count/crate-versions), hash `f6ec2387…`. The interim reconstruction was **discarded** and this Day-2 manifest was regenerated to match Day 1's exact schema and hashing (verified: `f6ec2387…` reproduces from `blake3(json.dumps(sort_keys=True, separators=(",",":")))` with `manifest_hash` removed). Chain coherence is preserved; nothing was fabricated. (Note: Day 1's receipt suggests a `jq -cS | b3sum` verification; that command yields a different digest due to jq/b3sum newline+encoding handling — the reproducible canonical form is the Python `json.dumps` one, which both manifests use.)

2. **SHACL conformance dimensions not mapped (deliberate).** `sr:conformance` (fitness/precision/lifecycle) is *optional* (no `sh:minCount`), so omitting it is conformant. In praxis these are computed by `receipt_validator` as a separate replay concern; adding them to `ReceiptRecord` would duplicate state the validator owns. `duration_ms` *was* a real gap and was added as a native optional field (`0`/`None` on the live law path, populatable by callers that time admission).

3. **Five-way hash taxonomy is a vocabulary mismatch.** praxis's chain has three hashes with *chain* semantics; `sr:` expects five with *execution* semantics. The `output`/`config`/`plan` mappings are documented re-uses of praxis's three hashes, not distinct artifacts praxis produces.

4. **Open seam: `receipt` noun verb collision** (tracked, not yet resolved). The affidavit `receipt` path (via lsp-max) shadows praxis `show`/`replay`/`export-ocel` on the `receipt` noun. Does not affect the Day-2 pipe (which uses `law receipt`), flagged for a later day.

5. **Concurrent workflow active in-repo.** A separate Genesis workflow (Day 6 "church pack" + differential-testing/frontier lanes) is editing this repo concurrently. It added `church`/`engine` to praxis-proposer and `frontier` to the root crate, independent of the Day-2 work and with Day-2 exports intact. One monolithic `--all-features` run caught 2 `snapshots_verbs` tests mid-edit by that workflow; on isolated re-run they pass. Day-2 changes touch no file that workflow owns.

---

## Test summary

`cargo test --workspace --all-features` — run at receipt time, exit code **0**. Aggregate across all test binaries: **486 passed, 0 failed, 8 ignored**. No compile errors, no panics, no `FAILED` result lines. This full run includes the concurrent workflow's `church`/`engine`/`frontier` additions and is entirely green (the mid-edit `snapshots_verbs` flakiness noted in the phase reports did not reproduce in this completed run). Representative tail:

```
test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 64 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out   (revenue_pipe / verbs)
...
Doc-tests rust_fable_testbed
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Owned Day-2 surfaces, all green within the above: `praxis-proposer` lib (MRR + RevTAC + engine), root lib, `tests/revenue_pipe.rs`, and the `receipt_shacl` SHACL module.

---

## Chain

Manifest algorithm (matching Day 1 exactly, per `DAY_1_RECEIPT.md`): `manifest_hash = blake3(canonical_json(manifest))`, where canonical JSON is Python `json.dumps(obj, sort_keys=True, separators=(",",":"))` with the `manifest_hash` field removed. Verified reproducible: recomputing over `MANIFEST_DAY_1.json` gives `f6ec2387…` and over `MANIFEST_DAY_2.json` gives `cb184872…`. Constellation = the 11 repos Day 1 recorded: praxis, wasm4pm-compat, wasm4pm, bcinr, star-toml, cargo-cicd, ggen, stpnt, affidavit, lsp-max, semantic_bit — each with HEAD, branch, dirty-file count, and crate versions.

- **prev (Day 1)** `manifest_hash` = `f6ec2387af8c0a6493f3f03c7fb918b7d0879434da5b4afb07458da585fa5dba`
  (`docs/genesis/MANIFEST_DAY_1.json`; `prev_day_hash = 0`×64 genesis anchor)
- **this day (Day 2)** `manifest_hash` = `cb184872b7c8bd5030b524a5430bd44d208db5b151f6f21066514293dea4e8c9`
  (`docs/genesis/MANIFEST_DAY_2.json`; `prev_day_hash =` the Day-1 hash above)

Day-2 `praxis` HEAD recorded in the manifest: `54e6c9be33b7aed770eb9348f506f629792c8f60` (the HEAD immediately before this receipt commit; the receipt commit itself is the next commit). All 11 repos' Day-2 HEAD/branch/dirty-count/versions are captured in `MANIFEST_DAY_2.json`.

---

## What Day 3 inherits

- **A live, receipted revenue pipe** (`observe → propose → goal → plan → admit → receipt`) with a proposal_hash-bound chain — the exact admission boundaries Day 3's fuzz + proptest + mutation sweep must harden (quarantine, config loader, receipt validator, PDDL parser inputs, and the evidence gate shared by propose-filter and admit).
- **A stable determinism anchor**: `revenue_demo`'s `chain_hash 229a4fe9…` and `proposal_hash 81393deaf9b84ced…` are fixed targets a mutation test can assert against (flip/drop/reorder must change the chain hash).
- **MRR as an invariant to attack**: order-invariance and evidence-monotonicity are already property-tested; Day 3 can proptest them across generated states.
- **A SHACL conformance contract** (`sr:SharedReceiptV1`) the receipt validator must keep satisfying under mutation.
- **An open chain-hygiene debt to close**: Day 1's manifest was reconstructed here; Day 3+ should treat manifest emission as a hard gate so no future day reconstructs a predecessor.
- **The chain anchor**: Day 3's manifest chains `prev_day_hash = cb184872b7c8bd5030b524a5430bd44d208db5b151f6f21066514293dea4e8c9`, using the same `blake3(python json.dumps canonical)` algorithm and 11-repo schema.
