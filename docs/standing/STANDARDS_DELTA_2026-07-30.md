# Seven-Day Standards Delta — 2026-07-30

## Standing

**PARTIAL_ALIVE** for repository-wide adoption.

The normative contract, hosted-agent transport contract, cognition contracts,
and structural drift checker are present on the adoption branch. Application
subsystems have not been individually migrated or reverified against every new
rule by this change.

## Window

This ledger covers standards observed or established from 2026-07-24 through
2026-07-30 in the active Praxis-adjacent repositories and the live operating
ontology used for this update.

## Observed source heads

| Repository | Observed commit | Relevant delta |
|---|---|---|
| `seanchatmangpt/wasm4pm` | `f1d4d7ac8b2f9a0265be82991487766eb35b4675` | Sole normative `AGENTS.md`; evidence-state separation; exact boundary execution; generated cognition surfaces; algorithm closure; multi-agent git discipline |
| `seanchatmangpt/wasm4pm` | `dfcf98f93e276f73e3ba438598fd0cddb57a7ab3` | Downstream cognition breed composition with per-breed, projection, duplicate-observation, multi-run-state, and typed-failure tests |
| `seanchatmangpt/wasm4pm` | `b74afe712331995419761aa56b64b019f81e2e07` | Combinatorial pattern language with explicit context, problem, forces, solution, falsifier, and composition rules |
| `seanchatmangpt/ggen` | `faa52dac474d456ae00105869770161d666ba31f` | Consolidated architecture, autonomics, self-hosting, and evidence-backed capability standing |
| `seanchatmangpt/ggen` | `2364ccda8d1a38f14e314365583c99f3fb81357d` | One receipted CI/CD release-law control plane and Gall-checkpoint roadmap |
| `seanchatmangpt/ggen` | `742dcb34b5e539137f89a99af8fa734b22b9aaac` | Replay-safe exact-head finalization |
| `seanchatmangpt/ggen` | `f8f8e6c7719f5c68f45e5a12a0df8ac4f6cbc602` | Bounded engine failure receipts |
| `seanchatmangpt/ggen` | `b4c0119e368f019b8f792b33dd501a93fffd8458` | Typed dry-run refusal ordering |
| `seanchatmangpt/ggen` | `28a545edf0fb8ec11bfb86cd938e229ba8caf427` | Logical time carried into POWL/OCEL records |

The source commits are provenance anchors, not dependencies and not claims that
all source-repository behavior was independently re-executed inside Praxis.

## Adopted standards

### 1. Normative authority

- `AGENTS.md` is the sole repository-wide agent contract.
- Path-specific `AGENTS.md` files may refine but not weaken root law.
- Compatibility files cannot silently fork policy.

### 2. Manufacturing calculus

```text
O → O* → A = μ(O*) → R → replay/hook
```

- `O*` must be admitted, aligned, grounded, and bounded.
- Actuation without a recomputable receipt is forbidden.
- Refusal remains refusal through every wrapper.

### 3. Standing lattice

The canonical states are:

```text
PARTIAL_ALIVE | ALIVE | BLOCKED | BUILD_BROKEN | UNKNOWN | UNSUPPORTED
```

Checkpoint evidence cannot promote crown closure. Unknown observations are not
admitted facts. Unsupported behavior is not misreported as failure.

### 4. Chesterton fence and falsifiers

Every replacement or refutation must preserve the strongest current statement,
identify the protected invariant, match the same objects and boundaries, state
exclusions, and provide an executable falsifier.

### 5. Exact repository transport

Resolve exact base SHA first. Use a transport ladder rather than treating one
failed clone or CLI edge as graph failure. Exact tree reconstruction is enough
for local verification; connector-backed blob/tree/commit/ref publication is a
lawful fallback.

### 6. Generated-surface governance

Generated files are projections of admitted sources. Change the source, run the
generator, prove idempotence, and preserve receipts. Hand-edited generated
standing or registries are refused.

### 7. Cognition composition

Observation, proposal, authority, projection, and actuation are separate
morphisms. Per-breed correctness does not prove composed-pipeline correctness.
Composition must verify deterministic ordering, obligation handoff, typed
failure propagation, multi-run isolation, projection stability, and replay.

### 8. Real-boundary evidence

Primary evidence paths cannot be replaced by mocks, fake telemetry, synthetic
OCEL, fabricated receipts, hardcoded success, or missing-fixture skips. Tests
must have teeth through mutation or disposable tamper rejection.

### 9. Gall checkpoints

Each increment must preserve a working receiptable system. Finalizers are
exact-head, replay-safe, idempotent, bounded in failure, and removed after their
one-use purpose is complete.

### 10. Release law

Capability and release standing are manufactured from exact package identity,
commit, artifacts, boundary execution, receipt verification, refusal ordering,
logical time, clean-consumer evidence where applicable, and controlled
actuation.

## Files introduced or changed

- `AGENTS.md`
- `CHATGPT-CLOUD-AGENTS.md`
- `.claude/rules/_core/absolute.md`
- `.claude/rules/cognition-contracts.md`
- `scripts/verify-agent-standards.sh`
- `.github/workflows/agent-standards.yml`

## Exclusions

This change does not claim:

- every existing Praxis subsystem already satisfies every adopted rule;
- all current builds and tests pass;
- generated surfaces have been regenerated;
- application behavior, release standing, or crown closure changed;
- source-repository commits were replayed inside this repository.

Those require subsystem-specific implementation and execution receipts.

## Falsifier

The structural checker must fail when a required contract, state-lattice term,
zero-unreceipted-actuation invariant, generated-surface law, cognition rule, or
workflow binding is removed or renamed. CI invokes the checker on relevant
policy changes.
