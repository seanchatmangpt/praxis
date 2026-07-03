# Receipts, replay, and foreign verification

Sources: `crates/praxis-synthesis/src/firing.rs` (outer chain + replay),
`src/graph.rs` (inner v1 receipts), `scripts/foreign_verify_graph.py`
(the second implementation). Every hash is computed, never asserted.

## The inner v1 chain (`praxis:workflow:v1`)

One executed workflow fragment yields one `WorkflowReceipt` whose chain
folds the graph/IR/plan/topology/geometry/exec stages. The hook layer folds
this chain AS AN EVENT and never mutates it: direct execution and
hook-fired execution of the same fragment produce byte-identical inner
chains. Evidence: `tests/prayer_kernel.rs ::
v1_chain_golden_pin_direct_execution_unchanged_by_the_hook_layer`.

## The outer firing chain (`praxis:hook-firing:v1`)

One firing = one admitted event judged by the full registry. Fold order
(`firing.rs :: fold_firing_chain`):

```
genesis(praxis:hook-firing:v1)
  ⊳ event_hash          (delta canonical form)
  ⊳ admission_hash      (AdmissionRecord JSON)
  ⊳ handler_hash        (canonical binding lines)
  ⊳ hook_hash           (verdict record list JSON — NotFired included)
  ⊳ history_hash        (window-history commitment: the first 7 history
                         deltas' computed event hashes — the max window − 1
                         that can influence any verdict)
  ⊳ inner chain per fired action, in verdict order
      (none fired -> the sentinel "praxis:no-action")
  ⊳ outcome_hash        (FiringOutcome JSON)
```

`delta_ttl_hash` (exact surface bytes) is a receipt FIELD only — never
folded (the ttl_hash doctrine). Outcomes: `Completed`, or
`Refused { stage, reason }` with stage `handler` | `kernel-boundary` |
`delegability` | `declared-refusal` — lawful refusals are chained, never
silent (knhk Covenant-2, imported as policy). The `kernel-boundary` stage
is the surrender invariant enforced as a runtime law: when the post-state
declares a prayer kernel, no `god-receives-unbounded` clause may be routed
toward computation (`kernel.rs :: enforce_surrender_boundary`,
`tests/repair_loop.rs`).

Evidence: `tests/firing_chain.rs :: completed_firing_chains_and_replays`,
`unknown_handler_is_refused_before_solving_and_still_chained`,
`human_only_binding_is_a_chained_delegability_refusal`,
`declared_refusal_surrender_is_chained_with_the_graph_reason`.

## Replay and payload binding

`firing.rs :: replay_firing(receipt, base_ttl, source, registry, history)`
re-derives the WHOLE firing from the base TTL and delta documents, compares
stage by stage in fold order (event, admission, handler, hook, history,
outcome, chain), and then binds every embedded payload to the hash just verified:
the admission record, the bindings list, the verdict list, the outcome, and
the inner chains must each reproduce their hash. A receipt whose hashes are
honest but whose bodies are forged is refused by name — a receipt cannot
vouch for itself.

Evidence: `tests/firing_chain.rs ::
forged_payloads_behind_honest_hashes_are_refused_by_name`.

## Foreign verification (second implementation)

**Scope statement.** Foreign verification scope for v26.7.3: The foreign
firing verifier independently re-derives the graph-side authority chain
through admission and handler binding. It computes the event hash from the
RDF delta, applies the delta to derive the post-state graph, re-canonicalizes
that graph, derives the admission record from computed graph hashes, and
extracts declared handler bindings from the resulting graph. The verifier
does not independently re-run the hook evaluator or execution runtime. Hook
verdicts and outcomes are verified by payload binding: the embedded
verdict/outcome bodies are hashed, compared to the receipt folds, and
included in the receipt-chain verification. Therefore v26.7.3 proves foreign
graph/admission/binding verification plus payload-bound verdict/outcome
integrity. Full foreign semantic re-execution of hook evaluation is a
withheld claim unless and until a Python-side hook evaluator mirror is
implemented.

`scripts/foreign_verify_graph.py` — Python stdlib + the `b3sum` binary, no
Rust, no crate source. (`scripts/foreign_verify.py`, the original workflow
verifier, is byte-frozen.) Exit 0 = verified, 1 = MISMATCH (first divergent
stage printed), 2 = usage/IO error.

### `graph <ttl-file> <receipt.json>` — inner v1 receipts

Recomputed: the Turtle-subset parse, the canonical form, the graph_hash,
the WorkflowIr extraction and its ir_hash, the chain refold, the
plan-payload binding, and the exec-payload hash.

Evidence: `tests/foreign_graph_tests.rs` (honest receipt agrees; reformat
agrees; first divergent stage named; graph-consistent IR forgery caught;
constraint-bearing workflow agrees).

### `firing <base.ttl> <adds.ttl> <removes.ttl> <receipt.json>` — outer chain

Re-derived from inputs: the event canonical form and event_hash (parse,
sort, dedup, cap-64, both-sides refusal mirrored); the post-state apply and
its hash; the AdmissionRecord JSON in serde field order and its hash; the
handler bindings re-extracted from the post-state (ill-formed refusals
mirrored) and their canonical lines; the chain refold.

Evidence: `tests/foreign_firing.rs ::
foreign_firing_verifier_agrees_on_an_honest_completed_receipt`,
`foreign_firing_verifier_agrees_on_a_declared_refusal_receipt`,
`foreign_firing_verifier_fails_a_tampered_verdict_payload`.

### Named limitations (what the foreign verifier does NOT re-derive)

Stated in the script itself and repeated here so no claim outruns them:

1. `hook_hash` and `outcome_hash` are REFOLDED FROM THE RECEIPT'S EMBEDDED
   PAYLOADS (verdict list, outcome JSON): this binds bytes to hash and
   catches payload tampering, but hook evaluation itself is not re-run in
   Python — re-derivation needs the Rust evaluator (`replay_firing` covers
   that side).
2. Inner v1 chains inside a firing receipt are refolded as claimed by the
   `firing` subcommand; each is independently verifiable with the `graph`
   subcommand, whose own named limitation is that plan/topology/geometry
   stage hashes are refolded as claimed, not re-derived (re-derivation
   needs the Rust replayer).
3. `history_hash` (the window-history commitment) is folded as claimed
   from the receipt field: the `firing` subcommand takes no history input,
   so it binds the fold position, not the history bytes. Re-deriving the
   commitment from an actual history needs the Rust side
   (`replay_firing`, which refuses a mismatched history —
   `tests/repair_loop.rs ::
   replaying_a_firing_against_a_different_history_is_refused`).

## Trustless replay

`bash scripts/trustless_replay.sh` verifies the packaged receipts in a bare
directory whose PATH contains nothing but `python3` and `b3sum` — no cargo,
no crate source. `package` (needs cargo) regenerates
`receipts/trustless/`; `verify` is the default subcommand.
