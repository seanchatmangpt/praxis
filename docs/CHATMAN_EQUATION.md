# The Chatman Equation and Chatman Engine as its Realized μ

Chatman Engine is the concrete realization of μ in the Chatman Equation
`A = μ(O*)`, `R = receipt(A)` — where `O` is raw observation, `O*` is admitted
observation, `μ` is the lawful manufacturing transformation, `A` is an
artifact with standing, and `R` is a receipt proving consequence. Before
Chatman Engine existed, μ was abstract; Chatman Engine makes it concrete. This
document exists so future sessions do not miscategorize the engine as a
narrower, more familiar system.

## What Chatman Engine Is Not

Chatman Engine is not a planner, workflow engine, ontology system, receipt
system, document generator, or agent runtime in isolation. It is the lawful
transformation surface that turns admitted reality into standing-bearing
artifacts — doing all of: admit, classify, plan, sequence, route, render,
actuate, refuse, receipt, replay, as one manufacturing function. Any
description that reduces it to one of these individual capabilities is a
projection of μ, not μ itself.

## The Stronger Formulation

```
A_n = mu_n(O*, P, H, C, E, T)
```

Where:

- `O*` = admitted observation graph
- `P` = public ontology coverage
- `H` = human authority / avatar decisions
- `C` = constraints, policies, rules
- `E` = evidence and provenance
- `T` = time, deadlines, sequence, state
- `A_n` = standing-bearing artifact at process stage `n`

Example staged artifacts for a legal matter: opened matter, evidence packet,
demand package, filed complaint, served defendant, discovery record,
mediation packet, judgment or settlement, payment ledger, disbursement,
closed matter. Each artifact has standing only if manufactured through μ —
not by any other path.

## Why Public Ontologies Matter

Public ontologies are part of `O*`. They prevent the engine from inventing
private meaning too early. Lawful input is public person/organization/event/
document/medical/financial/provenance/legal-process terms, with minimal
bridge terms used only where no public term exists.

## Why Humans-in-the-Loop Matter

Human avatars are part of `O*` too. The human carries authority: the client
approves settlement, the attorney approves legal sufficiency, the judge
enters judgment, the clerk files the document, the bookkeeper approves
disbursement, the lienholder asserts a lien. These are standing-changing
acts, not workflow labels. μ must preserve who had authority, what they saw,
what they decided, what evidence supported it, what downstream state
changed, and what refusal would happen without it.

## Why PDDL v3.1 and POWL v2 Are Projections, Not the Source

```
RDF/Turtle matter graph
  -> PDDL v3.1 planning projection
  -> POWL v2 workflow projection
  -> RDF-native event/evidence graph
  -> OCEL 2.0 export
```

The admitted graph determines what goals are reachable. The planning
projection makes reachability explicit. The workflow projection makes lawful
process order explicit. The event graph makes execution evidence explicit.
The receipt makes standing explicit. Replay makes consequence
non-ornamental. None of these projections is the source of truth — the
admitted graph and μ are.

## The Paradigm Shift

Most systems do "data → app logic → output." Chatman Engine does "admitted
public meaning → lawful manufacturing → standing-bearing artifact" — a
different category. Common miscategorizations to actively avoid when
describing it: workflow automation, document generation, test harness,
ontology project, legal tech intake tool, agent orchestration. These are
projections of μ, not μ itself.

## Closing Thesis

> Chatman Engine is the concrete μ of the Chatman Equation: a
> standing-preserving manufacturing function that transforms admitted
> public-ontology reality, human authority, evidence, constraints, and
> process state into lawful artifacts, standard projections, typed
> refusals, receipts, and deterministic replay.

## See Also

- `docs/chatman-engine/chicago_tdd_final_report.md`
- `docs/releases/v26.7.9/ARD.md`
- `docs/releases/v26.7.9/PRD.md`
- `crates/praxis-graphlaw/src/chatman/engine.rs`
