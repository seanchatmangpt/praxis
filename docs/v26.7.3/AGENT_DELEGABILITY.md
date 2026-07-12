# Agent assignment and the delegability lattice

Source: `crates/praxis-synthesis/src/handlers.rs`. The binding of "who
executes what" is graph-declared, judged against a closed registry, and
graded on an ordered lattice. Nothing here was inherited: the lattice was
authored fresh for v26.7.3 (the archaeology found it nowhere in the
constellation).

## The lattice

```
human-only  <  assistive  <  automatable  <  verifiable
```

- `human-only` — only the human acts (forgiveness, confession, surrender).
  Agents may support, never execute.
- `assistive` — the human acts with agent assistance; execution still parks
  for the human.
- `automatable` — an automated handler may execute.
- `verifiable` — an automated handler may execute AND the result is
  independently verifiable by replay.

Implemented as the ordered enum `Delegability` (derives `Ord`, so the
lattice comparison is the language's own).

## Graph declaration

A `wf:Capability` node opts in with `wf:handler <iri>` and MUST then carry
`wf:delegability "<grade>"` — there is no default grade; a handler without
a declared grade is `Refusal::WorkflowIllFormed`
(`src/handlers.rs :: handler_without_delegability_is_ill_formed`).
Capabilities without `wf:handler` are legacy-lawful (the default
deterministic runner applies).

## The closed registry, exact keys only

`HandlerRegistry::builtin()` contains exactly one handler:
`http://seanchatmangpt.github.io/praxis/handler#deterministic-v1`. Lookup
is exact-key IRI membership — prefix, suffix, and string-convention
matching are unrepresentable in the API. An unknown IRI is
`Refusal::UnknownHandler` naming the known table, and it fires BEFORE any
solving.

Evidence: `src/handlers.rs :: unknown_handler_refused_exact_key_only`
(a SUFFIX of a registered IRI is unknown);
`tests/firing_chain.rs :: unknown_handler_is_refused_before_solving_and_still_chained`.

## Eligibility rule (exact, as implemented)

An automated runner may execute a fired action iff:

1. Every `wf:handler` IRI anywhere in the admitted graph is in the closed
   registry (`judge_known` — global, pre-solve; one unknown IRI refuses the
   whole firing).
2. For every capability the fired action's DERIVED PLAN actually uses that
   carries a handler binding, the declared delegability is `automatable` or
   above (`judge_delegability`, scoped to the used-capability set). Below
   `automatable` is `Refusal::DelegabilityViolation` — the action parks for
   the human, receipted, never silent.

The scoping matters: a `human-only` binding on a capability NO fired plan
touches must not poison unrelated firings.

## Blocked examples (the agent cannot forgive)

- `release-resentment` declared `human-only`: a firing whose derived plan
  would execute it is refused with `DelegabilityViolation` and chained.
  Evidence: `tests/deviation_routes.rs ::
  human_only_release_resentment_blocks_the_debt_firing`;
  `tests/firing_chain.rs :: human_only_binding_is_a_chained_delegability_refusal`.
- The same acts by kernel boundary: forgiveness, confession, and surrender
  are `human-only` clause boundaries in `ontology/lord_prayer.ttl`
  (forgive-debtors, will-be-done) — see LORD_PRAYER_KERNEL.md.

## Allowed examples

- `write-prayer-receipt` declared `automatable`: the firing completes.
  Evidence: `tests/deviation_routes.rs :: automatable_write_prayer_receipt_is_allowed`.
- A `human-only` binding on an UNUSED capability does not refuse an
  unrelated firing. Evidence:
  `tests/deviation_routes.rs :: human_only_release_resentment_does_not_block_an_unrelated_firing`;
  `tests/firing_chain.rs :: human_only_binding_on_an_unused_capability_does_not_refuse`.

## Binding hash

Bindings render to canonical `capability\thandler\tdelegability` lines
(sorted, trailing newline); `handler_hash` is the content address of that
form and is fold 3 of the outer firing chain — the graph decides the
binding; the hash proves which graph
(`src/handlers.rs :: two_declarations_two_binding_hashes`).
