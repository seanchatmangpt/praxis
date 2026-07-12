# Rail A/B Status — External Cut Projection and AIR Compilation

Updated 2026-07-11 (superseding an earlier same-day version of this doc — see the "Update"
section at the bottom; do not trust anything in this file that isn't in that section without
re-checking). Reconciles `PRD.md` sections 7.4 ("Layer 4 — External Cut Projection"),
7.5 ("Layer 5 — Arazzo"), 7.6 ("Layer 6 — wasm4pm AIR Compiler"), and section 22 Rail A
("Projection: POWL external cut -> SPARQL render model -> Tera -> receipted Arazzo") and
Rail B ("Compilation: Arazzo -> wasm4pm parser -> AIR -> typed refusals") against what is
actually on disk. Status vocabulary per `.claude/rules/no-overclaiming.md`.

## Rail A — Projection: **PARTIAL, scaffolding only**

Commit `b404c53e` ("feat: PRD v26.7.11 Rail A - POWL external cut projection and multifractal
execution logic") is the only commit claiming this. What it actually added:

- `crates/powl2-decompose/src/powl.rs:65-72` — a new `Powl::ExternalCut{region, projection,
  renderer}` enum variant. Typed, real, compiles. Holds the projection/renderer as opaque
  `String`s.
- `crates/powl2-decompose/src/external_cut.rs` (new, 156 lines) —
  `validate_external_cut(cut: &Powl) -> Result<(), ExternalCutRefusal>`. **Structural
  validation only**: checks `projection`/`renderer` strings are non-empty and the nested
  `region` is a non-empty admitted shape. Does not parse the SPARQL text. Does not invoke
  Tera. Does not check the SPARQL is even syntactically valid.
- `crates/praxis-graphlaw/src/chatman/powl_projection.rs:310-325` (in `emit_powl_node`) —
  when it sees `ExternalCut`, emits three TTL triples that **echo the raw strings as
  literals** (`powl2:sparqlProjection "<projection text>"`,
  `powl2:teraRenderer "<renderer text>"`). No SPARQL execution against any graph.
- `crates/praxis-core/src/arazzo.rs` (new, 81 lines) — `ArazzoProjectionReceipt` /
  `compute_digest()`. Takes seven **already-computed** hex-digest strings and BLAKE3-hashes
  them. Nothing in the repo constructs a populated instance outside its own unit test — no
  real caller supplies those seven digests from an actual SPARQL/Tera run.
- `crates/wasm4pm-arazzo/queries/render_model_projection.rq` (new, 47 lines) — a real,
  well-formed SPARQL SELECT (POWL element -> flattened relational row, deterministic
  `ORDER BY`). **Unwired**: `grep -rn "render_model_projection"` finds only this file itself.
  No Rust code references it.
- `tera` is a real workspace dependency but is only used by `crates/ggen` (a general-purpose
  templating tool, unrelated to this pipeline). No code path renders `Powl::ExternalCut`'s
  `renderer` string through a Tera engine.

**What Rail A actually requires** (`PRD.md:294-314`): \(A_z = T(Q(W))\) — a declared SPARQL
query \(Q\) executed against the admitted region \(W\), rendered through a declared template
\(T\), producing a receipted Arazzo document. **None of the three stages (query execution,
template rendering, Arazzo emission) exists.** What exists is the typed *contract* for
carrying those three declarations plus a receipt shape that assumes they already ran.

Status: **PARTIAL** — the type system and validation shell are real; the actual computation
(\(Q\), \(T\), and their composition into \(A_z\)) is not implemented anywhere.

## Rail B — Compilation: **PARTIAL, disconnected halves**

- **Arazzo parsing (Layer 5) — PARTIAL/ALIVE for the parsing step itself**: real Arazzo 1.1.x
  document parsing and URI resolution exist (`crates/wasm4pm-arazzo/src/parse.rs`,
  `resolve.rs`), fixed this session for a determinism bug and silent step-data loss (see
  git history on `crates/wasm4pm-arazzo/src/compile.rs`, `temporal.rs`, `normalizer.rs`).
- **The bridge from a parsed document to AIR does not exist.** No function anywhere converts
  `wasm4pm_compat::arazzo::ArazzoDescription` into `wasm4pm-arazzo`'s own `air::AirProgram`.
  The two type families never meet outside hand-built test fixtures.
- **AIR compilation to an executable form does not exist.** `AirCompiler::compile_to_wasm`
  (fixed this session to be deterministic and to stop dropping step data) produces a
  placeholder WASM module — no host imports, no HTTP dispatch, no execution semantics. Its
  own doc comment (as of this session's fix) states this plainly and cites the PRD gap
  directly.
- **Zero downstream consumers.** No other crate in the workspace depends on
  `wasm4pm-arazzo`. There is no WASM runtime anywhere in the repo (`wasmtime`/`wasmer`/etc.)
  to execute compiled output even if it were semantically real.

Status: **PARTIAL, blocked on two missing bridges** — (1) `ArazzoDescription -> AirProgram`
lowering, (2) `AirProgram -> executable-with-real-semantics`, neither of which is a small
patch; both are the actual substance the PRD requires from this layer.

## Layer 7-10 (Erlang transition core, OTP, AtomVM, BCINR)

Out of scope for this document — see the ticket-drafting workflow's Rail C/D/E findings for
those layers. Per `PRD.md:1083`, "No later rail SHALL be used to backfill authority missing
from an earlier rail" — since Rail A/B are PARTIAL, any Rail C/D/E ticket must be scoped as
depending on Rail A/B completion, not as independently startable.

## Update (later the same session): everything above is now the "before" state

Sections above describe Rail A/B as found at the start of this session's PRD-reconciliation
work. A subsequent 10-agent build pass (5 tickets, each independently implemented and then
independently re-verified by a second agent with fresh commands — see
`tickets/index.md` PROJ-750 through PROJ-754) closed every gap this document identified except
one:

- **Rail A**: `validate_external_cut` is now called from the real admission path
  (`powl_to_turtle`), not just unit-tested in isolation. `render_model_projection.rq`'s
  vocabulary now fully matches what `powl_projection.rs` emits — the claim above that it
  "would match nothing if run" **is no longer true**; it was independently re-verified this
  session by direct predicate-set comparison. Real SPARQL execution via oxigraph (already a
  workspace dependency) now runs the corrected query and returns real, exact-value result rows
  against a real fixture. Real Tera rendering now produces a genuine Arazzo 1.1.0 JSON document
  that round-trips through `wasm4pm-arazzo`'s own real parser. `ArazzoProjectionReceipt` now
  hashes real material bytes, independently re-verified by recomputing the digests outside the
  constructor.
- **Rail B**: the `ArazzoDescription -> AirProgram` bridge — this document's single biggest
  named gap — now exists (`crates/wasm4pm-arazzo/src/lower.rs`), proven end-to-end: a real
  parsed Arazzo document survives through `parse -> resolve -> lower -> normalize ->
  compile_to_wasm` with real source content (step ids, URLs, routing action names) present in
  the final compiled WASM bytes, independently re-verified twice this session.

**What is still genuinely true from this document, unchanged**: `AirCompiler::compile_to_wasm`
still has no execution semantics (no host imports, no HTTP dispatch) — that was never this
document's Rail A/B claim to fix, and remains real, disclosed scope for later PRD layers.

**The one gap that survived the build pass, confirmed by direct code inspection (not
inference)**: `ChatmanEngine::admit_transition` (`crates/praxis-graphlaw/src/chatman/
engine.rs:566`) calls none of PROJ-751/752/753's new functions. Every stage above is real and
independently tested, but the full composition — POWL admission through compiled AIR — is only
ever exercised inside `#[cfg(test)]` code. Tracked as PROJ-796. Until that lands, the honest
status is **ALIVE as a library, PARTIAL as a production pipeline** — a materially smaller and
more precisely located gap than "scaffolding only," but still a real one; do not round up to
`ALIVE` for the rail as a whole.

## See also

- `RAIL_G_MEASUREMENT_DESIGN.md` — the multifractal measurement instrumentation plan, which
  depends on Rail A/B's projection actually producing real Arazzo output before Track 2a
  (real receipt-stream measurement) becomes possible. That precondition is now met.
- `PRD.md` sections 7.4-7.6, 22.
- `tickets/index.md` PROJ-750 through PROJ-754 (the build) and PROJ-796 (closed, see below).
- `crates/wasm4pm-arazzo/` — this session's determinism/data-loss/causality fixes (see git
  log on `compile.rs`, `temporal.rs`, `normalizer.rs`, `lib.rs`), plus the new `lower.rs`.

## Closing note: PROJ-796 is now ALIVE, not the surviving gap

The "Update" section above named PROJ-796 (`ChatmanEngine::admit_transition` calling none of
PROJ-751/752/753's new functions) as the one gap that survived the build pass. That gap is now
closed — re-verified fresh this session, not taken on report:

- `crates/praxis-graphlaw/src/chatman/engine.rs:716` defines
  `ChatmanEngine::admit_transition_with_external_cut`, a real production entry point (not
  `#[cfg(test)]`-gated) adjacent to `admit_transition` (`engine.rs:616`). Confirmed via
  `grep -n "admit_transition_with_external_cut" crates/praxis-graphlaw/src/chatman/engine.rs`.
- `crates/praxis-core/src/arazzo.rs:678` defines `ArazzoProjectionReceipt::project_and_compile`,
  the real Rail A/B pipeline entry the engine calls through the `ExternalCutCompiler` trait
  seam. Confirmed via `grep -n "fn project_and_compile" crates/praxis-core/src/arazzo.rs`.
- `crates/praxis-core/tests/rail_ab_external_cut_wiring.rs` (10,839 bytes) exists on disk and
  independently recomputes the engine's private digest-#10 formula, asserting equality to what
  the sealed engine produces.

`tickets/index.md`'s own PROJ-796 row already carries the full **ALIVE** writeup and disclosed
gaps (replay-mismatch detection for digest #10 not yet wired; two pre-existing, out-of-scope
`praxis-graphlaw` build failures). This section exists only to correct the "Update" section
above, which is now stale on this one point: do not read PROJ-796 as an open gap from this
file: it is **ALIVE**, per `tickets/index.md` row 796.
