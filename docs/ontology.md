# Ontology Specifications (`ontology/`)

The [`ontology/`](file:///Users/sac/praxis/ontology/) directory contains semantic descriptions of classes, properties, and constraints modeled in **RDF/Turtle (`.ttl`)** notation. 

These specifications represent the declarative ground truth ($O$ in the Post-Chatman Equation $A \cong O \cong L$).

## Directory Files

- **[`lawobject.ttl`](file:///Users/sac/praxis/ontology/lawobject.ttl)**: The primary schema modeling obligations, validators, and lifecycles.
- **[`workflow_demo.ttl`](file:///Users/sac/praxis/ontology/workflow_demo.ttl)**: Turtle model mapping a multi-agent validation graph workflow.

## Structure of the Ontology

Ontologies are written using standard prefixes:
```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix pdl: <http://praxis.seanchatmangpt.com/ontology/lawobject#> .
```

### Class Definitions
Classes model key system entities, such as the `LawObject`, validators, and obligations. Inheritance is defined using `rdfs:subClassOf`.

### Property Constraints
Properties link subjects to objects (e.g., `pdl:hasEvidence`) and specify domains and ranges.

## Compiling to Logic
The `ggen` compilation pipeline reads these Turtle schemas, parses the directed graph, and emits a STRIPS8 PDDL planning domain. This guarantees that runtime logic models match declarative schemas.
