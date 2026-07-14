# Operation Dogfood — Product Requirements Document

**Release:** v26.7.13  
**Product:** Multifractal Workflow  
**Status:** DRAFT — target-state requirements; standing is assigned per claim below  
**Crown use case:** Rust dry-run publish through the complete Claude Code lifecycle

## Claims Reconciliation

This table is authoritative for this document. `ALIVE` means the exact scoped capability has direct
evidence. `PARTIAL_ALIVE` means a real slice exists but the complete lifecycle claim does not.
`PLANNED` means required by this release but not yet admitted. `REFUSED` means a real run reached a
typed blocking condition. `UNKNOWN` means the evidence available to this document is insufficient.

| # | Claim | Standing | Scope and caveat | Required promotion evidence |
|---|---|---|---|---|
| C1 | MFW models bounded PDDL and projects POWL workflow structure | ALIVE | Existing TOGAF and SOC2 patterns demonstrate plan structure and hierarchical projection; this does not prove command actuation | Existing tests remain green and are reused by Operation Dogfood |
| C2 | MFW uses RDF as the lifecycle authority from user intent through replay | PLANNED | Existing RDF planning and evidence slices do not yet envelope the complete Claude Code lifecycle | One end-to-end run whose intent, observations, admission, plan, approval, tool events, artifacts, receipts, and replay are reconstructible from RDF |
| C3 | Reconnaissance and Explore work are dogfooded through MFW | PLANNED | Read-only research may still use ordinary tools, but every task, observation, result, and derived claim must be planned or recursively grafted and recorded in RDF | Explore activity cannot complete without RDF task binding, result admission, provenance, and receipt |
| C4 | MFW discovers an unfamiliar Rust system rather than requiring a hand-authored release workflow | PLANNED | Discovery may use Claude Code and repository tools; resulting facts do not become authority until admitted | Successful dry-run planning for a repository without a prewritten Operation Dogfood workflow |
| C5 | MFW creates a bounded execution plan and asks the user for permission before mutation | PLANNED | Read-only observation is allowed before approval; writes, commands with side effects, agent edits, commits, tags, pushes, and publishing are actuation | ODRL-backed permission bound to the exact plan digest, mutation set, bounds, and expiration |
| C6 | MFW launches and governs Claude Code when implementation or repair is required | PLANNED | Claude Code is an actuator and proposer, not standing authority | A failed gate produces RDF residue, a bounded repair child workflow, a Claude Code invocation, verified patch, and replayable receipt |
| C7 | Every Claude Code tool event is represented in RDF end to end | PLANNED | Native code, diffs, logs, and archives remain native payloads referenced by content-addressed RDF entities | Pre/post tool lifecycle events cover reads, searches, shell commands, edits, tests, agents, and stop outcomes with no orphan actuation |
| C8 | The actual Rust dry-run publish currently succeeds for the whole workspace | REFUSED | Current recorded reconnaissance identifies unversioned path dependencies, license gaps, a missing root license, path leaks, and only a near-term subset of crates as publishable | Real dry-run execution closes or explicitly excludes every blocker and produces an ALIVE receipt |
| C9 | Receipt and replay cover the complete dogfood lifecycle | PARTIAL_ALIVE | Receipt/replay machinery exists in project slices; complete Claude Code lifecycle coverage is not yet demonstrated | Byte-stable replay of the admitted plan and semantic replay of the RDF event graph |
| C10 | Public ontologies are used before private vocabulary | PARTIAL_ALIVE | Existing work uses public ontologies but retains necessary engine ABI vocabularies | Namespace report showing public terms for lifecycle semantics and a bounded private ABI list |
| C11 | MFW can truthfully distinguish success, exhaustion, bounds, unsupported capability, and inconsistency | PARTIAL_ALIVE | The outcome doctrine is accepted; planner implementations may still conflate depth exhaustion with infeasibility | Typed outcome algebra preserved through planner, harness, Claude Code adapter, CLI, receipt, and replay |
| C12 | Operation Dogfood permits autonomous external publication | REFUSED | v26.7.13 is a dry-run release; external registry mutation is outside the admitted goal | A later, separately approved release may define a publish permission surface |

## 1. Product Summary

Operation Dogfood makes Multifractal Workflow the governing workflow for the entire Claude Code
lifecycle.

A developer supplies an intended outcome, beginning with **Rust dry-run publish**. MFW admits the
repository as partial observation, discovers the actual system, creates a bounded plan, presents the
plan for permission, executes the approved work, launches Claude Code when implementation is needed,
verifies every repair, and returns a receipted, replayable outcome.

RDF is the authoritative state carrier throughout. It represents the request, repository snapshot,
research tasks, observations, claims, plan, permission, agent invocations, tool events, artifacts,
test results, refusals, receipts, and replay. Native files remain native; RDF identifies them,
content-addresses them, relates them, and governs their standing.

## 2. Narrative Frame

MFW cannot claim to manufacture other organizations' workflows while its own development lifecycle
is coordinated outside MFW.

Operation Dogfood closes that gap. The system becomes customer zero. Every capability used to build,
repair, verify, and release MFW must enter through the same lifecycle MFW offers to users:

\[
\text{intent}
\rightarrow \text{observation}
\rightarrow \text{admission}
\rightarrow \text{plan}
\rightarrow \text{permission}
\rightarrow \text{actuation}
\rightarrow \text{receipt}
\rightarrow \text{replay}
\]

Dogfooding is not achieved merely because the final deliverable contains PDDL or POWL. It is achieved
when the complete process that produced the deliverable is itself an MFW run.

## 3. Customer Problem

Rust developers attempting a release must reconstruct knowledge scattered across manifests,
workspaces, build scripts, generators, tests, CI files, documentation, package metadata, dependency
order, and prior failures. Existing automation usually assumes that this knowledge has already been
translated into a script.

Claude Code can investigate and implement repairs, but without a governing workflow:

- research tasks are not bound to a shared plan;
- observations can silently become assumptions;
- implementation can expand beyond permission;
- a successful command can be mistaken for a successful release;
- agent work can be lost outside the release evidence graph;
- a timeout can be mistaken for infeasibility;
- the same release must be rediscovered during the next attempt;
- there is no complete receipt proving why the final outcome has standing.

The customer needs an outcome-oriented system that discovers the workflow, not another tool that
requires the customer to encode the workflow first.

## 4. Product Position

Operation Dogfood is not a release script, a Claude prompt template, a CI wrapper, or an RDF logging
export.

It is a recursive manufacturing control plane:

- **MFW owns lifecycle state and workflow geometry.**
- **RDF owns admitted instance state and provenance.**
- **PDDL owns bounded feasibility and next-action planning.**
- **POWL owns structured process geometry and recursive composition.**
- **The permission broker owns the mutation boundary.**
- **Claude Code owns bounded implementation tasks assigned by MFW.**
- **The Rust harness owns actual command execution.**
- **Receipts and replay own consequence standing.**

## 5. Core Equations

\[
A = \mu(O^*)
\]

\[
R = \operatorname{receipt}(A)
\]

For Operation Dogfood:

- \(O\) is the initially observed repository and user intent;
- \(O^*\) is the RDF-admitted repository snapshot, goal, capability surface, constraints, and
  exclusions;
- \(\mu\) is the approved MFW plan executed through the broker, harness, and Claude Code child
  workflows;
- \(A\) is the dry-run publication outcome and its manufactured artifacts;
- \(R\) is the canonical receipt proving the complete lifecycle and replay result.

The hard invariant is:

\[
\{a \mid \operatorname{actuated}(a) \land \neg\operatorname{receipted}(a)\} = \varnothing
\]

## 6. Doctrine

### 6.1 RDF is the lifecycle authority

RDF must exist before and after every lifecycle transition. JSON, terminal output, code, patches,
archives, and prompts may remain native payloads, but none may become authoritative lifecycle state
until represented as a content-addressed RDF entity with provenance and standing.

### 6.2 Research is part of the workflow

Explore agents may use grep, file reads, Git history, Cargo metadata, and other read-only tools. Those
tools are mechanisms, not exemptions. The research task, its bounds, the agent invocation, every
material observation, and the derived conclusion must be bound into the MFW graph.

### 6.3 Permission precedes mutation

MFW may inspect before permission. It may not edit, generate, install, build with external side
effects, commit, tag, push, or publish until the user approves the plan containing those effects.

### 6.4 Claude Code is governed execution

MFW may launch Claude Code whenever a plan gate requires diagnosis, repair, implementation, or
verification. Claude Code cannot authorize its own scope, admit its own claims, or promote its own
result.

### 6.5 Real effects come from the harness

PDDL and POWL model lawful plan structure. They do not pretend a command succeeded. The Rust harness
executes a real command; its actual result becomes a new RDF observation; admission turns that result
into a fact that subsequent planning may consume.

### 6.6 Outcomes remain truthful

Every terminal result is exactly one of:

- `Found` — a valid, verified consequence exists;
- `Exhausted` — the exact admitted finite search space contains no valid path;
- `Bounded` — a declared search or execution bound was reached;
- `Unsupported` — a required capability is absent;
- `Inconsistent` — authoritative state or evidence disagrees.

No implementation may collapse these outcomes for convenience.

## 7. Primary Release Goal

Deliver one complete Operation Dogfood run in which MFW governs the full Claude Code lifecycle for a
Rust dry-run publish:

1. accept the user goal;
2. capture it in RDF;
3. inspect and model the repository through receipted reconnaissance;
4. admit the repository snapshot and release constraints;
5. generate a bounded PDDL plan and RDF POWL process;
6. present the exact plan and mutation surface for approval;
7. execute the approved gates using the Rust harness;
8. launch Claude Code for at least one genuine discovered repair or implementation task;
9. re-admit and verify the resulting patch;
10. attempt the real dry-run package workflow;
11. produce a typed outcome without external publication;
12. seal and replay the complete RDF lifecycle receipt.

The release succeeds even if the first whole-workspace dry run truthfully terminates `REFUSED`,
`Bounded`, `Unsupported`, or `Inconsistent`, provided the blocking outcome is real, typed, receipted,
and replayable. The product capability is the lawful lifecycle, not a fabricated green result.

## 8. MVP Definition

The v26.7.13 MVP contains:

1. **Operation Dogfood RDF pack** for lifecycle concepts, public-ontology mappings, SHACL admission,
   status schemes, and templates.
2. **Rust dry-run publish domain pack** containing the release gate model and typed refusal surface.
3. **Repository admission** for Cargo metadata, package manifests, generator configuration,
   verification recipes, release documentation, and current Git identity.
4. **Reconnaissance workflow** that binds Claude Code and Explore-agent research into RDF.
5. **Plan manufacture** from admitted repository facts into bounded PDDL and hierarchical POWL RDF.
6. **Permission artifact** using an RDF policy bound to the plan digest and mutation surface.
7. **Claude Code lifecycle adapter** covering launch, subagents, tool intents, tool results, patches,
   tests, and stop outcomes.
8. **Rust command harness** that executes approved commands and emits RDF observations.
9. **Recursive repair** that turns a failed gate into residue, plans a child workflow, invokes Claude
   Code, validates the result, and returns to the parent socket.
10. **Receipt and replay** over canonicalized RDF plus content-addressed native payloads.
11. **Human-readable report** projected from the graph, never maintained as a second authority.

## 9. Personas

### Rust developer

Wants a dry-run publish without first reverse-engineering and scripting the entire release process.

### Maintainer

Needs exact control over what MFW and Claude Code may change, along with a reviewable plan and patch.

### Release engineer

Needs dependency ordering, package validation, clean-room evidence, and truthful blocking outcomes.

### Auditor or reviewer

Needs to reconstruct who proposed, approved, executed, verified, and receipted every consequence.

### MFW contributor

Needs the MFW development process itself to prove that the workflow engine is usable on real work.

## 10. Functional Requirements

### FR-1 — Goal admission

The system shall create an RDF goal entity for the user request before reconnaissance begins.

### FR-2 — Repository snapshot

The system shall create a content-addressed RDF snapshot covering Git identity, manifests, lockfiles,
relevant configuration, release documentation, and discovered package topology.

### FR-3 — Research planning

Every Explore or reconnaissance task shall correspond to an MFW plan step or recursively grafted
observation child workflow.

### FR-4 — Observation capture

Each material read-only result shall be represented as a `prov:Entity` generated by a
`prov:Activity`, attributed to its agent, related to its source, and assigned an admission status.

### FR-5 — Claim derivation

Every claim used by the plan shall identify its supporting observations and derivation path. Search
snippets, agent summaries, and terminal prose shall not become admitted facts by themselves.

### FR-6 — Bounded plan

The system shall generate a plan whose steps, dependencies, bounds, goals, and expected evidence are
represented in RDF and projected to PDDL/POWL as required.

### FR-7 — Approval request

Before mutation, the system shall present the plan, expected changes, commands, agent authority,
bounds, exclusions, and falsifiers to the user.

### FR-8 — Approval binding

Approval shall be represented as an RDF permission bound to the user identity, plan digest, admitted
repository snapshot, action classes, resource scope, bounds, and expiration or run identifier.

### FR-9 — Pre-actuation guard

Every mutating Claude Code tool call or harness command shall prove that it is bound to an approved
plan step before execution.

### FR-10 — Claude Code launch

MFW shall be able to launch Claude Code with an RDF-derived task packet containing the admitted
problem, permitted scope, relevant payload references, required result, and verification ladder.

### FR-11 — Tool lifecycle capture

The adapter shall emit RDF for Claude Code session start, task assignment, subagent creation, tool
intent, policy decision, tool result, artifact generation, verification, and stop outcome.

### FR-12 — Native payload integrity

Files, prompts, patches, command output, package archives, and logs shall be stored or referenced as
native payloads with stable content digests. RDF shall identify and relate them without lossy
re-encoding.

### FR-13 — Real command outcomes

The harness shall emit actual exit code, stdout/stderr digests, duration, environment fingerprint,
and produced artifact digests. Planned effects shall never substitute for observed command results.

### FR-14 — Typed gate result

Each dry-run gate shall terminate with a typed result and generate the RDF facts required for the
next plan step or repair workflow.

### FR-15 — Recursive repair

A failed gate may generate a bounded residue entity and child workflow. The child may diagnose,
edit, test, and verify only within the parent permission or must request expanded permission.

### FR-16 — Patch admission

Claude Code changes shall remain proposed until formatting, targeted tests, integration gates,
scope inspection, and required adversarial checks pass.

### FR-17 — Replanning

After each admitted result, MFW shall plan from the new RDF state rather than from an unrecorded
in-memory assumption.

### FR-18 — Dry-run non-actuation

The release workflow shall not create a registry release, Git tag, push, deployment, or other public
mutation.

### FR-19 — Receipt

The final receipt shall bind the goal, snapshot, plan, approval, actions, agents, native payload
digests, test evidence, terminal outcome, and exclusions.

### FR-20 — Replay

Replay shall verify the canonical RDF event graph, native payload digests, plan binding, permission
binding, and resulting dry-run outcome.

### FR-21 — Human projection

The system shall generate a concise plan, progress view, final report, and refusal explanation from
the RDF graph.

### FR-22 — No lifecycle orphan

The verifier shall refuse a run containing an unbound agent task, orphan tool actuation, result with
no producing activity, artifact with no digest, approval with no plan digest, or completed step with
no receipt.

## 11. Non-Functional Requirements

### NFR-1 — Determinism

Given the same admitted RDF state, payload digests, toolchain versions, and approved plan, planning
and projection shall be byte-deterministic. Nondeterministic Claude output shall be frozen as an
observed payload before admission.

### NFR-2 — Fail closed for actuation

If the graph store, permission check, receipt writer, or lifecycle adapter is unavailable, mutating
actuation shall be refused.

### NFR-3 — Public ontology first

Lifecycle semantics shall use PROV-O, P-Plan, ODRL, SHACL, DCTERMS, DCAT, SKOS, SOSA, EARL, SPDX,
DOAP, QUDT, and other applicable public terms before adding private vocabulary.

### NFR-4 — Bounded private ABI

Private namespaces are permitted only for engine-specific concepts without a faithful public term,
including PDDL/POWL carrier bindings and typed internal refusals. Every private term shall be listed
in a generated namespace report.

### NFR-5 — Canonical identity

Immutable payloads shall use content-addressed identities. RDF receipts shall use canonicalized
datasets. Blank-node identity shall never cross a receipt boundary without canonicalization.

### NFR-6 — Information preservation

Native payloads shall remain retrievable by digest. RDF summaries shall not replace source bytes.

### NFR-7 — Resumability

An interrupted run shall resume from the last receipted state without repeating completed mutation.

### NFR-8 — Idempotence

Replaying an already completed and receipted step shall either prove the same consequence or refuse
on changed state; it shall not duplicate side effects.

### NFR-9 — Explainability

Every plan step and refusal shall expose its supporting RDF facts, rule, permission, and expected
evidence.

### NFR-10 — Performance

Lifecycle capture shall not require embedding full source files or logs in the graph. Large payloads
remain external and content-addressed.

### NFR-11 — Security

Secrets shall never be copied into RDF or receipts. Sensitive payloads shall be redacted,
access-controlled, or represented only by digest and classification.

### NFR-12 — Compatibility

The architecture shall support Claude Code as the initial implementation agent without making the
RDF lifecycle model Claude-specific.

## 12. Out of Scope

- Actual publication to crates.io or another registry.
- Automatic Git tag creation, push, merge, deployment, or production actuation.
- Proving that arbitrary Rust programs are correct.
- Serializing all source code, logs, archives, or prompts directly into RDF.
- Treating RDF capture alone as proof that an action was lawful.
- Replacing Cargo, Git, Claude Code, Lean, ggen, or the existing command harness.
- Claiming planner completeness outside the exact bounded model searched.
- Claiming all MFW crown edges are contiguous.
- Completing every open v26.7.13 feature before the dogfood lifecycle can be demonstrated.
- SOC2 compliance, audit opinion, or any other external attestation.
- General autonomous publication without a new permission surface.

## 13. Day-One Finish Plan

### Phase 1 — Declare the lifecycle graph

Create the Operation Dogfood pack, public ontology mappings, SKOS status/outcome schemes, SHACL
shapes, named-graph layout, and deterministic templates.

### Phase 2 — Admit Claude Code reconnaissance

Wrap session start, Explore tasks, read/search tools, material observations, claims, and provenance in
RDF. Demonstrate that the release plan can cite exact admitted evidence rather than prose summaries.

### Phase 3 — Manufacture and approve the plan

Generate the Rust dry-run publish plan from the graph, project hierarchical POWL RDF, display the
plan, and record user approval as a plan-digest-bound ODRL policy.

### Phase 4 — Execute through the broker

Run real gates through the Rust harness. Capture every command result as RDF before replanning.

### Phase 5 — Repair through Claude Code

Select one real blocker, generate a child repair workflow, launch Claude Code, validate the patch,
and return the receipted result to the parent workflow.

### Phase 6 — Seal and replay

Attempt the dry-run publish, preserve the truthful terminal outcome, seal the complete receipt, and
replay from the admitted graph and payloads.

## 14. Acceptance Criteria

Operation Dogfood v26.7.13 is `ALIVE` only when all of the following hold:

- [ ] A user request creates the root RDF goal and run identity.
- [ ] The repository snapshot and every plan-driving observation are content-addressed RDF entities.
- [ ] Every Explore agent and research task is bound to an MFW activity and plan step.
- [ ] No claim used by the plan lacks provenance to admitted observations.
- [ ] The complete plan exists as RDF and has bounded PDDL plus hierarchical POWL projections.
- [ ] The user approves the exact plan digest before the first mutation.
- [ ] A pre-actuation guard refuses a mutation not covered by the approval policy.
- [ ] MFW launches Claude Code for a real repair or implementation task.
- [ ] Claude Code session, subagent, tool, edit, test, and stop events appear in the RDF lifecycle.
- [ ] Native payloads remain byte-preserved and are referenced by digest.
- [ ] Planned command effects never substitute for actual harness results.
- [ ] At least one failed gate becomes a recursive repair workflow and re-enters the parent plan.
- [ ] The dry-run publish ends in a truthful typed outcome.
- [ ] No registry publication, tag, push, or production mutation occurs.
- [ ] The final receipt contains no orphan lifecycle event or unreceipted actuation.
- [ ] Replay verifies the RDF dataset, payload digests, approval binding, and terminal outcome.
- [ ] A human-readable verifier report is generated from the graph.

If any blocking item fails, the release status is not `ALIVE`. It must be reported as `REFUSED`,
`BOUNDED`, `UNSUPPORTED`, `INCONSISTENT`, or `PARTIAL_ALIVE` according to the exact failure.

## Grounding Appendix

Everything above this heading is the verbatim source PRD
(`/Users/sac/Downloads/Operation_Dogfood_PRD_v26.7.13.md`, adopted unchanged). This appendix is
new. It folds in the grounding record from this session's plan
(`~/.claude/plans/sequential-cooking-metcalfe.md`), which reports that 10 Explore agents
independently checked all 12 rows of the Claims Reconciliation table above against the repository
and found every `Standing` value correct — no row in that table is changed by this appendix. What
follows are caveats the grounding session flagged as missing nuance in the table's "Scope and
caveat" / "Required promotion evidence" columns, plus a deferred-gaps register. Citations below
that name a specific file were re-checked directly in this session via `Read`/`grep`; citations to
the grounding plan alone are marked as such and not independently re-verified beyond what is cited.

### C12 — the GitHub Release path is a public mutation the REFUSED scope does not name

C12 is `REFUSED` ("Operation Dogfood permits autonomous external publication") on the grounds that
"external registry mutation is outside the admitted goal." That scoping is accurate for
crates.io: `.github/workflows/release.yml` lines 103-119 show the `cargo publish` step commented
out under the heading "Uncomment to publish to crates.io on release. Requires
`CARGO_REGISTRY_TOKEN` secret."

The same workflow, however, defines a `release` job (lines 88-101) that is not commented out and
runs unconditionally on every push of a tag matching `[0-9][0-9].[0-9]+.[0-9]+` (lines 3-6): it
downloads the build matrix's artifacts and calls `softprops/action-gh-release@v2` with
`generate_release_notes: true`, which creates a public GitHub Release and attaches the built
binaries. That is a real, autonomous, external mutation outside crates.io and outside the current
C12 scope note. It is not gated by the same registry-publish switch. Treat this explicitly as an
exclusion Operation Dogfood does not yet cover: FR-18 ("The release workflow shall not create a
registry release, Git tag, push, deployment, or other public mutation") speaks to the *dry-run
workflow's own actions*, not to whether a downstream CI trigger fires independently once a tag is
pushed. A future publish-permission surface (the promotion evidence C12 already calls for) must
account for the GitHub-Release-on-tag path, not only the crates.io path.

### C8 — the REFUSED evidence understates and partly conflates the known blocker set

C8's "Scope and caveat" column names "unversioned path dependencies, license gaps, a missing root
license, path leaks, and only a near-term subset of crates as publishable." The repository's own
authoritative blocker taxonomy is `docs/PUBLISH_ALL_PRAXIS_PLAN.md` (section "Go/No-Go blockers"),
which enumerates seven distinct blockers B1-B7. Cross-checking C8's prose against that taxonomy
this session:

- B2 ("BUSL-1.1-licensed `wasm4pm` deps enter MIT/Apache-declared crates non-optionally" — a
  license-lineage/compatibility problem) and B3 ("four crates —`audit-tools`, `air_core_nif`,
  `tmp_sparql2`, `mfact-core` — have no `license` field at all" — a missing-declaration problem)
  are two different failure modes. C8's single phrase "license gaps" does not distinguish them.
- B4 ("`tmp_sparql2` is entirely git-ignored, zero packageable files") is omitted from C8's prose
  entirely.
- B7 ("`praxis-lean` has 3 untracked-but-not-ignored files that would ship") is also omitted. Per
  `docs/releases/v26.7.13/DRY_RUN_PUBLISH_VERDICT.md` ("Evidence references" / "still open today"
  note), B7 was confirmed still open in this session's own `git status --short`
  (`crates/praxis-lean/src/closure.rs`, `crates/praxis-lean/src/receipt_gate.rs`,
  `crates/praxis-lean/tests/receipt_closure_gate.rs` untracked).

This does not change C8's `REFUSED` standing — it names where the standing's supporting evidence
is narrower than its prose summary. `docs/releases/v26.7.13/DRY_RUN_PUBLISH_VERDICT.md`, an
independent verdict written this session, reaches the same `REFUSED` conclusion by a different
route (Gate 1 checkbox failures plus an unimplemented Gates 2-6 harness) and cites the B1-B7
taxonomy by reference for remediation scope.

Separately: `packs/dry-run-publish-pack/` (the "Kestrel Toolkit" case study — a fictional 3-crate
publish set, confirmed by reading
`crates/cng/tests/fixtures/dry-run-publish/dry-run-publish-case-study.ttl`, which states in its
own header comment that it is "chosen precisely so this fixture can never be read as this repo's
own real crates.io publish set") is an in-flight modeling artifact for the six-gate PDDL domain.
It exercises no real crate and executes no harness. Its existence does not move C8 off `REFUSED`;
the PRD's "Required promotion evidence" column ("Real dry-run execution closes or explicitly
excludes every blocker and produces an ALIVE receipt") still names the correct bar, and this
fixture does not clear it.

### C7 — the in-repo OCEL sources checked this session are OTel spans and chatman fixtures

C7's "Scope and caveat" column is accurate as written: it describes an FR-11 requirement, not a
present-tense claim of OCEL coverage. Confirmed this session, for context on what OCEL material
does exist today (so a later promotion pass does not mistake it for C7 evidence): `crates/cng/src/
otel_ocel.rs` derives `urn:graph:ocel` from `urn:graph:otel` via a SPARQL `CONSTRUCT` query over
admitted OTLP span data (`otel_rdf::project_admitted_spans`), and the OCEL JSON fixtures under
`.cargo-cicd/ocel/chatman/*.ocel.json` (e.g. `hook.ocel.json`) carry `case_id: "chatman-hook"` and
`fixture_id` attributes — hand-authored chatman-engine fixtures, not captured Claude Code tool
events. Neither source is a Claude Code session transcript or hook payload. FR-11's adapter (tool
intent, tool result, subagent creation, stop outcome) has no OCEL-producing implementation yet;
the grounding plan's Agent 6 finding that session transcripts under `~/.claude/projects/*.jsonl`
are the raw material FR-11 would need to consume is not independently re-verified in this pass
beyond noting the file-shape claim is plausible and unexplored here.

### Outcome-algebra materialization note

PRD section 6.6 declares a 5-element outcome vocabulary: `Found`, `Exhausted`, `Bounded`,
`Unsupported`, `Inconsistent`. Checked this session against `docs/releases/v26.7.13/ARD.md`
(owned by a different agent in this same release; read-only citation here): lines 167-179 of that
document define `SearchOutcome<T>` inside a code block explicitly marked `// PLANNED` with only
three variants — `Found(T)`, `Bounded`, `Exhausted` — and no `Unsupported` or `Inconsistent` arm.
A repository-wide search this session (`grep -rln "enum SearchOutcome\|struct SearchOutcome"
crates/`) returns zero matches: no crate defines this type anywhere. Both the PRD's 5-element
vocabulary and the ARD's 3-element vocabulary are therefore aspirational text, not implemented
code, and they disagree with each other on cardinality. C11's `PARTIAL_ALIVE` standing and its
"Required promotion evidence" ("Typed outcome algebra preserved through planner, harness, Claude
Code adapter, CLI, receipt, and replay") already anticipate this gap; this note adds that a
promotion pass must first reconcile which vocabulary (5-element or 3-element, or a documented
superset) is the one actually implemented, since the two design docs currently disagree.

### Public-ontology usage (NFR-3's twelve terms), checked this session

NFR-3 lists PROV-O, P-Plan, ODRL, SHACL, DCTERMS, DCAT, SKOS, SOSA, EARL, SPDX, DOAP, and QUDT.
Usage checked this session by grep/Read against `crates/cng/src/otel_receipt.rs`,
`crates/praxis-graphlaw/ontologies/`, and `vendors/oxigraph/testsuite/`:

| Ontology | Status this session | Evidence |
|---|---|---|
| PROV-O | In use | `crates/cng/src/otel_receipt.rs` mints `prov:Activity`/`prov:Entity`/`prov:Plan` nodes linked by `prov:used`/`prov:hadPlan`/`prov:generated` (module doc comment, lines 27-45) |
| SHACL | In use | Runtime-enforced admission gates (per grounding plan Agent 4; not independently re-run this pass) |
| DCTERMS | In use | `dcterms:description` on every entity in `crates/cng/tests/fixtures/dry-run-publish/dry-run-publish-case-study.ttl` |
| SKOS | In use | `skos:notation`/`skos:Concept` as the deterministic join key in the same fixture file |
| DCAT | Vendored, no consuming code | Local `.ttl`/`.rdf` copies under `crates/praxis-graphlaw/ontologies/{core,provenance,catalogs,industry}/`; no Rust code constructs `dcat:` triples |
| SOSA | Vendored, no consuming code | `crates/praxis-graphlaw/ontologies/domain/sosa.ttl` present; no consuming code found |
| QUDT | Vendored, no consuming code | `crates/praxis-graphlaw/ontologies/domain/qudt-ontology.ttl` present; no consuming code found |
| ODRL | Vendored, no consuming code | `crates/praxis-graphlaw/ontologies/industry/energy/9-odrl-vocabulary.rdf` present; matches C5's `PLANNED` standing (permission artifact not yet built) |
| SPDX | Catalog reference only, no local copy | `crates/praxis-graphlaw/ontologies/catalog.ttl` lines 202-208 register SPDX with a remote `dcat:downloadURL`, not a vendored file — a step below the DCAT/SOSA/QUDT/ODRL tier |
| DOAP | Catalog reference only, no local copy | `crates/praxis-graphlaw/ontologies/catalog.ttl` lines 212-218, same remote-reference pattern as SPDX |
| P-Plan | Absent | No file or code hit for `p-plan`/`P-Plan` anywhere under `crates/` this session |
| EARL | Absent from first-party code | `earl:` hits only inside vendored `vendors/oxigraph/testsuite/src/report.rs` (an `earl:Assertion` emitter shipped with the oxigraph test harness); no first-party praxis code uses it |

This sharpens the grounding plan's "4 used / 6 vendored-inert / 2 absent" summary: SPDX and DOAP
are weaker than DCAT/SOSA/QUDT/ODRL (catalog-registered by remote URL rather than locally
vendored), so a more precise split is 4 in use, 4 vendored-inert (DCAT, SOSA, QUDT, ODRL), 2
catalog-reference-only (SPDX, DOAP), 2 absent (P-Plan, EARL first-party). This does not change
C10's `PARTIAL_ALIVE` standing; it gives the "Namespace report" promotion evidence a more exact
starting inventory.

### Deferred-gaps register

Carried verbatim from the grounding plan's recommended-plan step 4, as a named, tracked list of
gaps this adoption pass does not close. None of these are represented as done anywhere in this
document; each remains `PLANNED` or `UNKNOWN` until a future increment closes it with cited
evidence:

1. TTL/expiration on permission artifacts — no implementation exists anywhere in the repository.
2. The ODRL policy artifact itself — an empty, quarantined dialect slot with no mechanism list.
3. Multi-graph dataset blank-node canonicalization — existing canonicalization sorts N-Quads text
   per graph, which is sound only for ground (blank-node-free) quads.
4. Permission-binding-in-replay — no replay surface currently re-verifies the approval/permission
   binding, only digests and plan binding.
5. f25 L7 post-restart replay — disclosed as `NotImplemented` where it was checked.
6. A residue-from-real-failure synthesizer — today's residues are hand-authored Turtle fixtures;
   nothing programmatically turns a failed gate into a `domain_pddl` + `problem_pddl` + blocked
   socket.
7. Programmatic patch admission — `git apply` plus `just` is manual practice today; nothing
   parses or applies diffs, or runs gates as an admission step, in Rust.
8. A `claude -p` headless launcher — zero in-repo precedent found; the designated integration seam
   is the existing `ExternalCutCompiler` trait (a new trait implementation, not new architecture).
9. EARL adoption as a first-party vocabulary — the oxigraph-testsuite emitter is a ready precedent
   but is not wired into any praxis-owned code path.
10. An `mfw` binary — `multifractal-workflow` is library-only today (no `[[bin]]` target); its
    families are reachable only through tests or the `cng` bench harness.
11. `dispatch_bridge` task-packet generalization — the transport is real but hardwired to a fixed
    22-field PDDL contract; an FR-10 free-form task packet needs a new artifact type and handler.

None of items 1-11 above is asserted `ALIVE` or `PARTIAL_ALIVE` by this appendix; they are named
here so a later promotion pass has a fixed register to check off against, rather than
rediscovering them from scratch.
