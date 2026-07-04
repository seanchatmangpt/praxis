# ggen → praxis/crates/ggen port evaluation

Evaluated `~/ggen`'s 9 crates against praxis's iron invariants (typed
`Refusal` enum not panics; receipts computed via BLAKE3 never asserted-into;
no wall-clock in any hash/receipt path; closed vocabulary tables that refuse
unknown predicates by name) and its `backup_template` crate shape. Nothing
in ggen is grandfathered — every module below earned its verdict by being
read against real praxis code, not by existing.

Legend: **PORT** (real, compliant or trivially made compliant) ·
**REIMPLEMENT-DIFFERENTLY** (concept sound, current code violates an
invariant, needs a fresh implementation) · **DROP** (dead weight,
decorative, duplicative, or out of ggen's own stated scope).

---

## ggen-graph — the most consequential crate (already a live dependency)

`praxis/Cargo.toml:52,80` already depends on it via
`{ path = "../ggen/crates/ggen-graph", optional = true }` behind the `ggen`
feature, with live call sites in `src/corpus.rs`, `src/mfg.rs`,
`src/receipt_shacl.rs`, `verbs/plan.rs`, `verbs/doctor.rs`.

**PORT as-is**: `graph/hash.rs` (canonicalize-then-BLAKE3, no time input),
`graph/dataset.rs`, `graph/parse.rs`, `graph/canonical.rs`, `graph/quad.rs`,
`delta/mod.rs`, `shacl.rs`, `sparql.rs`.

**REIMPLEMENT-DIFFERENTLY**: `receipt/mod.rs` and `coherence.rs` — the
single most serious finding of this whole evaluation: `receipt/mod.rs:35,42`
(and ~118, 126, 234; `coherence.rs:13,95,114,128`) hash `Utc::now()` directly
into the receipt signature. Identical transitions at different instants
produce different signatures — a direct, repeated violation of the
no-wall-clock invariant baked into cryptographic output, not a cosmetic one.
`ocel/projection.rs:266,645-646` compounds it with a fail-open
`.unwrap_or_else(|_| chrono::Utc::now())` substitution on parse failure.
`GraphError` (thiserror, 6 variants, `#![deny(unwrap_used, expect_used,
panic)]`) is a good start but 4/6 variants carry untyped `String` payloads.

**DROP**: `ocel/*` (`pack_events`, `lifecycle`, `projection`, `dfg`,
`ocel_types`) — this is ggen's own stated out-of-scope process-analysis
boundary (see `~/ggen/CLAUDE.md`'s Process Intelligence Boundary table),
and it's also where the wall-clock/fail-open issues concentrate.

**No closed vocabulary anywhere** — `vocab/*` are bare IRI constants, no
`vocab_check`/`WF_PREDICATES`-equivalent admission gate. Build one fresh,
mirroring `praxis-synthesis/src/graph.rs`'s `WF_PREDICATES` pattern.

**Blast radius if dropped with no replacement**: all usage is
feature-gated (`#[cfg(feature = "ggen")]`), so default builds are
unaffected; with the feature on, `mfg.rs` loses Turtle parsing,
`corpus.rs` loses graph construction, `receipt_shacl.rs`'s SHACL test
breaks, `verbs/doctor.rs:226` permanently reports the `ggen` capability
absent.

---

## ggen-core — the largest crate, mostly DROP/REIMPLEMENT

~143k lines, 300+ files, with **three-to-four competing pipeline
implementations** (`pipeline.rs` 869 lines, `pipeline_engine/pipeline.rs`
777, `codegen/pipeline.rs` 2076, `codegen/executor.rs` 1248) and no single
authoritative path — itself a violation of praxis's smallest-diff/reuse
doctrine before any invariant check even applies.

**DROP, rebuild small**: pipeline/executor. 1,822 `.unwrap()`/`.expect()` in
non-test production code crate-wide. Reimplement as a small staged
pipeline modeled directly on `praxis-core/src/receipt_validator.rs:100-111`
(fixed ordered stages, each `Result<_, Refusal>`, no short-circuit —
run-every-stage-report-all).

**PORT (adapted)**: `manifest/types.rs` — `serde(deny_unknown_fields)` +
`BTreeMap` determinism is genuinely good and matches praxis instincts; carry
the schema shape over, but wrap its errors in `Refusal`, not whatever
`manifest/validation.rs` returns today.

**REIMPLEMENT-DIFFERENTLY**: ontology/SPARQL resolver — `rdf/query.rs:191`
and `domain/packs/sparql_executor.rs:77` accept arbitrary predicate/query
strings with zero admission-time closed-vocabulary check (the opposite of
`WF_PREDICATES`). Oxigraph wrapper mechanics are reusable prior art; the
open-vocabulary admission model is not — port the query engine, then layer
a fresh `PREDICATES`/`vocab_check` gate on top, exactly like
`graph.rs:798-800` layers enforcement over its bounded Turtle parser rather
than trusting parser output directly.

**PORT (thin)**: Tera rendering — conventional wrapper, no hash/receipt
entanglement as long as template contexts are audited for timestamps.

**DROP, reimplement from scratch**: error handling — 8 fragmented error
enums (`domain/`, `lifecycle/`, `ontology/`, `ontology_core/`, `receipt/`,
`security/`, `transport/`, `validation/`) plus 42 files still on
`anyhow`/`Box<dyn Error>`. None of this is portable as-is against praxis's
single exhaustive, no-wildcard-arm `Refusal` enum pattern
(`praxis-core/src/refusal.rs:82-159`).

**Wall-clock-in-hash violations found** (invented, not just missing-field
omitted, which is the actual rule):
`receipt/receipt_impl.rs:51` (`Receipt::new()` sets `timestamp: Utc::now()`
alongside hashes/signature), `pipeline_engine/receipt.rs:203,680,879`
(`BuildReceipt::new()`, 3 separate call sites), `agent/receipt.rs:115`,
`codegen/canonicalize.rs:367` (wall-clock inside a routine named for
*canonicalization*), `codegen/execution_proof.rs:44,139`.

**Asserted-not-computed hash risk**: `receipt/receipt_impl.rs:51` builds
`Receipt` from caller-supplied `input_hashes`/`output_hashes: Vec<String>`
and `signature: String::new()` — adjacent to the exact anti-pattern
`graph.rs:643-649` calls out by name.

---

## ggen-cli / ggen-config

Uses `clap-noun-verb` auto-discovery (`cmds/mod.rs:29-64`), not a plain
`Subcommand` enum — each `pub mod <noun>` is a noun, each `#[verb]` fn a
verb. Real noun surface: `sync`, `init`, `inverse_sync`, `git_hooks`,
`agent`, `capability`, `doctor`, `graph`, `ontology`, `pack`, `packs`,
`packs_receipt`, `policy`, `receipt`, `utils`, plus archived/feature-gated
`lsp`/`a2a`/`mcp`/`framework`/`sigma`/`wizard` and dead unregistered
`template.rs`.

**PORT**: `sync`, `init`, `inverse_sync` (maps directly onto praxis's
`[[generation.rules]]`), `receipt` verify/info (matches the computed-receipt
invariant conceptually, needs a Refusal rewrite).

**REIMPLEMENT-DIFFERENTLY**: `graph`/`ontology` (praxis already has its own
stricter closed-vocabulary `graph.rs` — reconcile, don't duplicate),
`pack`/`packs`/`packs_receipt` (marketplace concept transfers, code doesn't),
`git_hooks` (trivial, hand-write against praxis's actual hook model).
ggen-config: port the manifest *schema concept*, reimplement the
string-keyed validator (`config_lib/error.rs:71`, `match (loc_str.as_str(),
code)`) as a closed enum. No fail-open `warn!`/`eprintln!` pattern found in
ggen-config — a real point in its favor, worth preserving.

**DROP**: `agent`/`capability`/`policy` — these are process-intelligence/
analysis surfaces ggen's own CLAUDE.md explicitly forbids ggen from owning;
porting them re-imports the exact violation ggen's own docs warn against.
Also drop `utils`, all archived/experimental nouns, and dead `template.rs`.

**Violations**: `main.rs:6-18` bare `exit(0)`/`exit(1)`, no Refusal→exit-code
mapping; `cmds/lsp.rs:132-140` a verb body calling `std::process::exit`
directly; `lib.rs:60`
`#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]` — a
crate-wide suppression hiding exactly what praxis's `-D warnings` gate
exists to catch; `receipt_manager.rs` mixes `.expect()`-as-state-machine
(112,139,185) with 17 sites of stringly-typed `Error::new(&format!(...))` —
the single biggest gap vs. praxis's ~30-variant typed Refusal enum;
`progress.rs` has 13× `.lock().unwrap()` with no poisoned-mutex handling.

**Self-hosting caveat**: draft `praxis:CliCommand` TTL entries transfer
cleanly (see roadmap below), but praxis's own `ggen.toml:153-166` `src-cli`
rule currently only SPARQL-selects `?name` from `praxis:Project` — it does
not yet query `praxis:CliCommand` instances into route wiring. That wiring
must be built, not assumed to already work.

---

## ggen-marketplace / ggen-lsp

**ggen-marketplace** — real RDF/SPARQL package registry (~25.7k LOC).
`rdf/control.rs` (profile enforcement, flagged as authoritative by ggen's
own `coding-agent-mistakes.md`) is **REIMPLEMENT-DIFFERENTLY**: real logic,
but `.lock().unwrap()` at 12+ sites, and 3 sites (857, 936, 1022) unwrap by
fabricating a fake version instead of refusing. `metadata.rs:280-282`
**fail-opens on a security-relevant field** — unknown trust values silently
coerce to `Experimental` with only a `warn!`. `models.rs`'s Draft/Published
typestate is **DROP** — its own comment at line 25 calls it deprecated, zero
call sites workspace-wide. `v3.rs:530-583` package lookup is **DROP** —
builds a SPARQL query then explicitly returns "not implemented" (decorative
completion). `install.rs`/`compatibility.rs`/`network.rs`/`pki.rs`/
`search_sparql.rs` are **REIMPLEMENT-DIFFERENTLY**: real domain logic,
unwrap-riddled.

**ggen-lsp** — `analyzers/*` + `check.rs` are **PORT**: genuine
diagnostic computation from parsed ASTs, no praxis equivalent exists yet
(`backup_template/src/lsp.rs` is a 136-line pure delegate to `lsp-max` with
zero analyzer logic) — this is the one real asset, to graft onto praxis's
existing LSP scaffold. Everything else is **DROP**: `handlers/*` (13-40 line
passthroughs), `a2a_mcp/a2a_generated/*` (generated boilerplate). Most
importantly: `intel/mine.rs:13` imports `discover_dfg` from
`ggen-graph/src/ocel/dfg.rs:38`, and `intel/field.rs:40-113` computes
"van der Aalst variant analysis" metrics — **a confirmed, concrete
violation of ggen's own stated Process Intelligence Boundary rule**,
sitting inside the very crate whose CLAUDE.md defines that rule.

---

## genesis-types-v2 / genesis-core-v2 / cpmp

**genesis-types-v2**: `lib.rs` error/event types are **PORT** (typed
`thiserror` enum, matches Refusal discipline). `schema.rs::RdfOntology`
(hand-rolled Turtle string emitter, no validation) and `PatternRegistry`
(bare `HashMap`, no closed-world enforcement, duplicated a third time across
genesis-core-v2 and cpmp) are both **DROP**.

**genesis-core-v2**: `primitives.rs` (`Receipt::generate`,
`ReplayCursor::advance`) is **PORT** — genuinely computes chained BLAKE3
hashes and refuses on signature mismatch rather than trusting input, and its
"zero-alloc" claim is real here (verified via `#[repr(C)]` compile-time size
assertions). The `Pattern` trait + registry + 3 concrete pattern impls are
**DROP, duplicative**: praxis-synthesis's `hooks.rs` (closed condition-kind
vocabulary, `REFUSED_KINDS` at lines 49-56) and `kernel.rs` (delegation as a
non-executable string property) already solve this problem correctly, while
genesis's `Pattern::execute()` is itself arbitrary executable code — the
opposite of praxis's "delegation never executable" rule. The touted "43
YAWL patterns" is unbacked: only 3 of 43 have any real body, the rest exist
only as doc-comment aspiration; "zero-copy" is false for the Pattern system
specifically (`Arc<dyn Pattern>` is heap-allocated dynamic dispatch).

**cpmp**: `receipt.rs::aggregate_hash` is **PORT** — genuinely computes
BLAKE3 over sorted (path, content-hash) pairs, order-independent, tested.
`receipt.rs::generate_receipt` is **REIMPLEMENT-DIFFERENTLY**: calls
`chrono::Utc::now()` directly inside receipt construction (line 41) —
same wall-clock-in-hash violation as ggen-graph and ggen-core.
`classification.rs::classify_file` is **PORT as reference** (honest,
explainable heuristic). `symbol.rs` regex extraction and
`receipt.rs::verify_no_deletion` are **REIMPLEMENT-DIFFERENTLY**: the
former panics via `Regex::new(...).unwrap()` on hardcoded patterns, the
latter fail-opens — on missing files it only `println!`s "REFUSAL: ..."
and still returns `Ok(())`, never a typed `Err`.

---

## Cross-crate pattern: the recurring violation is wall-clock-in-hash

Five independent modules across four different crates make the exact same
mistake — hashing `Utc::now()`/`SystemTime::now()` directly into a receipt
or signature: `ggen-graph/receipt/mod.rs`, `ggen-graph/coherence.rs`,
`ggen-core/receipt/receipt_impl.rs`, `ggen-core/pipeline_engine/receipt.rs`
(×3), `ggen-core/agent/receipt.rs`, `ggen-core/codegen/canonicalize.rs`,
`ggen-core/codegen/execution_proof.rs`, `cpmp/receipt.rs`. This is the
single most consistent, structural gap between ggen and praxis's invariant
#3 — not an isolated bug, a repeated design habit. Every receipt-shaped
module being ported needs this specific check applied, not just a general
code review.

---

## Target module layout for `praxis/crates/ggen/src/`

Following `backup_template`'s shape:

| `backup_template` file | ggen concept it absorbs |
|---|---|
| `lib.rs` | Manifest schema (adapted `ggen-core/manifest/types.rs`), staged pipeline modeled on `praxis-core/receipt_validator.rs` |
| `cli.rs` | Generated (not hand-written) from `praxis:CliCommand` ontology instances via the existing `src-cli` rule, once its SPARQL is extended past `?name` |
| `verbs/*.rs` | One file per surviving noun: `sync`, `init`, `inverse_sync`, `receipt`, `pack`/`packs` (reimplemented), `graph`/`ontology` (reconciled with existing `graph.rs`) |
| `error.rs` | Fresh `Refusal`-style exhaustive enum replacing ggen's 8 fragmented error types + all `anyhow` usage |
| `discovery.rs` | Fresh closed-vocabulary `vocab_check` (none exists in any evaluated ggen crate) mirroring `praxis-synthesis/graph.rs`'s `WF_PREDICATES` |
| `lsp.rs` | ggen-lsp's `analyzers/`+`check.rs` grafted onto the existing `lsp-max` delegate |
| (new) `graph.rs` or extend `praxis-synthesis/graph.rs` | ggen-graph's `graph/{hash,dataset,parse,canonical,quad}.rs`, `delta/`, `shacl.rs`, `sparql.rs` ported as-is; `receipt/mod.rs`+`coherence.rs` reimplemented with `Utc::now()` removed from the hash path |

Not represented in the layout at all (fully DROP): ggen-core's 3 duplicate
pipeline implementations, ggen-graph's `ocel/*`, ggen-lsp's
`intel/`+`route/`+`a2a_mcp/a2a_generated/`, ggen-marketplace's
`models.rs` typestate + `v3.rs`, genesis-core-v2's `Pattern`
trait/registry, genesis-types-v2's `RdfOntology`/`PatternRegistry`,
ggen-cli's `agent`/`capability`/`policy`/`utils`/archived nouns/dead
`template.rs`.

## `praxis:CliCommand` ontology entries needed

Following the exact shape of `CmdDodRun`/`CmdVerifierVerify`
(`schema/praxis.ttl:229-237`):

```turtle
praxis:CmdSyncRun a praxis:CliCommand ;
    praxis:noun "sync" ; praxis:verb "run" ; praxis:handler "handle_sync_run" .

praxis:CmdInitRun a praxis:CliCommand ;
    praxis:noun "init" ; praxis:verb "run" ; praxis:handler "handle_init_run" .

praxis:CmdReceiptVerify a praxis:CliCommand ;
    praxis:noun "receipt" ; praxis:verb "verify" ; praxis:handler "handle_receipt_verify" .

praxis:CmdReceiptInfo a praxis:CliCommand ;
    praxis:noun "receipt" ; praxis:verb "info" ; praxis:handler "handle_receipt_info" .
```

Plus one entry each for the reimplemented `pack add/remove/list/show`,
`graph validate/load/query`, `ontology list/status/info` verbs once their
fresh (non-ported) implementations exist.

**Prerequisite work, not yet done**: root `ggen.toml`'s `src-cli` rule
(`ggen.toml:153-166`) only SPARQL-selects `?name` from `praxis:Project`
today — it doesn't query `praxis:CliCommand` instances into generated
routes. Extending that query/template pair to actually wire
noun/verb/handler triples into `cli.rs` is required before any of the
above TTL entries produce real code, and should be the very first
follow-up step (see roadmap).

## Porting roadmap for follow-up sessions

Ordered by leverage and risk, each step gated on `just verify-all` before
the next begins:

1. **Wire `src-cli` to `praxis:CliCommand`** (currently stops at
   `?name` — see prerequisite above). Nothing else self-hosts until this
   works; validate with the 4 draft TTL entries above.
2. **`ggen-graph`'s clean modules** (`graph/hash.rs` et al., `delta/`,
   `shacl.rs`, `sparql.rs`) — highest confidence PORT, immediately
   unblocks replacing the external path dependency for the non-receipt
   surface.
3. **Fresh `Refusal`-style error enum + closed-vocab `vocab_check`** — both
   are prerequisites every other module's port depends on; build once,
   reuse everywhere (this is also where the cross-crate wall-clock-in-hash
   fix pattern gets applied uniformly).
4. **`ggen-graph`'s receipt/coherence layer**, reimplemented with
   `Utc::now()` removed from the hash input — this retires the external
   dependency entirely once done, per the user's stated goal.
5. **Staged pipeline** (`sync`/`init`/`inverse_sync`) modeled on
   `receipt_validator.rs`'s run-every-stage shape, replacing ggen-core's
   3-4 competing pipelines with one.
6. **`ggen-lsp`'s `analyzers/`+`check.rs`**, grafted onto
   `backup_template/src/lsp.rs`'s existing `lsp-max` delegate — decoupled
   first from `intel/`+`route/` (the confirmed process-boundary
   violation), which get dropped, not ported.
7. **`ggen-config` manifest schema**, `deny_unknown_fields`+`BTreeMap`
   shape preserved, validator reimplemented as a closed enum.
8. **`ggen-marketplace`'s real domain logic** (`install`, `compatibility`,
   `network`, `pki`, `search_sparql`, `rdf/control.rs`) — lowest priority,
   highest unwrap-removal effort; only take this on once 1-7 are stable.
9. **`cpmp`'s `aggregate_hash` + `classify_file`** — small, low-risk,
   can slot in any time after step 3 provides the Refusal enum.

Everything marked DROP above (ggen-core's dead pipelines, all `ocel/*`,
`ggen-lsp`'s `intel/`+`route/`+generated protocol code, marketplace's dead
typestate, genesis's `Pattern` system) is excluded from this roadmap
entirely — it does not get ported, reimplemented, or scheduled.
