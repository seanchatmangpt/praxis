# Genesis Day 4 Receipt — The Membrane Ships

**Date:** 2026-07-02 (Day 4 of the seven-day program)
**Program:** GENESIS Day 4 — productize the MCP membrane so a second agent (any MCP client) drives the full Day-2 revenue pipe through the membrane *alone* — no repo access, no CLI — proving agents build under law across a wire. Plus `agent8`: an 8-bit agent projection and the 64-byte wire ABI that projection rides on, with a branchless fleet kernel.
**Principle:** build beyond human reading, within human verification. No claim below exceeds a mechanism you can re-run.

> This receipt is written **out of order**, after Days 5, 6, and 7 were already receipted and after the Day-7 `GENESIS_SEAL.json` was cast. That is unusual and is stated plainly rather than hidden. Day 4's work (the `agent8` crate + the productized MCP membrane + the external-agent demo) genuinely landed in the `praxis` history, but at Day 7 it had sealed no manifest of its own (see `GENESIS.md` Day 4 row and `GENESIS_SEAL.json`, which record it as *unsealed*). This closer seals Day 4 for real. The chain consequences of sealing out of order are handled honestly in **Chain** below — nothing is back-dated and no committed record is rewritten.

---

## The day's thesis, proven in code

**An external agent with only membrane access can complete a receipted revenue mission.** The proof is not asserted — it is a script that speaks raw JSON-RPC to the shipped server over stdio, exactly as any MCP client would, and drives the complete Genesis Day-2 pipe end to end:

```
initialize → tools/list → propose_revenue → propose_goal → plan_solve
           → judge → admit → receipt → whoami (+ fleet_status)
```

No repo path, no CLI verb, no reaching around the membrane. Every response is asserted; the run ends by printing the receipt `chain_hash` and the session's resident `AgentByte`, which must carry `RECEIPTED` (`0x40`). Harness: `scripts/membrane_demo.sh` (a small Python driver used only for robust subprocess/JSON framing).

---

## What landed

### 1. `agent8` — 8-bit agent projection + 64-byte wire ABI + fleet kernel

Crate at `crates/agent8/` (workspace member; `#![forbid(unsafe_code)]`, `missing_docs = warn`), three layers smallest-to-largest:

| Layer | File (LoC) | What |
|---|---|---|
| Projection | `src/byte.rs` (241) | `AgentByte` — a `#[repr(transparent)] u8` newtype projecting one agent's governance posture into eight named bits, with a const `GRANT_REQUIRED` mask, and const `with`/`without`/`carries`/`select`. `AgentSelect { Grant, Deny }`. Hand-authored sibling of the *generated* `Status8Field` in `semantic_bit` (prior art cited inline). |
| Wire ABI | `src/abi.rs` (378) | `Env64` + `Pulse64` — `#[repr(C, align(64))]` ports of the bytestar ABI (`env64_t` / `pulse64_t`), **each exactly 64 bytes, compile-time asserted**. `pulse64_from_receipt_record` bridges a praxis `ReceiptRecord` onto the wire. The `Env64` pattern byte *is* the `AgentByte` wire slot. |
| Fleet | `src/fleet.rs` (273) | SWAR kernel packing 8 agents per word; `sweep_admit` (ported `unibit` admit/denial-polarity primitive — **zero = admitted, nonzero = denied**) + popcount `FleetStats`. |

Bit vocabulary (`byte.rs`): `ADMITTED 0x01`, `EVIDENCE_OK 0x02`, `WITHIN_BUDGET 0x04`, `AUTHORITY_BOUND 0x08`, `HEALTHY 0x10`, `CONFORMANT 0x20`, `RECEIPTED 0x40`, `REPLAYABLE 0x80`. `GRANT_REQUIRED = 0x6F` — the six *load-bearing governance* bits; `HEALTHY` (operational) and `REPLAYABLE` (post-hoc) are deliberately not demanded before acting. `src/lib.rs` (37) re-exports the surface.

**Tests (re-run at receipt time): 15 unit + 4 integration + 3 doc = 22, all pass.** `cargo clippy -p agent8 --all-targets` clean. `cargo build --workspace` exit 0 (2m16s cold).

The integration suite (`tests/integration.rs`) carries the two load-bearing invariants:
- `abi_structs_are_cache_line_sized` — `Env64`/`Pulse64` are each 64 bytes.
- `fleet_sweep_matches_naive_loop_differential` — the SWAR kernel is byte-equal to a naive per-agent oracle across a pseudo-random fleet (differential correctness, not just self-consistency).

### 2. Fleet sweep benchmark (measured live at receipt time, release)

```
agent8 fleet sweep: 10000000 agents (1250000 words) in 2.242 ms
  (admitted=156504, blocked=9843496, receipted=4999687, replayable=5000859)
throughput: 4.46 G agents/s
```

Harness: `cargo bench -p agent8` (`benches/fleet_sweep.rs`, `harness = false`). **Honesty note:** the original Day-4 build report claimed `1.024 ms / 9.76 G agents/s`. That number is not reproduced here — this receipt records what I actually measured on this machine at close time (`2.242 ms / 4.46 G agents/s`). Both are real numbers from the same harness on different hardware/thermal states; the receipt reports the run it can attest to, not the one it inherited. The debug-profile run (via `cargo test --all-targets`) measured `13.403 ms / 0.75 G/s`, consistent with an unoptimized build.

### 3. The membrane, productized — 13 tools, single-source with the CLI (AR-2)

`src/bin/mcp_lawobject_server.rs` exposes **13 tools**, covering the entire revenue demo drivable through MCP alone (verified live by `tools/list`):

```
admit, fleet_status, inspect_obligation, judge, plan_solve, promote,
propose_goal, propose_revenue, receipt, receipt_replay, receipt_validate,
show_andon, whoami
```

Zero drift: the Day-2 logic lives once in `my_conforming_project::ops` (`plan_solve_payload`, `propose_revenue_payload`, `propose_goal_payload`, and the shared PDDL/JSON helpers). The MCP tools and the CLI verbs (`src/verbs/plan.rs`, `src/verbs/propose.rs`) are thin call-throughs to that one implementation. Tool results are cached (`src/mcp_cache.rs`, `ToolResultCache`).

### 4. Session `AgentByte` adapter — `agent8` wired behind the membrane

`ServerState` holds a resident `AgentByte`. Each `judge`/`admit`/`receipt` outcome folds forward into it (validated → `CONFORMANT|EVIDENCE_OK`; admitted → `ADMITTED|WITHIN_BUDGET|AUTHORITY_BOUND`; receipted → `RECEIPTED|REPLAYABLE`), on both cache-hit and fresh paths. `whoami` reports the byte / flags / `select` / `missing_for_grant`; `fleet_status` runs the `agent8` SWAR popcount kernel over a supplied fleet.

---

## The proof artifact — external-agent membrane transcript

Run: `./scripts/membrane_demo.sh` (built `--features mcp,proposer`; server `rmcp 0.11.0`). Every line is a live assertion that passed:

```
[ok] initialize            → server rmcp 0.11.0
[ok] tools/list            → 13 tools; pipe coverage complete
     tools: admit, fleet_status, inspect_obligation, judge, plan_solve, promote,
            propose_goal, propose_revenue, receipt, receipt_replay,
            receipt_validate, show_andon, whoami
[ok] propose_revenue       → 4 ranked proposals (observation, not authority)
[ok] propose_goal          → goal (stage acct-1 closed-won)  (proposal_hash 6966f539b7a64923…)
[ok] plan_solve            → plan_len 1 (goal reachable in shipped domain)
[ok] judge                 → verdict validated
[ok] admit                 → status admitted
[ok] receipt               → status receipted; chain_hash 831ae41cb38995a91af5c6d44e2602a2a3bd0a76a51bc2818eac31cbbe629cf6
[ok] whoami                → byte 0xff flags PRCHUBEA select Grant (RECEIPTED set)
[ok] fleet_status          → 2/4 admitted, 3 receipted (SWAR popcount kernel)
```

Final proof object emitted by the demo:

```json
{
  "mission": "revenue-physics-day2",
  "goal": "(stage acct-1 closed-won)",
  "proposal_hash": "6966f539b7a649230bf891d2fe9f61fdcd0beb2e0eed99d8e1abb1a370465dc9",
  "plan_len": 1,
  "chain_hash": "831ae41cb38995a91af5c6d44e2602a2a3bd0a76a51bc2818eac31cbbe629cf6",
  "final_agent_byte_hex": "0xff",
  "final_agent_flags": "PRCHUBEA",
  "final_select": "Grant",
  "fleet_stats": { "total": 4, "admitted": 2, "blocked": 2, "receipted": 3, "replayable": 1 }
}
```

**What this proves.** An MCP client that never touched the repo (1) observed a revenue snapshot and got 4 *ranked* lawful proposals — proposals are observations, not authority (AR-9); (2) lifted the top-ranked proposal to a goal atom bound by `proposal_hash 6966f539…`; (3) spliced that goal into the shipped `ontology/revenue.pddl` and found a plan (`plan_len 1`); (4) passed the goal through `judge` (verdict `validated`) and the evidence-gated `admit` (status `admitted`); (5) obtained a BLAKE3 receipt `chain_hash 831ae41c…` binding the proposal; and (6) watched the session's resident `AgentByte` reach `0xff` (`PRCHUBEA` — RECEIPTED set), which `select`s to `Grant`. The timestamp is pinned (`DEMO_TS_NS`) so the `chain_hash` is stable across runs — the transcript is reproducible, not a one-shot.

---

## Refusals / gaps (receipted, not papered over)

1. **No `MANIFEST_DAY_3.json` to chain from — recorded, not fabricated.** The Day-4 closer was instructed `prev = MANIFEST_DAY_3.json hash`. That manifest does not exist: Day 3 did real work (fuzz + mutation suites — `crates/praxis-core/tests/{fuzz_boundaries,mutation_chain}.rs`, task #49 still `in_progress`) but sealed no manifest. Per the standing rule that *silent gaps are the only forbidden artifact*, this manifest chains the last **genuinely sealed** link — Day 2, `cb184872…` — and records the Day-3 gap explicitly in its `chain_note`, rather than inventing an absent hash. This mirrors Day 5 and Day 6, which also chain Day 2. The blake3 method was confirmed by recomputing `MANIFEST_DAY_2.json` and reproducing `cb184872…` before generating this day's hash.

2. **Sealed out of order, after the Day-7 seal — not hidden.** `GENESIS_SEAL.json` (committed Day-7 artifact, `seal_hash 9c666317…`) recorded Day 4 as *unsealed* and covered only the two contiguous links 1→2. This receipt does **not** rewrite that seal to retroactively claim it committed Day 4. Day 4 now has a genuine manifest chaining Day 2; the canonical week-seal remains the true Day-7 record. Re-casting the week-seal to fold in Days 4 and 6 is a follow-up for whoever next seals over a quiescent tree.

3. **Bench number not reproduced from the build report.** See §2 — the report's `1.024 ms / 9.76 G/s` was not reproduced; this receipt records the measured `2.242 ms / 4.46 G/s`. No claim rests on the faster number.

4. **One clippy style warning on the MCP server bin (non-blocking).** `src/bin/mcp_lawobject_server.rs:340` — `pub struct FleetStatusParams` draws a `clippy::redundant_pub_crate`-style visibility suggestion (`consider pub(crate)`). It is a lint *warning*, not an error; `cargo build --workspace` exits 0 and the demo runs. Left as a Day-5 cleanup rather than silently churned in a seal commit.

5. **Non-quiescent tree; a known sibling test-isolation flake persists.** `praxis` has 115 dirty files at close (concurrent agents live-editing). The `praxis-core` `law-signed` receipt-validator tests race on the process-global `PRAXIS_SIGNING_KEY` env var under parallel `--all-features` execution — the same defect class Days 6 and 7 documented; it passes in isolation / single-threaded. Not a Day-4 surface and not a logic regression. Push / tag / publish therefore remain correctly out of scope for this additive closer.

---

## Test summary

- **`agent8` (the day's owned crate):** 15 unit + 4 integration + 3 doc = **22 passed, 0 failed**; `cargo clippy -p agent8 --all-targets` clean.
- **Membrane demo (`scripts/membrane_demo.sh`):** every JSON-RPC assertion passed; receipted mission completed through the wire alone.
- **`cargo build --workspace`:** exit 0 (green).

Representative green tail:
```
test result: ok. 15 passed; 0 failed; 0 ignored   (agent8 unit)
test result: ok.  4 passed; 0 failed; 0 ignored   (agent8 integration.rs — incl. cache-line + differential)
test result: ok.  3 passed; 0 failed; 0 ignored   (agent8 doctests)
```

---

## Chain

Manifest algorithm (matches Days 1, 2, 5, 6 exactly): `manifest_hash = blake3(json.dumps(obj, sort_keys=True, separators=(",",":")))` with the `manifest_hash` field removed. **Verified reproducible at receipt time:** recomputing over `MANIFEST_DAY_2.json` reproduces `cb184872…` (the same script that produced this Day-4 hash), and reloading the written `MANIFEST_DAY_4.json` reproduces its own stated hash. Constellation = the same 11 repos Days 1–2 recorded; every sibling HEAD was re-scanned live and **verified identical to the Day-6 record** (only `praxis` advanced: `dd09ee6…` → this HEAD), so their versions carry accurately; `praxis` crate versions were re-derived live from `[package]` sections.

- **prev (last genuinely sealed link — Day 2)** `manifest_hash` = `cb184872b7c8bd5030b524a5430bd44d208db5b151f6f21066514293dea4e8c9`
  (`docs/genesis/MANIFEST_DAY_2.json`). Day 3 sealed no manifest; that gap is recorded in the Day-4 manifest's `chain_note`.
- **this day (Day 4)** `manifest_hash` = `0745965ad3f4b0bf071cff45987e6350eac50bec51a466923bb5c847df0f019d`
  (`docs/genesis/MANIFEST_DAY_4.json`; `prev_day_hash =` the Day-2 hash above).

Day-4 `praxis` HEAD recorded in the manifest: `0903989cd91864a2472ef304282c28407de90a26` (the HEAD immediately before this receipt commit).

---

## What Day 5 inherits

> Days 5, 6, 7 already ran. What follows is what a **re-seal / release pass over a quiescent tree** now inherits from a genuinely-sealed Day 4.

- **A fourth genuine, independently-reproducible manifest** (`0745965a…`) chaining Day 2 — the week now has, in sealed form, links 1→2 (contiguous) plus 4→2 and 6→2 (out-of-order siblings). A future `GENESIS_SEAL` recomputation can fold Days 4 and 6 in honestly; the current `seal_hash 9c666317…` remains the true Day-7-state record and must not be silently overwritten.
- **The membrane thesis proven, not asserted:** `scripts/membrane_demo.sh` — the external-agent transcript the Day-4 row listed as missing — now exists, is reproducible (pinned timestamp → stable `chain_hash 831ae41c…`), and drives the full Day-2 pipe through MCP alone. The `GENESIS.md` Day-4 row is updated from "⚠️ work landed, not sealed" to sealed by this closer.
- **`agent8` as a reusable substrate:** an 8-bit governance projection + a 64-byte wire ABI (`Env64`/`Pulse64`, cache-line exact, bridged from `ReceiptRecord`) + a branchless fleet kernel with a differential-verified sweep. The session `AgentByte` behind the membrane is the first consumer; any fleet-scale admission surface can reuse it.
- **Four debts to close before any irreversible release action:** (1) `MANIFEST_DAY_3.json` — Day 3's fuzz/mutation work is still unsealed (task #49 `in_progress`); (2) the `PRAXIS_SIGNING_KEY` parallel-test race in `praxis-core` / sibling receipt tests; (3) the `FleetStatusParams` visibility lint on the MCP bin; (4) tree quiescence — push, tag, and `cargo publish` remain correctly refused until the tree is committed and green.
