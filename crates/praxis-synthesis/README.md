# praxis-synthesis

Prototype crate combining the deep-research findings (2026-07-02) into one
bounded, receipted pipeline. Each layer maps to one research result and rides
on a substrate praxis already owns — nothing is duplicated.

## Layer → research map

| Layer | Module | Research finding | Substrate reused | What's new |
|---|---|---|---|---|
| 1 | `datalog` | **Nemo** — ultra-scalable Datalog reasoning (arXiv:2308.15897) | `pddl_index::{Dict, FactStore, atom_key}` — the same interned u32 ID space the grounder uses; generalizes `IndexedGroundProblem::build`'s delete-relaxed fixpoint | **Semi-naive** evaluation (delta joins) for arbitrary stratified Horn rules; `fixpoint_hash` content-addresses the reasoned state |
| 2 | `sequence` | **SMT capability sequencing** (arXiv:2312.08801) — solver discovers execution order *and* parameter bindings; no hand-authored PDDL | Layer 1's saturated database is the binding-enumeration source (the "Datalog feeds the solver" stack) | `Capability` declarations + `BoundedCsp`: deterministic branch-and-bound with `before(a,b)` ordering constraints; the `Solver` trait is the seam for a real SMT backend |
| 3 | `dag` | **OxyMake** — content-addressable workflows (arXiv:2606.20989) | `chatman_common::provenance::{content_address, fold_event, genesis_seed}` | First DAG executor in the tree: data-dependency edges derived from effect→precondition flow, BLAKE3 per-node outputs, memo-cache replay, **order-independent** `root_hash` |
| 4 | `verify` | **Flux** — refinement types (arXiv:2207.04034), read as *refinements over artifacts*, not a type checker | style of `praxis-core::verify::run_pipeline` (all checks run, no short-circuit) | Six machine-checkable refinements incl. `PlanReachesGoal` (independent replay), `ChainRecomputes` (byte-compare refold), `FixpointClosed` (extra round derives nothing) |

`pipeline::Synthesis::run` composes them: **facts → saturate → sequence →
dag-execute → verify → one `SynthesisReceipt`** whose `chain` folds every
stage hash into a single BLAKE3 value — the whole run as one auditable object.

## The flagship proof

`tests/sequence_tests.rs::solver_rediscovers_the_five_step_lawobject_order`:
the five lawobject capabilities (supply-evidence, clear-obligations, judge,
admit, receipt) are *declared* — preconditions, effects, cost — and the solver
**discovers** the order and the `o1` binding that Genesis Day 2 hand-authored
in PDDL. `tests/pipeline_tests.rs` then executes the discovered plan as a
content-addressed DAG (second run: 100% memoized, byte-identical receipt) and
the verifier admits it.

## Doctrine

Bounded everything, receipted refusals, no silence:

- Caps: `MAX_TUPLES` 1M, `MAX_ITERATIONS` 10k, `MAX_STRATA` 8, `MAX_VARS` 8,
  `MAX_STEPS` 16, `MAX_BINDINGS_PER_STEP` 256, `MAX_NODES` 100k.
- Every cap violation is a `Refusal` variant carrying reason + salvage data
  (partial progress, best-plan-so-far, frontier size) — never a panic, never a
  silent truncation.
- Deterministic: same input → byte-identical receipts, plans, and hashes
  (locked by tests).

## Refusal register (what this prototype deliberately does NOT do)

| Refused | Reason | Salvage path |
|---|---|---|
| Real SMT backend (z3, cvc5) | Native library dependency breaks the pure-Rust / forbid-unsafe / deterministic-build doctrine | `sequence::Solver` trait is the seam; `BoundedCsp` proves the architecture |
| Parallel DAG execution | Prototype scope; determinism first | Topo order + content addressing already make nodes with disjoint inputs safely parallelizable later |
| Memo-cache persistence | Prototype scope | `MemoCache` is a plain map; a JSONL/sled store slots behind the same key scheme |
| CLI verb (`synth …`) | Surface stability — the shape should settle before it becomes a public noun | Follow-up once the API survives contact |
| Nemo-scale (10⁸ facts) benchmarks | This is an architecture proof at 10³–10⁵ tuples; the nested-loop join would need index-backed joins first | Layer 1's storage is already the interned sorted-ID form index joins want |
| Delete-conflict DAG edges | Data edges only (producer→consumer); delete-effects don't order otherwise-independent nodes | Documented; conservative fallback is the plan's linear order, which `Dag::from_plan` never violates |

## Run

```sh
cargo test -p praxis-synthesis     # 20 tests
cargo clippy -p praxis-synthesis   # clean
```

## The Supervision Layer (fault-tolerant plans)

Lineage: **CNS 8T/8H/8M** (tick/hop/memory bounds) → **BitActor** (`TICK_BUDGET 8`,
sealed spec↔exec) → **ByteActor V3** (bounded rings, crystal envelopes) →
**knhk** (TickBudget, R1/W1/C1 classification, ParkManager — observation
without actuation) → **praxis** (this crate: the actuation layer, closed).

| module | provenance | what it adds |
|---|---|---|
| `budget` | PORT(knhk timing.rs) | branchless TickBudget, CHATMAN_CONSTANT=8, compile-time ChatmanBounded |
| `fault` | PORT(knhk runtime_class/failure_actions) | R1/W1/C1 tiers; budget breach = *certified* refusal (knhk had a metric) |
| `park` | PORT(knhk park.rs) + gaps closed | kill-9-durable quarantine (WAL) + ReAdmission{OnInputChange,AfterRuns,Manual} |
| `supervise` | net-new | topology DERIVED from the plan's dependency structure; OneForAll refused by absence; intensity ≤ 8 |
| `geometry` | net-new | 8 failure classes; branches mined from the solver's own fragility analysis; GeometryGap implicit + unshadowable |
| `dag::execute_supervised` | net-new | crashes as values; classify→actuate closed; GaveUp = lawful Ok-receipt; crash chain anchored to geometry_hash |
| `cell_supervise` | net-new | combo status lanes (provably-unreachable bit sets); MAPE-K at epoch boundaries; quarantine by cross-group quorum |

Measured (`receipts/supervised_cell.json` v2, 10k members, release, under
the honesty-audit discipline of `tests/honesty_audit.rs`): supervision
overhead is −0.9% **of medians over 5 paired runs**, inside the baseline's
±5.7% run spread — noise, and the harness self-refutes if a negative
overhead ever exceeds spread (the v1 single-sample −2.7% figure was retired
by exactly that guard); per-member latency at 10% faults is p50 34µs /
p99 1.42ms / worst 2.29ms — the tail the aggregate was masking; verdicts
are never minimums; composition ratio (whole cell vs sum of members)
measured at 1.0004; recovery counts track injected rates exactly
(69/608/3,179 at 1%/10%/50%); throughput flat ~15k members/s across fault
rates, recomputed from (count, elapsed); crashloop template quarantined by
epoch 2 in every run; every cell verified, and `foreign_verify.py` passes
**unmodified** on supervised receipts.

### Supervision refusals (receipted)
| Refused | Reason | Salvage |
|---|---|---|
| Path-deps on knhk | verified: even `genesis-runtime-primitives` drags tokio/reqwest/otel; mu-kernel ships test libs as regular deps + unsafe | ports with greppable `PORT(knhk)` provenance |
| rdtsc tick measurement | knhk's own hot receipts carried dummy ticks; declared costs are the honest model here | `Ticks` abstract; bench follow-up |
| OneForAll strategy | acyclic data-flow cannot express shared mutable fate | absence enforced by exhaustive match |
| Novelty-curve-under-faults measurement | member-level injection doesn't re-run solver work; a work-proxy would overstate cache dividends | node-level fleet injection is the follow-up |
| Beat scheduling, full SLO matrix, quarantine parole, supra-cell | v1 scope | receipted in supervised_cell.json |
