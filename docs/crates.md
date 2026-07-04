# Crate Modules (`crates/`)

The core functionality of Praxis is split across modular Rust crates. This separation of concerns enforces clear logical boundaries between typestates, agent projections, planning engines, and proposal mechanisms.

## Crate Directory Index

All crates live in the root directory under [`crates/`](file:///Users/sac/praxis/crates/).

| Crate Name | Path | Primary Purpose |
|---|---|---|
| **[`praxis-core`](crate_praxis_core.md)** | `/crates/praxis-core` | State machines, typestate lifecycles, and obligation validation. |
| **[`agent8`](crate_agent8.md)** | `/crates/agent8` | 8-bit status byte fleet projection and SWAR popcount kernels. |
| **[`praxis-proposer`](crate_praxis_proposer.md)** | `/crates/praxis-proposer` | Proposal generation engines (such as revenue and goal planners). |
| **[`chatman-common`](crate_chatman_common.md)** | `/crates/chatman-common` | Deterministic BLAKE3 hashing, error models, and testing suites. |
| **[`praxis-synthesis`](crate_praxis_synthesis.md)** | `/crates/praxis-synthesis` | Conformance reports and graph assembly validators. |
| **[`praxis-retrofit`](crate_praxis_retrofit.md)** | `/crates/praxis-retrofit` | CLI helpers to retrofit existing projects to Praxis. |
| **[`praxis-reconciler`](crate_praxis_reconciler.md)** | `/crates/praxis-reconciler` | State sync checkers. |
| **[`powl2-decompose`](crate_powl2_decompose.md)** | `/crates/powl2-decompose` | POWL workflow decomposition routines. |
| **[`pddl-index`](crate_pddl_index.md)** | `/crates/pddl-index` | Bounded planning indices and cached states. |
| **[`rust-fable-testbed`](crate_rust_fable_testbed.md)** | `/crates/rust-fable-testbed` | Allegorical testbeds and mock simulations. |

---

## Technical Details

### `praxis-core`
The central kernel. It compiles the `LawObject<State>` struct which implements typestate transitions (e.g., `Raw` to `Validated`). It is also responsible for executing the Rice Quarantine rules on incoming observation JSON payloads.

### `agent8`
Handles massive fleet state coordination. The status of any given agent is projected into a single byte (`PRCHUBEA`), where bits correspond to:
- **P**: Replayable
- **R**: Receipted
- **C**: Conformant
- **H**: Healthy
- **U**: Authority
- **B**: Budget
- **E**: Evidence
- **A**: Admitted
This crate implements high-performance SWAR (SIMD within a register) popcount routines to compute health statistics over entire arrays of agent bytes.
