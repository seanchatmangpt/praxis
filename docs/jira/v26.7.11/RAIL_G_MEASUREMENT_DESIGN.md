# Rail G Measurement Design — Multifractal Execution Density

Updated 2026-07-11. Companion to `PRD.md` section 16 ("Multifractal Measurement Rail") and
section 22 Rail G ("OCEL execution measure -> declared scale profiles -> Z(q,epsilon) ->
tau(q) -> f(alpha) -> standing report").

## Executive summary

"Multifractal" has zero grounding in this repository today. `git grep -rli multifractal`
returns exactly one file: `PRD.md`. The one commit whose message claims "multifractal
execution logic" (`b404c53e`) did not implement the formalism — it added a typed `ExternalCut`
scaffold with no SPARQL execution, no Tera rendering, and a receipt struct that hashes
pre-computed digests nobody ever supplies. See `RAIL_A_B_STATUS.md` (companion doc,
Rail A/B reconciliation) for that finding in detail.

This document is not a status report — it is the instrumentation design needed to make a
multifractal claim real: which measurable quantity to partition, at which scales, computed by
which code, and what a non-trivial result would look like versus a trivial (monofractal) one.
Two independent, complementary tracks are specified below rather than one, per the standing
instruction to cover the space combinatorially instead of picking a single metric prematurely.

## 1. What "multifractal" operationally means here

Given a measure \(\mu\) distributed over some space, partitioned into boxes of size
\(\epsilon\), with box masses \(\mu_i(\epsilon)\):

- Partition function: \(Z(q,\epsilon) = \sum_i \mu_i(\epsilon)^q\)
- Mass exponent: \(\tau(q)\), fit from \(\log Z(q,\epsilon)\) vs. \(\log \epsilon\) across
  several \(\epsilon\) (slope of the linear regression)
- Generalized dimension: \(D(q) = \tau(q)/(q-1)\), \(q \neq 1\)
- Singularity spectrum via Legendre transform: \(\alpha(q) = d\tau/dq\),
  \(f(\alpha) = q\alpha(q) - \tau(q)\)

**The system is multifractal only if \(D(q)\) is non-constant across \(q\)** (equivalently,
\(f(\alpha)\) spans a non-trivial range of \(\alpha\)). A flat \(D(q)\) is a legitimate,
honest result — it means the measured process is monofractal (uniform scaling), not a failed
experiment. Report which outcome was actually measured; do not presuppose multifractality.

Three preconditions are non-negotiable before this is worth computing at all:
1. A genuinely multiscale structure to partition (not just "run it twice at different sizes").
2. Enough distinct \(\epsilon\) values (several, ideally spanning an order of magnitude or
   more) to fit \(\tau(q)\) with statistical confidence, not eyeball two points.
3. Enough real heterogeneity in the underlying process that \(D(q)\) has a chance of varying —
   otherwise the honest finding is "monofractal," which should be reported as such.

None of \(Z(q,\epsilon)\), \(\tau(q)\), or \(f(\alpha)\) exists anywhere in this codebase
today (confirmed by exhaustive grep). This is new code, not a wiring exercise.

## 2. Track 1 — Structural (spatial) multifractal over the POWL decomposition tree

The best-grounded candidate: `Powl` (`crates/powl2-decompose/src/powl.rs:47-72`) is already a
genuine recursive tree — `PartialOrder{children: Vec<Powl>, order}` /
`Choice{children: Vec<Powl>, graph: ChoiceGraph}` — built by
`decompose::convert_rec(net, depth, budget)` (`crates/powl2-decompose/src/decompose.rs:137`),
which already threads a `depth: usize` parameter through the recursion. Recursive tree depth
*is* a natural box-size scale for multifractal analysis (this is the same structure used for
multiplicative cascades in the classical multifractal literature) — no synthetic scale needs
to be invented.

- **Box size \(\epsilon\)**: tree depth \(d\) (or, more finely, cumulative subtree "reach" —
  number of leaf `Powl` nodes under a subtree rooted at depth \(d\)).
- **Mass \(\mu_i(\epsilon)\)**: for the subtree rooted at node \(i\) at depth \(d\), a real
  work quantity attributable to that subtree. Candidates, in order of how directly they're
  already computed:
  - Leaf count under the subtree (purely structural; zero new instrumentation, computable
    directly from the existing `Powl` tree with a simple recursive count).
  - Fan-out-weighted node count: sum of `children.len()` (or `ChoiceGraph.n`,
    `powl.rs:22`) over all nodes in the subtree — captures branching intensity, not just size.
  - Receipt count: number of `ChatmanEngine::admit_transition` calls
    (`crates/praxis-graphlaw/src/chatman/engine.rs:566`) attributable to executing that
    subtree, if/when POWL execution is wired to the chatman engine (it is not today — see
    §4 "what has to be built").
- **What a real result looks like**: compute \(D(q)\) for \(q \in \{-5,...,-1,0,1,...,5\}\)
  (standard range) across several real or synthetic POWL workflows. If some workflows are
  sequential-heavy (deep, narrow — mostly `PartialOrder` chains) and others are
  parallel-heavy (shallow, wide — mostly `Choice` fan-out), \(D(q)\) should genuinely differ
  by region within a single large mixed workflow. That heterogeneity is the multifractal
  claim; a uniformly-shaped workflow will legitimately measure as monofractal.

## 3. Track 2 — Temporal multifractal over a single execution trace

Independent of Track 1: partition *time* (or, natively to this codebase, **ticks** — see
`crates/praxis-synthesis/src/budget.rs:25-28`, `CHATMAN_CONSTANT: u64 = 8`, `Ticks(pub u64)`,
explicitly documented as a declared bounded-work unit, not wall-clock — the correct native
unit here, not `Instant::now()`) into windows of size \(\epsilon\), from one execution trace.

- **Box size \(\epsilon\)**: window width in ticks (e.g. 1, 2, 4, 8, 16, 32, ... — geometric
  spacing gives the log-log regression real dynamic range).
- **Mass \(\mu_i(\epsilon)\)**: count of `admit_transition` receipts
  (`engine.rs:566-616`, real, already-computed: each call already produces one sealed
  `EngineProcessReceipt` with a `receipt_root` over 9 stage digests, `engine.rs:217-241,
  270-278`) falling in window \(i\). A lower-cost proxy that exists today without any new
  wiring: `crates/cng/src/bench/manufacture.rs:69`'s `manufacture_set` already returns a
  `SetOutcome{stage_ns, graph_triples, transitions, tape_ops, ...}`
  (`manufacture.rs:21-45`) per artifact set, driven by `WorkdayConfig{ticks, ...}`
  (`crates/cng/src/bench/workday.rs:222-227`) — `tape_ops`/`transitions` counts binned by
  tick-window is buildable from existing fields with no new measurement code, only new
  aggregation code.
- **What a real result looks like**: if admission is bursty (long quiet stretches punctuated
  by dense admission clusters — plausible for a workflow with parallel fan-out gated by a
  shared precondition), \(D(q)\) will show real spread. If admission is close to uniform in
  time, the honest measurement is monofractal.

## 4. What has to be built (none of it exists today)

1. A partition-function module: `Z(q, epsilon)`, `tau(q)` via log-log linear regression
   across the epsilon sweep, `alpha(q)`/`f(alpha)` via the Legendre transform (finite-
   difference derivative of the fitted `tau(q)` curve is sufficient at this stage — no need
   for a closed-form derivative). This is genuinely new mathematical code; propose it as its
   own module (e.g. `crates/praxis-core/src/multifractal.rs`) rather than folding it into
   `cng`'s bench harness, since both Track 1 and Track 2 need to call it.
2. For Track 1: a subtree-mass walker over `Powl` (recursive, `O(n)` in tree size) computing
   whichever mass definition is chosen, at every depth — this does not require executing
   anything, it's a pure structural computation over trees `powl2-decompose` already
   produces.
3. For Track 2: tick-windowed aggregation over either (a) a real `admit_transition` receipt
   stream, which requires POWL execution to actually be wired to the chatman engine (it is
   not — see `RAIL_A_B_STATUS.md`), or (b) `cng`'s existing `manufacture_set` outputs across
   a `WorkdayConfig.ticks` sweep, which is buildable now with zero new execution wiring.
   **Track 2b is therefore the lowest-cost path to a first real (not decorative) data point**
   — it needs only the aggregation/partition-function module, not new execution plumbing.
4. A "declared scale profile" schema (PRD §17 item 21 lists "Multifractal measurement profile
   schema" as a required, currently-unbuilt artifact, `PRD.md:830`) — the epsilon sweep values
   and q range need to be declared and receipted, not chosen ad hoc per run, to satisfy the
   PRD's determinism/receipt discipline (`CLAUDE.md` invariant 2/5).

## 5. The "1000x phase change" claim

A phase change is a threshold-crossing qualitative shift, not a magnitude ratio by itself. Two
real, already-present threshold parameters are candidates for where such a shift could
legitimately occur — use one of these, not an invented parameter:

- **`convert_with_budget(net, budget: usize)`** (`decompose.rs:109`) — `budget` is a hard
  ceiling; behavior changes discontinuously from "succeeds" to "refused" when the recursive
  decomposition exceeds it. This is the more classical sense of "phase change" (a
  discontinuity in behavior class at a threshold), and it is already a real, tested code path
  — not hypothetical.
- **`partition_mg` vs. `partition_sm` branch selection** (`decompose.rs:154` vs. `:165`) — a
  marked-graph vs. state-machine structural classification that already picks a qualitatively
  different decomposition strategy. Measuring Track-1 mass/density on either side of this
  branch is a real regime-comparison, not a fabricated one.

**No claim about "1000x" specifically is currently supportable — that number is not measured
anywhere in this repo.** The correct next step is: pick one candidate threshold above, measure
whichever Track-1/Track-2 metric on both sides of it across a range of workload sizes, and
report whatever ratio is actually observed. It may be 3x, 40x, or 1000x; report the number the
instrumentation produces, scoped to the metric and the workload class it was measured on — per
`PRD.md:1109` itself, which already anticipates this rail may be `PARTIAL_ALIVE` rather than
`ALIVE` at initial release.

## 6. Recommended build order (not exclusive — both tracks are independently valuable)

1. Track 2b first (lowest cost: aggregation-only, no new execution wiring) to get one real,
   non-trivial `tau(q)`/`f(alpha)` data point end to end, proving the partition-function module
   itself is correct.
2. Track 1 (structural, POWL-tree) next — pure-function, no execution dependency, and directly
   answers whether workflow *shape* is multifractal, which is closer to what PRD §7.4's
   external-cut projection is actually about.
3. Track 2a (real `admit_transition` receipt stream) only after POWL execution is actually
   wired to the chatman engine (a Rail B/C dependency — see `RAIL_A_B_STATUS.md`); until then
   it has no real data source and should not be attempted.

## See also

- `PRD.md` — sections 16 (formalism definitions), 17 item 21 (required schema artifact), 22
  Rail G, 23 (Definition of Done, `PARTIAL_ALIVE` allowance for this rail).
- `RAIL_A_B_STATUS.md` — Rail A/B reconciliation (external-cut projection and AIR compiler
  actual status; the projection this rail's Track 1 would eventually measure real data from).
- `crates/powl2-decompose/src/decompose.rs`, `crates/powl2-decompose/src/powl.rs` — Track 1
  hook points.
- `crates/praxis-graphlaw/src/chatman/engine.rs` — the real receipt cycle (`admit_transition`)
  Track 2a would consume once wired.
- `crates/cng/src/bench/manufacture.rs`, `crates/cng/src/bench/workday.rs` — Track 2b's
  existing scale-varying benchmark harness.
