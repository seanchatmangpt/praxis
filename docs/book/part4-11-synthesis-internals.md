# Synthesis Internals

This chapter covers `crates/praxis-synthesis`, walking outward from the
wire-format contract (`docs/SYNTH_V1.md`) through the eight v26.7.3 internals
documents in `docs/v26.7.3/`. Every constant and file/line citation below was
re-read from source for this chapter rather than copied from a prior audit.

## synth/v1: the wire contract

`docs/SYNTH_V1.md` specifies the one payload shape a foreign agent needs to
drive the synthesis pipeline, whether over the CLI (`synth run` / `synth
solve`) or the membrane (`docs/SYNTH_V1.md:1-5`). A **term** is either a
variable (`"?N"`, N < 8) or an interned constant; an **atom** is `["pred",
["t1", ...]]` with arity at most 8; facts must be ground
(`docs/SYNTH_V1.md:7-10`).

The payload has seven top-level fields — `facts`, `rules`, `capabilities`,
`goal`, `horizon`, `constraints`, `solver` — each with a stated bound
(`docs/SYNTH_V1.md:15-42`): facts capped at ≤10⁸ total tuples; rules must be
stratified Horn clauses with every head/negated variable bound by a positive
body atom, ≤8 vars; capabilities' effect variables must be bound by
preconditions; horizon ≤16 steps; constraints ≤64 (8×8, eight kinds). The
solver is either `"solver8"` (a propagating solver that certifies
unsatisfiability) or `"brute"` (described as the differential oracle).

The eight constraint kinds are listed exhaustively
(`docs/SYNTH_V1.md:44-55`): `Before`, `After` (sugar for reversed `Before`),
`NotLater`, `NotEarlier`, `Excludes`, `Requires`, `AtMost`, `Budget`.

A successful `synth solve` returns a `status: "solved"` document with a
saturation fixpoint hash, a plan (steps with capability + binding, cost), and
a receipt (`nodes_explored`, `problem_hash`, `plan_hash`)
(`docs/SYNTH_V1.md:59-62`). `synth run` additionally executes the plan as a
content-addressed DAG with BLAKE3 per-node outputs and memoized replay, then
admits it through six refinements, returning one `SynthesisReceipt` whose
`chain` commits the entire run (`docs/SYNTH_V1.md:64-67`). Refusals — the
document's example is an `UnsatProof` for a missing precondition fact — are
results, not errors, and carry a `core` that the document calls a
certificate: re-propagating the core alone reproduces the impossibility,
without re-searching (`docs/SYNTH_V1.md:69-81`). The conformance fixture is
named directly: the five capabilities in
`crates/praxis-synthesis/tests/common/mod.rs`, under `solver8` at horizon 6,
must yield exactly `supply-evidence → clear-obligations → judge → admit →
receipt` at cost 5 (`docs/SYNTH_V1.md:85-89`).

## Datalog engine bounds — verified against source

The synth/v1 contract's per-field bounds (≤8 vars, stratified rules) trace
to concrete constants in `crates/praxis-synthesis/src/datalog.rs`, which this
chapter re-reads directly rather than trusting the wire-format doc's prose:

```
pub const MAX_TUPLES: u64 = 100_000_000;      // datalog.rs:40
pub const MAX_ITERATIONS: u64 = 10_000;       // datalog.rs:42
pub const MAX_STRATA: usize = 8;              // datalog.rs:44
pub const MAX_VARS: usize = 8;                // datalog.rs:46
```

`MAX_TUPLES` backs the relation store's capacity
(`rels: RelStore::with_cap(MAX_TUPLES)`, `datalog.rs:171`) and is enforced on
every ground-fact insertion (`datalog.rs:226`). `MAX_VARS` is enforced when a
rule is added — any bound variable index ≥ `MAX_VARS` is refused
(`datalog.rs:236-241`). `MAX_STRATA` bounds stratification during rule
safety/negation-cycle checking, refusing a predicate whose stratum need
reaches the bound with an explicit "negation cycle" message
(`datalog.rs:327-343`). `MAX_ITERATIONS` bounds the semi-naive fixpoint
evaluation loop itself, checked at two evaluation sites
(`datalog.rs:561-564`, `datalog.rs:646-649`), each returning a budget-exceeded
error naming `MAX_ITERATIONS`. Together these four constants are the actual
enforcement mechanism behind SYNTH_V1's "≤8 vars," "stratified," and
"≤10⁸ tuples" language — the wire-format document states the contract; the
datalog engine is where it is mechanically checked.

`crates/praxis-synthesis/src/agent_registry.rs` defines a fifth bound in the
same family:

```
pub const MAX_AGENTS: usize = 8;              // agent_registry.rs:42
```

enforced at agent-declaration time — a subject count exceeding `MAX_AGENTS`
is refused as `Refusal::AgentIllFormed`, with the actual count and the bound
both named in the error message (`agent_registry.rs:95-98`). This is a
distinct registry from the handler registry described below (`handlers.rs`),
but shares the same "8-bound, closed-vocabulary" discipline that recurs
across the crate: 8 constraint kinds, 8 vars, 8 strata, 8 hooks (see below),
8 agents.

## Rice quarantine: the only door into the admitted graph

`docs/v26.7.3/RICE_QUARANTINE.md` describes `crates/praxis-synthesis/src/quarantine.rs`
and `src/delta.rs` as the sole entry point for any candidate change to the
admitted RDF graph (`RICE_QUARANTINE.md:1-3`). The pipeline is: a
`MeaningSource` (raw `adds_ttl`/`removes_ttl` bytes plus a declared `Origin`
— `Operator`, `Proposer`, or `Bridge`) goes through `RiceQuarantine::inspect`,
which runs only decidable checks and produces a `GraphDelta`; then
`Admission::admit` applies that delta against a `Reference`, checks the
post-state's closed-world `wf:` vocabulary, recomputes the post hash, and
increments an epoch counter (`RICE_QUARANTINE.md:6-14`).

The document is explicit that Origin carries no trust weight: every origin
passes the identical decidable checks, so an LLM proposer's output (Origin
`Proposer`) faces the same parser, the same caps, and the same admission gate
as an operator's — and there is no executable-content shape at all, only
triples, so nothing an LLM proposes can execute past this gate
(`RICE_QUARANTINE.md:20-22`, `RICE_QUARANTINE.md:64-67`). The cap named is
`MAX_DELTA_TRIPLES = 64` per side, in `src/delta.rs`
(`RICE_QUARANTINE.md:31`). Removing a triple absent from the base graph is a
distinct refusal (`Refusal::AdmissionRefused`), because retracting something
never admitted would silently rewrite history
(`RICE_QUARANTINE.md:42-44`). Hash discipline separates two hashes: the
delta's `event_hash` is computed from its canonical (surface-invariant) form,
while `delta_ttl_hash` names the exact surface bytes but is a receipt field
only, never folded into any chain (`RICE_QUARANTINE.md:69-74`).

## Hooks and RDF-to-PDDL grounding

`docs/v26.7.3/RDF_TO_PDDL_HOOKS.md` covers `src/hooks.rs` (the hook registry)
and `src/ground.rs` (the RDF-event-to-action connector). A hook is a tuple
`h = (trigger, check, act, receipt)` declared as a `hook:Hook` node directly
in the admitted graph, so hook declarations are content-addressed by the same
law-hash that covers the rest of the graph (`RDF_TO_PDDL_HOOKS.md:6-12`). The
registry bound is `MAX_HOOKS = 8`, and every registered hook produces a
verdict — `Fired`, `NotFired`, or `Gated` — on every event, so silence about
whether a hook fired is provable rather than assumed
(`RDF_TO_PDDL_HOOKS.md:22-25`).

Five condition kinds are supported: `datalog`, `delta`, `threshold`, `count`,
`window` (`RDF_TO_PDDL_HOOKS.md:29-40`). A separate, explicit list of refused
kinds — `sparql-ask`, `sparql-select`, `shacl`, `n3`,
`semantic-inference` — is refused by name at registration, each paired with
its honest bounded analog rather than silently dropped
(`RDF_TO_PDDL_HOOKS.md:44-55`). The grounding connector,
`ground_fired_action` in `src/ground.rs`, restricts the admitted post-graph
to exactly the fired action's `wf:Workflow` fragment (closure under `wf:`
references, minus every other `wf:Workflow` typing) before running it through
the existing IR-extraction → lowering → Solver8 → topology/geometry →
supervised-execution chain — no new solver, no synthesized actions at firing
time (`RDF_TO_PDDL_HOOKS.md:57-77`).

## AA livelock modeling

`docs/v26.7.3/AA_LIVELOCK.md` documents `src/livelock.rs` as a model of six
"human livelock" classes, framed as workflow soundness rather than clinical
diagnosis: a livelock is a loop that consumes cycles without progressing the
graph (`AA_LIVELOCK.md:1-5`). The six classes — Resentment, Shame, SelfWill,
Fear, ReliefSeeking, SpilledMilk — each have an "open when" condition and a
"closed by" condition expressed as datalog goal predicates
(`AA_LIVELOCK.md:13-20`), evaluated through the same hook engine
(`hooks.rs::eval_datalog`) that fires knowledge hooks, so livelock detection
and hook firing share one evaluator (`AA_LIVELOCK.md:23-26`).

The document maps the traditional twelve-step recovery statements onto twelve
named "soundness operations" (e.g., step 3, "decided to turn will and life
over," maps to "self-will control transfer: the selfPlan gains a surrendered
edge; scheduling moves to the solver") as a code constant,
`livelock.rs::STEPS`, which the document calls "testable and
content-addressable, not prose" (`AA_LIVELOCK.md:34-39`). A no-infinite-
rehearsal law, `rehearsal_exceeded(history, loop_iri, bound)`, is true iff at
least `bound` deltas in history touch the loop IRI; at the bound, the
rehearsal must "PARK" — an open loop can only be lawfully revisited a bounded
number of times before being handed off instead of replayed
(`AA_LIVELOCK.md:56-63`). The document is careful about scope in its own
closing section: "This model detects graph-shaped livelocks and bounds
rehearsal. It does not diagnose people, replace recovery programs, or claim
therapeutic effect" (`AA_LIVELOCK.md:68-72`).

## Agent delegability lattice

`docs/v26.7.3/AGENT_DELEGABILITY.md` documents `src/handlers.rs`, describing
an ordered lattice `human-only < assistive < automatable < verifiable`
implemented as a derived-`Ord` Rust enum, `Delegability`
(`AGENT_DELEGABILITY.md:9-23`). A `wf:Capability` node opts into automated
handling with `wf:handler <iri>`, and must then also declare
`wf:delegability` — there is no default grade, and an undeclared grade on a
handler-bearing capability is refused as `Refusal::WorkflowIllFormed`
(`AGENT_DELEGABILITY.md:26-31`). The closed handler registry
(`HandlerRegistry::builtin()`) is stated to contain exactly one handler IRI,
matched by exact-key membership only — no prefix/suffix matching is
representable in the API — with an unknown IRI refused before any solving
begins (`AGENT_DELEGABILITY.md:36-42`).

The eligibility rule for automated execution is two-part: every `wf:handler`
IRI anywhere in the admitted graph must be in the closed registry
(a global, pre-solve check), and for every capability the fired action's
*actually-used* derived plan touches, the declared delegability must be
`automatable` or above — scoped narrowly enough that a `human-only` binding
on a capability no fired plan touches cannot poison an unrelated firing
(`AGENT_DELEGABILITY.md:48-62`). The document's worked example: a
`release-resentment` capability declared `human-only` causes any firing whose
derived plan would execute it to be refused with `DelegabilityViolation`,
chained rather than silent (`AGENT_DELEGABILITY.md:64-70`).

## The Lord's Prayer kernel — with its disclaimer intact

`docs/v26.7.3/LORD_PRAYER_KERNEL.md` documents `src/kernel.rs` and
`ontology/lord_prayer.ttl`. Reading `src/kernel.rs` directly confirms the
document's central structural claim: the file's own module comment states
"The Lord's Prayer kernel — all 11 clauses as typed, closed-world nodes"
(`kernel.rs:1`), and `pub const CANONICAL_CLAUSES: [&str; 11]` is declared at
`kernel.rs:39`, with `pub const BOUNDARIES: [&str; 3]` at `kernel.rs:34` — an
independent confirmation of the 11-clause, 3-boundary structure the doc
describes.

The kernel declares all 11 clauses of the prayer as typed graph nodes under
the namespace `http://seanchatmangpt.github.io/praxis/prayer-kernel#`, with
extraction (`extract_kernel`) enforcing exact coverage — all 11, no more, no
fewer, no duplicates, any deviation refused as `Refusal::KernelIllFormed`
naming the offending clause (`LORD_PRAYER_KERNEL.md:5-11`). The 11-clause
table, reproduced from `ontology/lord_prayer.ttl` as the document states it:

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

(`LORD_PRAYER_KERNEL.md:22-35`)

The three lawful boundary strings, per the document and independently visible
as the `BOUNDARIES` constant in `kernel.rs:34`:

- `human-only` — the act is reserved for the human; agents may support,
  never execute.
- `god-receives-unbounded` — the object of the clause is surrendered, not
  computed.
- `automatable-support` — a bounded support workflow may assist the human
  act.

(`LORD_PRAYER_KERNEL.md:37-44`)

On the God boundary specifically, the document states: God is never modeled
as an agent, handler, capability, or tool; the boundary is a string property
on a clause node that never resolves to an executable node of any kind, and
the deliverance hook's effect is `refuse` with a declared surrender reason —
an `UnboundedThreat` fact yields a chained refusal receipt, never a plan
(`LORD_PRAYER_KERNEL.md:45-59`).

`kernel_hash` is the content address of byte-sorted canonical lines
(`name\tproblem_class\tboundary\taction`), computed from the extracted
clauses rather than asserted in source, and the document states it is stable
under any surface reordering of the TTL because the extractor sorts clauses
into canonical scriptural order first and the hash lines are byte-sorted
besides (`LORD_PRAYER_KERNEL.md:62-69`).

**The document's own disclaimer, reproduced exactly as it frames it**
(`LORD_PRAYER_KERNEL.md:71-78`, section heading "What the kernel is not"):

> The kernel does not automate prayer, interpret scripture, or claim moral
> completeness (see `docs/claims/WITHHELD_CLAIMS.md`). It is a typed index
> from clause to problem class to a bounded, pre-declared support action —
> and the clause execution ORDER in the demo workflow is derived by the
> solver from declared preconditions, never authored
> (`tests/prayer_kernel.rs :: provision_anxiety_grounds_the_daily_prayer_workflow`).

This disclaimer is load-bearing for how the kernel should be read: it is a
typed vocabulary and delegation-boundary index over an ontology file, not an
automation of the prayer or a theological claim, and the source document
frames it that way itself rather than leaving that inference to the reader.

## Receipts, replay, and foreign verification

`docs/v26.7.3/RECEIPTS_REPLAY_VERIFY.md` describes two nested receipt chains.
The inner chain, `praxis:workflow:v1`, is produced once per executed
workflow fragment and folds graph/IR/plan/topology/geometry/exec stages; the
document states the hook layer folds this chain as an event and never
mutates it, so direct execution and hook-fired execution of the same
fragment produce byte-identical inner chains
(`RECEIPTS_REPLAY_VERIFY.md:7-14`). The outer chain,
`praxis:hook-firing:v1`, folds in order: `event_hash`, `admission_hash`,
`handler_hash`, `hook_hash`, `history_hash` (a window-history commitment
over the first 7 preceding deltas), then the inner chain per fired action,
then `outcome_hash` (`RECEIPTS_REPLAY_VERIFY.md:16-33`). `delta_ttl_hash` is
again named as a receipt field only, never folded — the same ttl_hash
doctrine as in the quarantine document.

Replay (`firing.rs::replay_firing`) re-derives the whole firing from the base
TTL and delta documents and compares stage by stage in fold order, then binds
every embedded payload (admission record, bindings, verdicts, outcome, inner
chains) to the hash just verified — the document's framing: "a receipt
cannot vouch for itself" (`RECEIPTS_REPLAY_VERIFY.md:50-59`).

Foreign verification is scoped carefully rather than oversold. The document's
own scope statement: the foreign firing verifier independently re-derives
the graph-side authority chain (event hash, post-state apply,
re-canonicalization, admission record, handler bindings) but does not
independently re-run the hook evaluator or execution runtime — hook verdicts
and outcomes are verified by payload binding (hashing the embedded bodies and
comparing to the receipt folds), not by re-execution
(`RECEIPTS_REPLAY_VERIFY.md:66-79`). Three named limitations follow this
statement explicitly: `hook_hash`/`outcome_hash` are refolded from embedded
payloads rather than re-run in Python; inner v1 plan/topology/geometry stage
hashes inside a firing receipt are refolded as claimed rather than re-derived;
and `history_hash` is folded as claimed from the receipt field rather than
re-derived from an actual history, since the `firing` CLI subcommand takes no
history input (`RECEIPTS_REPLAY_VERIFY.md:109-129`). `bash
scripts/trustless_replay.sh` verifies packaged receipts in a bare environment
containing only `python3` and `b3sum` on PATH — no cargo, no crate source
(`RECEIPTS_REPLAY_VERIFY.md:131-136`).

## Port candidate census and Definition of Done

`docs/v26.7.3/PORT_CANDIDATE_CENSUS.md` records a scan of 386 repositories
against 8 candidate components, resulting in 8 IMPORT recommendations and 0
ADAPT/REWRITE recommendations, with 3 REFUSE entries and no security flags
(`PORT_CANDIDATE_CENSUS.md:3-13`). The four candidates it calls decisive are
the Lord's Prayer Kernel, Rice Quarantine, the Solver8 planner, and the
Replay/Foreign Verifier — each already resident at the source paths named
above (`PORT_CANDIDATE_CENSUS.md:11`, `PORT_CANDIDATE_CENSUS.md:52-98`). The
refuse list is instructive about what this crate deliberately does not pull
in: a blank-node color-refinement canonicalizer from `ggen-graph` is refused
because blank nodes are prohibited in praxis-synthesis to preserve a strict
linear-sorting invariant; an Oxigraph SPARQL/SHACL validator is refused
because the crate uses a zero-dependency stratified Datalog engine instead,
avoiding a heavy SPARQL engine on the hot path; and a Go governance registry
is refused as non-functional in the Rust runtime target
(`PORT_CANDIDATE_CENSUS.md:102-118`).

`docs/v26.7.3/DEFINITION_OF_DONE.md` is a 20-gate table, each row citing both
an implementation file and a named test proving it, with the document's own
framing that "a gate without a cited test is not claimed" and that every hash
in the system is computed at run time and re-derived at verification time,
never asserted (`DEFINITION_OF_DONE.md:3-6`). Gate 18 is "No LLM in
runtime," backed by `Cargo.toml`'s stated six-offline-dependency allowlist
and a test named `dependencies_are_exactly_the_offline_allowlist`
(`DEFINITION_OF_DONE.md:29`). The document's closing section states plainly
what "done" does not mean: no claim of completeness beyond the cited tests,
with a `docs/claims/WITHHELD_CLAIMS.md` register listing what this version
deliberately does not claim (`DEFINITION_OF_DONE.md:57-61`).

## The shape that recurs

Across all eight documents, three disciplines repeat: an 8-bound closed
vocabulary wherever a registry or grammar could otherwise grow unboundedly
(8 constraint kinds, `MAX_VARS = 8`, `MAX_STRATA = 8`, `MAX_HOOKS = 8`,
`MAX_AGENTS = 8`); a hash-then-verify discipline where every receipt field is
computed and re-derivable rather than asserted, with a named `_ttl_hash`
convention kept deliberately out of any hash chain; and a refuse-by-name
discipline where anything the crate cannot bound or decide is rejected with
a typed `Refusal` naming the exact reason, rather than silently ignored or
best-effort approximated. The Rice quarantine's Origin-blindness, the hook
registry's named refusal of `sparql-ask`/`shacl`/`n3`, and the delegability
lattice's exact-key-only handler lookup are three independent
instantiations of that same refuse-by-name discipline.
