# Genesis Day 6 Receipt — Mission Physics beyond revenue

**Date:** 2026-07-02 (Day 6 of the seven-day program)
**Program:** GENESIS Day 6 — a second domain pack (church operations) on the *identical* substrate as revenue, proving the law is domain-independent: two institutions, one admission boundary.
**Principle:** build beyond human reading, within human verification. No claim below exceeds a mechanism you can re-run.

> This receipt is written **after** the Day-7 seal was already cast. That is unusual and is stated plainly here rather than hidden: Day 6's work (church pack + generic mission substrate) genuinely landed, but at Day 7 it had sealed no manifest and `tests/two_domains.rs` + `docs/MISSION_PHYSICS.md` were still pending (see `DAY_7_RECEIPT.md`). Those files now exist and pass; this closer seals Day 6 for real. The chain consequences of sealing out of order are handled honestly in **Chain** below — nothing is back-dated and no committed record is rewritten.

---

## The day's thesis, proven in code

**The substrate — proposer + planner + admission gate + receipt chain — is domain-independent. Only the ontology and the *authored* objective function change.** Revenue and church operations are now the *same generic pipeline* instantiated at two `Pack`s. The proof is structural, not asserted: the church proposer is literally `praxis_proposer::engine::Proposer<ChurchDomain>` — zero new proposer/scorer/ranker/hasher code — and `tests/two_domains.rs` runs one loop body over both packs.

This is grounded in real service (ZOE Church welcome team). The discretization — turning "no one who came for help gets lost" into evidence flags and stages — is operational, to make the care auditable. It is not a reduction of the spiritual to numbers.

---

## What landed

### 1. The church-operations domain pack — mirrors revenue exactly

Structural parallelism *is* the proof, so the church pack mirrors the revenue pack file-for-file:

| Concern | Revenue | Church |
|---|---|---|
| Ontology (pddl: vocabulary) | `ontology/revenue.ttl` | `ontology/church.ttl` — `Person` states `FirstTime → Returning → Connected → Serving → Leading` (ordered stage enum), evidence flags `welcomed`, `followed_up`, `in_small_group`, `care_assigned` |
| PDDL domain (PDDL8-safe, positive-precondition) | `ontology/revenue.pddl` | `ontology/church.pddl` — evidence-gated stage-advance actions (`advance-to-connected` requires `welcomed ∧ followed_up`, etc.) |
| Authored objective | `revenue_objective.json` | `crates/praxis-proposer/church_objective.json` |
| Proposer | `engine::Proposer<RevenueDomain>` | `engine::Proposer<ChurchDomain>` |

The authored church objective (`zoe-welcome-team-connection`, weights are **domain-authored data**, the algebra is the shared one):

```json
{ "people_connected": 1000.0, "care_completion_rate": 250.0,
  "volunteer_capacity_used": -50.0, "first_time_followup_within_48h": 500.0 }
```

The lawfulness pre-filter and the admission gate use the **same** evidence flags: a visitor missing `followed_up` is never proposed past `Returning` **and** would be refused by `admit` if forced — the identical mechanism revenue uses for `legal_approved`. Asserted in `crates/praxis-proposer/tests/church_proposer_tests.rs` (**25 tests, green**).

### 2. The Domain-trait refactor — needed, and revenue still passes

Revenue *was* hardcoded. The refactor introduced a domain-independent engine and made both domains implement one trait:

- `crates/praxis-proposer/src/engine.rs` (new) — the `Domain` trait plus generic `Proposal<D>`, `Proposer<D>`, the single shared `score()` (weighted sum in fixed fluent order + cited rationale), the ranking comparator, and blake3 canonical hashing. Written once, reused by every pack.
- `objective.rs` — the `deny_unknown_fields` loader + finite-weight rule + weighted algebra is shared verbatim; only the allowed fluent vocabulary differs (`validate_fluents` / `from_json_str_for`). Revenue's `score()` now routes through `engine::score`, byte-identical output — so revenue's Day-2 determinism anchors are preserved.

### 3. The generic mission substrate — one mission language above all packs

- `src/mission.rs` (759 lines) — the generic substrate:
  - **`Pack` trait** (`src/mission.rs:66`) — extends `engine::Domain` with the planning + admission surface a pipeline needs: `pddl_domain_text`, `build_problem`, `stage_required_evidence`, `entity_evidence`, `evidence_permits`, `ceiling_fluents`. Implementing it is the *entire* cost of a new institution.
  - **`run_pipeline<P: Pack>`** (`src/mission.rs:258`) — one generic function whose body names no institution: `Proposer::<P>::propose` → top goal → `bcinr_pddl` `plan solve` → the shared `ops::judge_payload`/`ops::admit_payload` gate → `ops::receipt_payload` chain binding `proposal_hash`.
  - **`ceiling<P: Pack>`** (`src/mission.rs:411`) — Maximum Reachable objective, MRR generalized to church (max reachable `people_connected` / care under the evidence gates).
  - `impl Pack for RevenueDomain` and `impl Pack for ChurchDomain`.
- `src/verbs/mission.rs` — the `mission` verb generalizing RevTAC to any pack: `mission run --pack <revenue|church> --objective <path> --state <path>` and `mission ceiling --pack <p> --state <path>`. The `match` on `--pack` is the *only* institution-specific line; everything below it is pack-independent.
- `docs/MISSION_PHYSICS.md` — the mission language documented with a revenue example and a church example side by side, identical in structure.

### 4. Two institutions, one substrate — the integration proof

`tests/two_domains.rs` runs the **identical pipeline code path** over both packs. Verified by running the compiled test binary directly at receipt time (see the external-breakage note below for why direct):

```
running 3 tests
test church_ceiling_respects_evidence_gates ... ok
test revenue_ceiling_equals_bespoke_mrr ... ok
test two_institutions_one_substrate ... ok
test result: ok. 3 passed; 0 failed; 0 ignored
```

- `two_institutions_one_substrate` — the same substrate functions serve both packs; only ontology + objective + state differ; both produce valid receipt chains; both enforce their evidence gate via the same admission mechanism. This test *is* the domain-independence proof — one loop, two packs.
- `revenue_ceiling_equals_bespoke_mrr` — the generalized `ceiling<RevenueDomain>` reproduces Day 2's bespoke MRR, so the generalization did not change revenue's numbers.
- `church_ceiling_respects_evidence_gates` — a visitor's contribution to the church ceiling is zero unless its evidence gate is satisfied — the same lawful-ceiling semantics as MRR.

---

## Refusals / gaps (receipted, not papered over)

1. **No `MANIFEST_DAY_5.json` to chain from — recorded, not fabricated.** The Day-6 workflow instructed `prev = MANIFEST_DAY_5.json hash`. That manifest does not exist: Days 3, 4, 5 did real work but sealed no manifest (see `GENESIS_SEAL.json` and `GENESIS.md`). Per the standing rule that *silent gaps are the only forbidden artifact*, this manifest chains the last **genuinely sealed** link — Day 2, `cb184872…` — and records the Day 3–5 gap explicitly in its `chain_note`, rather than inventing three absent hashes. This mirrors the seal's own methodology ("binds the links that genuinely exist, not a fabricated seven").

2. **Sealed out of order, after the Day-7 seal — not hidden.** `GENESIS_SEAL.json` (a committed Day-7 artifact, `seal_hash 9c666317…`) recorded Day 6 as *unsealed* and covered only the two contiguous links 1→2. This receipt does **not** rewrite that seal to retroactively claim it committed Day 6 — that would misrepresent what the Day-7 seal actually covered. Day 6 now has a genuine manifest chaining Day 2; the canonical week-seal remains the true Day-7 record. Re-casting the week-seal to fold in Day 6 is a follow-up for whoever next runs a seal over a quiescent tree.

3. **Non-quiescent tree; two external breakages observed and receipted (neither is Day-6 work).** This closer touched only additive docs + JSON (`DAY_6_RECEIPT.md`, `MANIFEST_DAY_6.json`, `GENESIS.md`). During the run, concurrent agents were live-editing the tree (tasks #65 `in_progress` at receipt time; #54, #64 flipped to `completed` mid-run):
   - **`src/verbs/doctor.rs:380` type mismatch** (task #54, mid-edit) briefly broke the root bin between a green `cargo build --workspace --all-features` and the test sweep. It was fixed by its own agent during this run. Not a Day-6 file; the `two_domains` **test binary itself compiled** throughout (verified by running it directly).
   - **`praxis-core receipt_validator::tests::tampered_payload_hash_fails_chain_recompute`** failed once under the parallel `--all-features` run, then **passed in isolation** and the **whole `praxis-core` lib passed 64/64 single-threaded**. This is the same test-isolation defect class the Day-7 receipt documented: `law-signed` (pulled by `--all-features`) tests race on the process-global `PRAXIS_SIGNING_KEY` env var under parallel execution. It is a sibling test-isolation defect, not a logic regression, and touches no Day-6 surface. Recommended fix (unchanged from Day 7): thread the signing key through the payload call instead of mutating a global env var, or make every signing-dependent test hold the shared guard.

---

## Test summary

- **Day-6 owned surfaces — all green** (run at receipt time):
  - `praxis-proposer` (church pack + engine + objective): **25 + 13 + 9 passed, 0 failed**.
  - `tests/two_domains.rs`: **3 passed, 0 failed** (the two-institutions proof).
- **`praxis-core` lib, single-threaded:** **64 passed, 0 failed.**
- **Full `cargo test --workspace --all-features`:** every test binary green except the one parallel-race flake above (which passes in isolation). No Day-6 surface fails under any run.
- **`cargo build --workspace --all-features`:** exit 0 (green) at the start of this closer's run.

Representative green tail (full-sweep):
```
test result: ok. 127 passed; 0 failed; 0 ignored   (my-conforming-project lib, single-threaded siblings)
test result: ok.  57 passed; 0 failed; 0 ignored
test result: ok.  25 passed; 0 failed; 0 ignored    (church_proposer_tests)
test result: ok.   3 passed; 0 failed; 0 ignored    (two_domains)
```

---

## Chain

Manifest algorithm (matches Days 1, 2, 7 exactly): `manifest_hash = blake3(json.dumps(obj, sort_keys=True, separators=(",",":")))` with the `manifest_hash` field removed. **Verified reproducible at receipt time:** recomputing over `MANIFEST_DAY_2.json` reproduces `cb184872…` (the same script that produced this Day-6 hash), confirming the method is the established one. Constellation = the same 11 repos Days 1–2 recorded (praxis, wasm4pm-compat, wasm4pm, bcinr, star-toml, cargo-cicd, ggen, stpnt, affidavit, lsp-max, semantic_bit), each with HEAD, branch, dirty-file count, and crate versions re-scanned live (8 of 11 sibling HEADs had advanced since Day 2, so versions were re-derived, not carried forward).

- **prev (last genuinely sealed link — Day 2)** `manifest_hash` = `cb184872b7c8bd5030b524a5430bd44d208db5b151f6f21066514293dea4e8c9`
  (`docs/genesis/MANIFEST_DAY_2.json`). Days 3–5 sealed no manifest; that gap is recorded in the Day-6 manifest's `chain_note`.
- **this day (Day 6)** `manifest_hash` = `9f976d7e108ecfa8cf98b9f1ec4607e17f4d44142beb4c81488c809ac505e9ca`
  (`docs/genesis/MANIFEST_DAY_6.json`; `prev_day_hash =` the Day-2 hash above).

Day-6 `praxis` HEAD recorded in the manifest: `dd09ee61422a260ecc0cbdc3f2126eacfd54adf9` (the HEAD immediately before this receipt commit).

---

## What Day 7 inherits

> Day 7 already ran (`DAY_7_RECEIPT.md`, `GENESIS_SEAL.json`). What follows is what a **re-seal / release pass over a quiescent tree** now inherits from a genuinely-sealed Day 6.

- **A third genuine, independently-reproducible manifest** (`9f976d7e…`) chaining Day 2 — so the week now has three sealed links (1→2, and 6→2 out of order), where before it had two. A future `GENESIS_SEAL` recomputation can fold Day 6 in honestly; the current `seal_hash 9c666317…` remains the true Day-7-state record and should not be silently overwritten.
- **The domain-independence thesis proven, not asserted:** `tests/two_domains.rs` (one loop, two packs) and `docs/MISSION_PHYSICS.md` — the two artifacts the Day-7 receipt listed as "still pending" — now exist and are green. The Day-7 honesty table's Day-6 row ("tests/two_domains.rs + docs/MISSION_PHYSICS.md still pending") is now superseded by this receipt.
- **A generic `Pack` substrate** (`src/mission.rs`): adding a third institution costs exactly one `impl Pack` — no new proposer, scorer, ranker, hasher, or admission code.
- **Two live external debts to close before any irreversible release action:** (1) the `PRAXIS_SIGNING_KEY` parallel-test race in `praxis-core` / sibling receipt tests; (2) tree quiescence — push, tag `v26.7.2`, and `cargo publish` remain correctly refused until the tree is committed and green, exactly as Day 7 recorded.
