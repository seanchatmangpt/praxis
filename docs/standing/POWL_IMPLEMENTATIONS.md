# POWL 2.0 Implementations — the 3-way fork

v26.8.16 · PROJ-815

## Summary

The same POWL 2.0 structure (Leaf / PartialOrder / ExternalCut, Kourani et al. Def 3.7) is
modeled independently in at least three places in this ecosystem. This document names each,
states which is canonical for the Chatman Engine, which is a disclosed clean-room fork and why,
and which two are cross-repo and out of scope for reconciliation in this milestone.

## 1. `powl2_decompose::Powl` — canonical for chatman

`crates/powl2-decompose/src/powl.rs:94`. Defined in the `powl2-decompose` crate, whose
`lib.rs` doc comment describes it as a faithful implementation of Kourani, Park & van der
Aalst's recursive WF-net → POWL 2.0 decomposition (arXiv:2602.15739), admitting only the
**separable** class and refusing (typed `Refusal`, Rice-style) everything else.

This is the type imported directly by
`crates/praxis-graphlaw/src/chatman/powl_projection.rs:26`:

```rust
use powl2_decompose::{validate_external_cut, ExternalCutRefusal, GNode, Powl, SocketPath};
```

`powl_projection.rs`'s own doc comment states it "match[es] crates/powl2-decompose's Powl
type" — this is the canonical-for-chatman implementation.

## 2. `cng::powl::Powl` — disclosed clean-room duplicate

`crates/cng/src/powl.rs:19`. The `cng` crate's module doc comment self-discloses this as a
"[c]lean-room implementation of the invariants proven in the praxis test surface
(`chatman_pddl_to_powl_*`)" — the fork exists because `cng` does not depend on
`praxis-graphlaw` (`cng` is a standalone noun-verb CLI; see
`~/.claude/projects/-Users-sac-praxis/memory/cng-cli.md`), so it re-derives the same POWL 2.0
subset (`Leaf` / `PartialOrder` / external cut) rather than taking a cross-crate dependency.

Both implementations maintain the **same O(n²) transitive-closure algorithm** independently:

- `cng::powl::project_tape_to_powl` (`crates/cng/src/powl.rs:822-850`)
- `powl_projection::project_pddl_tape_to_powl` (`crates/praxis-graphlaw/src/chatman/powl_projection.rs:45-75`)

Both build one `Leaf(Some(label))` per tape op and then explicitly compute the full transitive
closure of the total order (`for i in 0..n { for j in (i+1)..n { order.insert((i, j)); } }`),
each with its own doc comment justifying the same O(n²)-in-output-size complexity bound. This is
a disclosed, intentional fork (cng's own doc comment says so) — not silent drift — but it is
still two hand-maintained copies of one algorithm, in case a future correctness fix to one is
needed in the other.

## 3. wasm4pm-compat POWL type and the Lean 4 formalization in `~/mfact` — cross-repo, unreconciled, not independently re-verified

`crates/powl2-decompose/src/powl.rs:111-122`'s doc comment on the `Powl` enum itself names a
third, structurally different formalization:

> `~/mfact/procint/ProcInt/Models/Powl.lean` also defines a Lean `Powl` inductive type, but by
> its own doc comment it formalizes a *different* source: Kourani and van Zelst, BPM 2023,
> Definitions 1–2 — the original tree-structured POWL (`atom` / `silent` / `xor` / `loop` /
> `po`), not the choice-graph-based POWL 2.0 this enum implements.

Per that same comment, the two are not structural analogs: `powl2_decompose::Powl::Choice`
routes through a `ChoiceGraph` (exclusive paths *and* cycles), while the Lean formalization's
exclusive choice is an n-ary tree node (`xor`, arity ≥ 2) with a dedicated `loop (doP redoP :
Powl α)` constructor rather than a graph cycle.

Separately, `powl_projection.rs`'s module doc comment references a `wasm4pm-compat` POWL
vocabulary (`wasm4pm-compat/ontologies/powlv2.ttl`) used for Turtle serialization/admission.
`wasm4pm-compat` is a path dependency resolved outside this repo (`../wasm4pm-compat`, per
`crates/praxis-graphlaw/Cargo.toml:46` and root `Cargo.toml:110,221`) — its own POWL-related
type(s), if any, live in that external tree.

**Both of these (the `~/mfact` Lean formalization and the wasm4pm-compat vocabulary/type) are
named here as unreconciled with the two Rust implementations above, per the source doc
comments cited.** Neither was independently re-verified against current `~/mfact` or
`../wasm4pm-compat` source during this session — this document states what the cited praxis
repo doc comments already say, not a fresh audit of those external trees. Reconciling all three
(or four) representations under one shared trait/type is out of scope for this ticket and this
milestone.

## What this closes

The next person who encounters `cng::powl::Powl` and wonders whether it's an accidental fork of
`powl2_decompose::Powl` (canonical for chatman) doesn't need to re-derive the answer: it's a
disclosed, intentional clean-room duplicate, for the dependency reason stated above, maintaining
the same O(n²) algorithm twice. The `~/mfact` Lean type and the wasm4pm-compat type are known,
named, and out of scope — not silently forgotten.

## See Also

- `crates/powl2-decompose/src/lib.rs` — canonical decomposition crate, Rice-style admission
- `crates/powl2-decompose/src/powl.rs:94-122` — `Powl` enum and its own cross-reference to the
  `~/mfact` Lean formalization
- `crates/praxis-graphlaw/src/chatman/powl_projection.rs` — chatman's projection, imports
  `powl2_decompose::Powl` directly
- `crates/cng/src/powl.rs` — cng's disclosed clean-room duplicate
- `docs/jira/v26.8.16/tickets/PROJ-815.md` — the ticket this document fulfills
