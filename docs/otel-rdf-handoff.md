# OTEL → RDF → OCEL Handoff Boundary

Version: v26.7.9-era. Updated 2026-07-11 (G11 status corrected — the mapper described below
as future work is built; see `docs/jira/v26.7.11/tickets/index.md` PROJ-763/764/765 for the
authoritative, independently-verified writeups).

This document defines the boundary between the OTel Weaver live-check campaign (admission of
telemetry against semantic conventions) and the RDF/OCEL projection increment: mapping admitted
OTEL signals into RDF and deriving OCEL from RDF.

Status, re-verified fresh this session (not taken on report):

- G10 (boundary identified): ALIVE as this document only.
- G11 (OTEL → RDF mapper): **ALIVE** — `crates/cng/src/otel_rdf.rs` (PROJ-763). Re-run this
  session: `CARGO_TARGET_DIR=target/agent-g11-verify cargo test -p cng --features bench --lib
  otel_rdf` → `10 passed; 0 failed`.
- SPARQL-CONSTRUCT-derived OCEL (`G_OCEL = CONSTRUCT_P(G_OTEL)`): **ALIVE** —
  `crates/cng/src/otel_ocel.rs` (PROJ-764), `queries/otel-to-ocel.construct.rq`.
- PROV-O transformation ancestry + receipt (`digest(P) + digest(G_OTEL) -> digest(G_OCEL)`):
  **ALIVE** — `crates/cng/src/otel_receipt.rs` (PROJ-765).

## What exists at the boundary today

Weaver live-check (`just otel-weaver-live`, weaver 0.22.1) admits or refuses telemetry emitted
by the `otel-live` binary (`cargo run -p cng --features otel-live --bin otel-live`) against the
`registry/otel/` semantic-convention registry. Admission is the gate: telemetry that fails
conventions (e.g. missing `process.outcome`) is refused with
`NEGATIVE_REFUSAL_CODE=WEAVER_SEMANTIC_CONVENTION_REFUSED` and never crosses this boundary.

`crates/cng/src/otel_rdf.rs`'s `admit` function re-validates the identical
`event.praxis.activity_executed` five-attribute contract in-process (required attributes
present, `process.outcome` restricted to the closed `completed`/`refused` vocabulary),
refusing via typed `CngRefusal::OtelSpanRefused` (`CNG_R27`) before any triple is produced.
`project_admitted_spans` then projects admitted spans into the named graph `urn:graph:otel`
per the mapping below.

## RDF projection (BUILT — `crates/cng/src/otel_rdf.rs`, `otel_ocel.rs`)

Admitted OTEL signals are mapped into named RDF graphs; OCEL is derived exclusively via SPARQL
CONSTRUCT:

```text
G_OCEL = CONSTRUCT_P(G_OTEL)
```

1. Admitted OTEL signals land in the named graph `urn:graph:otel`.
2. A fixed set `P` of SPARQL CONSTRUCT queries projects `urn:graph:otel` into `urn:graph:ocel`.
3. No other derivation path exists — OCEL is a pure function of the OTEL graph and the queries.

### Named-graph layering

The mission's graph layering applies unchanged:

```text
urn:graph:source    (authored TTL: plans, ontologies)
urn:graph:otel      (admitted OTEL signals, this boundary's output)
urn:graph:ocel      (CONSTRUCT-derived OCEL, never hand-built)
urn:graph:results   (verdicts, gate outcomes)
urn:graph:receipts  (sealed receipt envelopes)
```

### Provenance (PROV-O)

Each CONSTRUCT run records PROV-O provenance of the construction: the query digest and the
input-graph digest derive the output-graph digest. A consumer can replay
`digest(P) + digest(G_OTEL) → digest(G_OCEL)` and refuse on mismatch — same discipline as the
engine's computed (never asserted) BLAKE3 receipts.

### Generation source

**Corrected from the original plan below**: the built `P` (`crates/cng/src/queries/
otel-to-ocel.construct.rq`, PROJ-764) is a hand-written SPARQL file compiled in via
`include_str!` — not ggen-generated. It is checked against the crate's own
no-inline-SPARQL convention (`tests/no_inline_ttl_guard.rs`), which requires the query live in
its own `.rq` file rather than as a Rust string literal; that convention is satisfied, but
ggen generation from `otel-bridge.ttl` via `just sync` was not how this ticket built it. Left
here as the originally-planned alternative, not current behavior.

### Alignment targets

- Vocabulary: `crates/praxis-graphlaw/ontologies/core/ocel2.ttl`.
- Serialization layout: the OCEL JSON layout already used under `.cargo-cicd/ocel/`.

## Forbidden alternatives

These derivation paths are refused by design, not deferred:

1. Log-file parsing (scraping weaver or emitter stdout/logs into OCEL).
2. Imperative Rust OCEL construction (building OCEL objects in code instead of CONSTRUCT).

Either alternative would make OCEL an asserted artifact rather than a computed projection of
the admitted OTEL graph, breaking the receipt discipline.

## References

- `crates/cng/src/otel_rdf.rs` — G11 OTEL → RDF admission mapper (PROJ-763, built)
- `crates/cng/src/otel_ocel.rs`, `crates/cng/src/queries/otel-to-ocel.construct.rq` —
  `G_OCEL = CONSTRUCT_P(G_OTEL)` (PROJ-764, built)
- `crates/cng/src/otel_receipt.rs` — PROV-O ancestry + `digest(P) + digest(G_OTEL) ->
  digest(G_OCEL)` receipt (PROJ-765, built)
- `docs/jira/v26.7.11/tickets/index.md` PROJ-763/764/765 — authoritative, independently
  re-verified build status for this whole boundary
- `crates/praxis-graphlaw/ontologies/core/ocel2.ttl` — OCEL 2.0 vocabulary (alignment target)
- `crates/praxis-graphlaw/ontologies/core/otel-bridge.ttl` — OTEL bridge vocabulary
- `docs/CHATMAN_EQUATION.md` — A = μ(O*) formulation this projection serves
- `justfile` — `# --- OTel Weaver live-check ---` recipe section (the admission side)
