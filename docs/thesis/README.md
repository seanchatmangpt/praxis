# The Chatman Equation — Thesis Program

A reader's guide to the six-paper thesis series and the `praxis` codebase that
implements it. If you are new to this work, read this file first, then follow
[`ONBOARDING.md`](./ONBOARDING.md) for a hands-on 30-minute path through the code.

---

## What the equation says (in plain language)

Most software turns raw input straight into action: a log line, a model output,
or a sensor reading arrives and something happens. The Chatman Equation refuses
that shortcut. It says an actuated result **A** is the image, under a
deterministic manufacturing step **μ**, of an *admitted* observation — not a raw
one:

> **A = μ(O\*)**

Read it right to left. **O** is the raw observation space: arbitrary finite
records with no decidable meaning (logs, tool responses, human claims). **O\***
(written *Adm* in the papers) is the **admitted subspace** — the records that
have passed a finite, computable battery of obligations. **μ** is a bounded,
deterministic map from admitted observations to artifacts. Everything μ produces
leaves a **receipt**: a small, hash-committed record a human can verify without
re-running the work.

The one-line consequence, called *Conservation of Consequence*: nothing is
actuated without an admitted cause, and nothing is actuated without a receipt.
The earlier published form of the law was `A = μ(O)` (see *Prior art* below);
this series **factors** that μ into two stages — admission (`adm`) then
manufacture (μ) — so the gate that decides *what may enter at all* and the
receipt that *proves what happened* are first-class, not implicit.

Formal statement: `00_foundations.tex`, Eq. `eq:chatman` (the boxed identity)
and Def. `def:five` (the five objects O, O\*, μ, A, Rec).

---

## Read the papers in this order

All PDFs live in `/Users/sac/praxis/docs/thesis/`. The bound master
(`chatman_thesis_master.pdf`, 137pp) contains all six in this same order with a
generated table of contents and PDF bookmarks.

1. **Part 0 — Foundations**
   `/Users/sac/praxis/docs/thesis/00_foundations.pdf`
   Fixes the five objects and states the Bounded Receipted Chatman Equation
   (BRCE) as four invariants B1–B4: admission gate, bounded manufacture, receipt
   totality, conformance. Start here; every later paper cites its definitions.

2. **Part I — Admission Algebra**
   `/Users/sac/praxis/docs/thesis/01_admission_algebra.pdf`
   Develops the algebra of admission: refusal as a first-class value and the
   composition law that lets many obligation checks combine into one verdict.
   This is the theory behind "a refusal is an output, not an exception."

3. **Part II — Receipt Cryptography**
   `/Users/sac/praxis/docs/thesis/02_receipt_cryptography.pdf`
   Supplies the cryptography the keystone leaves "on credit": what a receipt
   commits to, why a single flipped byte is caught, and how chaining preserves
   order. This is where B3 (receipt totality) gets its teeth.

4. **Part III — Planning Geometry**
   `/Users/sac/praxis/docs/thesis/03_planning_geometry.pdf`
   Recasts conformance as geometry: the *marking polytope* from the Petri state
   equation `m = m0 + N·x`, and a Farkas separation certificate for a trace that
   leaves it. This is the semantics beneath the B4 conformance invariant.

5. **Part IV — Projection (Keystone)**
   `/Users/sac/praxis/docs/thesis/projection_thesis.pdf`
   The load-bearing paper. Proves the **Faithful Projection Theorem** (a bounded
   verifier reads O(1) receipt symbols yet the receipt is faithful to the
   interior under collision resistance) and the Comprehension–Verification Gap
   that makes trust cost linear across a trillion-agent fleet.

6. **Part V — Projection and Scale**
   `/Users/sac/praxis/docs/thesis/04_projection_and_scale.pdf`
   Pushes projection to fleet scale: the status-byte / lane view of an agent and
   what stays constant-cost as the number of agents grows. Read last; it assumes
   the keystone.

---

## Math-to-code map

Each core result and the `praxis` file/type that carries it. Paths are relative
to the repo root `/Users/sac/praxis/`. (Entries verified against the tree on the
date of writing — see the note on `agent8`.)

| Paper result | Implemented by |
|---|---|
| **Faithful Projection Theorem** (Part IV `thm:faithful`); receipt chaining (Part II) | Receipt chain in `crates/praxis-core`: `src/receipt_record.rs` (`ReceiptRecord`, fields `payload_hash_hex` / `prev_chain_hash_hex` / `chain_hash_hex`), the BLAKE3 fold `fold()` / `recompute_chain()` in `src/chain.rs`, ed25519 in `crates/praxis-core/src/signing.rs` |
| **Admission monoid** / refusal algebra (Part I; keystone §"The admission monoid") | `crates/praxis-core/src/refusal.rs` — `RefusalCategory`/`RefusalScenario`, `compose_denials()` (the monoid compose), `denial_lane()` / `scenario_for_denial_lane()` as its inverse |
| **BRCE B1 admission gate** — `A = μ(O*)`, Raw→Validated→Admitted (Part 0 `def:brce`) | `crates/praxis-core/src/law.rs` (`LawObject`, `Admit`, `Judge`, `Obligation`) + `src/default_law.rs`; driven by the `law` verb (`src/verbs/law.rs` `judge`/`admit`) |
| **Marking polytope** / conformance geometry, unit-fitness replay (Part III `def:polytope`, `thm:farkas`) | `PowlReplayVerifier` (from `bcinr_powl_receipt`) wired in `crates/praxis-core/src/replay_adapter.rs`; exposed via `receipt replay` / `receipt validate` (`src/verbs/receipt.rs`) |
| **Rice quarantine** — undecidable observations refused at the boundary (Part 0 `thm:rice`) | `crates/praxis-core/src/quarantine.rs` (`RiceQuarantine`, `BoundarySchema`, `JsonBoundarySchema`) |
| **Lifecycle category** — illegal transitions uninhabited (Part 0 `thm:typestate`) | `crates/praxis-core/src/lifecycle.rs`; enforced by the `typestate` default feature |
| **Receipt verdict / verifier** (Part IV bounded verifier) | `crates/praxis-core/src/receipt_validator.rs` (`ReceiptValidator`, `Verdict`, `Clock`); affidavit pipeline in `src/verify.rs` + `src/verbs/verifier.rs` (`verify` verb) |
| **Agent-8 projection / trillion-agent fleet** (keystone §"The trillion-agent projection", §"A worked status byte") | **Not yet in the tree as a crate.** The math is in the keystone; the code target `crates/agent8` (an `AgentByte` newtype + `Env64`/`Pulse64` wire ABI) is *specified* in `workflows/genesis/day4.js` but has not landed under `crates/`. Its nearest real analog today is the node-bit/token-bit status projection in `crates/praxis-core/src/replay_adapter.rs` (`PowlReplayFrame`). See the honesty note below. |

**Honesty note on `agent8`:** the task brief lists `agent8 projection -> crates/agent8`.
Spot-checking the tree, `crates/` contains `chatman-common`, `praxis-core`,
`praxis-proposer`, `praxis-reconciler`, `praxis-retrofit`, and
`rust-fable-testbed` — **no `agent8`**. The only references to `agent8` are in
`workflows/genesis/day4.js` (a build plan) and `tests/frontier_matrix.rs` (a
capability-frontier cell). Treat that row as *planned*, not shipped. Every other
row above points at a file that exists and a symbol re-exported from
`crates/praxis-core/src/lib.rs`.

---

## Prior art

- **`[chatman2025]`** — *The Chatman Equation and the Industrial Revolution of
  Knowledge: A = μ(O), Knowledge Hooks, and Production-Verified Enterprise
  Execution* (The Praxis / Capability Physics Program). This is the published
  paper that introduced the single governing law `A = μ(O)` and demonstrated it
  at enterprise scale — knowledge hooks over RDF/SHACL, the KNHK hot-path
  executor, the `ggen` bounded projection, and Lockchain provenance. The thesis
  series does not restate that result; it *grounds* it by factoring μ into
  admission + manufacture (`00_foundations.tex` §"Prior work", `\cite{chatman2025}`,
  cited by all six papers). Nothing in it is retracted: its ingress guards *are*
  the obligation battery, and its cryptographic receipts *are* the receipt space.

- **bytestar / C-era prehistory** — the fixed-size 64-byte wire frames and the
  8-bit agent-status projection descend from a pre-Rust C substrate. That
  lineage survives today as the `#[repr(C, align(64))]` frame idea and the
  status-byte view discussed in the keystone (§"A worked status byte"). In the
  current codebase the surviving, *ported* form is the compile-time-sized POWL
  replay frame (`crates/praxis-core/src/replay_adapter.rs`) and the
  capability-frontier bytestar row recorded in `workflows/genesis/day7.js`
  ("bytestar — C stubs/dormant — design ported"). The full `Env64`/`Pulse64` ABI
  port is the same not-yet-landed `agent8` work noted above.

---

## Where to go next

- Hands-on: [`ONBOARDING.md`](./ONBOARDING.md) — clone, build, and run one verb
  from each noun (law, plan, receipt, propose), read a receipt, verify a chain.
- The end-to-end skeptic's demo: `/Users/sac/praxis/scripts/walkthrough.sh`
  (narrated in `/Users/sac/praxis/docs/WALKTHROUGH.md`).
