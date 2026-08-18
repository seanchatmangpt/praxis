# Semantic Runtime Contracts

This rule refines `AGENTS.md`; it may not weaken it. It governs semantic
runtime design, public ontologies, admitted observations, Knowledge Hooks,
N3, process evidence, and editor-time semantic infrastructure.

## 1. BRCE is the authority root

BRCE is upstream of POWL v2, TOGAF, SAFe, Jira, Spec Kit, agents, builds,
release rails, and dashboards. Those systems are projections or consumers of
admitted state; they do not manufacture authority by displaying it.

The Broker is the only `DO` path. Every other component may select, construct,
propose, diagnose, route, or verify, but it may not actuate machine state
without broker admission and a receipt.

```text
SELECT / observe / construct != DO
DO = admitted broker actuation + receipt + replay hook
```

The invariant remains **zero unreceipted actuation**.

## 2. Combinatorial maximalism and reversible construction

Maximize graph-domain construction before machine-state mutation. Explore
alternatives in a reversible, content-addressed domain; admit one bounded
choice; actuate only the admitted choice.

```text
partial observation O
  → reversible graph construction
  → admitted observation O*
  → lawful intent
  → brokered actuation A
  → receipt R
```

`SELECT/DO → CONSTRUCT` is a category change: the system should manufacture
inspectable candidate structure before crossing an irreversible boundary.
Memory or graph growth may be a theorem witness when it records reversible
alternatives; it is not itself permission to actuate.

## 3. O*.toml admission carrier

`O*.toml` is the canonical human-readable carrier for admitted bounded
observation when TOML is used. A TOML document is not admitted merely because
it parses.

Admission requires, as applicable:

- exact source identities and precedence;
- closed keys or explicitly exploratory mode;
- typed environment overrides;
- path and traversal policy;
- deterministic merge winner map;
- canonical serialization;
- BLAKE3 witness over sources, order, overrides, and canonical bytes;
- positive and negative fixtures;
- no editor, LSP, telemetry stream, or OCEL history granting standing.

Typestate constructors and witness fields must prevent external forgery.

## 4. Public-ontology-first law

Use existing public vocabularies before minting local terms:

- PROV-O for provenance and activities;
- DCAT and DCTERMS for datasets, catalogs, identity, and document relations;
- SKOS for controlled concept schemes;
- SHACL for executable shape constraints;
- ODRL for permissions, prohibitions, and duties;
- FOAF for agents where applicable;
- OCEL for object-centric event history;
- FIBO for financial and legal-entity concepts;
- QUDT for quantities, units, and dimensions;
- SOSA/SSN for observations, sensors, and measurements;
- OWL-Time for logical and graph-carried time.

Mint a local term only after documenting the public-vocabulary gap, its
boundary, its closed-value policy, and the shape or verifier that enforces it.
A vendored ontology is not an active dependency until a real query, shape,
projection, or runtime path consumes it.

## 5. Knowledge Hook law

Knowledge Hooks manufacture intents; they do not perform side effects.

```text
O → O* → C → I → Ia → A → R
```

Where:

- `O` is partial observation;
- `O*` is admitted observation;
- `C` is a derived claim or condition;
- `I` is an intent manufactured by routing `ρ(Cd) = I`;
- `Ia` is an admitted, authorized intent;
- `A` is brokered actuation;
- `R` is the receipt.

`Hd != actuate`: a hook decision or derivation is not machine-state mutation.
Hook failures such as `ToolFailed`, `TimedOut`, `Unsupported`, malformed
program, or missing authority remain typed outcomes and never collapse into
truth, permission, or successful effect.

Proposal visibility is not authority. A constructed triple is not proof that
the represented external action occurred.

## 6. N3 quarantine

N3 is a last-resort reasoning surface, never the default actuation language.
Where enabled, it is quarantined:

- server-side execution only;
- explicit builtin whitelist;
- no network access;
- no OS process execution;
- no direct broker or canonical-store mutation;
- no semantic-log self-authorization;
- bounded recursion depth `<= 8`;
- bounded derived facts `<= 4096` per admitted run;
- deterministic ordering and canonical output;
- receipt/replay and tamper rejection;
- typed timeout, unsupported-builtin, and bound-exceeded refusals.

`LogWebOperation`, `LogSemantics`, and `OsProcess` remain quarantined capability
classes unless separately admitted through a brokered boundary.

## 7. LSP and semantic ownership

Editor-time systems observe and diagnose; they do not grant standing.

Ownership boundaries:

- Tree-sitter owns concrete syntax trees;
- Salsa memoizes computations and dependencies but does not own or persist a
  Tree-sitter tree as semantic authority;
- Oxigraph or the governed RDF store owns semantic graph state;
- the LSP owns document snapshots, diagnostics, and ephemeral analysis;
- admission witnesses and runtime receipts own standing.

The invariant is `SalsaDoesNotOwnTreeSitterTree`.

An LSP may report parse, shape, path, vocabulary, or likely-admission errors.
It may not construct an authoritative admitted configuration, mutate the
canonical graph, manufacture `q_config`, or promote telemetry into standing.

## 8. OCEL and process evidence

OCEL records lifecycle history and object relations. It does not grant
admission or authority by itself.

A process claim must bind:

- the exact emitting boundary;
- event and object identity;
- logical time and deterministic ordering;
- event-to-object qualifiers;
- receipt-chain position;
- replay or conformance verifier;
- named exclusions between observed history and authorized actuation.

A process trace is evidence about execution, not automatic proof of semantic
correctness, authorization, or theorem standing.

## 9. Deterministic ticket law

A ticket must be deterministic. It identifies:

- exact repository and base SHA;
- admitted inputs and exclusions;
- bounded output;
- owner and actuation surface;
- acceptance command or inferred verifier;
- positive and negative fixture;
- receipt location;
- replay/falsifier condition;
- standing transition permitted by successful execution.

A narrative aspiration without an executable acceptance boundary is not an
implementation ticket.

## 10. Falsifiers

This contract is falsified if any runtime path can:

- actuate outside the Broker;
- treat a hook proposal as permission;
- treat OCEL or LSP telemetry as admission authority;
- mint local ontology terms without a documented public-vocabulary gap;
- execute N3 with unbounded derivation, network, process, or canonical
  mutation capability;
- let Salsa own the syntax tree or an editor create standing;
- accept an `O*.toml` carrier without recomputable provenance and witness;
- close a ticket without an exact acceptance boundary and receipt.
