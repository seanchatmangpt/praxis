# Praxis v26.7.6 "After Neon" — Ecosystem Inventory

Snapshot date: 2026-07-05/06. Every classification cites a path and, where a repo
exists, its last commit date (`git -C <dir> log -1 --format=%ci`). Classifications:
ALIVE / PARTIAL / BROKEN / STALE / DUPLICATE / MISSING / OUT_OF_SCOPE / UNKNOWN.

## Repositories

| Surface | Path | Last commit | Class | Evidence |
|---|---|---|---|---|
| praxis (this workspace) | `/Users/sac/praxis` | 2026-07-05 18:27:40 -0700 | ALIVE | `cargo check --workspace` exit 0 (warnings only) on 2026-07-06 |
| roxi (fork of pbonte/roxi) | `/Users/sac/roxi` | 2026-07-05 22:46:50 -0700 | ALIVE → REFERENCE-ONLY after adoption | Native N3/Datalog/SPARQL 1.1/SHACL/ShEx engine, 29,625 LOC Rust (`find … wc -l`); lib crate at `roxi/lib` with `[lib] name = "minimal"` (`lib/Cargo.toml:33`). Being adopted into praxis as `crates/praxis-graphlaw`; once the clean-room adoption lands, mark this repo REFERENCE-ONLY |
| ggen (standalone repo) | `/Users/sac/ggen` | 2026-07-03 22:43:34 +0000 | OUT_OF_SCOPE | Frozen by user decision. Coupling to remove: praxis root `Cargo.toml:80` still carries `ggen-graph = { path = "../ggen/crates/ggen-graph", optional = true }` (feature `ggen`, `Cargo.toml:52`) |
| bcinr | `/Users/sac/bcinr` | 2026-07-03 14:54:55 -0700 | ALIVE | praxis deps: bcinr-powl-receipt 26.6.24 (`Cargo.toml:96`), bcinr-pddl 26.6.26 (`:97`), bcinr-powl 26.6.25 (`:100`) |
| wasm4pm (canonical) | `/Users/sac/wasm4pm` | 2026-07-05 20:55:07 -0700 | ALIVE | praxis deps: prolog8 26.7.1 (`Cargo.toml:101`), wasm4pm-cognition 26.7.1 (`:103`), wasm4pm-planner 26.7.1 (`:116`) |
| wasm4pm-compat | `/Users/sac/wasm4pm-compat` | 2026-07-02 00:25:03 -0700 | ALIVE | praxis dep wasm4pm-compat 26.6.29 (`Cargo.toml:102`) |
| wasm4pm_copy | `/Users/sac/wasm4pm_copy` | — | DUPLICATE | Duplicate of canonical `/Users/sac/wasm4pm`; path exists (ls verified) |
| wasm4pm-wt-p1..p4 | `/Users/sac/wasm4pm-wt-p{1,2,3,4}` | — | DUPLICATE | Worktree-style duplicates of canonical wasm4pm; all four paths exist |
| dev/wasm4pm | `/Users/sac/dev/wasm4pm` | — | DUPLICATE | Duplicate of canonical wasm4pm |
| chatmangpt/{wasm4pm,pictl,pm4wasm} | `/Users/sac/chatmangpt/…` | — | DUPLICATE | All three paths exist; duplicates of the canonical wasm4pm family |
| tower-lsp-max | `/Users/sac/dev/tower-lsp-max` | — | MISSING | Broken symlink → `/Users/sac/tower-lsp-max` (target absent). Nearest relatives: `/Users/sac/tower-lsp-composition`, `/Users/sac/lsp-types-max` (both exist). Note: praxis `Cargo.toml:153` records a patch because bcinr-pddl-lsp hardcodes `lsp-max = { path = "../../../lsp-max" }` |

## Praxis workspace crates (`/Users/sac/praxis/crates/`)

Classified by lib.rs header inspection + workspace membership (`Cargo.toml:168-178`).

| Crate | Class | Evidence |
|---|---|---|
| agent8 | ALIVE | `src/lib.rs`: "8-bit agent projection plus the 64-byte wire ABI"; workspace member; optional root dep (`Cargo.toml:89`) |
| chatman-common | ALIVE | Shared house crate (errors, telemetry, provenance, test infra); workspace member; frozen-dep of praxis-synthesis |
| ggen (crate, distinct from ~/ggen repo) | ALIVE | `src/lib.rs`: `#![deny(clippy::print_stdout)]`, `#![deny(unsafe_code)]`; workspace member; primary factory surface for this release |
| ocel | STALE (data-only) | Not a crate: directory holds `anti_llm_cheat_lsp_ocel.json`, `.receipt.json`, `ocel_gap_report.md`; no `src/`, not a workspace member |
| pddl-index | ALIVE | Dictionary-encoded lazy grounding for PDDL8; workspace member, version 26.7.2 (`Cargo.toml:98`); frozen-dep of praxis-synthesis |
| powl2-decompose | ALIVE | Kourani Stage-1 WF-net → POWL 2.0 decomposition; workspace member |
| praxis-core | ALIVE | "Fused Law Object abstraction: obligation + lifecycle + receipt + OCEL"; `#![deny(unsafe_code)]`, `#![warn(missing_docs)]`; workspace member (`Cargo.toml:88`) |
| praxis-lean | ALIVE | Lean 4/Lake deterministic admission-authority wrapper; workspace member (`Cargo.toml:178`); gauge surface for this release |
| praxis-proposer | ALIVE | PR-14 proposer layer (lawful candidate goal states); workspace member, optional root dep (`Cargo.toml:90`) |
| praxis-reconciler | PARTIAL | Only `src/loop_logic.rs`; NOT in workspace members list (`Cargo.toml:168-178`); not built by `cargo check --workspace` |
| praxis-retrofit | ALIVE | House-style standardization retrofits; workspace member (`Cargo.toml:91`) |
| praxis-synthesis | ALIVE | Bounded, receipted deep-research pipeline; deps frozen to pddl-index, chatman-common, blake3, serde, serde_json, thiserror (enforced by `tests/no_llm_runtime.rs`) |
| rust-fable-testbed | ALIVE | Deterministic Rust-eval pipeline, RDF/Turtle task specs; workspace member, optional root dep (`Cargo.toml:92`) |

## PROJ-305 surface

- `docs/v26.7.3/COGNITIVE_BREED_MAPPING.md` EXISTS (ls verified 2026-07-06).
- `crates/praxis-synthesis/src/breeds.rs:15` defines `pub const BREED_MODULE_MAP: &[(&str, &str)]`, with a test iterating it (`breeds.rs:28-33`).

## Known couplings to resolve in v26.7.6

1. Remove/replace optional `ggen-graph` path dep on frozen `~/ggen` (`Cargo.toml:52,80`) — target replacement: `crates/praxis-graphlaw` (roxi adoption).
2. `lsp-max` hardcoded-path patch (`Cargo.toml:153`) tied to the MISSING tower-lsp-max lineage.
3. `praxis-reconciler` is orphaned from the workspace — adopt into members or fold `loop_logic.rs` elsewhere.
