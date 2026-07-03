# Rice quarantine and admission

Source: `crates/praxis-synthesis/src/quarantine.rs` and `src/delta.rs`.
This is the only door into the admitted graph.

## The pipeline

```
MeaningSource (raw bytes + Origin)
  -> RiceQuarantine::inspect   decidable checks only -> GraphDelta
  -> Admission::admit          apply vs Reference, post-state vocab check,
                               recompute post hash, epoch += 1
  -> AdmittedEvent             { AdmissionRecord, post triples, delta }
```

### MeaningSource and Origin

A `MeaningSource` is raw candidate bytes (`adds_ttl`, `removes_ttl`) plus a
declared `Origin`: `Operator`, `Proposer` (e.g. an LLM — advisory until
admitted), or `Bridge`. Origin is provenance ONLY: every origin passes the
identical decidable checks; no origin is trusted more than another.

### inspect — decidable checks only

`RiceQuarantine::inspect` runs the bounded Turtle-subset parser with hard
caps and nothing else: no semantic evaluation of raw content, ever. This is
the Rice boundary — properties the parser and caps cannot decide about the
bytes are not decided; they are refused with a typed `Refusal` naming the
culprit (`Refusal::GraphMalformed`, `Refusal::GraphCapExceeded`,
`Refusal::InvalidInput` for a triple asserted and retracted at once).
Caps: `MAX_DELTA_TRIPLES = 64` per side (`src/delta.rs`).

Evidence: `src/quarantine.rs :: quarantine_refuses_malformed_bytes_decidably`;
`src/delta.rs :: delta_cap_fires`, `add_and_remove_same_triple_is_refused`.

### admit — judged against the Reference

`Admission::admit` applies the delta to the admitted base
(`Reference { triples, graph_hash, epoch }`), enforces the closed-world
`wf:` vocabulary on the POST-state, recomputes the post-state hash by
re-canonicalizing (never taken from the event), and increments the logical
epoch. There is no wall clock anywhere. Removing a triple absent from the
base is `Refusal::AdmissionRefused` — retracting what was never admitted
would silently rewrite history. Refusal paths get a receipted
`AdmissionRecord` too (`Admission::refusal_record`, verdict `Refused`,
post hash = base hash).

Evidence: `src/quarantine.rs :: quarantine_admission_computes_post_hash_and_epoch`,
`post_state_vocab_violation_is_refused_at_admission`,
`refusal_record_binds_base_and_event_without_state_change`;
`src/delta.rs :: apply_adds_removes_and_refuses_phantom_removal`.

## Raw meaning cannot execute

Nothing executes from bytes. Execution happens only downstream of an
`AdmittedEvent`, and only through hooks that are themselves declared in the
admitted graph (`src/firing.rs :: fire_hooks` takes a `MeaningSource` and
runs it through inspect -> admit before any hook is even extracted).

- Raw scripture is the worked case: a scripture-text literal admitted into
  the graph is DATA — it triggers no hook, grounds no action, and asserting
  law-vocabulary alongside it is refused by the closed-world check.
  Evidence: `tests/kernel_coverage.rs :: raw_scripture_is_quarantined_data_not_law`.
- An LLM proposer's output is just a `MeaningSource` with
  `Origin::Proposer` — it faces the same parser, the same caps, the same
  admission gate, and can never carry executable content past them
  (there is no executable content shape at all; only triples).

## Hash discipline

`GraphDelta::event_hash` is computed from the delta's canonical form
(surface-invariant); `delta_ttl_hash` names the exact surface bytes but is
a receipt field only — never folded into any chain. Evidence:
`src/delta.rs :: event_hash_is_surface_invariant_and_ttl_hash_is_not`.
