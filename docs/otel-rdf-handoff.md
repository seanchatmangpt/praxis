# OTEL → RDF → OCEL Handoff Boundary

Version: v26.7.9-era. Updated 2026-07-10.

This document defines the boundary between the OTel Weaver live-check campaign (admission of
telemetry against semantic conventions) and the next increment: mapping admitted OTEL signals
into RDF and deriving OCEL from RDF. It is a boundary definition, not a description of shipped
code.

Status honesty: everything in "The next increment" below is FUTURE work. The mapper does NOT
exist today. Only the boundary definition in this document is current.

- G10 (boundary identified): ALIVE as this document only.
- G11 (OTEL → RDF mapper + SPARQL-derived OCEL): BLOCKED — not implemented.

## What exists at the boundary today

Weaver live-check (`just otel-weaver-live`, weaver 0.22.1) admits or refuses telemetry emitted
by the `otel-live` binary (`cargo run -p cng --features otel-live --bin otel-live`) against the
`registry/otel/` semantic-convention registry. Admission is the gate: telemetry that fails
conventions (e.g. missing `process.outcome`) is refused with
`NEGATIVE_REFUSAL_CODE=WEAVER_SEMANTIC_CONVENTION_REFUSED` and never crosses this boundary.

## The next increment (FUTURE — not implemented)

After live-check admits telemetry, the next increment maps admitted OTEL signals into named RDF
graphs and derives OCEL exclusively via SPARQL CONSTRUCT:

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

The CONSTRUCT queries in `P` will be ggen-generated from the same
`crates/praxis-graphlaw/ontologies/core/otel-bridge.ttl` source that defines the OTEL bridge
vocabulary, via `just sync` semantics — never hand-written per deployment.

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

- `crates/praxis-graphlaw/ontologies/core/ocel2.ttl` — OCEL 2.0 vocabulary (alignment target)
- `crates/praxis-graphlaw/ontologies/core/otel-bridge.ttl` — OTEL bridge vocabulary (ggen source)
- `docs/CHATMAN_EQUATION.md` — A = μ(O*) formulation this projection serves
- `justfile` — `# --- OTel Weaver live-check ---` recipe section (the admission side)
