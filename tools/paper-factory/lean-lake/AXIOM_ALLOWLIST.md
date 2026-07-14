# Axiom Allowlist — `Praxis/Corpus`

Inventory of every `axiom` declaration in `tools/paper-factory/lean-lake/Praxis/Corpus/*.lean`
that lives outside an `ax_*.lean` file (files named `ax_*.lean` are the designated home for
axioms and are not tracked here). This file is the disclosure record for today's known axioms
and the input to the regression gate (`just praxis-lean-axiom-gate`) that blocks any *new*,
undocumented axiom from being added.

Recomputed from scratch on 2026-07-12 via:

```bash
grep -rn "^\s*axiom\s" tools/paper-factory/lean-lake/Praxis/Corpus/*.lean | grep -v "^ax_"
```

That grep returns 65 lines. Of those:

- **6 are false positives** — prose inside doc comments that happens to start with the word
  `axiom` (e.g. `def_genesis.lean:22: axiom needed.`, `thm_sep.lean:19: axiom is needed
  since...`). Not real declarations; excluded from every count below.
- **59 are real `axiom` declarations.**
- **18 of the 59** live in the 7 files another agent is actively reproving/reclassifying
  (`prop_intauth.lean`, `refusal_simpleoneforone.lean`, `ref_curve.lean`,
  `lineage_armstrong.lean`, `def_obsauth.lean`, `def_body.lean`, `prop_topology.lean`).
  These are **excluded, in-progress** — not listed below, neither passed nor failed by the
  gate. Re-run this inventory once that agent's work lands.
- **41 real, in-scope axioms remain** — these are the rows below, and the exact set the gate
  enforces.

Prior-audit cross-check: the lean-pilot corpus was reported at 245 axioms; that figure is a
different tree (`lean-pilot`, not `lean-lake/Corpus`) and is not comparable to the 41/59/65
figures above, which were counted fresh from `lean-lake/Corpus` for this task.

## Classification key

- **(a) External primitive postulate** — models something genuinely outside Lean's reach (a
  concrete hash function's behavior, an OS/process call, a hardware counter reading). Defensible
  to leave axiomatized indefinitely.
- **(b) Property/theorem-shaped axiom** — asserts a fact, estimate, or relationship that could
  in principle be defined or proved inside Lean. Documented here as a known gap, not hidden;
  candidate for future proof or downgrade to a cited empirical assumption.

## Excluded, in-progress (owned by another agent — not in this inventory or gate)

| File | Axiom count |
|---|---|
| `def_body.lean` | 2 |
| `def_obsauth.lean` | 4 |
| `lineage_armstrong.lean` | 4 |
| `prop_intauth.lean` | 2 |
| `prop_topology.lean` | 1 |
| `ref_curve.lean` | 2 |
| `refusal_simpleoneforone.lean` | 3 |
| **Total excluded** | **18** |

## In-scope inventory (41 axioms, gated)

| File:Line | Name | Class | Justification |
|---|---|---|---|
| `con_commit.lean:23` | `chainH` | a | Content-addressing hash function (`Payload → BitVec 256`); models a real cryptographic hash's behavior, outside Lean's computational reach without a verified hash implementation. |
| `con_fablechain.lean:54` | `chainH` | a | Same hash primitive re-postulated in this module's namespace; external cryptographic hash. |
| `con_merklecell.lean:41` | `chainH` | a | Same hash primitive re-postulated for Merkle-cell chaining; external cryptographic hash. |
| `con_xorf.lean:41` | `fnv1a` | a | FNV-1a 32-bit non-cryptographic hash; fixed external standard with no Mathlib equivalent (per module docstring). |
| `def_contentaddr.lean:37` | `chainH` | a | Canonical definition site for the `chainH : ByteArray → Digest` hash primitive that other modules re-postulate. |
| `def_receipt.lean:48` | `chainH` | a | Same hash primitive, receipt-chaining context. |
| `def_residual.lean:39` | `RepairBand` | b | Opaque `Type` standing in for an undefined repair-band domain; could be given a concrete inductive/structure definition instead of being left abstract. |
| `def_residual.lean:45` | `repairOp` | b | Behavior of a repair operation over `RepairBand`/`Fin k`/`ℝ` asserted rather than defined; a concrete implementation is plausible. |
| `def_sandbox.lean:29` | `cargoBuild` | a | Invokes an external process (`cargo build`); genuinely outside Lean's reach without an FFI/IO model. |
| `def_sandbox.lean:30` | `cargoTest` | a | Invokes an external process (`cargo test`); same as above. |
| `def_sandbox.lean:31` | `cargoClippy` | a | Invokes an external process (`cargo clippy`); same as above. |
| `def_sandbox.lean:32` | `safetyAudit` | a | External audit procedure over a `CodeBlock`; not computable inside Lean. |
| `def_sandbox.lean:50` | `blake3Chain` | a | External cryptographic hash-chaining primitive (BLAKE3). |
| `def_vizgap.lean:27` | `DiffBlock` | b | Opaque `Type` for a diff-block domain; could be given a concrete definition. |
| `def_walframe.lean:31` | `chainH` | a | Same hash primitive, wal-frame chaining context. |
| `est_bit_supply.lean:65` | `llmJudgmentThroughput` | b | Empirical estimate parameter (real-valued); a modeling assumption about external system throughput, not a Lean-internal fact — should eventually cite a source or be replaced with a parameterized bound. |
| `est_bit_supply.lean:66` | `llmJudgmentThroughput_eq` | b | Pins the above estimate to the literal value `10^6`; an empirical assumption asserted as axiom rather than derived. |
| `est_comp_supply.lean:25` | `decisionCostLowerBound` | b | Empirical cost-estimate parameter (`Float`); external economic assumption. |
| `est_comp_supply.lean:27` | `decisionCostUpperBound` | b | Empirical cost-estimate parameter; external economic assumption. |
| `est_comp_supply.lean:29` | `decisionCostLowerBound_eq` | b | Pins the lower bound to `1e-2`; empirical assumption asserted as axiom. |
| `est_comp_supply.lean:30` | `decisionCostUpperBound_eq` | b | Pins the upper bound to `1e0`; empirical assumption asserted as axiom. |
| `est_comp_supply.lean:33` | `decisionsPerSecPerAcceleratorLower` | b | Empirical throughput-estimate parameter; external hardware assumption. |
| `est_comp_supply.lean:35` | `decisionsPerSecPerAcceleratorUpper` | b | Empirical throughput-estimate parameter; external hardware assumption. |
| `est_comp_supply.lean:37` | `decisionsPerSecPerAcceleratorLower_eq` | b | Pins lower bound to `1e0`; empirical assumption asserted as axiom. |
| `est_comp_supply.lean:38` | `decisionsPerSecPerAcceleratorUpper_eq` | b | Pins upper bound to `1e2`; empirical assumption asserted as axiom. |
| `est_comp_supply.lean:41` | `fleetSizeLower` | b | Empirical fleet-size estimate parameter; external assumption about deployed hardware count. |
| `est_comp_supply.lean:43` | `fleetSizeUpper` | b | Empirical fleet-size estimate parameter; external assumption. |
| `est_comp_supply.lean:45` | `fleetSizeLower_eq` | b | Pins lower bound to `1e6`; empirical assumption asserted as axiom. |
| `est_comp_supply.lean:46` | `fleetSizeUpper_eq` | b | Pins upper bound to `1e7`; empirical assumption asserted as axiom. |
| `est_comp_supply.lean:49` | `lambdaCompLower` | b | Derived-rate lower bound; empirical assumption asserted as axiom rather than computed from the other parameters. |
| `est_comp_supply.lean:51` | `lambdaCompUpper` | b | Derived-rate upper bound; same as above. |
| `est_comp_supply.lean:53` | `lambdaCompCentral` | b | Derived-rate central estimate; same as above. |
| `est_comp_supply.lean:55` | `lambdaCompLower_eq` | b | Pins lower bound to `1e7`; empirical assumption asserted as axiom. |
| `est_comp_supply.lean:56` | `lambdaCompUpper_eq` | b | Pins upper bound to `1e9`; empirical assumption asserted as axiom. |
| `est_comp_supply.lean:57` | `lambdaCompCentral_eq` | b | Pins central estimate to `1e8`; empirical assumption asserted as axiom. |
| `meas_recoveryinvisible.lean:56` | `recoveryinvisible` | b | Theorem-shaped `Prop` over `CrashKind`/`BitVec 256`; module comment states it is not derivable from Mathlib composition alone, but it is still a property claim, not an external primitive — candidate for future proof. |
| `prop_boundary.lean:52` | `manufactureProp` | b | Asserts an `ObsStar → AdmProp` mapping exists without construction; a property/relationship claim about the admission model, not an external-world primitive. |
| `refusal_noknhkpath.lean:39` | `knhkTransitiveDeps` | a | Stands in for the real `cargo tree` transitive-dependency set of a candidate crate; genuinely external build-tool output, not computable inside Lean. |
| `refusal_noknhkpath.lean:44` | `knhk_drags_disallowed_deps` | b | Asserts the intersection with disallowed deps is nonempty; a property claim about the (externally sourced) dependency set, theorem-shaped. |
| `refusal_nomeasuredticks.lean:46` | `declaredTicks` | a | Planner's own declared per-frame cost, computed from the plan by an external process; modeled as an opaque function rather than derived in Lean. (Line renumbered from `:39` after the `Frame`→`TickFrame` concept-identity rename added doc-comment lines; same axiom.) |
| `refusal_nomeasuredticks.lean:54` | `rdtscTicks` | a | Stands in for an actual hardware `rdtsc` cycle-counter reading; genuinely external (hardware timing), and per the no-wall-clock invariant deliberately never given a concrete value in-model. (Line renumbered from `:47`; same axiom.) |
| `def_walframe.lean:30` | `chainH` | a | Collision-resistant hash primitive (BLAKE3-style), signature `String → Digest`. Part of the still-open `chainH` collision (7 sites total: `def_walframe`, `con_merklecell`, `def_contentaddr`/`contentAddrChainH`, `def_receipt`, `con_commit`, `con_fablechain`, `Praxis/Mathlib/DefReceipt`) — the Concept Identity Report's `chainH` section recommends canonicalizing on one base axiom plus computable wrappers, but that merge has not been executed yet. Disclosed here as a known-open item, not silently hidden. |
| `con_merklecell.lean:40` | `chainH` | a | Same collision-resistant hash primitive, signature `List Digest → Digest` (Merkle aggregation variant). Same still-open `chainH` collision as above. |
| `def_contentaddr.lean:50` | `contentAddrChainH` | a | Same collision-resistant hash primitive, signature `ByteArray → Digest`, renamed from bare `chainH` during this session's `Frame`/`Receipt` cluster resolution to unblock an unrelated build break. Still part of the open `chainH` collision pending a full merge. |

## Regression gate

`just praxis-lean-axiom-gate` re-runs the grep above, drops the known prose false positives,
drops the 7 excluded/in-progress files, and diffs the remaining axiom names against this table.
It fails if the code contains any in-scope axiom name not listed here (a new, undisclosed
axiom). It does **not** fail on axioms already listed here, including class-(b) ones — the
point of this gate is disclosure of new axioms, not prohibition of documented ones.
