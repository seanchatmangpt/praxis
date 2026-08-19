# PROJ-815: Document the POWL 3-way fork and cross-reference the two quarantine doctrines

**Status**: DONE — `docs/standing/POWL_IMPLEMENTATIONS.md` added; cross-reference doc comments
added to both `crates/praxis-synthesis/src/quarantine.rs` and
`crates/praxis-graphlaw/src/chatman/quarantine.rs` (no logic changed; `just fmt-check` clean
for both files).
**Dependencies**: none (docs-only, no build gate required)

## Scope

Two zero-code-risk documentation items from the chatman cross-analysis, bundled because both are
"make existing drift visible, don't change behavior" moves:

### 1. POWL 3-way fork

The same POWL 2.0 structure (Leaf/PartialOrder/ExternalCut, Kourani et al. Def 3.7) is modeled
three times:

- **`powl2_decompose::Powl`** (`powl.rs:94`) — canonical for chatman, imported directly by
  `crates/praxis-graphlaw/src/chatman/powl_projection.rs:26`.
- **`cng::powl::Powl`** (`crates/cng/src/powl.rs:19`) — a disclosed clean-room duplicate,
  because `cng` doesn't depend on `praxis-graphlaw`. Same O(n²) transitive-closure algorithm
  maintained twice (`cng/src/powl.rs:822-850 project_tape_to_powl` vs.
  `powl_projection.rs:45-75 project_pddl_tape_to_powl`).
- A `wasm4pm-compat` POWL type and a Lean 4 formalization in `~/mfact`, both named as
  unreconciled in `powl2-decompose.rs:11-22` but not independently re-verified this session.

**Action**: write `docs/standing/POWL_IMPLEMENTATIONS.md` naming all of the above, which is
canonical-for-chatman, which is a disclosed fork and why, and which two are known-but-out-of-
scope (cross-repo). This closes a knowledge gap without touching any code — the next person who
finds `cng::powl::Powl` doesn't have to re-derive that it's a deliberate fork.

### 2. Quarantine doctrine cross-reference

`praxis-synthesis::quarantine.rs` (`RiceQuarantine`/`Admission`/`Origin`, lines ~27,50,114) and
`praxis-graphlaw::chatman::quarantine.rs` independently implement the same named "Rice/
decidable-checks-only" doctrine with no shared trait or type. The chatman review classified this
as the one true drift pattern in this ecosystem review — not self-disclosed as intentional the
way the other overlaps were.

**Action**: add a one-line doc comment at the top of each file pointing at the other, naming
both as independent implementations of the same doctrine, e.g.:
```rust
// NOTE: praxis-graphlaw::chatman::quarantine implements the same "Rice/decidable-checks-only"
// doctrine independently — no shared trait/type exists between the two as of PROJ-815.
// See docs/jira/v26.8.16/tickets/PROJ-815.md.
```
This does not fix the drift (that's the medium-risk "shared AdmissionDoctrine trait" item the
chatman review deferred, not part of this milestone) — it makes the drift visible to the next
reader without changing behavior.

## Verification plan

Doc/comment-only change. `just fmt-check` is the only gate that could plausibly matter (comment
syntax), and is independent of PROJ-811's blocker since no `cargo check`/`test` is required for
this ticket to land safely.
