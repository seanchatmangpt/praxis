# AA livelock model — human livelocks as workflow soundness

Source: `crates/praxis-synthesis/src/livelock.rs`. A livelock is a loop
that consumes cycles without progressing the graph: rehearsal without
release, shame without an amend candidate, self-will without surrender.
Each class carries a DETECTION program in the same bounded datalog
micro-syntax the hook registry uses, evaluated by the same engine over
`t(s, p, o)` facts — detection is a query on the admitted graph, never a
judgment call.

## Livelock classes (closed vocabulary — 6 of the 8-bound)

| Class | Open when | Closed by (life# vocabulary) | Goal predicate |
|-------|-----------|------------------------------|----------------|
| Resentment | a `ResentmentLoop` exists | any act with `releases` pointing at it | `openresentment` |
| Shame | a `Harm` exists | any candidate with `amendsFor` pointing at it | `openshame` |
| SelfWill | a `SelfPlan` exists | the plan gains a `surrendered` edge | `openselfwill` |
| Fear | a `ProvisionAnxiety` exists | it carries `hasBreadReceipt` | `openfear` |
| ReliefSeeking | a `TemptationRisk` exists | any guard act with `guards` pointing at it | `openrisk` |
| SpilledMilk | an `IrreversibleEvent` exists | ANY of `repairs`, `learnsFrom`, `releases` | `openspill` |

A loop praxis cannot classify is not "detected loosely"; it simply is not a
livelock this model speaks about. `detect(class, post_triples)` builds the
class's program via `detection_program` and evaluates it through the hook
engine (`hooks.rs :: eval_datalog`), so livelock detection and hook firing
share one evaluator.

Evidence: `tests/livelock.rs :: every_class_program_parses_and_evaluates`,
`test_14_resentment_livelock_detected`,
`test_15_release_fact_closes_and_inventory_sees_it`,
`test_16_spilled_milk_closes_through_any_of_three`;
`src/livelock.rs :: self_will_closes_via_surrendered_edge`.

## Steps 1-12 mapped to soundness operations

The mapping is the code constant `livelock.rs :: STEPS` — testable and
content-addressable, not prose
(`tests/livelock.rs :: twelve_steps_map_to_soundness_operations`;
`src/livelock.rs :: steps_cover_one_through_twelve_in_order`).

| Step | Recovery statement (abridged) | Soundness operation |
|------|-------------------------------|---------------------|
| 1 | admitted the loop is unmanageable from inside | livelock detection: the datalog goal is derivable and no local transition closes it |
| 2 | a power greater than the loop can restore sanity | external recoverability: soundness is judged from outside the stuck component |
| 3 | decided to turn will and life over | self-will control transfer: the selfPlan gains a surrendered edge; scheduling moves to the solver |
| 4 | made a searching and fearless inventory | life-to-graph translation: every loop, debt, and harm becomes a typed triple |
| 5 | admitted the exact nature of the wrongs to another | external witness node: the graph is shared with a second verifier, not self-audited |
| 6 | entirely ready to have defects removed | defective transition readiness: dead transitions are marked removable, not defended |
| 7 | humbly asked to have shortcomings removed | removal request: the retraction delta is proposed through the quarantine door |
| 8 | listed all persons harmed, willing to make amends | repair queue: open debts and harms become an ordered amends worklist |
| 9 | made direct amends except when it would injure | bounded repair / safe withholding: repair fires within budget; harmful repair is refused |
| 10 | continued personal inventory, promptly admitted wrongs | daily livelock detection: the detection programs re-run on every admitted delta |
| 11 | sought conscious contact through prayer and meditation | daily external alignment: the kernel re-orients against the reference, not against itself |
| 12 | carried the message and practiced the principles | service output: the recovered workflow grounds actions for graphs beyond its own |

## The no-infinite-rehearsal law

`rehearsal_exceeded(history, loop_iri, bound)` is the window-hook law
applied to rumination: it is true iff at least `bound` deltas in the
history touch the loop IRI (as subject, predicate, or object, on either
delta side). At the bound the rehearsal must PARK — revisiting an open loop
is lawful only a bounded number of times before the loop is handed off
instead of replayed. `bound == 0` is trivially exceeded (no rehearsal
budget at all).

Evidence: `tests/livelock.rs :: test_17_infinite_rehearsal_refused_at_bound`.

## Scope

This model detects graph-shaped livelocks and bounds rehearsal. It does not
diagnose people, replace recovery programs, or claim therapeutic effect —
see `docs/claims/WITHHELD_CLAIMS.md`.
