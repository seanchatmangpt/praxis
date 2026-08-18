# RDF Blue Ocean / TRIZ Innovation Law Pack

This directory is an executable GraphLaw reference pack for the Post-AGI doctrine that **cognition is a fallback cost, not an architectural primitive**.

The pack turns Blue Ocean Strategy and TRIZ into bounded graph-law operations:

```text
RDF innovation profile + industry state
  -> Datalog/N3 closure
  -> SPARQL opportunity discovery
  -> full Blue Ocean ERRC derivation
  -> TRIZ contradiction principles
  -> candidate future RDF graphs
  -> ShEx structural grammar
  -> SHACL admission
  -> denial/falsifier checks
  -> DfCM planning frontier
  -> PDDL / POWL v2
```

The crown ends at a reversible planning frontier. It does **not** actuate. Any eventual world transition remains downstream of ggen manufacture and the BRCE/GymAct consequence boundary.

## Semantic boundary

`ontology.ttl` defines the narrow innovation-law profile used by the pack. It composes W3C RDF/RDFS, OWL, SKOS, and Dublin Core Terms for vocabulary identity and controlled concepts. It is not an attempt to replace domain ontologies: real industry graphs align their own activities, capabilities, buyer factors, constraints, and contradictions into these bounded innovation roles.

## Dialect responsibilities

| GraphLaw surface | Innovation responsibility |
|---|---|
| RDF | Industry, capability, buyer/noncustomer, contradiction, candidate state |
| SPARQL SELECT | Discover opportunity intersections |
| SPARQL CONSTRUCT | Manufacture reversible candidate/planning graphs |
| Datalog/N3 | Derive ERRC consequences and TRIZ principles |
| ShEx | Structural grammar of a complete candidate future |
| SHACL | Admission constraints before planning handoff |
| Denials | Falsifiers and Post-AGI regression guards |
| RSP | Future continuous opportunity detection surface |
| DRed / IMaRS | Future incremental maintenance of candidate closure |

N3 remains a last-resort rule surface. The reference pack uses simple monotonic rules that are expressible without LLM inference. No rule grants actuation authority.

## Reference crown

`industry-fixture.n3` models a bounded incumbent value system:

- manual coordination is high cost and low buyer utility, so it is an **ELIMINATE** candidate;
- bespoke reporting is heavily invested in with low buyer utility, so it is a **REDUCE** candidate;
- verified outcome speed is highly important with low current performance, so it is a **RAISE** candidate;
- manual coordination blocks a noncustomer while a deterministic capability performs the same function, allowing a **CREATE** future to be derived;
- the incumbent assumes increasing speed reduces control, creating a TRIZ contradiction.

`blue-ocean-triz.n3` derives the complete ERRC surface plus:

- a zero-human/zero-agent `CandidateFuture`;
- both `SeparationInTime` and `Intermediary` TRIZ principles;
- all applicable principles preserved on the candidate;
- no selected winner.

The denial rules refuse a candidate that declares agent mediation required while an equivalent deterministic morphism already exists, and also refuse contradictory `None` + `Required` agent-mediation state.

`candidate.shacl.ttl` and `candidate.shex.json` independently gate completeness and structural form. `construct-futures.rq` then projects admitted candidates into a PDDL planning frontier.

## Crown invariants

```text
ERRC surface                   = Eliminate + Reduce + Raise + Create
human selection required      = false
agent selection required      = false
LLM inference required        = false
CONSTRUCT mutates source RDF  = false
candidate implies authority   = false
candidate implies actuation   = false
DfCM winner selected upstream = false
```

The corresponding executable tests are:

- `tests/sparql_construct_projection.rs`
- `tests/chatman_rdf_innovation_crown.rs`

The design deliberately treats innovation as **closure + contradiction + construction + constraint + falsification + planning**, not as an inherently cognitive act.
