# Blue River Dam Benchmarks — v26.7.6

Divan micro-benchmarks over the Praxis control layer, measured on this
machine on 2026-07-06. Every number in this document comes from the bench
runs recorded in Section 3; nothing is asserted-in.

## 1. Executive summary

**The Praxis control layer operates below the latency horizon of AI agents.**

An LLM agent step is a seconds-scale event (order of magnitude: 1–30 s per
tool call or reasoning turn — public order-of-magnitude knowledge, labeled as
such in Section 8; no agent was benchmarked here). The measured Praxis
control operations that govern such a step are nanosecond-to-microsecond
events:

| Control operation | Median (measured) | vs. a 1 s agent step |
|---|---|---|
| `bcinr_transition_table` (LUT dispatch) | 0.581 ns | ~1.7 × 10⁹ per step |
| `powl_step_tick` (workflow advance) | 3.465 ns | ~2.9 × 10⁸ per step |
| `standing_transition` (Raw → Validated) | 19.94 ns | ~5.0 × 10⁷ per step |
| `little_law_snapshot` (64-receipt ledger) | 47.45 ns | ~2.1 × 10⁷ per step |
| `action_precondition_mask` | 57.86 ns | ~1.7 × 10⁷ per step |
| `receipt_frame_link` (BLAKE3 chain link) | 245.6 ns | ~4.1 × 10⁶ per step |
| `pddl_action_filter` (full grounding) | 6.54 µs | ~1.5 × 10⁵ per step |
| `verify_gate_dispatch` (5-gate pipeline, 8 records) | 11.41 µs | ~8.8 × 10⁴ per step |
| `ggen_render_report_small` (Tera render) | 17.33 µs | ~5.8 × 10⁴ per step |
| `graphlaw_materialize_delta` (rule fixpoint) | 922.5 µs | ~1.1 × 10³ per step |

Even the most expensive control operation measured (GraphLaw rule
materialization at 922.5 µs) fits more than a thousand times inside a single
1-second agent step. The full deterministic spine of one governed action —
transition-table dispatch + standing transition + precondition mask +
scheduler tick + receipt link (0.581 + 19.94 + 57.86 + 3.465 + 245.6 ns ≈
327 ns) — costs roughly a third of a microsecond. Praxis governance is not
the bottleneck of an AI-operated process; it is invisible next to it.

## 2. Little's Law framing

Little's Law: **L = λ·W** — work in process equals arrival rate times cycle
time.

AI agents raise λ: a fleet of agents can propose actions at rates no human
change-control process was sized for. If W (the time to judge, admit,
schedule, receipt, and verify each action) stays at human scale — minutes to
days — L explodes: an unbounded queue of unjudged, unreceipted actions.

Praxis attacks W. The measured control path holds per-action governance
overhead W_control at ~327 ns (Section 1 spine) and even the heavyweight
steps (grounding, verification, rendering, law materialization) at
microseconds to sub-millisecond. With W_control ≈ 10⁻⁶–10⁻³ s, the control
layer's own contribution to L is ≈ λ × 10⁻³ or less — at λ = 1,000
actions/second, the control layer holds about one action in process at the
worst case (materialization-bound) and about 10⁻³ actions in process on the
receipt spine. The dam holds; the reservoir is the agents' own think time,
not the law.

The `little_law_snapshot` operation itself — computing L, λ, W from a live
64-record receipt ledger — costs 47.45 ns, so WIP can be observed
continuously without perturbing the system it measures.

## 3. Benchmark environment

| | |
|---|---|
| Hardware | Apple M3 Max, 48 GB RAM |
| OS | macOS 26.2 (Darwin 25.2.0) |
| rustc | 1.97.0-nightly (a5c825cd8 2026-04-14) |
| Bench harness | divan 0.1.21, `harness = false`, bench profile (release-inherited, `panic = "unwind"`) |
| Date | 2026-07-06 |
| Repo state | v26.7.6 working tree, commit lineage from `1ea2385` |

Exact commands:

```sh
cargo bench --bench blue_river_dam -p my-conforming-project   # root: transitions, planner, receipts, verify, Little's Law
cargo bench --bench blue_river_dam -p ggen                    # ggen: sync-engine Tera render
cargo bench --bench blue_river_dam -p praxis-graphlaw         # graphlaw: materialize-over-delta
```

Divan defaults: 100 samples per benchmark; iteration counts auto-scaled
(reported per bench below). All fixtures are deterministic and wall-clock
free in the hash paths (invariant 3): receipt frames carry `ts_ns = 0` at
emission; ledger fixtures use synthetic monotonic `ts_ns`.

Honest-proxy notes (per the rule: name the proxy or refuse):

* `standing_transition` measures `DefaultLaw::judge` (Raw → Validated) on a
  `LawObject` with zero obligations — the same praxis-core status path
  `ops::receipt_issue_payload` (and therefore `plan run`) drives, isolated
  from I/O.
* The PDDL benches run over a bench-owned 5-action STRIPS mirror of the
  golden law lifecycle (supply-evidence → clear-obligations → judge → admit
  → receipt), not the `mfg`-manufactured text (which is behind the `ggen`
  feature); the grounder, mask check, and scheduler are the production code
  paths.
* `ggen_render_report_small` renders a bench-owned small report template
  through the exact Tera engine `ggen::sync` builds (`template::build_tera`
  over a preloaded `GraphLawStore`); the sync stage's private `render_str`
  wrapper (error mapping only) is not included.
* `graphlaw_materialize_delta` re-runs `TripleStore::materialize()` to
  fixpoint after adding a 1-triple delta to a preloaded, already-materialized
  32-node chain with transitive rules — materialization is fixpoint-based,
  so the number covers re-derivation, not a DRed-style incremental delta.

## 4. Deterministic transition benchmarks

`benches/blue_river_dam.rs` (root crate):

| Benchmark | Fastest | Median | Mean | Samples × iters |
|---|---|---|---|---|
| `standing_transition` | 19.61 ns | 19.94 ns | 19.91 ns | 100 × 25,600 |
| `bcinr_transition_table` | 0.571 ns | 0.581 ns | 0.582 ns | 100 × 819,200 |
| `action_precondition_mask` | 56.56 ns | 57.86 ns | 57.86 ns | 100 × 12,800 |
| `action_precondition_mask_condition_tree` | 93.35 ns | 94.66 ns | 95.6 ns | 100 × 3,200 |
| `pddl_action_filter` | 6.29 µs | 6.54 µs | 6.841 µs | 100 × 100 |

* `standing_transition` — one Raw → Validated standing transition through
  `praxis_core::DefaultLaw::judge`, the status machine every receipted
  action passes through.
* `bcinr_transition_table` — one `bcinr_powl::admit::admit` call: an 8-bit
  key extraction plus a single index into the compile-time 256-entry
  topology LUT. Sub-nanosecond: this is the raw dispatch floor.
* `action_precondition_mask` — evaluate every ground action's precondition
  set against the state and fold the eligibility bits into a u64 fired-set
  mask (the per-node check of `find_plan`'s BFS).
* `action_precondition_mask_condition_tree` — the recursive
  `eval_condition` variant used by the temporal path, over the golden goal.
* `pddl_action_filter` — full candidate admit/refuse filtering:
  `GroundProblem::build` type-checks and grounds all 5 action schemas over
  the problem objects, refusing type-incompatible candidates.

## 5. Workflow control benchmarks

| Benchmark | Fastest | Median | Mean | Samples × iters |
|---|---|---|---|---|
| `powl_step_tick` | 3.384 ns | 3.465 ns | 3.456 ns | 100 × 102,400 |

One `bcinr_powl::scheduler::scheduler_tick` over the compiled golden 5-slot
POWL sequence tape from a fresh run state — the exact step-advance
`plan_run::execute_receipted` loops on. At 3.465 ns per tick, advancing a
full 64-slot tape costs on the order of 0.2 µs of scheduler time.

## 6. Receipt-link benchmarks

| Benchmark | Fastest | Median | Mean | Samples × iters |
|---|---|---|---|---|
| `receipt_frame_link` | 243 ns | 245.6 ns | 245.4 ns | 100 × 3,200 |
| `verify_gate_dispatch` | 11.24 µs | 11.41 µs | 11.57 µs | 100 × 100 |
| `little_law_snapshot` | 46.8 ns | 47.45 ns | 47.32 ns | 100 × 12,800 |

* `receipt_frame_link` — building one `OcelCausalFrame` (`ts_ns = 0`,
  `DenialPolarity::ADMITTED`) and chaining it onto a genesis-folded
  `OcelCausalReceipt`: one BLAKE3 chain-hash step, the per-fired-atom cost
  of receipted execution. 245.6 ns/link ⇒ ~4 million tamper-evident chain
  links per second per core.
* `verify_gate_dispatch` — the full 5-gate
  `praxis_core::verify::run_pipeline` (format, chain integrity, continuity,
  commitments, profile) over an 8-record lawful ledger, including chain-hash
  recomputation per record: 11.41 µs ≈ 1.4 µs per record verified.
* `little_law_snapshot` — `verify_ops::little_law_snapshot` (new in this
  change, unit-tested in `src/verify_ops.rs`): L = λ·W over a 64-record
  in-memory ledger in 47.45 ns.

## 7. GraphLaw materialization benchmarks

`crates/praxis-graphlaw/benches/blue_river_dam.rs`:

| Benchmark | Fastest | Median | Mean | Samples × iters |
|---|---|---|---|---|
| `graphlaw_materialize_delta` | 886.2 µs | 922.5 µs | 923.2 µs | 100 × 100 |

Setup (untimed): load a 32-edge `ex:links` chain plus two N3 rules (direct +
transitive `ex:reach`) and materialize to fixpoint (~561 derived facts).
Timed: add one new edge and call `materialize()` again. As noted in Section
3 this is a fixpoint re-derivation, the worst honest reading of "small
delta" — and it still completes in under a millisecond.

ggen render surface (`crates/ggen/benches/blue_river_dam.rs`):

| Benchmark | Fastest | Median | Mean | Samples × iters |
|---|---|---|---|---|
| `ggen_render_report_small` | 16.95 µs | 17.33 µs | 19.39 µs | 100 × 100 |

One small report template (heading, 4-row loop, interpolations) rendered
through the sync engine's Tera instance built by `template::build_tera` over
a preloaded `GraphLawStore`.

## 8. LLM-agent latency comparison

No agent latency was measured in this work. As order-of-magnitude public
knowledge (labeled as such): a single LLM agent step — one reasoning turn or
tool call round-trip against a frontier hosted model — is a
**seconds-scale** event, commonly ~1–30 s depending on model, context
length, and output length; multi-step agent tasks run minutes.

Placing the measured table (Sections 4–7) against a conservative 1 s agent
step:

| Layer | Latency | Ratio to 1 s agent step |
|---|---|---|
| Agent reasoning step (order of magnitude, not measured here) | ~10⁰–10¹ s | 1 |
| GraphLaw law re-materialization | 9.2 × 10⁻⁴ s | ~10³ smaller |
| Render / verify / ground | 10⁻⁵ s | ~10⁵ smaller |
| Receipt chain link | 2.5 × 10⁻⁷ s | ~4 × 10⁶ smaller |
| Standing transition / masks / snapshot | 2–6 × 10⁻⁸ s | ~10⁷–10⁸ smaller |
| Scheduler tick / transition table | 10⁻⁹ s | ~10⁹ smaller |

The control layer sits three to nine orders of magnitude below the agent
layer. Governance at this cost is effectively free relative to the thing
being governed.

## 9. WIP explosion scenario (worked example)

Scenario: a fleet of 100 agents, each completing one governed action every
10 s ⇒ arrival rate **λ = 10 actions/s** into the control layer.

Ungoverned baseline for comparison: if each action instead waited on a
human change-approval step with W ≈ 4 h (14,400 s), Little's Law gives
L = 10 × 14,400 = **144,000 actions in process** — the WIP explosion. The
queue grows faster than any review board can drain it.

With the measured Praxis path, per-action control work (using medians):

| Step | Cost |
|---|---|
| `pddl_action_filter` (ground once per action) | 6.54 µs |
| `action_precondition_mask` | 57.86 ns |
| `powl_step_tick` × 5 slots | 17.3 ns |
| `standing_transition` | 19.94 ns |
| `receipt_frame_link` × 5 atoms | 1.23 µs |
| `verify_gate_dispatch` (8-record window) | 11.41 µs |
| `graphlaw_materialize_delta` (law delta per action, worst case) | 922.5 µs |
| **W_control total (worst case)** | **≈ 941.7 µs** |

L_control = λ × W = 10 × 9.417 × 10⁻⁴ ≈ **0.0094 actions in process** —
the control layer holds less than one-hundredth of one action at any
instant. Even at λ = 1,000 actions/s (a thousand-agent fleet at one action
per second each), L_control ≈ 0.94: the dam holds a single action while
144,000 would have pooled behind a human gate. Headroom to saturation of one
core is λ_max ≈ 1/W ≈ 1,060 actions/s on the worst-case path, and ≈ 52,000
actions/s if the law graph is not re-materialized per action (W ≈ 19.2 µs).

Observability closes the loop: at 47.45 ns per `little_law_snapshot`, the
L/λ/W gauges can be recomputed on every single receipt with no measurable
cost.

## 10. Fortune 5 deployment implication

A Fortune-5-scale deployment is a λ problem: tens of thousands of daily
changes across ERP, supply chain, and financial systems, about to be
multiplied by AI agents that never sleep. The measured numbers say the
Praxis control layer changes which term of L = λ·W the enterprise pays for:

* **Governance is off the critical path.** Sub-microsecond standing
  transitions and receipt links mean every agent action can be judged,
  admitted, scheduled, and chain-receipted inline — with total overhead
  (≈ 327 ns spine, ≈ 942 µs with a full law re-materialization) invisible
  next to the agent's own seconds-scale step.
* **One core is a control plane.** At ~1,000 governed actions/s per core on
  the worst-case measured path (~52,000/s without per-action law
  re-materialization), a single commodity node governs an action volume
  larger than a Fortune 5's daily change count per minute. Capacity planning
  for the dam is a rounding error.
* **Audit is continuous, not quarterly.** 11.41 µs to run all five verifier
  gates over a ledger window, 1.4 µs per record, means the entire receipt
  chain can be re-verified continuously; the Little's Law snapshot (47 ns)
  gives management a real-time WIP gauge instead of a post-hoc report.
* **The refusal path costs the same as the admit path.** These are the same
  table lookups and mask evaluations; saying "no" by name is as cheap as
  saying "yes" — so the dam does not create pressure to bypass it.

The deployment claim, stated as findings: with AI raising λ by orders of
magnitude, the measured Praxis W keeps L bounded near zero at the control
layer. The latency budget of enterprise governance moves from the approval
queue to the agents themselves — which is exactly where a Fortune 5 wants
its constraint.

---

Bench sources: `benches/blue_river_dam.rs`,
`crates/ggen/benches/blue_river_dam.rs`,
`crates/praxis-graphlaw/benches/blue_river_dam.rs`.
Snapshot function: `src/verify_ops.rs::little_law_snapshot`.
