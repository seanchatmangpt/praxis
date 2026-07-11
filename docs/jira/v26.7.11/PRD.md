# Multifractal Workflow v26.7.11

## Product Requirements Document

**Version:** 26.7.11  
**Status:** BUILD TARGET  
**Product:** Multifractal Workflow Runtime  
**Architecture:** Chatman Ecosystem / GraphLaw / wasm4pm  
**Canonical thesis:** Standing-preserving recursive work across heterogeneous local process geometries  
**Core law:** \(A=\mu(O^*)\)  
**Receipt law:** \(R=\operatorname{receipt}(A)\)

---

# 1. Product Decision

v26.7.11 SHALL implement Multifractal Workflow as the bounded outer-workflow architecture of the Chatman Ecosystem.

The release SHALL preserve the existing standing law at every execution scale:

\[
O^* \rightarrow \mu \rightarrow A \rightarrow R
\]

The release SHALL NOT create a new general-purpose workflow authority.

The canonical production path SHALL be:

\[
\text{admitted graph}
\rightarrow
\text{PDDL}
\rightarrow
\text{POWL v2}
\rightarrow
\text{external cut}
\rightarrow
\text{SPARQL projection}
\rightarrow
\text{Tera rendering}
\rightarrow
\text{Arazzo}
\rightarrow
\text{wasm4pm AIR}
\rightarrow
\text{Erlang transition core}
\rightarrow
\text{OTP or AtomVM}
\rightarrow
\text{broker}
\rightarrow
\text{admitted consequence}
\rightarrow
\text{receipt/replay}
\]

POWL v2 SHALL remain the process-geometry authority.

Arazzo SHALL be a manufactured inter-engine workflow artifact.

wasm4pm SHALL parse and compile Arazzo into AIR.

Erlang SHALL own the shared outer transition semantics.

OTP SHALL supervise long-lived distributed workflow execution.

AtomVM SHALL preserve equivalent AIR transition semantics on constrained execution surfaces.

BCINR SHALL remain the local POWL runner for chip-scale execution.

The broker SHALL remain the only DO path.

---

# 2. Problem

Enterprise work is currently reconstructed by human interpreters from partially typed observations.

Humans infer:

- process membership;
- current state;
- ownership;
- authority;
- unresolved goals;
- dependency structure;
- child obligations;
- evidence of completion;
- safe next actions;
- compensation requirements;
- residual open work.

This creates an undocumented semantic execution layer between observation and actuation.

The system already has standing-preserving manufacture for admitted artifacts. The missing product surface is a lawful outer runner for recursive workflows that cross independent Chatman Engines, networks, human wait states, long timeouts, and runtime restarts.

A local POWL/BCINR runner is insufficient for that scale.

A single-engine multi-agent runtime is explicitly rejected because it collapses independent process cells into ambient shared execution state.

v26.7.11 SHALL add an outer workflow rail without moving standing authority out of POWL, GraphLaw, admission, or the broker.

---

# 3. Product Thesis

Work is not admitted as unrestricted semantic behavior.

Raw observation \(O\) SHALL first become admitted observation \(O^*\).

Manufacture \(\mu\) SHALL operate only over \(O^*\).

Artifacts SHALL gain standing only through declared manufacture and admission.

Real-world consequence SHALL re-enter as raw observation and SHALL NOT unlock dependent work until re-admitted.

The product SHALL treat recursive workflow as scale-invariant execution law with heterogeneous local geometry.

Every workflow activity MAY be a typed socket for an admitted child workflow.

A child workflow MAY execute:

- locally in the same process cell;
- in another Chatman Engine;
- through a human execution surface;
- on a constrained AtomVM surface.

Substitution SHALL preserve parent standing only when type, authority, boundary, and closure law are satisfied.

Recursive descent SHALL be bounded.

---

# 4. Goals

v26.7.11 SHALL:

1. Detect external execution cuts in admitted POWL v2 geometry.
2. Deterministically project those cuts into Arazzo.
3. Compile manufactured Arazzo into a compact AIR representation in wasm4pm.
4. Execute AIR through a pure Erlang transition core.
5. Run long-lived inter-engine workflows under OTP supervision.
6. Preserve the same transition semantics under AtomVM.
7. Dispatch only through the broker.
8. Correlate every returned consequence with its dispatch and source workflow.
9. Re-admit external consequences before process closure.
10. Enforce explicit parent-child closure laws.
11. Manufacture compensation as workflow.
12. Emit declared OpenTelemetry evidence.
13. Admit telemetry into RDF and derive OCEL by SPARQL CONSTRUCT.
14. Produce BLAKE3 receipt chains and replay witnesses.
15. Measure heterogeneous workflow scaling from admitted OCEL evidence.
16. Preserve executable GraphLaw authority boundaries.
17. Add theorem-bearing product laws to Lean/Lake where the claims are formalizable.

---

# 5. Non-Goals

v26.7.11 SHALL NOT:

- accept arbitrary Arazzo as production workflow authority;
- allow Tera templates to define topology;
- allow Arazzo to define parent-child closure law;
- replace POWL v2 with Arazzo;
- replace BCINR for local chip-scale POWL execution;
- create a second workflow planner;
- permit language models to invent production plans from unrestricted text;
- permit agents, scripts, CLIs, N3, SPARQL, or workflow runners to actuate directly;
- share hidden mutable semantic state between independent Chatman Engines;
- treat Erlang PID as workflow identity;
- treat logs as receipts;
- treat child-reported `done` as parent closure;
- imperatively construct OCEL in business code;
- make multifractal classification a prerequisite for basic workflow correctness;
- silently promote `UNKNOWN` or `UNSUPPORTED` to an admitted state.

---

# 6. Governing Laws

## 6.1 Standing Law

\[
A=\mu(O^*)
\]

Raw observation has no production standing.

## 6.2 Receipt Law

\[
R=\operatorname{receipt}(A)
\]

A receipt SHALL bind artifact identity, inputs, manufacture path, conformance, actuation consequence, and replay identity.

## 6.3 Broker Law

\[
\{a:\operatorname{actuate}(a)\land\neg(R\vdash a)\}=\varnothing
\]

There SHALL be zero unreceipted actuation.

## 6.4 Execution-Law Invariance

For every admitted child workflow \(W_i\) of workflow \(W\):

\[
\mathcal L(W_i)\cong\mathcal L(W)
\]

The structural classes observation, admission, authority, manufacture, actuation, consequence, refusal, receipt, and replay SHALL recur at every workflow scale.

## 6.5 Substitution Closure

For admitted workflow \(W\), socket \(a\), and compatible child \(W'\):

\[
W[a\mapsto W']\in\mathcal W
\]

only when the child satisfies the socket's declared type, authority, boundary, and closure contract.

## 6.6 Bounded Descent

Each recursive substitution SHALL consume budget or decrease a declared well-founded measure.

The initial cost vector SHALL support:

\[
C(W)=\langle d,a,u,r\rangle
\]

where:

- \(d\): remaining decomposition depth;
- \(a\): unresolved activities;
- \(u\): unresolved uncertainty;
- \(r\): unresolved resource dependencies.

Required relation:

\[
C(W_{child}) <_{lex} C(W_{parent})
\]

A child that cannot establish descent SHALL be refused.

---

# 7. Canonical Architecture

## 7.1 Layer 1 — Admitted Graph

Oxigraph-backed admitted graph state SHALL be the process truth substrate.

GraphLaw SHALL route declared dialects under explicit authority.

No runtime layer SHALL infer ambient authority from syntax.

## 7.2 Layer 2 — PDDL

PDDL SHALL own admitted action possibility.

PDDL MAY declare:

- predicates;
- objects;
- action parameters;
- preconditions;
- effects;
- initial state;
- goals.

PDDL SHALL NOT own workflow runtime semantics.

## 7.3 Layer 3 — POWL v2

POWL v2 SHALL own canonical process geometry:

- partial order;
- parallelism;
- choice;
- bounded loop;
- hierarchy;
- recursive workflow sockets;
- external cuts;
- parent-child closure.

Every POWL activity SHALL be addressable as a potential workflow socket.

## 7.4 Layer 4 — External Cut Projection

A declared external cut SHALL identify a POWL region whose execution boundary leaves the current process cell.

Projection SHALL be deterministic.

The only production projection SHALL be:

\[
A_z=T(Q(W))
\]

where:

- \(W\) is the admitted POWL region;
- \(Q\) is the declared SPARQL projection;
- \(T\) is the declared Tera renderer;
- \(A_z\) is the manufactured Arazzo artifact.

The projection receipt SHALL bind all four objects plus their digests.

## 7.5 Layer 5 — Arazzo

Arazzo SHALL carry generated inter-engine workflow structure.

Arazzo MAY represent:

- step dependencies;
- operation dispatch;
- workflow invocation;
- criteria;
- success routing;
- failure routing;
- retries;
- timeouts;
- correlation;
- output binding.

Arazzo SHALL NOT invent process topology or authority.

Production Arazzo without an admitted POWL source and projection receipt SHALL be refused.

## 7.6 Layer 6 — wasm4pm AIR Compiler

The Arazzo parser/compiler SHALL live in wasm4pm.

wasm4pm SHALL perform:

- complete document parsing;
- reference resolution;
- dependency normalization;
- expression compilation;
- criterion normalization;
- operation classification;
- timeout normalization;
- correlation extraction;
- typed refusal.

wasm4pm SHALL emit AIR.

AIR SHALL remove YAML/JSON syntax, JSON Pointer traversal, and Arazzo document interpretation from the runtime hot path.

The AIR semantic core SHALL be limited to:

- dependency readiness;
- dispatch;
- workflow invocation;
- expression evaluation;
- criteria evaluation;
- success routing;
- failure routing;
- retry;
- timeout;
- correlation;
- output binding;
- completion.

## 7.7 Layer 7 — Shared Erlang Transition Core

The AIR state transition implementation SHALL be pure Erlang.

The canonical transition SHALL be modeled as:

\[
\delta_{AIR}:(S,E)\rightarrow(S',C)
\]

where:

- \(S\) is workflow state;
- \(E\) is an admitted runtime event;
- \(S'\) is the next state;
- \(C\) is a finite set of requested commands/consequences to route through lawful surfaces.

The transition core SHALL NOT directly perform I/O or actuation.

OTP and AtomVM SHALL wrap the same transition core.

## 7.8 Layer 8 — OTP Outer Runner

Each external workflow instance SHALL run as a supervised process.

Workflow semantic identity SHALL be independent of PID.

Minimum workflow identity:

- workflow ID;
- parent workflow ID;
- Arazzo workflow ID;
- source POWL region ID;
- dispatch ID;
- correlation ID;
- source digest;
- projection digest;
- receipt head;
- replay ID.

The OTP runner SHALL survive execution-process restart by reconstructing from admitted state and replay surfaces.

The runner SHALL react to:

- start;
- dispatch-ready;
- acknowledgment;
- result;
- timeout;
- retry-due;
- child-complete;
- child-refused;
- admission-result.

## 7.9 Layer 9 — AtomVM Runner

AtomVM SHALL execute the same AIR transition semantics.

The product SHALL NOT maintain a separate semantic implementation.

For identical AIR and identical ordered admitted event corpus, OTP and AtomVM SHALL produce equivalent:

- state digest;
- result digest;
- refusal class;
- command sequence.

## 7.10 Layer 10 — BCINR Local Runner

BCINR SHALL remain the local runner for POWL geometry inside a process cell.

BCINR SHALL answer:

- activity eligibility;
- dependency satisfaction;
- socket attachment;
- child closure;
- next local transition.

BCINR SHALL NOT own:

- remote process survival;
- partition recovery;
- long human wait states;
- external restart semantics;
- inter-engine correlation.

---

# 8. Independent Process Cells

Agents SHALL NOT be modeled as roles sharing one workflow engine.

Each Chatman Engine SHALL be an independent process cell with:

- its own admitted graph view;
- its own bounded authority;
- its own local POWL/BCINR execution;
- broker-mediated actuation;
- explicit external dispatch and return admission.

An inter-engine workflow SHALL cross a declared external cut.

No engine SHALL unlock another engine's process state through hidden memory or shared agent context.

Returned results SHALL follow:

\[
O_{external}
\rightarrow
correlation
\rightarrow
provenance
\rightarrow
authority
\rightarrow
structure
\rightarrow
semantic\ conformance
\rightarrow
O^*\ \text{or refusal}
\]

Only admitted returned consequence MAY unlock a parent, sibling, or dependent workflow.

---

# 9. Parent-Child Closure

Every recursive socket SHALL declare its closure law.

Initial closure types SHALL include:

- `all_required`;
- `any_sufficient`;
- `quorum(q)`;
- `ordered_subset`;
- `policy_decides`;
- `first_conformant`.

For `all_required`:

\[
Close(W)\iff\forall c\in C(W),TerminalAdmitted(c)
\]

For `quorum(q)`:

\[
Close(W)\iff|\{c\in C(W):TerminalAdmitted(c)\}|\ge q
\]

Closure SHALL belong to POWL/GraphLaw.

A child completion signal SHALL be treated as observation until admitted.

Arazzo SHALL transport the outer execution but SHALL NOT define the authoritative closure law.

---

# 10. Compensation

Compensation SHALL be modeled as workflow.

A partial real-world consequence SHALL never be erased from process history.

When an external consequence is refused, the parent SHALL remain open, refused, or blocked according to declared law.

When a prior actuation requires remediation, the runtime SHALL manufacture a compensation workflow with:

- authority;
- admitted inputs;
- expected consequence;
- dispatch;
- receipt;
- replay.

Compensation examples include cancellation, revocation, corrective communication, provisional-state reversal, replacement-document request, exception review, and responsible-role notification.

There SHALL be no generic `rollback()` assumption for real-world work.

---

# 11. GraphLaw Authority Registry

v26.7.11 SHALL make the dialect registry executable.

Each dialect declaration SHALL include:

- admitted input classes;
- output classes;
- authority;
- quarantine state;
- refusal codes;
- receipt requirements;
- replay surface;
- executable route.

Initial authority map:

| Dialect | Authority |
|---|---|
| RDF | public graph representation |
| SPARQL SELECT | observe admitted graph state |
| SPARQL CONSTRUCT | manufacture graph consequence |
| Datalog | stable bounded closure derivation |
| N3 | quarantined bounded implication/refinement |
| SHACL | admission/refusal by shape law |
| ShEx | structural admission/refusal |
| PDDL | admitted action possibility |
| POWL v2 | process geometry and closure |
| Arazzo | manufactured inter-engine workflow carrier |
| OCEL | object-centric execution evidence |
| PROV-O | derivation and ancestry |
| ODRL | declared policy where profile-enabled |
| Lean/Lake | theorem admission |

No dialect SHALL acquire authority from another dialect merely because it can encode equivalent syntax.

---

# 12. N3 Quarantine

N3 SHALL remain unavailable by default.

N3 execution SHALL require:

- explicit profile capability;
- declared cost bounds;
- builtin whitelist;
- controlled execution surface;
- receipt support;
- replay support;
- zero direct actuation.

Default routing SHALL prefer SHACL/ShEx, Oxigraph SPARQL, SPARQL CONSTRUCT, Datalog, and bounded closure.

An N3 rule that requests direct actuation SHALL be refused.

---

# 13. Broker Requirements

The broker SHALL be the only actuation route.

Before actuation the broker SHALL verify:

- current artifact standing;
- actor role;
- capability authority;
- hook contract;
- input conformance;
- artifact lineage;
- idempotency key;
- correlation ID;
- required prior receipts.

After actuation the broker SHALL:

- capture the real consequence;
- bind consequence to actuation;
- emit runtime evidence;
- return raw consequence for re-admission;
- hash the consequence;
- extend the receipt chain;
- preserve replay identity.

No workflow runner SHALL mark external work complete solely from a successful HTTP status, process return code, or model response.

---

# 14. Evidence Pipeline

Business code SHALL emit telemetry under generated semantic conventions.

Business code SHALL NOT construct OCEL directly.

The canonical evidence path SHALL be:

\[
public\ semantics
\rightarrow
GGEN
\rightarrow
Weaver\ registry
\rightarrow
generated\ telemetry\ surface
\rightarrow
execution
\rightarrow
OTLP
\rightarrow
RDF\ admission
\rightarrow
SPARQL\ CONSTRUCT
\rightarrow
OCEL
\]

Graph layers SHALL remain separate:

- `G_SOURCE`;
- `G_OTEL`;
- `G_OCEL`;
- `G_RESULT`;
- `G_RECEIPT`.

Weaver SHALL govern what telemetry may say.

SPARQL CONSTRUCT SHALL determine what process evidence follows from admitted telemetry.

SPARQL SELECT SHALL measure.

ASK/SHACL SHALL admit or refuse.

PROV-O SHALL represent the transformation ancestry.

---

# 15. Receipt and Replay

Every manufactured Arazzo artifact SHALL have a receipt binding:

- source POWL digest;
- external-cut identity;
- SPARQL projection digest;
- Tera template digest;
- Arazzo digest;
- compiler version;
- AIR digest.

Every workflow execution SHALL extend a BLAKE3-linked receipt chain.

Minimum event receipt fields:

- workflow semantic ID;
- parent semantic ID;
- event type;
- event digest;
- prior receipt head;
- resulting state digest;
- command digest;
- runtime profile;
- timestamp or declared logical clock;
- replay ID.

Replay SHALL:

1. resolve the AIR artifact by digest;
2. restore the admitted initial state;
3. apply the admitted ordered event corpus;
4. recompute state and command digests;
5. verify receipt-head equivalence.

Replay mismatch SHALL be a typed refusal or build failure; it SHALL NOT be logged and ignored.

---

# 16. Multifractal Measurement Rail

The product SHALL derive workflow geometry measurements from admitted OCEL evidence.

Measurement SHALL be profile-driven and receipted.

For workflow family \(x\), define execution measure \(\mu_x\).

For declared scale \(\epsilon\), define normalized mass:

\[
p_i(\epsilon)=
\frac{\mu_x(B_i(\epsilon))}
{\sum_j\mu_x(B_j(\epsilon))}
\]

Partition function:

\[
Z(q,\epsilon)=\sum_i p_i(\epsilon)^q
\]

Estimate mass exponent \(\tau(q)\) from:

\[
Z(q,\epsilon)\sim\epsilon^{\tau(q)}
\]

Derive:

\[
\alpha=\frac{d\tau}{dq}
\]

and:

\[
f(\alpha)=q\alpha-\tau(q)
\]

Supported process scales SHALL initially include:

- enterprise goal;
- program;
- process;
- subprocess;
- workflow;
- activity;
- child workflow;
- broker actuation;
- recursive POWL depth;
- object-centric aggregation level;
- bounded execution cost band.

The selected scale, q-range, fitting method, minimum evidence threshold, confidence criteria, and source OCEL digest SHALL be part of the measurement profile.

The system SHALL NOT relabel a workflow family `MULTIFRACTAL_ADMITTED` merely because a plotted curve appears nonlinear.

Measurement status SHALL use explicit standing.

Initial statuses:

- `DECLARED`;
- `PARTIAL_ALIVE`;
- `ALIVE`;
- `BLOCKED`;
- `BUILD_BROKEN`;
- `UNKNOWN`;
- `UNSUPPORTED`.

Multifractal admission SHALL require declared statistical and replay criteria.

Failure or incompleteness of the measurement rail SHALL NOT silently invalidate an otherwise correct outer workflow runner.

---

# 17. Required Product Artifacts

v26.7.11 SHALL manufacture at minimum:

1. POWL v2 external-cut declaration schema.
2. External-cut validator.
3. SPARQL render-model projection.
4. Tera Arazzo renderer.
5. Arazzo projection receipt.
6. wasm4pm Arazzo parser.
7. AIR type system.
8. AIR typed refusal catalog.
9. Pure Erlang AIR transition core.
10. OTP supervised outer runner.
11. AtomVM wrapper over the same transition core.
12. Inter-engine dispatch envelope.
13. Correlation and return-admission pipeline.
14. Parent-child closure evaluator.
15. Compensation workflow manufacture path.
16. Broker integration.
17. Weaver semantic convention declarations.
18. OTLP-to-RDF admission path.
19. RDF-to-OCEL CONSTRUCT profile.
20. OCEL measurement queries.
21. Multifractal measurement profile schema.
22. Receipt-chain implementation.
23. Replay verifier.
24. OTP/AtomVM differential conformance corpus.
25. Negative fixtures for every prohibited authority escape.
26. Lean/Lake law pack for formalizable invariants.
27. Manifest/standing report for all v26.7.11 artifacts.

---

# 18. Typed Refusals

The implementation SHALL provide stable typed refusals.

Minimum refusal surface:

- `RAW_OBSERVATION_NOT_ADMITTED`
- `POWL_REGION_NOT_ADMITTED`
- `EXTERNAL_CUT_UNDECLARED`
- `EXTERNAL_CUT_TYPE_MISMATCH`
- `EXTERNAL_CUT_AUTHORITY_MISMATCH`
- `BOUNDED_DESCENT_NOT_PROVEN`
- `ARAZZO_UNMANUFACTURED`
- `ARAZZO_SOURCE_RECEIPT_MISSING`
- `ARAZZO_PROJECTION_DIGEST_MISMATCH`
- `AIR_PARSE_REFUSED`
- `AIR_REFERENCE_UNRESOLVED`
- `AIR_EXPRESSION_UNSUPPORTED`
- `AIR_CRITERION_UNSUPPORTED`
- `AMBIENT_AUTHORITY_REFUSED`
- `DIRECT_ACTUATION_REFUSED`
- `BROKER_RECEIPT_PRECONDITION_MISSING`
- `CORRELATION_MISSING`
- `CORRELATION_MISMATCH`
- `RETURN_PROVENANCE_MISSING`
- `RETURN_AUTHORITY_REFUSED`
- `RETURN_STRUCTURE_REFUSED`
- `RETURN_SEMANTIC_REFUSED`
- `CHILD_COMPLETION_UNADMITTED`
- `PARENT_CLOSURE_UNSATISFIED`
- `COMPENSATION_REQUIRED`
- `REPLAY_DIGEST_MISMATCH`
- `OTP_ATOMVM_SEMANTIC_DRIFT`
- `N3_CAPABILITY_MISSING`
- `N3_COST_BOUND_EXCEEDED`
- `N3_BUILTIN_REFUSED`
- `N3_DIRECT_ACTUATION_REFUSED`
- `MEASUREMENT_PROFILE_MISSING`
- `MEASUREMENT_EVIDENCE_INSUFFICIENT`
- `MULTIFRACTAL_CLASSIFICATION_UNADMITTED`

Refusal codes SHALL be machine-stable and receipt-visible.

---

# 19. Acceptance Scenarios

## 19.1 Local POWL Remains Local

Given an admitted POWL region with no external cut, the region SHALL execute through BCINR and SHALL NOT manufacture Arazzo.

## 19.2 External Cut Manufactures Arazzo

Given an admitted POWL region with a valid external cut, the system SHALL project a render model with SPARQL, render Arazzo with Tera, bind the source/projection/template/output digests, and admit the Arazzo artifact.

## 19.3 Handwritten Arazzo Is Refused

Given production Arazzo without a projection receipt and admitted POWL source, execution SHALL return `ARAZZO_UNMANUFACTURED`.

## 19.4 Remote Child Closure

Given a parent with `all_required` closure and two external children, one admitted terminal child and one unadmitted completion observation SHALL leave the parent open.

## 19.5 Quorum Closure

Given `quorum(2)` over three children, two admitted terminal children SHALL close the parent even if the third remains open, provided policy declares no contrary blocking condition.

## 19.6 Crash and Replay

Given an OTP workflow process crash after receipt head \(R_n\), the supervisor SHALL restore execution machinery, replay admitted events through \(R_n\), reproduce the state digest, and continue without changing workflow semantic identity.

## 19.7 Duplicate Result

Given the same correlated result twice, the second return SHALL not duplicate consequence or advance the workflow twice.

## 19.8 Refused Remote Consequence

Given a correlated remote result that fails SHACL or authority admission, the result SHALL not unlock dependent steps. The workflow SHALL enter its declared refused/blocked/compensation path.

## 19.9 Compensation Manufacture

Given admitted evidence of a partial real-world consequence plus a failure requiring remediation, the runtime SHALL manufacture and dispatch a compensation workflow through the same broker/receipt law.

## 19.10 OTP/AtomVM Equivalence

Given the same AIR artifact and ordered admitted event corpus, OTP and AtomVM SHALL emit identical semantic state digest, result digest, refusal class, and command sequence.

## 19.11 Evidence Reconstruction

Given admitted OTLP-derived RDF and a declared CONSTRUCT profile, replay SHALL regenerate graph-equivalent OCEL with the expected transformation receipt.

## 19.12 Multifractal Non-Promotion

Given insufficient scale coverage or evidence mass, the measurement rail SHALL return `MEASUREMENT_EVIDENCE_INSUFFICIENT` or `PARTIAL_ALIVE`; it SHALL NOT emit `MULTIFRACTAL_ADMITTED`.

---

# 20. Verification Ladder

## Unit

- AIR parser normalization.
- dependency readiness.
- criteria evaluation.
- retry/timeout transitions.
- bounded-descent comparator.
- closure laws.
- receipt hashing.
- idempotency behavior.
- measurement profile validation.

## Integration

- POWL external cut → SPARQL model.
- SPARQL model → Tera → Arazzo.
- Arazzo → wasm4pm → AIR.
- AIR → Erlang transition core.
- Erlang command → broker.
- broker consequence → return admission.
- OTLP → RDF → OCEL.
- OCEL → measurement query.

## End-to-End

Execute a parent POWL workflow that dispatches to at least two independent Chatman Engines, admits child consequences, closes according to explicit law, actuates through the broker, emits evidence, creates receipts, and replays to the same terminal standing.

## Chaos

Inject:

- OTP process death;
- remote engine restart;
- duplicate delivery;
- event reordering where ordering is not guaranteed;
- delayed acknowledgment;
- timeout;
- partition;
- stale result;
- malformed result;
- receipt corruption.

No chaos case may create unreceipted actuation or false parent closure.

## Stress

Measure:

- concurrent workflow instances;
- dispatch fan-out;
- child depth;
- receipt-chain length;
- event corpus replay size;
- OCEL object/event volume.

Stress limits SHALL be declared by profile rather than inferred from successful local execution.

## Benchmark

Benchmark separately:

- BCINR local transition latency;
- wasm4pm Arazzo-to-AIR compile cost;
- Erlang AIR transition cost;
- OTP supervision/recovery cost;
- AtomVM transition cost;
- broker dispatch overhead;
- replay throughput;
- RDF-to-OCEL construction cost;
- multifractal measurement cost.

No aggregate benchmark SHALL hide scale changes between local BCINR execution and outer OTP workflow execution.

## Verifier Report

The release verifier SHALL report:

- declared artifacts;
- manufactured artifacts;
- admitted artifacts;
- refused fixtures;
- orphan counts;
- projection digest consistency;
- AIR conformance corpus result;
- OTP/AtomVM differential result;
- broker bypass search result;
- replay equivalence result;
- OCEL transformation equivalence result;
- measurement rail status;
- Lean/Lake build status.

---

# 21. Lean/Lake Formalization Targets

The v26.7.11 law pack SHOULD formalize, where the model is finite and explicit:

1. substitution closure under compatible socket predicates;
2. bounded descent under lexicographic cost vectors;
3. absence of direct actuation in the declared transition relation;
4. parent closure for `all_required`;
5. parent closure for quorum;
6. idempotent duplicate-result transition;
7. semantic identity independence from runtime PID;
8. shared-transition-core equivalence premise for OTP/AtomVM wrappers;
9. receipt-chain head determinism over an ordered event corpus.

Lean SHALL admit theorem-bearing claims.

Lake SHALL establish composition/build standing.

Generated theorem candidates SHALL not be labeled proven until Lake admits the relevant source.

The paper claim surface SHALL remain downstream of theorem and verifier standing.

---

# 22. Delivery Order

The required implementation order is:

### Rail A — Projection
POWL external cut → SPARQL render model → Tera → receipted Arazzo.

### Rail B — Compilation
Arazzo → wasm4pm parser → AIR → typed refusals.

### Rail C — Pure Semantics
AIR state/event model → pure Erlang transition core → deterministic corpus.

### Rail D — Runtime
OTP supervision → broker dispatch → correlation → return admission → parent closure.

### Rail E — Equivalence
AtomVM wrapper → differential corpus → semantic drift refusal.

### Rail F — Evidence
Weaver conventions → OTLP → RDF admission → OCEL CONSTRUCT → provenance receipt.

### Rail G — Measurement
OCEL execution measure → declared scale profiles → \(Z(q,\epsilon)\) → \(\tau(q)\) → \(f(\alpha)\) → standing report.

### Rail H — Formal Standing
Lean/Lake laws → negative fixtures → manifest → verifier report.

No later rail SHALL be used to backfill authority missing from an earlier rail.

---

# 23. Definition of Done

v26.7.11 is `ALIVE` only when:

- an admitted POWL v2 workflow can declare an external cut;
- the cut deterministically manufactures receipted Arazzo;
- wasm4pm compiles the artifact to AIR;
- the pure Erlang transition core executes the AIR semantics;
- OTP supervises a long-lived inter-engine workflow;
- AtomVM passes the shared semantic conformance corpus;
- every real actuation routes through the broker;
- external results re-enter through correlation and admission;
- parent-child closure is explicit and tested;
- compensation is represented as workflow;
- runtime evidence is admitted as RDF;
- OCEL is constructed by declared SPARQL CONSTRUCT;
- receipt/replay reproduces terminal standing;
- authority-escape negative fixtures refuse;
- the verifier finds zero broker bypasses in declared production routes;
- Lean/Lake admits the selected formal law pack;
- the release manifest reports each rail without silent promotion.

The multifractal measurement rail MAY be `PARTIAL_ALIVE` at initial runtime release if the implementation, measurement profiles, receipt path, and insufficient-evidence refusals are complete but production evidence is not yet sufficient to admit a multifractal spectrum.

The runtime crown SHALL NOT be called `ALIVE` if any declared actuation path is unreceipted.

---

# 24. Crown Requirement

Multifractal Workflow v26.7.11 SHALL make recursive work a standing-preserving manufacturing surface across independent execution cells.

The product is complete when work may descend from enterprise goal to atomic actuation and return from real consequence to parent closure without requiring a human to manually reconstruct graph-derivable state, manually stitch child workflows, infer hidden ownership, poll remote work, or narrate provenance.

The execution law SHALL remain invariant.

The local geometry MAY vary.

The descent SHALL remain bounded.

The actuation SHALL remain brokered.

The consequence SHALL be re-admitted.

The receipt SHALL prove the chain.

The replay SHALL reproduce it.
