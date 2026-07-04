# The Nine Crates

The workspace root `Cargo.toml` declares its members in a single list:

```toml
[workspace]
resolver = "2"
members = [
    "crates/agent8",
    "crates/ggen",
    "crates/chatman-common",
    "crates/praxis-core",
    "crates/praxis-proposer",
    "crates/praxis-retrofit",
    "crates/rust-fable-testbed",
    "crates/powl2-decompose",
    "crates/pddl-index",
    "crates/praxis-synthesis",
]
```

(`/Users/sac/praxis/Cargo.toml:165-178`)

That is nine crates, plus the workspace root package itself (`my-conforming-project`, `/Users/sac/praxis/Cargo.toml:1-13`). What follows is one description and one verified, cited fact per crate, in the order they appear in the `members` list.

## `crates/agent8`

Description: "8-bit agent projection + 64-byte wire ABI (Env64/Pulse64) + branchless fleet kernel." (`crates/agent8/Cargo.toml:7`)

`AgentByte` is a `#[repr(transparent)]` wrapper around a `u8`, with eight named single-bit constants: `ADMITTED` (`0x01`), `EVIDENCE_OK` (`0x02`), `WITHIN_BUDGET` (`0x04`), `AUTHORITY_BOUND` (`0x08`), `HEALTHY` (`0x10`), `CONFORMANT` (`0x20`), `RECEIPTED` (`0x40`), `REPLAYABLE` (`0x80`) (`crates/agent8/src/byte.rs:48-70`). Its `GRANT_REQUIRED` constant is defined as the OR of six of those eight bits — `ADMITTED | EVIDENCE_OK | WITHIN_BUDGET | AUTHORITY_BOUND | CONFORMANT | RECEIPTED`, i.e. `0x6F` — deliberately excluding `HEALTHY` and `REPLAYABLE` as advisory/post-hoc signals (`crates/agent8/src/byte.rs:72-88`).

## `crates/ggen`

Description: "SPARQL-in-Tera code generation" (`crates/ggen/Cargo.toml:8`).

`crates/ggen/src/pack.rs` documents a pack as a directory containing `pack.toml`, `ontology.ttl`, and a `templates/` directory of `*.tmpl` files, resolved fail-closed: a missing pack directory, missing manifest, missing ontology, unknown manifest keys, or an empty template set all refuse by name with an `FM-PACK-*` code (`crates/ggen/src/pack.rs:1-7`). Its `content_hash` function computes a deterministic BLAKE3 hash over the pack's ontology and templates as sorted `(relative_path, bytes)` pairs, and is defined at `crates/ggen/src/pack.rs:177`. The on-disk `pack.toml` schema is deserialized with `#[serde(deny_unknown_fields)]`, i.e. unknown keys in a pack manifest are a hard error rather than silently ignored (`crates/ggen/src/pack.rs:37-39`).

## `crates/chatman-common`

Description: "Shared house crate for the seanchatmangpt Rust fleet" (`crates/chatman-common/Cargo.toml:9`).

Its `signed_receipt` module wraps a BLAKE3 chain hash with an ed25519 signature. The signing key is loaded, in priority order, from the `PRAXIS_SIGNING_KEY` environment variable (64 lowercase hex chars — 32 bytes of the ed25519 secret seed) or from a file path given in `PRAXIS_SIGNING_KEY_FILE` (`crates/chatman-common/src/signed_receipt.rs:1-11`). The module is compiled only when the `signed-receipts` feature is enabled (`crates/chatman-common/src/signed_receipt.rs:13-15`).

## `crates/praxis-core`

Description: "Fused Law Object abstraction: obligation + lifecycle + receipt + OCEL." (`crates/praxis-core/Cargo.toml:6`).

`praxis-core/src/refusal.rs` defines `RefusalCategory`, an 8-bucket refusal classification: `Identity`, `Capacity`, `Topology`, `Temporal`, `Lifecycle`, `Authorization`, `Prerequisites`, `Reserved` (`crates/praxis-core/src/refusal.rs:41-59`). The module doc states this design was "ported from stpnt ... as prior art; stpnt is not a dependency" (`crates/praxis-core/src/refusal.rs:40-41`).

## `crates/praxis-proposer`

Description: "PR-14 proposer layer: ranks candidate goal states from a domain-authored objective function. Proposals are untrusted observations (O, not O*) per AR-9." (`crates/praxis-proposer/Cargo.toml:6`).

`objective.rs` fixes the scoring vocabulary as a 4-element array, `FLUENT_NAMES: [&str; 4] = ["realized_revenue", "pipeline_value_at_risk", "time_penalty", "stage_advance"]`, and the module doc states scoring iterates this array (never a map) so floating-point summation order is deterministic (`crates/praxis-proposer/src/objective.rs:39-45`). The weights themselves are author-supplied JSON (`revenue_objective.json`), not discovered or invented by the crate — the module doc cites "Vision 2030 Non-goal 1 ('No value discovery')" as the reason (`crates/praxis-proposer/src/objective.rs:1-8`).

## `crates/praxis-retrofit`

Description: "Retrofit automation tool: Apply praxis standards across Rust ecosystem" (`crates/praxis-retrofit/Cargo.toml:8`).

`ci_gate.rs` defines a `GateConfig` struct for CI/CD compliance gating, with fields `min_score: f32`, `block_on_drop: bool`, `critical_categories: Vec<ComplianceCategory>`, `auto_remediate: bool`, and `generate_badge: bool` — i.e. a PR can be blocked on a numeric compliance-score threshold and/or on specific categories failing (`crates/praxis-retrofit/src/ci_gate.rs:13-25`).

## `crates/rust-fable-testbed`

Description: "Deterministic Rust-eval testbed for Claude models: RDF/Turtle task specs -> compiled prompts -> sandboxed cargo verification -> BLAKE3-chained receipts." (`crates/rust-fable-testbed/Cargo.toml:8`).

`receipt.rs` documents its ledger as "a flat JSONL append log, not the full `law.rs` Raw→Validated→Admitted→Receipted lifecycle (a deliberate v1 simplification per the plan; `law.rs`'s receipt pattern is inspiration only, this chain is new code)" (`crates/rust-fable-testbed/src/receipt.rs:1-4`). Its `TestbedReceipt` struct chains entries via `chain_hash = blake3(prev_chain_hash || json(task_id, prompt_hash, model, metrics_summary, prev_chain_hash))`, with the genesis entry's `prev_chain_hash` being `"0".repeat(64)` (`crates/rust-fable-testbed/src/receipt.rs:16-29`).

## `crates/powl2-decompose`

Description: "Kourani et al. Stage-1 decomposition: safe & sound WF-nets -> POWL 2.0. Separability is the admission predicate; non-separable nets are refused with a receipt." (`crates/powl2-decompose/Cargo.toml:7`).

The module doc for `decompose.rs` states that Algorithm 3's fall-through branch — "neither a base case, nor a conflict-hiding partition, nor a concurrency-hiding partition exists" — is not approximated but refused, "emitting a `Refusal` carrying a machine reason and a BLAKE3 receipt over the offending (sub-)net" (`crates/powl2-decompose/src/decompose.rs:1-18`). It further notes that "every separable net is free-choice" (a corollary of Def 3.13), so non-free-choice nets are refused up front (`crates/powl2-decompose/src/decompose.rs:16-18`).

## `crates/pddl-index`

Description: "Dictionary-encoded, XOR-filter-pruned lazy grounder for PDDL8 (the qlever treatment): grounding-as-join over compact u32 ID space, materializing only reachable ground actions." (`crates/pddl-index/Cargo.toml:7`).

Its `xorf.rs` module implements a XOR filter — an approximate-membership structure with no false negatives — described as "ported (design, not code)" from `bytestar/bytecore/abi/tables.h`'s `bs_xorf_maybe_has` function, citing Graf & Lemire's 2020 paper "Xor Filters: Faster and Smaller Than Bloom and Cuckoo Filters" (`crates/pddl-index/src/xorf.rs:1-9`). The module doc explains the filter's role: the sorted fact store is the authoritative membership answer, and the XOR filter is "the cheap gate in front of it" so the store is "only touched for atoms that *might* exist," with no false negatives keeping the gate sound (`crates/pddl-index/src/xorf.rs:11-19`).

## `crates/praxis-synthesis`

Description: "Prototype synthesis of the deep-research stack: semi-naive Datalog saturation (Nemo), bounded constraint-driven capability sequencing (SMT lesson), content-addressed DAG execution (OxyMake), and refinement-style admission (Flux) — pure Rust, bounded, receipted." (`crates/praxis-synthesis/Cargo.toml:7`).

`datalog.rs` defines `pub const MAX_TUPLES: u64 = 100_000_000;` (`crates/praxis-synthesis/src/datalog.rs:40`) and `pub const MAX_STRATA: usize = 8;` (`crates/praxis-synthesis/src/datalog.rs:44`). `MAX_TUPLES` is used to size the relation store's initial capacity (`crates/praxis-synthesis/src/datalog.rs:171`) and to cap EDB fact insertion, per the doc comment "Refuses past `MAX_TUPLES` or arity cap" (`crates/praxis-synthesis/src/datalog.rs:226`). `MAX_STRATA` is checked during stratification and, if a predicate's stratum count reaches it, the code refuses with the message `"predicate {} exceeds MAX_STRATA ({MAX_STRATA}) — negation cycle"` (`crates/praxis-synthesis/src/datalog.rs:340-343`).

---

## `praxis-reconciler` is not a workspace member

There is a directory at `/Users/sac/praxis/crates/praxis-reconciler/` (`crates/praxis-reconciler/`), but it contains no `Cargo.toml` — only a `src/` subdirectory holding a single file, `loop_logic.rs`. Because it is not listed in the workspace `members` array (`/Users/sac/praxis/Cargo.toml:167-178`) and has no manifest of its own, it is not part of the nine crates above; `cargo build --workspace` never touches it.

`src/loop_logic.rs` opens with:

```rust
use genesis_types_v2::{
    BoundedRepairOperator, Error, RepairAdmissionReport, ResidualVector, Result, VisualGapReport,
};
```

(`crates/praxis-reconciler/src/loop_logic.rs:1-3`)

No crate named `genesis-types-v2` (or `genesis_types_v2`) exists anywhere in this filesystem's crate roots — it is absent from `/Users/sac` at the depth searched, and it is not a dependency declared anywhere in this workspace. `praxis-reconciler` is therefore dead source: a file referencing a nonexistent crate, sitting in a directory with no manifest to even attempt compiling it.

**Do not confuse this with the real, working tool of a similar name at `playground/tools/praxis-reconciler/`.** That directory has its own `Cargo.toml` — package `praxis-reconciler`, version `0.1.0`, edition 2021, depending on `clap` (with the `derive` feature), `notify` 6.1.1, and `anyhow` 1 (`/Users/sac/praxis/playground/tools/praxis-reconciler/Cargo.toml:1-9`). It is a standalone, independently-buildable playground tool, unrelated to the abandoned `crates/praxis-reconciler/` source file beyond sharing a name.
