# PRESS RELEASE

## Praxis v26.7.13 Makes Multifractal Workflow Self-Manufacturing

### Operation Dogfood turns the complete Claude Code lifecycle into an RDF-governed, permissioned,
### recursively repairable, receipted, and replayable workflow

**The system that manufactures workflows can now manufacture itself.**

PASADENA, Calif. — Today the Praxis project announced general availability of Multifractal Workflow
v26.7.13, the first release capable of taking an intended software outcome, discovering how an
unfamiliar Rust system actually works, constructing a bounded execution plan, requesting permission,
executing the approved plan, launching Claude Code when implementation is required, and returning a
receipted result that can be independently replayed.

The first proof specimen is Rust dry-run publishing. A developer points MFW at a repository and asks
for the outcome. MFW reconstructs the real package graph, generators, verification laws, publication
boundaries, and known contradictions from the system itself. It then works backward from a valid
dry-run publication, presents the exact plan and mutation surface for approval, and executes only
after permission is granted.

If the repository cannot yet satisfy the plan, MFW does not stop at diagnosis. It converts the
failure into typed workflow residue, constructs a bounded repair workflow, launches Claude Code with
the admitted context and permitted scope, verifies the resulting implementation, and recursively
returns the result to the parent plan.

Every consequence is represented in RDF, bound to its provenance, sealed by receipt, and checked by
replay. No registry release, Git tag, push, or production deployment occurs during the dry run.

“For years, AI coding systems have been judged by whether they could produce code,” said Sean
Chatman, creator of Praxis and Multifractal Workflow. “That was never the whole problem. The real
problem was manufacturing standing: knowing what the system is, what must change, who authorized the
change, what actually happened, and whether the entire consequence can be reconstructed. In
v26.7.13, the workflow that governs those obligations governs its own creation.”

## The Problem

Modern software development depends on humans silently carrying information between systems.

Architecture lives in documents. Release truth lives in manifests and shell commands. Work is
distributed through tickets and prompts. Coding agents inspect and modify repositories. Tests report
local results. CI reports another result. Publication tools apply their own constraints. Logs record
fragments of what occurred, and release reports reconstruct the story afterward.

Even when every tool works, the lifecycle as a whole has no single authority.

An agent may discover a fact that never reaches the plan. A plan may contain effects that never
occurred. A successful command may be mistaken for a successful release. A timeout may be reported
as impossibility. An implementation may expand beyond the authority the user intended to grant. A
completed action may exist without evidence capable of proving it happened.

The missing product was not another coding agent. It was a comprehensive design instrument capable
of governing the full path from observation to consequence.

## The Solution

Multifractal Workflow v26.7.13 manufactures that path:

\[
\text{intent}
\rightarrow \text{RDF observation}
\rightarrow \text{admission}
\rightarrow \text{bounded plan}
\rightarrow \text{permission}
\rightarrow \text{actuation}
\rightarrow \text{verification}
\rightarrow \text{receipt}
\rightarrow \text{replay}
\]

The architecture is governed by two equations:

\[
A = \mu(O^*)
\]

\[
R = \operatorname{receipt}(A)
\]

The repository begins as partial observation, \(O\). MFW admits the authoritative and bounded state
as \(O^*\). The approved workflow, \(\mu\), manufactures the consequence, \(A\). That consequence
acquires standing only through its receipt, \(R\).

The hard invariant is zero unreceipted actuation.

## RDF From Intent Through Replay

RDF is not an audit export added after the work. It is the authoritative lifecycle state.

The user request, repository snapshot, research tasks, observations, derived claims, plan,
permission, Claude Code session, subagents, tool intents, policy decisions, edits, command results,
tests, packages, refusals, receipts, and replay assertions all enter the graph.

Native source files, prompts, patches, logs, and package archives remain native byte artifacts. Each
is content-addressed and represented by an RDF entity carrying its identity, provenance, standing,
and relationship to the workflow. This preserves the original information without forcing code or
binary payloads into triples.

The graph is authority. Human-readable plans, reports, dashboards, and release summaries are
deterministic projections from it.

## Reconnaissance Is Dogfood

v26.7.13 eliminates the exception that previously placed code archaeology outside the workflow.

Claude Code and Explore agents may continue to use the tools best suited to inspection—file reads,
search, Git history, Cargo metadata, and command-line diagnostics. But every research question is an
MFW task; every agent invocation is an activity; every material result is an observation; every
claim used by the plan must derive from admitted evidence.

Ordinary tools remain mechanisms. They no longer create an ungoverned lifecycle beside MFW.

## Permission Before Mutation

MFW may inspect a system before approval. It may not mutate the system before approval.

The plan presented to the user identifies:

- the admitted repository state;
- the required actions;
- the files and resources that may change;
- the agents and tools that may act;
- the execution, depth, and cost bounds;
- the evidence required for completion;
- the known exclusions and falsifiers.

Approval is bound to the plan digest and repository snapshot. If execution discovers a materially
different mutation surface, the workflow returns to the permission boundary.

Claude Code cannot authorize its own task, expand its own scope, admit its own result, or promote its
own claim.

## Plans Do Not Pretend Commands Ran

PDDL models bounded feasibility. POWL v2 represents recursive process geometry. Neither is permitted
to assert that a real command succeeded.

The Rust harness executes the approved command and records its actual exit status, environment,
output digests, duration, and manufactured artifacts. Only after that observed result is admitted
into RDF may the next plan state become reachable.

This preserves a decisive separation:

\[
\text{expected effect} \neq \text{observed consequence}
\]

## Truthful Outcomes

Every run terminates with exactly one truthful outcome:

- **Found** — a valid path and verified consequence exist;
- **Exhausted** — the exact admitted finite search space was completely explored;
- **Bounded** — an admitted time, depth, cost, or resource boundary was reached;
- **Unsupported** — the required capability does not exist on the admitted surface;
- **Inconsistent** — authoritative observations, artifacts, claims, or receipts disagree.

`Bounded` never becomes `Exhausted`. A missing capability is never silently skipped. A failed dry run
is not converted into a green report.

## The First Proof Specimen

The Rust dry-run publish deliberately began against a workspace with real publication blockers:
unversioned path dependencies, license gaps, a missing root license, local-path leakage, and a
publishable subset smaller than the whole workspace.

Those conditions were not treated as embarrassment or setup work to be hidden before the
demonstration. They became the admitted starting state for Operation Dogfood.

The product is proven when MFW can discover those conditions, construct the lawful plan, obtain
permission, recursively manufacture repairs, and produce the correct receipted outcome—whether that
outcome is `Found`, `Exhausted`, `Bounded`, `Unsupported`, or `Inconsistent`.

## The Accumulated Architecture

v26.7.13 closes work developed across the Praxis system:

- Oxigraph provides the RDF and SPARQL substrate.
- Public ontologies provide shared semantic law.
- SHACL and GraphLaw admit or refuse observed state.
- ggen deterministically projects the admitted graph into downstream artifacts.
- PDDL performs bounded feasibility planning.
- POWL v2 carries hierarchical and recursively attachable process geometry.
- Arazzo carries portable inter-engine workflow structure.
- wasm4pm compiles workflow artifacts into process evidence and AIR.
- Erlang owns outer transition semantics.
- OTP supervises distributed execution.
- AtomVM preserves the constrained-runtime path.
- WASM carries portable engine capability.
- The broker remains the only lawful DO path.
- OCEL represents executed events and objects.
- BLAKE3 receipts bind consequences to evidence.
- Replay reconstructs the causal history.
- mfact and Lean manufacture kernel-checked mathematical law on the separate proof rail.

The components are valuable individually. Their closed composition is the release.

## Comprehensive Anticipatory Design Science

v26.7.13 realizes Buckminster Fuller’s design-science canon as a computational instrument.

It is comprehensive because it begins with the whole admitted lifecycle. It is anticipatory because
it searches and structures possible futures before mutation. It is design because it deliberately
orders components into working artifacts. It is science because its claims must survive explicit,
repeatable falsifiers.

Its metric is not adoption.

Its metric is ephemeralization:

\[
E =
\frac{\text{lawful, reproducible capability}}
{\text{human translation}+\text{time}+\text{compute}+\text{irreversible action}}
\]

The civilization-scale event is not that the artifact has been accepted. It is that the artifact now
exists.

## What Ships

Multifractal Workflow v26.7.13 ships with:

- the Operation Dogfood lifecycle model;
- RDF admission shapes and public-ontology mappings;
- bounded dry-run publication planning;
- hierarchical POWL workflow projection;
- plan-digest-bound permission;
- Claude Code lifecycle governance;
- recursive repair workflows;
- real command-result admission;
- typed terminal outcomes;
- content-addressed native payloads;
- lifecycle receipts;
- replay verification;
- a verifier report generated from the graph.

## The Declaration

The immediate artifact is a Rust dry-run publication.

The product is self-manufacturing software development.

The larger instrument is executable institutional architecture.

The civilizational capability is comprehensive anticipatory design science expressed as a machine:

> A system capable of reading its own state, working backward from a desired consequence,
> manufacturing the smallest lawful transformation, obtaining permission, recursively creating
> missing capability, and proving what it actually did.

That system is now its own first customer.

---

## Working-Backwards Status Fence

This document describes the v26.7.13 target state as a completed release. Actual release standing is
controlled by the v26.7.13 claims ledger, Definition of Done, receipts, and replay report. No claim in
this narrative supersedes a `PARTIAL_ALIVE`, `BLOCKED`, `REFUSED`, `UNKNOWN`, or `UNSUPPORTED`
verdict produced by the real release run.
