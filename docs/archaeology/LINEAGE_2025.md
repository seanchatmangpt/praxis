# LINEAGE_2025 — The Bit/Byte/CNS Fossil Record

Dossier of the 2025 bounded-actor lineage, compiled 2026-07-02 from a
read-only ten-agent sweep (5 explorers, 5 planners) over the local tree.
Every claim below names its evidence site. Verdict vocabulary:
**genuine** (enforcement read at the enforcement site), **partial**,
**theatre** (defective code path cited), **unverified**.

The organizing split this dossier establishes:

> **2025 built the particle. Praxis builds the court.**
>
> 2025 contained the physical runtime theory — the bounded, sealed,
> semantic execution particle. Praxis supplies the admissibility,
> verification, and derived-governance theory. Every forced 2025→praxis
> revision replaced a *chosen* thing with a *derived* thing.

---

## 1. Timeline

| Repo | Last commit | Tier | Role in the arc |
|---|---|---|---|
| `/Users/sac/seven_tick` | (no git) | docs | charter/context notes for the 7/8-tick doctrine |
| `/Users/sac/cns` | 2025-08-22 | C/Erlang/Elixir/TTL | seed: chatman nano stack, 8T/8H/8M trinity, OWL-AOT |
| `/Users/sac/bitstar` | 2025-08-20 | C/TTL | crystal envelope, BLAKE3, dispatch, 107-technique catalogue |
| `/Users/sac/bytestar` | 2025-10-29 | C/Erlang/TTL | productionized: bytecore ABI, 108 kernels, 42 NIFs |
| `/Users/sac/knhk` | 2026-06-03 | Rust/Erlang/TTL | genesis-* crates + custody/lockchain Erlang lineage |
| `/Users/sac/unibit`, `/Users/sac/bcinr` | 2026 | Rust | parallel Rust reimplementations |
| `/Users/sac/praxis` | current | Rust | the court: admission, receipts, derived supervision |

Corrections that matter for any future dig:
- `/Users/sac/bitactor` is a **build-artifact dump** (no git, checked-in
  `.o`/`.dSYM`/binaries) — not a source of truth; date by content only.
- knhk's `.rs` file count is inflated ~15× by `.claude/worktrees/`
  duplicates; the authoritative source is `rust/genesis-*`.

## 2. The recurring design (found four times)

One design, re-implemented across CNS → bitstar → bytestar → knhk:
a **tick-bounded (≤8), hop-bounded (≤8), 64-byte-crystal, hash-sealed,
ontology-compiled actor VM** with a typed refusal vocabulary.

Constant contracts, consistent across all four implementations:
`MAX_TICK_BUDGET 8`, `MAX_HOP_LIMIT 8`, `CRYSTAL_ENVELOPE_SIZE 64`
(`bitstar/bitactor/include/byteactor_core.h`), `BITACTOR_8T_MAX_CYCLES 8`
with `_Static_assert` size contracts (`cns` nano-stack headers), knhk
`CHATMAN_CONSTANT`. The refusal enums — `admission_reason_t`
(INVALID_MAGIC / INVALID_SIGNATURE / BOUNDS_EXCEEDED) and
`execution_status_t` (TICK_EXCEEDED / HOP_EXCEEDED / BUDGET_EXCEEDED) —
are a genuine typed-failure contract, not decoration.

**What was actually novel in 2025** (vs. ordinary actor theory): the actor
as a *measured particle of executable meaning* — cache-line physical
(64B), hard bounded action (≤8 ticks), a communication light cone
(≤8 hops, locally transported in the envelope), semantics manufactured
into execution (TTL/OWL → AOT C), declared meaning bound to runtime
(spec/exec/trace hash triple), bounded transport (SPSC, no unbounded
mailbox fiction), encoded failure, and honest FAIL artifacts.

## 3. Enforcement audit — genuine vs theatre

| Mechanism | Site | Verdict |
|---|---|---|
| Branchless tick trap (saturating table, trap-on-exceed) | `bitstar/.../techniques_v3/tick_hop_discipline/A25_TickBudgetCounter.c` | **genuine** — cleanest bounded-execution artifact in the tree |
| 108 productionized kernels, bytecore ABI (~50 headers) | `bytestar/byteactor/src/k*.c`, `bytestar/bytecore/abi/` | genuine tier (perf claims unverified) |
| SPSC Erlang bridge (Disruptor layout, real kernels included) | `bytestar/byteflow/c_src/byteflow_spsc_nif.c` (916 LOC) | **genuine** — the lineage's most credible bridge |
| Zero-tick elision + branchless dispatch | `bitstar/bitactor/src/bitactor_dispatch.c` | genuine mechanism; "zero cost" refuted by its own unit test (§5) |
| rdtsc on Apple Silicon | `cns` nano-stack `bitactor_80_20.c:34-76` | **theatre** — fabricated synthetic cycle counts; every macOS 8T claim self-fulfilling |
| ENTANGLE / COLLAPSE opcodes | nano-stack `bitactor_core.c` | **theatre** — printf stubs |
| Crystal-execute NIF "8-tick guarantee" | `bytestar/byteactor/erlang/c_src/byteactor_nif.c` | theatre (self-labelled simulation) |
| quantum / retrocausal / void_crystals / paradox headers | `bytestar/bytecore/abi/` fringe | design fiction; preserved as history only |

## 4. Provenance lineage (the seal)

| Generation | Mechanism | Defect |
|---|---|---|
| BitActor | `spec_hash` **asserted as a TTL literal**; XOR-threshold "verification" (`meta_probe.c:269`) | hash not derived from content — a forgery vector |
| ByteActor | real from-scratch BLAKE3 spec/exec/trace triple (`bitstar/bitactor/src/bitactor_blake3.c`) | acceptance by Hamming distance `popcount(spec⊕exec) < max_diff`; suspect multi-block finalize |
| CNS OWL-AOT | real rdflib reasoning → generated C with materialized closures (`cns/codegen/owl_aot_compiler.py`) | no hash-receipt layer (reproducibility by regeneration only) |
| praxis μ5 | strict-equality BLAKE3 + Ed25519 custody + receipted refusals | — |

The mathematics of why fuzzy acceptance had to die: under hash avalanche,
unequal inputs land at Hamming distance ~Binomial(256, ½) — any small ε
buys **zero semantic slack and pure false-accept surface**. Tolerance
belongs in canonicalization (before the hash), never in the codomain.
Strict equality is the ε→0 limit the geometry already imposed.

## 5. Measurement integrity register

**Honest failures (trust these — they are design law):**
- 426ns vs ≤8-tick target: `❌ FAIL` printed in its own results table
  (`bytestar/byteactor/BYTEACTOR_V2_VALIDATION_REPORT.md`).
- CNS 8T compliance **0/5**, integrated path **84 ticks vs 8 micro**
  (`cns/docs/COMPREHENSIVE_BENCHMARK_STATE_REPORT.md`) — the
  **composition penalty**: local boundedness does not compose; price the
  holonomy (roll-ups) or fuse at compile time.
- knhk "THE TRUTH" audit (`knhk/CRITICAL_AUDIT_FINDINGS.md`): engine 100%
  stubbed, `Receipt::default()` = cryptographically empty success — the
  refusal-to-lie artifact.
- The self-refuting zero-tick unit test
  (`cns/bitactor/tests/test_zero_tick_unit_validation.c`): a project-
  authored test that disproves its own marketing term. A bound must
  budget its own enforcement.

**Broken-by-construction (never cite as measurement):**
- Dummy-loop benchmark **template**
  (`cns/v1/core/weaver/templates/c_performance_contract.c.j2`) —
  the defect replicated into every generated contract benchmark.
- Min-of-samples PASS ("0 cycles ✓ PASS" while the same report's averages
  read 22.7 and 44 cycles) — `bitactor/cns/8T_FINAL_IMPLEMENTATION_REPORT.md`.
- "100% 8-tick compliance" computed over 14/16 **dead** combinations, with
  a tick silently redefined ×1000 (7,000ns called compliant) —
  `cns/bitactor/tests/BitStar_Combination_Test_Results.md`.
- Synthetic p95/p99 (`base_latency × variance × 2.5`) presented as SLA
  evidence — `cns/k8s_baseline_comparator.py`.

**The distilled failure classes** (now enforced against praxis's own
benchmarks by `crates/praxis-synthesis/tests/honesty_audit.rs` and the
reworked `tests/supervised_receipt.rs`):
1. min-of-samples verdicts (verdicts must be p99/worst-case),
2. dummy-loop harnesses (work product must be *verified*, not looped),
3. aggregate throughput masking per-op tails,
4. silent unit redefinition (ticks are declared op-counts, recomputed),
5. flattering artifacts (negative overhead beyond run spread **fails the
   harness** — triggered by the v1 receipt's own −2.68%),
6. asserted flatness of composition (the ratio is measured).

## 6. Supervision lineage

Classical side (cns/bitstar/bytestar): textbook hand-authored OTP —
strategy census across the tree: one_for_one ×613, one_for_all ×179,
rest_for_one ×113, simple_one_for_one ×76; restart-and-forget; failures
narrated, never classified into a decision surface (sole exception:
`bitstore_recovery_manager.erl`'s remediation taxonomy).

Divergent branch (knhk `genesis_custody` / `genesis_rc`): transient
actors whose state replays from an owned, Merkle-linked receipt log
(`genesis_custody_actor.erl`, `genesis_lockchain.erl` with associative
merge ⊕); restart intensity 1 — a restart storm is a recorded fault, not
absorbed churn. This is provenance-first custody: the structural
precondition for failure geometry, still without prediction or actuation.

The knhk gap — **classification without actuation** (MAPE-K observed,
never acted) — is what `praxis-synthesis` closed: derived
`SupervisionTopology` (strategies earned by dependency position;
OneForAll *inexpressible* — the absence is the theorem), `FailureGeometry`
with the intensional `GeometryGap` complement (unshadowable by
construction), `execute_supervised` (crashes as values, GaveUp lawful),
and the supervised cell (MAPE-K closed at epoch boundaries, quorum
quarantine, foreign verifier unmodified).

## 7. Terminology fossils

| Term | Origin | 2025 meaning | Praxis standing |
|---|---|---|---|
| tick / 8T | CNS | CPU cycles (aspirational; fabricated on ARM) | declared operation count, `CHATMAN_CONSTANT = 8`, no clock claim |
| hop / 8H | CNS | causal light-cone radius | bounded plan depth / hops ≤ 8 |
| crystal | bitstar | 64B cache-line envelope | not adopted as vocabulary; the *idea* lives as fixed-shape projections (status byte, [u32;8] tuples) |
| spec_hash/exec_hash | BitActor/ByteActor | asserted literal → real BLAKE3, fuzzy accept | strict-equality content addresses in receipts |
| zero-tick | bitstar | elision marketed as free (self-refuted) | refused — silent skipping is unauditable |
| ENTANGLE/COLLAPSE | nano-stack | printf stubs | banned from identifiers (theatre vocabulary) |
| lockchain | knhk | Merkle receipt merge ⊕ | cell roll-ups / fold_event chains |
| park | knhk | in-memory quarantine, no way back | WAL-durable `ParkManager` + `ReAdmission` |
| MAPE-K | knhk | classification without actuation | closed loop at epoch boundaries |

## 8. Import / refuse matrix (summary)

**Imported (with `PORT(...)` provenance headers):** TickBudget /
CHATMAN_CONSTANT (knhk timing.rs), RuntimeClass R1/W1/C1 + failure
actions, ParkManager + ReAdmission, admission-gate design, receipt-chain
semantics.

**Port-design (idea re-derived, no code):** hop budget (renamed,
de-theatred), spec/exec/trace triple under strict equality,
mailbox-replay custody, Merkle roll-up merge, OWL-AOT closure
materialization (future, Rust codegen).

**Refused (receipted reasons):** fuzzy hash acceptance;
`Receipt::default()` (success must not be default-constructible);
zero-tick elision; 2025 benchmark harnesses (fabrication instruments);
PQC claims (nothing real to import); C kernels as code (FFI vs
forbid-unsafe/zero-deps); theatre vocabulary in identifiers.

**Cross-cutting laws:** no asserted hashes; no default-constructible
success; **no performance claim without an anti-2025-theatre audit**;
theatre vocabulary banned.

## 9. Claim discipline

Strong (safe) claims:
- The 2025 work implemented a recurring bounded actor-particle design
  across C, Erlang/NIF, and Rust-adjacent systems, with consistent
  invariants: tick bounds, hop bounds, semantic compilation, sealed
  provenance, bounded communication.
- Praxis adds the missing admissibility layer: strict equality, foreign
  verification, unsat certificates, self-application, plan-derived
  topology, derived failure geometry, MAPE-K actuation closure.

Bounded claims:
- "The architecture *targets* trillion-agent composition by reducing each
  actor to bounded projections and gluing receipts" — production
  trillion-agent operation is not claimed. The overlap curve is a
  *measured curve* on one workload, not a law.

Refused claims (do not make):
- 2025 proved 8-tick execution generally; 2025 proved PQC security;
  2025 benchmarks are reliable; fuzzy hash acceptance is acceptable;
  phase change has occurred; **negative overhead is a success**.

## 10. Standing doctrine going forward

> **BitActor/ByteActor made agency small. Praxis makes agency
> admissible. The audit suite makes admissibility honest.**

No new performance claim ships without passing the honesty-audit suite;
the measurement harness self-refutes on flattering artifacts; the fossil
record above is the reason.
