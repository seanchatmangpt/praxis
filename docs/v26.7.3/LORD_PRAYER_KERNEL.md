# The Lord's Prayer kernel

Source: `crates/praxis-synthesis/src/kernel.rs` and
`crates/praxis-synthesis/ontology/lord_prayer.ttl`. The kernel declares all
11 clauses of the prayer as typed, closed-world graph nodes
(`prayer-kernel:Clause`, namespace
`http://seanchatmangpt.github.io/praxis/prayer-kernel#`). Each clause names
its problem class and its delegation boundary. Extraction
(`extract_kernel`) enforces EXACT coverage: all 11 canonical clauses, no
more, no fewer, no duplicates — any deviation is a typed
`Refusal::KernelIllFormed` naming the culprit clause.

Evidence: `tests/kernel_coverage.rs ::
all_11_clauses_extract_and_hash_is_stable_across_reorder`,
`ten_clause_kernel_refuses_naming_the_missing_clause`,
`unknown_clause_name_refuses`.

## The 11-clause table

Values below are exactly the ones declared in `ontology/lord_prayer.ttl`.

| Clause | Problem class | Boundary | Action |
|--------|---------------|----------|--------|
| our-father | orientation | human-only | `prayer:DailyPrayerWorkflow` |
| hallowed-name | reverence | human-only | — |
| kingdom-come | authority-transfer | god-receives-unbounded | — |
| will-be-done | will-surrender | human-only | — |
| on-earth-as-heaven | alignment | god-receives-unbounded | — |
| daily-bread | provision-anxiety | automatable-support | `prayer:DailyBreadHook` |
| forgive-debts | owed-debt | god-receives-unbounded | — |
| forgive-debtors | resentment-loop | human-only | `prayer:ForgiveDebtorsHook` |
| temptation-guard | temptation-risk | automatable-support | `prayer:TemptationGuardHook` |
| deliverance | unbounded-threat | god-receives-unbounded | `prayer:DeliveranceHook` |
| doxology | closure | human-only | — |

The three lawful boundary strings (`kernel.rs :: BOUNDARIES`):

- `human-only` — the act is reserved for the human; agents may support,
  never execute.
- `god-receives-unbounded` — the object of the clause is surrendered, not
  computed.
- `automatable-support` — a bounded support workflow may assist the human
  act.

## The God boundary

God is never modeled as an agent, handler, capability, or tool. The
boundary is a STRING property on a clause node — it never resolves to an
executable node of any kind. Concretely:

- No node in `ontology/lord_prayer.ttl` typed `wf:Capability`, `hook:Hook`,
  or carrying `wf:handler` represents God, and the test
  `tests/kernel_coverage.rs :: god_is_never_typed_executable_and_deliverance_is_surrendered`
  checks this at the graph level.
- Unbounded problems are surrendered, never computed: the deliverance hook's
  effect is `refuse` with a declared surrender reason, so an
  `UnboundedThreat` fact yields a chained refusal receipt, not a plan.
  Evidence: `tests/prayer_kernel.rs :: unbounded_threat_is_surrendered_not_computed`,
  `tests/firing_chain.rs :: declared_refusal_surrender_is_chained_with_the_graph_reason`.

## The kernel_hash law

`kernel_hash` (`kernel.rs`) is the content address of byte-sorted canonical
lines `name\tproblem_class\tboundary\taction`. It is COMPUTED from the
extracted clauses — never asserted in the source document — and is stable
under any surface reordering of the TTL (clauses are sorted into canonical
scriptural order by the extractor first, and the hash lines are byte-sorted
besides). Evidence:
`tests/kernel_coverage.rs :: all_11_clauses_extract_and_hash_is_stable_across_reorder`.

## What the kernel is not

The kernel does not automate prayer, interpret scripture, or claim moral
completeness (see `docs/claims/WITHHELD_CLAIMS.md`). It is a typed index
from clause to problem class to a bounded, pre-declared support action —
and the clause execution ORDER in the demo workflow is derived by the
solver from declared preconditions, never authored
(`tests/prayer_kernel.rs :: provision_anxiety_grounds_the_daily_prayer_workflow`).
