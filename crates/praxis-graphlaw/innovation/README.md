# RDF Blue Ocean / TRIZ Innovation Law Pack

This directory is an executable GraphLaw reference pack for the Post-AGI doctrine that **cognition is a fallback cost, not an architectural primitive**.

The pack turns Blue Ocean Strategy and TRIZ into bounded graph-law operations:

```text
RDF industry state
  -> Datalog/N3 closure
  -> SPARQL opportunity discovery
  -> Blue Ocean ERRC derivation
  -> TRIZ contradiction principles
  -> candidate future RDF graphs
  -> ShEx structural grammar
  -> SHACL admission
  -> denial/falsifier checks
  -> DfCM planning frontier
  -> PDDL / POWL v2
```

The crown ends at a reversible planning frontier. It does **not** actuate. Any eventual world transition remains downstream of ggen manufacture and the BRCE/GymAct consequence boundary.

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

`industry-fixture.n3` models one bounded incumbent contradiction:

- manual coordination is high cost and low buyer utility;
- it blocks a noncustomer;
- a deterministic capability performs the same underlying function;
- the incumbent assumes increasing speed reduces control.

`blue-ocean-triz.n3` derives:

- an `EliminateOpportunity`;
- a zero-human/zero-agent `CandidateFuture`;
- both `SeparationInTime` and `Intermediary` TRIZ principles;
- no selected winner.

The denial rules refuse a candidate that declares agent mediation required while an equivalent deterministic morphism already exists.

`candidate.shacl.ttl` and `candidate.shex.json` independently gate completeness and structural form. `construct-futures.rq` then projects admitted candidates into a PDDL planning frontier.

## Crown invariants

```text
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
