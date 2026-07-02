# The Skeptic's Walkthrough

**Run it:** `./scripts/walkthrough.sh` from the repository root.
**What it is:** Vision 2030 Release Criterion 5 in executable form — one scripted pass from a domain ontology to a replay-verified, tamper-evident, signed receipt chain, with every assertion checked by the script itself. If any claim below is not true of the running system, the script exits nonzero and names the criterion that failed. Exit code 2 means part of the target verb surface has not landed yet; the script names exactly which verb and fakes nothing.

This document is the narrative companion: what each step proves, **what it deliberately does not prove**, and how to read the artifacts. The calibration rule comes from the PRD success metrics: *claims in docs exceeding mechanism are defects, with the same severity as a failing test.* Hold this document to that rule.

---

## The three disclaimers, up front (PRD §4, binding non-goals)

1. **Integrity, not virtue.** A perfect receipt chain over wrong obligations proves nothing about whether the policy was *good*. The walkthrough proves that authored obligations were enforced and that the record of enforcement survives a hostile reviewer. It does not prove the obligations were the right ones. (PRD risk #3: "receipt theater" — we name it so you don't have to discover it.)
2. **Software-binding, not physics-binding.** Admission gates constrain code that goes *through the membrane*. Code that bypasses the praxis boundary — a direct database write, a shell command, an API call from elsewhere — is out of scope, and no artifact here claims otherwise. No claim of the form "this prevents X" where X can route around the process boundary is made or implied.
3. **No value discovery.** Every obligation, policy, and goal in this walkthrough was authored by a human and is visible in the script source. The system decided whether transitions were *lawful*, never whether they were *worth doing*. There is no LLM anywhere in the admission, receipting, or replay path — those are deterministic code (PRD non-goal 3), which is precisely why Step 1's determinism check and Step 3's hash-stability check can be byte-exact.

---

## Step-by-step: the claim, the evidence, the limit

### Step 0 — Preflight (Criterion 2: every promised noun exists)

**Proves:** the binary exists and every verb the demo calls (`law judge|admit|receipt|verify-signature`, `mfg pddl`, `plan solve|lawobject`, `receipt issue|validate|replay`, `config show|witness`, `dod matrix`, `doctor`) is really registered — probed via `--help`, not assumed. A throwaway ed25519 signing seed is generated into `PRAXIS_SIGNING_KEY`, the documented env mechanism.

**Does not prove:** anything about behavior — that is what the remaining steps are for. The throwaway key proves signature *mechanics*, not key custody or key-management practice; in production the seed would live in a secret store, and nothing here demonstrates that.

**Degradation rule:** this is the only step allowed to exit gracefully. A missing verb → exit 2 with the verb's exact name and a "not yet built" note (feature-gated lanes: `mfg` needs `--features ggen`, signing needs `--features law-signed`). Preflight also detects a *shadowed* verb: under `lsp`/`andon`, a dependency (affidavit, via lsp-max) registers its own `receipt replay` into the shared CLI registry, and last-write-wins can hide praxis's ledger replay — that is receipted as not-built rather than letting Step 5 exercise the wrong implementation. After Step 0, every step must pass or the script fails loudly.

### Step 1 — Manufacture: ontology → PDDL (PR-8)

**Proves:** `ontology/lawobject.ttl` (Turtle) projects to PDDL domain text by deterministic code — ordered SPARQL over the graph, a bounds-enforced STRIPS8 intermediate form, direct Rust emission. The script emits **twice** and byte-compares: identical output or fail. So the planning input's chain of custody starts at the ontology, not at a hand-edited text file.

**Does not prove:** that the ontology itself is correct or complete — it is an authored artifact, exactly like a policy. Manufacturing faithfully projects what a human wrote; garbage ontology in, faithfully-manufactured garbage out.

**How to read it:** the emitted `(define (domain ...))` text is shown in full in the transcript. The PDDL8 bounds (arity ≤ 8, etc.) are enforced in the emitter's Rust code, not by a template — that is why emission is trustworthy as a *law-bound* artifact (ADR AR-6).

### Step 2 — Plan: a hash-committed action sequence (PR-7)

**Proves:** a bounded, deterministic forward-search planner produces the lifecycle action sequence (supply-evidence → clear-obligations → judge → admit → receipt) over the manufactured domain, and the plan is committed under a BLAKE3 plan chain. The plan a reviewer reads is provably the plan that was computed. Had the goal been infeasible, the output would be a structured refusal (`admitted: false`) — a lawful, receiptable outcome, not a crash.

**Does not prove:** optimality. The planner is deliberately BFS/greedy with bounded depth (ADR AR-5): determinism and refusal-on-infeasible were chosen over optimal plans. It also does not prove the plan was *executed* — that is Step 3.

**How to read it:** `plan_chain` is a BLAKE3 hash over the plan's frame sequence. Two runs over the same domain/problem produce the same chain; any change to any action changes it.

### Step 3 — Execute through the gate: refusal, admission, signed receipt (PR-1, PR-4, PR-6)

This step runs the same pipeline twice, and the contrast is the point.

**3a proves (refusal as data):** a payload carrying an unmet obligation (a blocking constraint: "dual-control approval absent") is **halted**. The refusal is structured JSON: a verdict, an Andon state, and the exact unmet obligations. It is an output of the system working correctly — not an exception, not a log line. A reviewer can diff two refusals; you cannot diff a stack trace.

**3b/3c prove (admission and receipt):** the same action with its evidence obligation met passes Raw → Validated → Admitted, and receipting yields a BLAKE3 `chain_hash` that commits to the payload hash, the previous chain hash, and the caller-supplied metadata frame (instruction id, activity index, timestamp). The script fixes `ts_ns=42` and receipts **twice**: identical hash or fail. The receipt is signed ed25519 under `PRAXIS_SIGNING_KEY`, and `law verify-signature` re-verifies it — the signature is self-contained, so a third party can check it without out-of-band key exchange. Signing is fail-closed: with the key set, an unsigned receipt is a script failure, not a shrug.

**Does not prove:** that anything happened in the outside world. "Execution" here means the payload lawfully traversed the typestate lifecycle. The receipt binds *what was admitted and when*, not any external side effect — that binding is exactly the software boundary of non-goal 2. It also does not prove the compile-time typestate guarantee itself (receipting an un-admitted object is a *compile* error; a shell script cannot exhibit a program that doesn't compile — see `crates/praxis-core` and its tests for that claim).

**How to read a chain hash:** `chain_hash = BLAKE3(prev_chain_hash ‖ fixed-layout frame)`, where the frame carries the payload's own BLAKE3 hash plus the metadata. So the hash binds: this payload, after that predecessor, under this instruction/activity/timestamp. Change any of them — one byte — and the hash changes. It does **not** bind: who ran it (that's the signature's job), or whether the payload's claims about the world are true.

**How to read a refusal JSON:** `verdict: "halted"` + `unmet: [...]` names each obligation that blocked admission, by kind. A refusal means "the gate held"; it is the artifact you *want* to see when policy is violated.

### Step 4 — Tamper detection: one flipped byte (PR-5, PR-3)

**Proves:** the persisted store (`receipts/receipts.jsonl`, append-only) is tamper-evident. The script issues a chained 3-record trace (strictly increasing `instruction_id`s — the monotonic stage rejects reuse), validates it (passes), then flips exactly one hex digit of record 1's `payload_hash_hex` in a *copy* and validates again. (The persisted record carries the payload's *hash*, not the raw payload — so the attack mutates the hash commitment itself, the strongest single-byte case.) The mutated copy must be rejected **at the chain-integrity stage** (`chain_recompute`) — the script greps for the chain/hash stage in the verdict, so a rejection for the wrong reason (schema noise) does not count as a pass. The untouched original continues to validate in the same run, ruling out "the validator just rejects everything."

**Does not prove:** deletion-resistance of the *whole file*. If an attacker deletes the entire store and its copies, there is no chain left to fail — tamper-evidence covers mutation and truncation of records within a chain, and continuity between records. Distribution/anchoring of chain heads is future work (PR-16), not a current claim.

**Why the emit/recompute can't drift:** emission and validation share one frame-construction function (ADR AR-3). If they were two implementations, a "validator" could pass forever while checking the wrong thing.

### Step 5 — Replay: conformance against receipted intent (PR-5)

**Proves:** two complementary order guarantees, by two mechanisms. (5a) Each record's lifecycle token-replays against the POWL model (a strict judge → admit → receipt sequence) and the lawful trace scores **conformance fitness 1.0 per record**. (5b) *Inter-record* order is enforced by the ledger's validation stages: the same records in reversed order are **rejected at chain-linkage/monotonicity** — each record's `prev_chain_hash` must be the previous record's `chain_hash`, and instruction ids must strictly increase, so a reordered store cannot validate. Chain integrity (Step 4) proves no record was altered; replay + linkage prove the *order* of what happened matches receipted intent. These are different failures and both are caught.

**Does not prove:** conformance to any richer process than the one modeled. The replay model here is the 3-node lifecycle itself; fitness 1.0 means "each record's trace fits this model," not "the business process was followed" for any process not encoded in the model. Note the division of labor honestly: per-record replay does not itself detect a reordered *store* — that is deliberately the linkage stages' job, and 5b asserts it there.

**How to read fitness:** token-replay fitness is the proportion of a record's trace the model can consume without violation — 1.0 means every event fired a token the model had enabled; anything less means at least one event happened out of turn.

### Step 6 — The frontier matrix (Criterion 3, PR-11)

**Proves:** `dod matrix` produces `target/frontier-report.json` with `pass_rate: 1.0` over the capability × socket coverage matrix — where **refusals count as passes when expected**. Every integration that was considered and rejected (stpnt: no license; clnrm-core: dependency footprint; affidavit: incompatible chain rule; ...) is a receipted cell with a stated reason. Silent omission of a decision is a defect class, and this artifact is how it's caught.

**Does not prove:** that the enumerated space is the whole universe of possible integrations — it is the space the project *declared*, receipted, and now cannot silently shrink.

### Step 7 — Doctor (PR-13)

**Proves:** one command reports holistic health — build state, config witness, frontier coverage, receipts store, tool availability. It closes the loop: if the walkthrough passed but doctor is unhealthy, believe doctor.

**Does not prove:** anything beyond what its checks encode; it is a dashboard over the same mechanisms, not an extra guarantee.

### Step 8 — The membrane: an external agent builds through MCP alone (PR-14, AR-2, AR-9)

**Run it:** `just membrane-demo` (or `./scripts/membrane_demo.sh`). CI form: `just membrane-test`.

**Proves:** the entire Genesis Day 2 revenue pipe is drivable by an external agent that has **only** the praxis MCP membrane — no repo access, no CLI, no in-process Rust. `scripts/membrane_demo.sh` spawns `mcp_lawobject_server` and speaks raw newline-delimited JSON-RPC over its stdin/stdout: `initialize → tools/list → propose_revenue → propose_goal → plan_solve → judge → admit → receipt → whoami`. Every response is asserted; the run ends by printing the receipt `chain_hash` and the session's resident `AgentByte`, which must carry `RECEIPTED` (0x40). With the mission's evidence obligation satisfied, the byte lands at `0xFF` (`PRCHUBEA`, `select: Grant`) — every governance bit set.

Two load-bearing properties back this:

- **One implementation, no drift (AR-2).** Each tool calls the exact same `my_conforming_project::ops::*` function the CLI verb calls — `plan_solve` → `ops::plan_solve_payload` (the CLI `plan solve`), `propose_revenue`/`propose_goal` → `ops::propose_{revenue,goal}_payload` (the CLI `propose revenue`/`goal`), `judge`/`admit`/`receipt` → `ops::{judge,admit,receipt}_payload`. There is no second copy of the pipe living behind the membrane that could pass while the real one fails.
- **The resident byte is an adapter, not a claim (agent8).** The session's `AgentByte` is folded forward by the *outcomes* of judge/admit/receipt (validated → `CONFORMANT|EVIDENCE_OK`; admitted → `ADMITTED|WITHIN_BUDGET|AUTHORITY_BOUND`; receipted → `RECEIPTED|REPLAYABLE`), so `whoami` reads back exactly what the membrane admitted. `fleet_status` runs the same 8-bit projection through the SWAR popcount kernel over a whole fleet.

**Does not prove:** that a proposal is *worth doing* (a proposal is observation O, never authority O* — AR-9; it still has to pass judge/admit like any raw input), nor anything about the world outside the boundary (same software-binding limit as Step 3). The resident `AgentByte` is a projection of *what this session admitted*, not an independent attestation — its authority is exactly the authority of the judge/admit/receipt calls that moved it.

**How to read the AgentByte:** `whoami` returns `{byte, flags, select, missing_for_grant}`. `flags` is 8 chars high→low `P R C H U B E A` (Replayable Receipted Conformant Healthy aUthority Budget Evidence Admitted); a `-` marks a clear bit. `select` is `Grant` iff the six governance bits (`GRANT_REQUIRED = 0x6F`) are all set — `HEALTHY` and `REPLAYABLE` are advisory and excluded. `RECEIPTED` set is the proof that this session receipted a mission through the membrane.

---

## Reading the artifacts: a field guide

| Artifact | What it binds / measures | What it does NOT claim |
|---|---|---|
| `chain_hash` (64 hex) | BLAKE3 over previous hash ‖ frame(payload hash, instruction id, activity, ts_ns). This payload, after that predecessor, under this metadata. | Truth of the payload's content; any external side effect; identity of the runner (see signature). |
| `signature` (ed25519) | The chain hash was signed by the holder of `PRAXIS_SIGNING_KEY`; verifiable self-contained. | Key custody quality; that the signer was authorized in any organizational sense. |
| refusal JSON (`verdict: halted`, `unmet: [...]`) | The gate held: these named obligations were unmet at judgment time. A refusal is a success of the mechanism. | That the obligations were correct or sufficient policy. |
| `receipts/receipts.jsonl` | Append-only persisted chain; any in-place mutation or reorder is detectable by recompute + replay. | Resistance to wholesale deletion; off-host durability (not yet built — PR-16). |
| replay `fitness` | Fraction of the trace consumable by the POWL lifecycle model without a token violation. 1.0 = executed order matches receipted intent. | Conformance to any process not encoded in the model. |
| `target/frontier-report.json` `pass_rate` | Every declared capability × socket cell is admitted-and-working or refused-with-reason. Expected refusals pass. | Completeness of the declared space itself. |
| session `AgentByte` (`whoami`, 8-bit `PRCHUBEA`) | What *this MCP session* admitted: bits set by its own judge/admit/receipt outcomes. `RECEIPTED` (0x40) ⇒ a mission was receipted through the membrane; `select: Grant` ⇒ all six governance bits (`0x6F`) set. | An independent attestation; authority beyond the judge/admit/receipt calls that moved it. Advisory `HEALTHY`/`REPLAYABLE` do not gate Grant. |

## If it fails

- **Exit 2 (preflight):** the named verb has not landed. This is the honest state of a system under construction — the script refuses to demo around a hole.
- **Exit 1 (a step):** the message names the release criterion that is now *not demonstrated*. That is a defect with the severity of a failing test; the transcript above the failure is the reproduction.
- Set `PRAXIS_WALKTHROUGH_KEEP=1` to keep the temp working directory (stores, tampered copies, PDDL, plan JSON) for inspection after the run.
