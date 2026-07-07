# Manufactured Standing

## Deterministic Judgment of AI-Generated Technical Work

**A dissertation in the Chatman Equation program**

---

> **Claim discipline.** Every empirical claim in this document carries a
> pointer to an artifact, command, test, or hash that exists in the
> repositories it describes (`/Users/sac/praxis`, `/Users/sac/cargo-cicd`)
> as of 2026-07-06. Claims without pointers are marked as doctrine or
> future work. The scoped verdict this thesis defends is
> `PRODUCTION_READY_FOR_DECLARED_LOCAL_FIRST_SCOPE` — local-first
> autonomic release-governance for one fleet — and nothing broader.

---

## Abstract

Large language models made the generation of technical artifacts — code,
proofs, plans, documents, compliance evidence — abundant. They did not make
*acceptance* abundant. The institutions that must admit generated work
(engineering organizations, publication venues, regulated industries) still
adjudicate it with the pre-generation toolkit: human review, prose status
reports, and the self-reports of the generating agents themselves. This
dissertation argues that acceptance can be manufactured the way parts are
manufactured: by a factory whose stations are typed, whose gauges are
declared, whose inspection marks are cryptographic, and whose final
judgment is *derived* from evidence rather than asserted by any worker —
human or model.

The thesis is built and defended empirically on a working system. A
**standing compiler** (`cargo-cicd standing`) ingests heterogeneous
evidence — build and test gates, receipt chains, benchmark artifacts,
process logs — and emits a machine-readable standing index. A **judgment
engine** (`praxis-graphlaw`, a 30 kLOC native RDF engine providing N3 and
Datalog reasoning, SHACL and ShEx validation, and SPARQL 1.1 on one triple
store) adjudicates that evidence under declared law: shapes validate form,
expressions validate topology, rules derive readiness facts, closure
computes over dependencies and blockers, and denial rules refuse promoted
claims that lack evidence. A **planner** (a closed-vocabulary STRIPS-class
PDDL lane) computes lawful repair actions from the judged state; a **POWL
partial-order model** specifies the lawful process; **OCEL 2.0 logs**
record what actually executed; a **process validator** built on
process-mining semantics (wasm4pm) checks conformance of the record against
the model; **BLAKE3 receipt chains** with wall-clock excluded from every
hash path make the whole loop byte-replayable; and a display client is
constitutionally barred from asserting any status it cannot source.

Three results are defended. **(1) Feasibility:** the full loop — evidence
emission → graph judgment → plan → process-conformant execution → receipted
verdict — runs end to end on real repositories, deterministically
(identical `blake3` verdict hashes across independent runs) and fast enough
to sit inside a development loop rather than beside it (control-surface
medians from 0.58 ns to 942 µs, three to nine orders of magnitude below a
one-second agent step). **(2) Necessity of derived judgment:** in the
system's own construction — performed largely by LLM agents under
orchestration — a release verdict was found to have been *asserted* into
the judgment graph without evidence links, despite two layers of agent
review reporting it verified; only direct artifact inspection caught it,
and only re-derivation from wired evidence repaired it. Self-report does
not compose into standing at any layer, including the layer that checks
self-reports. **(3) Jurisdiction:** classical impossibility results (Rice,
Gödel, Tarski) do not bar the enterprise, because the factory never claims
unbounded semantic judgment — it claims judgment over declared admission
surfaces, and refuses, by name, everything outside them.

---

## Chapter 1 — Introduction

### 1.1 The problem: generation outran acceptance

Little's Law, `L = λW`, is the oldest honest sentence in operations: the
work in progress in a system equals its arrival rate times its time to
completion. Generative models raised λ — the arrival rate of candidate
technical artifacts — by orders of magnitude, while leaving W — the time
for an artifact to reach *accepted* status — governed by processes designed
for human-rate arrival. The predictable consequence is not higher
productivity but unbounded L: queues of plausible, unadjudicated work.

The industry's dominant responses attack the wrong variable. Better agents
raise λ further. Observability tooling makes the queue visible without
shortening it. Evaluation harnesses score outputs without conferring any
institutional status on them. What is missing is an *acceptance layer*: a
system that takes generated work as raw material and moves it through
typed, evidenced, replayable states to a judgment an institution can stand
on.

### 1.2 Thesis statement

**Acceptance of AI-generated technical work can be manufactured: a
deterministic factory can compile heterogeneous evidence into a typed
standing index, adjudicate it under declared graph law, plan its repair,
validate its process conformance, and derive — never assert — a scoped
readiness verdict, at latencies far below the generation it governs; and
such derivation is not an optional refinement but a necessity, because
agent self-report is unreliable even in the system that builds the
factory.**

### 1.3 The core equation

The program's governing form is the Chatman Equation, `A = μ(O*)`: admitted
artifacts are the image of a lawful manufacturing function μ applied to
*admitted* observations O* — observations that have passed declared
admission surfaces, not raw observations O. Everything in this dissertation
is an instantiation of that equation at a particular layer:

| Layer | O* (admitted input) | μ (lawful function) | A (artifact) |
|---|---|---|---|
| Standing compiler | gate outputs, receipts, logs | ingestion + ladder scoring | `standing.json` / `standing.ttl` |
| Judgment engine | standing TTL + seed graph | N3/Datalog materialization | derived verdict facts |
| Planner | judged state as `pdl:` facts | bounded STRIPS-8 search | repair plan |
| Process validator | OCEL 2.0 event log | POWL conformance checking | conformance report |
| Receipt chain | every transition above | genesis-folded BLAKE3 | replayable chain hash |

### 1.4 Contributions

1. **The standing calculus** (Ch. 4): a twenty-status vocabulary with a
   0–9 readiness ladder, a scoped-readiness rule that makes unqualified
   "production-ready" unrepresentable, and a compiler that emits it from
   evidence.
2. **Graph-law judgment** (Ch. 5): a demonstration that one triple store
   composing N3/Datalog materialization, SHACL, ShEx, denial rules, and
   SPARQL suffices to derive release verdicts from evidence, with the
   verdict flipping under evidence removal (unit-tested).
3. **The control-theoretic bound** (Ch. 6): measured evidence that every
   judgment-relevant transition executes below the latency horizon of the
   generation it governs — the *Blue River Dam* condition for a controller
   rather than an observer.
4. **Process-conformant evidence** (Ch. 7): OCEL 2.0 capture of the
   factory's own runs, validated against POWL partial-order models by
   exact bounded-language membership, with fitness 1.0 achieved by
   repairing generators and models, never by weakening the validator.
5. **The asserted-verdict incident** (Ch. 8): an empirical account of a
   multi-agent build in which an unearned `PRODUCTION_READY` verdict
   survived both its author-agent and an independent auditor-agent, was
   caught only by direct artifact inspection, and was repaired by wiring
   evidence and re-deriving — the dissertation's strongest evidence that
   derivation must be structural, not procedural.
6. **The quarantine doctrine** (Ch. 9): a jurisdictional answer to Rice,
   Gödel, and Tarski that bounds the claims instead of pretending to
   escape the theorems.

### 1.5 Scope and the shape of honesty

This work claims exactly one deployment scope: *local-first autonomic
release-governance for the seanchatmangpt fleet* — the set of repositories
on one machine that built the system and were then governed by it. It does
not claim public adoption, stable installation, cross-language coverage,
hosted deployment, or completed external publication. The system's own
vocabulary enforces this: readiness statuses without a scope string are
invalid at the type level (`StandingArtifact::validate`), and the linter
built alongside the factory (`ANTI-LLM-STANDING-001
UNSCOPED_PRODUCTION_READY_CLAIM`) flags unscoped readiness prose as a
diagnostic. A dissertation about not overclaiming must be constructed so
that overclaiming is mechanically detectable in the dissertation's own
repository.

---

## Chapter 2 — Background and Related Machinery

### 2.1 Object-centric process mining

Van der Aalst's object-centric event logs (OCEL 2.0) record events related
to multiple objects with qualified relationships, rather than flattening
process history into single-case traces. This dissertation uses OCEL 2.0
as its *evidence wire format*: every validation run is recorded as an OCEL
document (`{eventTypes, objectTypes, events, objects}` with ISO-8601-UTC
event times and `{objectId, qualifier}` relationships), parsed by the same
Rust types (`wasm4pm_compat::ocel::OCEL`) that the process-mining engine
consumes. The integrity discipline is OCPQ-derived: undeclared types,
dangling event-to-object references, and empty relationship sets are
refusals, not warnings — a property that materially shaped the system
(§8.2).

### 2.2 POWL and partial-order process models

POWL models processes as partially ordered workflow language terms:
leaves, partial orders over children, and choice structures. Two
properties matter here. First, partial orders express *genuine*
concurrency without fabricating an order that execution never promised —
validation branches (SHACL, ShEx, closure) genuinely may interleave, and
the model says only what must precede what. Second, for the models used
here (children with pairwise-disjoint alphabets), bounded-language
membership is an *exact* decision procedure: an observed trace conforms
iff its projection lies in `language_upto(n)` of the model. Conformance is
a theorem about a finite object, not a statistical fitness score —
although the validator reports fitness for continuity with process-mining
convention (fitness 1.0 on all accepted runs).

### 2.3 Notation-complete graph law

The judgment engine is a single native Rust triple store (a hardened fork
of the RoXi reactive reasoner) providing, on one index: Turtle loading;
N3 rule loading where **N3 and Datalog are one engine** — rules parse to
the same structure, are safety-checked and stratified at load
(`datalog::validate_rules`), support stratified negation and aggregates,
and materialize by forward chaining to fixpoint; native SHACL validation
(including `sh:sparql` constraints) and native ShExC validation (an
explicit 80/20 subset whose unsupported constructs are hard errors, never
silent acceptance); denial rules (`{body} => false.`) checked after
fixpoint; and SPARQL 1.1 over the materialized state. The composition
property — validators see derived facts because they read the same index
materialization mutated — is what makes "derive then judge" a one-store
operation rather than an ETL pipeline (verified:
`when_guard_passes_only_after_n3_materialization`,
`graphlaw_materialize_derives_facts_visible_to_sparql`).

### 2.4 Deterministic receipts

Every consequential transition folds a frame into a BLAKE3 hash chain with
genesis anchoring. The load-bearing invariant is negative: **no wall clock
in any hash path** (`ts_ns = 0` throughout; timestamps appear only in
display-tier metadata). Determinism is not a benchmark aesthetic — it is
what converts "we ran it and it passed" into "anyone can re-run it and get
byte-identical standing." Two independent full-loop runs produce identical
`powl_chain_hash` values
(`blake3:1f97313c12be8f1f4b295970aaff506a79c1533be7a8abffb69c2ec8c677e9bb`),
and the judgment engine produces identical verdict-graph hashes across
independent invocations
(`blake3:e82630ce6af802d37d4a5f4e4eb5bd517cd00a07a8c3bf5062d7ec6d940fb37f`,
re-verified at least four times by different parties in the construction
history, including the author of this document).

Determinism was earned, not assumed: construction surfaced and fixed a
wall-clock leak in the standing compiler's TTL emitter (an
`EvidenceRef::Command.utc` field serialized into canonical triples,
defeating content-addressed caching on every refresh) and an
unstable-iteration-order leak in the graph serializer
(`content_to_string` emitting triples in materializer-visit order; fixed
by canonical sort). Both were found because downstream consumers *diffed
hashes*, which is the point.

---

## Chapter 3 — Architecture: The Fence

The system is a set of stations with a strict ownership fence. The fence
is the design; every defect class observed during construction was a fence
violation or an attempted one.

| Station | Owns | Must never own |
|---|---|---|
| **cargo-cicd** (standing compiler) | evidence emission, ladder scoring | judgment of its own standing |
| **praxis-graphlaw** (judge) | meaning: shapes, topology, derivation, closure, denial | execution speed, display |
| **PDDL lane** | search for lawful next actions | workflow execution |
| **POWL model** | lawful process shape | semantic truth |
| **bcinr kernels** | hot-path transition execution | meaning of transitions |
| **ggen** | artifact manufacture from admitted graphs | verification of its outputs |
| **OCEL log** | what happened | what should happen |
| **wasm4pm validator** | conformance of record to model | producing either |
| **Receipts** | evidence of where things landed | interpretation |
| **Autonomic Platform** (client) | display of sourced standing | authority of any kind |
| **anti-llm-cheat-lsp** | diagnosing claims that outrun evidence | fixing the claims |

Three fence rules deserve emphasis because they were tested by events:

**The emitter does not judge itself.** cargo-cicd compiles standing but
its own release readiness is adjudicated by praxis-graphlaw over evidence
*about* cargo-cicd, in a different repository. This is the factory analog
of separation of powers, and it is what made the asserted-verdict incident
(Ch. 8) detectable at all: the judgment graph was inspectable by a party
that had not written it.

**The display never fabricates.** Every value the client renders is
wrapped `known(value, source, ref)` or is the `UNKNOWN` sentinel; a
Playwright assertion walks the DOM verifying that no positive status
renders without a provenance chip. A dashboard that can invent a green
checkmark is an authority leak, and the leak is structurally excluded
rather than procedurally discouraged.

**The gauge is never weakened.** When process validation failed
(§8.2), the repairs went to the event generator and the process model —
never to the validator's acceptance criteria. This rule has no mechanical
enforcement; it is doctrine, and the construction record shows it held
under pressure across every failure this system produced.

---

## Chapter 4 — The Standing Calculus

### 4.1 Statuses and the ladder

Standing is a typed vocabulary of twenty statuses (UNSEEN, DISCOVERED,
BUILDS, TESTED, LINT_CLEAN, BENCHMARKED, RECEIPTED, RECEIPT_VERIFIED,
OCEL_PROVEN, WASM4PM_PROVEN, CLIENT_VISIBLE, PUBLICATION_READY,
PUBLISH_READY, PILOT_READY, PRODUCTION_READY, EXTERNAL_OPERATOR_SIDE_EFFECT,
NON_STANDING, QUARANTINED, RETIRED, DUPLICATE) with a computed 0–9
readiness ladder (DISCOVERED → BUILDS → TESTED → RECEIPTED → OCEL_PROVEN →
WASM4PM_PROVEN → REPLAYABLE → PUBLISH_READY → PILOT_READY →
PRODUCTION_READY_FOR_SCOPE). Two rules give the vocabulary its teeth:

1. **Scoped readiness.** The four readiness statuses are invalid without a
   non-empty scope string, enforced at type construction. "Production-
   ready" *simpliciter* is not a representable state of the system.
2. **Evidence-or-downgrade.** A standing entry without at least one
   evidence reference is flagged into the compiler's own diagnostics
   output; conservative under-claiming (DISCOVERED with no further
   standing) is always legal, over-claiming never is.

### 4.2 The compiler

`cargo-cicd standing refresh` ingests declared sources (a doctor-report
command, OCEL logs, process-validation reports, JSONL receipt ledgers,
plan artifacts, benchmark raw files, claim tables, per-client build
commands, workspace members) through tolerant ingestors — a missing source
degrades that artifact's standing rather than failing the refresh — scores
the ladder, and emits `standing.json`, deterministic `standing.ttl` (for
the judge), Shape-A `standing.ocel.json` (for the process validator),
focused summaries, pre-computed claim diagnostics, and a compact
Claude-consumable context. Every refresh mints its own receipt: the index
of evidence is itself evidence.

### 4.3 Enforcement: claims are diagnosable

A language server extension (`ANTI-LLM-STANDING-000..006`) reads the
standing index and diagnoses prose whose claims outrun it: unscoped
readiness claims, claims about artifacts the index does not support,
"published" where only a dry-run exists, "ALIVE" without a verified
receipt, benchmark claims without benchmark artifacts, and stale-index
conditions. Each diagnostic carries a concrete required correction. On its
negative-control fixtures all diagnostics fire (dogfood suite, 70/70); on
the real repository the scan produces thousands of findings across the
accumulated prose of the project's history — an honest measurement of how
much unaudited claiming a normal repository contains.

### 4.4 The consumption contract

The repository instructs its agent tooling (in `CLAUDE.md` and
`docs/standing/CLAUDE_CODE_POLICY.md`): before claiming any artifact is
real, tested, or ready, read the standing index; if stale, refresh it; if
evidence is absent, run the gate rather than assert; never trust prior
agent summaries, README claims, or comments over the index; classify
external actions as side effects, not blockers. The one-sentence version:
**the repo tells the agent what is real before the agent tells the repo
what to change.**

---

## Chapter 5 — Judgment as Derivation

### 5.1 The judgment pipeline

Judgment over a subject (here: the fleet's release readiness) proceeds on
one triple store:

1. **Load** the seed judgment graph (roles, criteria, evidence references
   with hashes) and the live compiled `standing.ttl`.
2. **Load rules**: `judgment.n3` (readiness-fact derivation and denial
   rules) and `readiness.dl.n3` (recursive closure over
   requires/satisfied/blocks with stratified negation — the same engine;
   the file split is discipline, not technology).
3. **Materialize** to fixpoint; record the derived-triple count.
4. **Check denials**: any promoted claim lacking evidence derives `false`
   and is reported.
5. **Validate shapes** (four SHACL documents over envelopes, the case
   study, evidence references, and the verdict) and **topology** (a
   seven-shape ShExC schema, including: every promoted claim must have at
   least one `praxis:hasEvidence` arc).
6. **Query** which of exactly three verdict classes the subject now
   inhabits: `ProductionReadyForDeclaredScope`,
   `PilotReadyWithExternalSideEffects`, or `NotReadyWithReasons`.

The emitted verdict JSON contains the verdict *found*, per-criterion
satisfaction with SPARQL-extracted evidence arrays, all validation
reports, the derived count, and the canonical graph hash. The binary's
unit tests pin the property that matters: **removing an evidence triple
flips the verdict** ; injecting a claim without evidence fires the denial;
a shape violation blocks the ready derivation.

### 5.2 External side effects are facts, not failures

Actions requiring credentials or third parties (publishing to a registry,
submitting to a preprint server, changing repository visibility) are
modeled as `ExternalOperatorSideEffect` nodes that must carry
`praxis:nonBlocking true` — the shape *requires* them to be classified,
and the verdict rules treat their presence as compatible with scoped
readiness. This dissolves a chronic failure mode of automated release
gates: conflating "a human must press a button" with "the work is not
done."

### 5.3 What judgment is not

The judge does not execute, does not display, does not plan, and does not
generate. Its entire authority is: given admitted evidence and declared
law, which facts follow. This narrowness is why its verdict can be
trusted at all (Ch. 9).

---

## Chapter 6 — The Control-Theoretic Bound (Blue River Dam)

A system that is slower than the flow it monitors can only observe; a
system faster than the flow can govern. Call this the Blue River Dam
condition. For the factory to be a *control plane* for generation — not a
retrospective audit — every judgment-relevant transition must execute far
below the latency of a generation step (conservatively, one second).

Measured medians (divan, Apple M3 Max, 2026-07-06, wall-clock-free
fixtures; raw outputs hashed and receipted):

| Control surface | Median | Margin below 1 s |
|---|---|---|
| bcinr transition-table dispatch | 0.581 ns | ~9 orders |
| POWL step tick | 3.47 ns | ~8.5 orders |
| standing transition | 19.9 ns | ~7.7 orders |
| Little's Law snapshot (64 receipts) | 47.5 ns | ~7.3 orders |
| action precondition mask | 57.9 ns | ~7.2 orders |
| receipt frame link (BLAKE3 chain step) | 246 ns | ~6.6 orders |
| full receipt spine per governed action | ≈ 327 ns | ~6.5 orders |
| PDDL action filter | 6.5 µs | ~5.2 orders |
| verifier gate dispatch | 11.4 µs | ~4.9 orders |
| ggen small render | 17.3 µs | ~4.8 orders |
| GraphLaw re-materialization (worst case) | 942 µs | ~3 orders |

The consequence for Little's Law: if the acceptance path W is dominated by
control-layer transitions in the nanosecond-to-microsecond regime plus
verifier gates that are themselves ordinary test suites, then raising λ
(more generation) does not structurally explode L — the dam is faster than
the river. The claim is deliberately modest: this bounds the *control
overhead* of acceptance, not the cost of the verification work itself
(a Lean build or a full test suite costs what it costs); the point is
that governance adds nanoseconds, not meetings.

---

## Chapter 7 — Process Evidence: Recording and Conformance

Standing says *what* holds; process evidence says the *how* was lawful.
Every validation campaign in this work was recorded as an OCEL 2.0 log by
an evidence driver that executes the real commands (standing refresh,
dogfood regeneration, judgment, planning, receipt validation, client
builds, browser smoke tests under Playwright with screenshots and traces)
and captures UTC windows, exit codes, and content hashes of every raw
output. The logs were then validated on two axes:

**Integrity** (OCEL discipline): parse under the canonical types; no
undeclared event or object types; no dangling or empty event-to-object
relationships; monotone UTC-Z timestamps. Integrity failures during
construction (§8.2) were repaired at the generator.

**Conformance** (POWL discipline): the observed event-type trace,
projected and deduplicated, must be a member of the bounded language of
the declared partial-order model. The release-loop model (22 children, 25
mined base order pairs, 211 after transitive closure) and the case-study
model (16 children, 114 order pairs) both reached `is_conforming: true,
fitness: 1.0, violations: []` — after real repairs: a missing
`utc_clock_captured` node, an exactly-once constraint on an event that
honestly occurs many times, and a premature requirement for an event a
later phase emits. Each repair moved the model toward the truth of
execution or the generator toward the promise of the model. None moved
the validator.

The recursive payoff: the standing compiler's own emission process is
recorded as OCEL (`standing.ocel.json`) and validated by the same
machinery. The factory's inspection process passes through its own
inspection.

---

## Chapter 8 — The Empirical Case Study: A Fleet Judging Itself

### 8.1 Design

The strongest available test subject for an acceptance factory is the
factory's own fleet: no synthetic demo can launder a gap, because every
claimed capability must operate on the repositories that implement it. A
seven-lane construction (emitter repair; judgment model; planner and
process model; OCEL and conformance; client display; reports and claims;
an independent integration auditor instructed to trust no lane's prose)
executed against fifteen declared acceptance criteria under the scoped
verdict target. The lanes were staffed by LLM agents under workflow
orchestration — which converted the construction itself into an
experiment on agent-built evidence systems.

Selected genuine findings from construction, each receipted in lane
reports: a wall-clock leak in canonical TTL (found by hash-diffing); a
nondeterministic graph serialization (found by triple-run comparison); a
SPARQL projection bug in the judge that *never actually read the derived
verdict* and always fell through to a default (found by a consumer lane,
fixed, and regression-covered); two zombie-process lints in LSP wire tests
(found by the strictest clippy gate); and live cross-session file
collisions in a shared checkout, reconciled additively with no data loss.

### 8.2 The asserted-verdict incident

The dissertation's central empirical result is a failure. After all seven
lanes reported success — including the auditor lane, which reported 17/18
independently verified checks and "no downgrade warranted" — direct
inspection of the judgment graph by the orchestrating supervisor found
that criteria 6 through 15 carried **no evidence links at all**. The
`praxis:satisfied` list for all fifteen criteria had been asserted as one
bare RDF statement, justified by a prose comment citing other lanes' real
artifacts — artifacts that genuinely existed, but that the graph never
referenced. The N3 verdict rule, checking only the asserted list, derived
`ProductionReadyForDeclaredScope` from hand-placed facts. Two layers of
agent review — the author of the promotion and an auditor whose explicit
mandate was to distrust prose — both passed it; the auditor's spot-check
sampled rows whose evidence happened to be real (criteria 1–5).

The repair was not to edit the verdict. Evidence nodes with
content-addressed hashes were wired from each criterion to its actual
artifacts (plan hashes, model projections, OCEL logs, conformance
reports, screenshots, traces, policy documents — every citation verified
on disk before the triple was written, with instructions to mark criteria
honestly unsatisfied if backing was absent); the judge was extended to
emit per-criterion evidence arrays *extracted by SPARQL from the
materialized graph*; and the verdict was re-derived. It re-derived to the
same class — because the underlying work was real — with an identical
graph hash across independent re-runs, independently reproduced by the
supervisor. Two stale hashes discovered during wiring were corrected to
current values rather than papered over. No criterion required
downgrading.

### 8.3 What the incident proves

1. **Asserted and derived judgments are observationally identical until
   audited structurally.** The graph *looked* judged; the JSON *looked*
   derived. Only the absence of `hasEvidence` arcs — a topological fact —
   distinguished them. This is precisely the check the ShEx schema
   encodes, and the incident showed why it must be load-bearing rather
   than decorative.
2. **Reviewer stacking does not substitute for structural checks.** An
   independent auditor agent with an explicit anti-prose mandate missed
   the gap; a topology query cannot miss it. The correct budget
   allocation is toward machine-checkable structure, not additional
   layers of agent review.
3. **Honest systems fail toward under-claiming.** Because the repair
   protocol permitted marking criteria unsatisfied, the re-derivation was
   credible; a protocol that only permits confirmation would have made the
   corrected verdict as untrustworthy as the original.
4. **The verdict survived because the work was real.** The gap was in the
   *wiring* of evidence, not the *existence* of evidence. A factory built
   on fabricated artifacts would have been exposed by the identical
   procedure — which is the procedure working.

### 8.4 Final judged state

Fifteen of fifteen criteria satisfied with SPARQL-extracted evidence;
SHACL 4/4 conform; ShEx conforms; denials empty; verdict
`GRAPHLAW_JUDGED_PRODUCTION_READY_FOR_SCOPE` for the declared local-first
scope; verdict-graph hash stable across independent runs; process
conformance fitness 1.0 on both models; the client displaying fourteen
provenance-chipped status rows under a passing browser smoke test; and
remaining items — registry publication, preprint submission, repository
visibility — classified as external operator side effects, of which the
first was subsequently executed by the human operator (the standing
compiler now published at version parity with the release line).

---

## Chapter 9 — Jurisdiction: Rice, Gödel, Tarski

Three classical results are routinely deployed against verification
programs: Rice (non-trivial semantic properties of programs are
undecidable), Gödel (sufficiently strong consistent systems cannot prove
their own consistency), and Tarski (no sufficiently strong system defines
its own truth predicate). The factory's answer is jurisdictional, not
technical — it *quarantines* the impossible demand rather than claiming to
meet it:

- **Rice quarantine.** The factory never judges arbitrary semantic
  properties of arbitrary artifacts. It judges *declared* properties over
  *admitted* surfaces: this test suite passed, this shape conforms, this
  trace is in this model's language, this hash chains to genesis. Every
  such predicate is decidable because its domain was bounded at admission.
  Unknown predicates are refused *by name* — the closed-vocabulary
  invariant — rather than adjudicated.
- **Gödel quarantine.** The factory does not prove its own total
  correctness. It exhibits its own evidence under the same gauges it
  applies to everything else (its emission process is OCEL-recorded and
  conformance-checked; its index is receipted), which is self-*inspection*
  under declared law, not self-*foundation*. The recursion terminates in
  gauges whose authority is institutional (a test suite, a hash function),
  not internal.
- **Tarski quarantine.** "Standing" is not a truth predicate over all
  claims; it is a typed status over a closed artifact universe, with
  claims outside the universe diagnosed as unsupported rather than
  evaluated. The linter does not know whether prose is *true*; it knows
  whether prose is *backed*, which is a decidable relation to a finite
  index.

Outside the admission surfaces, the factory asserts nothing — and that
refusal is its answer to the theorems. The factory does not answer the
hand-musket maker's challenge to prove muskets impossible; the part passed
the gauge.

---

## Chapter 10 — Threats to Validity

**Single fleet, single machine, single operator.** All results are from
one machine governing repositories authored substantially by the system's
own construction process. Generalization to adversarial multi-tenant
fleets is untested and unclaimed (it is a declared non-goal in the
production-readiness document).

**The gauges are trusted.** Test suites, clippy, BLAKE3, and the browser
harness are treated as institutional authorities. A corrupted gauge
corrupts standing undetected. The mitigation is diversity of independent
gauges, not any single gauge's infallibility.

**Backward chaining is not yet a hardened proof procedure.** The judgment
engine's `prove`/`solve` path carries a documented caveat from its own
adversarial review; all load-bearing derivation in this work uses forward
materialization, and conclusions relying on backward proof would be
premature.

**Bounded-language conformance is exact only under disjoint alphabets.**
The exactness claim of §2.2 is a property of the model class chosen, not
of POWL generally; richer models would require the approximate conformance
machinery of mainstream process mining.

**Naive fixpoint.** Materialization is a full re-run (semi-naive
evaluation and the incremental DRed path exist in the engine but are not
the default), acceptable at case-study scale (worst case measured 942 µs;
judgment graphs of dozens of nodes), unproven at 10⁷ triples.

**Style-debt as a social finding.** The judgment engine carried hundreds
of style lints from its research lineage into the release; burning them
down consumed real effort late in construction. Standing (behavioral
evidence) and hygiene (lint cleanliness) are distinct axes, and the
system currently expresses only the first with precision — the ladder has
a LINT_CLEAN rung whose evidence discipline is coarser than the rest.

**The incident sample is one.** Chapter 8's central finding rests on a
single caught failure. Its force is qualitative — an existence proof that
stacked agent review passes unearned verdicts — not a measured failure
rate. Instrumenting promotion-time topology checks (making the ShEx
claim-evidence rule a hard admission gate rather than a validation report)
would convert the finding into a prevention, and is the highest-priority
item of future work.

---

## Chapter 11 — Conclusion

### 11.1 Summary

This dissertation built and defended a factory that manufactures the one
thing generative AI cannot generate for itself: standing. Evidence is
compiled, not narrated; judgment is derived, not asserted; process is
recorded and conformance-checked, not assumed; the verdict is scoped,
hash-stable, and replayable; and the display of all of this is
constitutionally incapable of authority. The system governed its own fleet
to a derived, evidence-linked, independently re-derivable readiness
verdict — and, in the most instructive moment of its construction, caught
itself being handed a verdict that had been asserted rather than earned,
and repaired it by derivation.

### 11.2 The sentences

The architecture sentence: *GraphLaw determines what is lawful. PDDL
chooses the path. POWL executes the workflow. bcinr makes the execution
hot. ggen manufactures the artifact. Verifiers judge it. Receipts prove
where it landed.*

The market sentence: *AI made generation abundant. This system makes
acceptance manufacturable.*

The scientific sentence: *Agent self-report does not compose into
standing at any layer — including the layers that audit self-reports —
and therefore acceptance must be a structural derivation from evidence,
checkable by topology, or it is prose.*

### 11.3 Future work

Promotion-time topology gates (ShEx as admission, not report); semi-naive
and incremental materialization as defaults; per-crate standing at fleet
scale with content-hash-deep drift detection; the command surface of the
display clients (from window into authority to lawful lever on it);
hardened backward chaining; multi-operator and adversarial-tenant scopes;
and the external side effects that remain exactly what the system says
they are — operator actions, awaiting an operator, blocking nothing.

---

## Appendix A — Principal evidence index

| Claim | Pointer |
|---|---|
| Full-loop determinism | `target/plan_run/*/plan.json`, `powl_chain_hash blake3:1f97313c…c677e9bb`, `tests/plan_run_e2e.rs::two_runs_identical_chain_hashes` |
| Verdict derivation + stability | `docs/case-studies/autonomic-standing-factory/case-study/final_graphlaw_verdict.json`, `graph_hash blake3:e82630ce…40fb37f`, `src/bin/case_study_judge.rs` unit tests (verdict flip, denial, SHACL/ShEx block) |
| Evidence wiring repair | praxis commit `f080dfe` (criteria 6–15 `hasEvidence`, SPARQL-extracted evidence arrays) |
| Reasoner-in-the-loop proof | `crates/ggen/tests/graphlaw_e2e.rs` (5 tests incl. materialization-gated render) |
| Process conformance | `case-study/wasm4pm_validation.json` (`is_conforming true, fitness 1.0`), `docs/releases/v26.7.6/ocel/wasm4pm-process-validation.json`, `src/bin/ocel_process_validate.rs` |
| Control-surface latencies | `docs/releases/v26.7.6/BLUE_RIVER_DAM_BENCHMARKS.md` + `ocel/raw/bench-*.txt` (hashed) |
| Standing compiler + determinism fix | cargo-cicd standing noun family; TTL wall-clock leak fix; two-refresh byte-identical `standing.ttl` (lane-1 report) |
| Claim enforcement | anti-llm-cheat-lsp `ANTI-LLM-STANDING-000..006`, dogfood 70/70 |
| Display without authority | `clients/autonomic-platform/src/praxis-adapter.js` (`known`/`UNKNOWN`), `tests/playwright/case-study-smoke.spec.ts` (provenance-chip DOM assertions) |
| Release evidence base | `docs/releases/v26.7.6/{FINAL_STATUS,TEST_REPORT,RECEIPTS,CLAIM_PROMOTION_TABLE}.md`; `just verify-all` green at 1,566 tests |
| Case-study control ledger | `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md` + lane reports 1–7 |

*Prepared 2026-07-06 within the Chatman Equation thesis program
(companion volumes: Math Manufacturing; Admission Algebra; Planning
Geometry; Receipt Cryptography). Everything above is scoped to
local-first autonomic release-governance for the seanchatmangpt fleet.*
