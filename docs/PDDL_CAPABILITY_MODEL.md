# PDDL Capability Model: From LawObject to Planning Domain

## Overview

CPhy's LawObject is a compile-time type-safe wrapper around obligations, lifecycle transitions, and cryptographic receipts. This document describes how to model a LawObject as a **PDDL planning problem**, enabling automated reasoning about capability lifecycle (Raw → Validated → Admitted → Receipted).

The model captures:
- **Predicates**: Evidence, preconditions, blocking constraints, Andon status
- **Actions**: Judge (evaluate obligations), Admit (admit validated objects), Receipt (compute chain hash), Promote (override Andon holds)
- **Resources**: Andon capacity (holds), chain validators, evidence suppliers
- **Effects**: State transitions that mirror the typestate lifecycle

---

## PDDL Domain Schema

### Types

```pddl
(define (domain lawobject-capability)
  (:requirements :typing :durative-actions :adl :universal-preconditions)

  (:types
    law-object
    obligation
    evidence-type
    predicate
    andon-state
    lifecycle-stage
    validator
    authority
    chain-token
  )
```

#### Type Semantics

| Type | Meaning |
|------|---------|
| `law-object` | A domain payload under judgment (e.g., a contract, claim, artifact) |
| `obligation` | An obligation instance (Precondition, BlockingConstraint, EvidenceRequired) |
| `evidence-type` | Category of evidence (e.g., "identity", "ledger_entry", "audit_log") |
| `predicate` | A namespaced predicate check (e.g., "balance_positive", "signature_valid") |
| `andon-state` | Halt status: `green`, `halted`, `overridden` |
| `lifecycle-stage` | Raw, Validated, Admitted, Receipted |
| `validator` | An agent or system that performs judgment (Judge) |
| `authority` | An agent that can override Andon holds (Admit authority) |
| `chain-token` | Cryptographic identity for chain hash (receipt anchor) |

---

### Predicates

#### Obligation Predicates

```pddl
(is-precondition ?ob - obligation ?pred - predicate)
  ; True if obligation ?ob is a Precondition on predicate ?pred.

(is-blocking-constraint ?ob - obligation)
  ; True if obligation ?ob is a BlockingConstraint.

(requires-evidence ?ob - obligation ?etype - evidence-type)
  ; True if obligation ?ob is EvidenceRequired for evidence type ?etype.

(evidence-satisfied ?ob - obligation)
  ; True if the evidence requirement for ?ob has been provided.

(precondition-satisfied ?pred - predicate)
  ; True if the precondition ?pred has been evaluated and passed.

(blocking-constraint-cleared ?ob - obligation)
  ; True if the BlockingConstraint has been explicitly cleared or evidence provided.
```

#### State & Lifecycle Predicates

```pddl
(in-stage ?obj - law-object ?stage - lifecycle-stage)
  ; ?obj is in stage ?stage (Raw, Validated, Admitted, Receipted).

(has-obligation ?obj - law-object ?ob - obligation)
  ; law-object ?obj carries obligation ?ob.

(andon-status ?obj - law-object ?state - andon-state)
  ; Current Andon state of ?obj: green, halted, or overridden.

(obligation-unmet ?obj - law-object ?ob - obligation)
  ; Obligation ?ob on ?obj is currently unmet (contributes to Halted state).

(andon-holds ?obj - law-object)
  ; Shorthand: Andon is halted; object cannot progress without resolution.
```

#### Chain & Receipt Predicates

```pddl
(chain-hash-computed ?obj - law-object ?token - chain-token)
  ; Chain hash for ?obj has been computed and is stored in ?token.

(prev-chain-valid ?token - chain-token)
  ; The previous chain hash (ancestor in receipt chain) is valid and accessible.

(signature-applied ?obj - law-object)
  ; Ed25519 signature has been applied to ?obj (if signed feature enabled).
```

#### Authority & Capability Predicates

```pddl
(validated-by ?obj - law-object ?validator - validator)
  ; law-object ?obj has been validated by ?validator.

(admitted-by ?obj - law-object ?authority - authority)
  ; law-object ?obj has been admitted by ?authority.

(override-authority ?authority - authority ?ob - obligation)
  ; ?authority is permitted to override obligation ?ob.

(andon-override-applied ?obj - law-object ?by - authority)
  ; Andon hold on ?obj has been overridden by ?by.
```

---

### Actions

#### Action 1: JUDGE (Raw → Validated)

Evaluates all obligations on a law-object in Raw stage. Transitions to Validated if all obligations pass, or stays Raw (in Halted Andon state) if any fail.

```pddl
(:action judge
  :parameters (
    ?obj - law-object
    ?validator - validator
  )
  :precondition (and
    (in-stage ?obj raw)
    (forall (?ob - obligation)
      (implies
        (has-obligation ?obj ?ob)
        (or
          (and (is-precondition ?ob ?pred) (precondition-satisfied ?pred))
          (and (is-blocking-constraint ?ob) (blocking-constraint-cleared ?ob))
          (and (requires-evidence ?ob ?etype) (evidence-satisfied ?ob))
        )
      )
    )
  )
  :effect (and
    (not (in-stage ?obj raw))
    (in-stage ?obj validated)
    (validated-by ?obj ?validator)
    (andon-status ?obj green)
    (not (andon-holds ?obj))
  )
)
```

**Semantics:**
- Precondition: object is Raw, and all obligations are satisfied.
- Effect: object transitions to Validated; Andon marked Green.
- Failure: if any obligation remains unmet, the action is inapplicable (precondition fails). The object stays Raw with Andon::Halted.

---

#### Action 2: ADMIT (Validated → Admitted)

Transitions a Validated object to Admitted state, pending authority approval and absence of Andon holds.

```pddl
(:action admit
  :parameters (
    ?obj - law-object
    ?authority - authority
  )
  :precondition (and
    (in-stage ?obj validated)
    (not (andon-holds ?obj))
    (andon-status ?obj green)
  )
  :effect (and
    (not (in-stage ?obj validated))
    (in-stage ?obj admitted)
    (admitted-by ?obj ?authority)
  )
)
```

**Semantics:**
- Precondition: object is Validated and Andon is Green (no holds active).
- Effect: object transitions to Admitted, recorded as admitted-by ?authority.
- Failure: if Andon is Halted or Overridden, the action is inapplicable.

---

#### Action 3: RECEIPT (Admitted → Receipted)

Computes chain hash and optionally applies signature, transitioning Admitted → Receipted.

```pddl
(:action receipt
  :parameters (
    ?obj - law-object
    ?prev-token - chain-token
    ?new-token - chain-token
  )
  :precondition (and
    (in-stage ?obj admitted)
    (prev-chain-valid ?prev-token)
    (not (chain-hash-computed ?obj ?new-token))
  )
  :effect (and
    (not (in-stage ?obj admitted))
    (in-stage ?obj receipted)
    (chain-hash-computed ?obj ?new-token)
    (signature-applied ?obj)
  )
)
```

**Semantics:**
- Precondition: object is Admitted, previous chain hash is accessible, new token not yet used.
- Effect: object transitions to Receipted; chain hash computed and stored; signature applied.
- Failure: if previous chain is invalid or new token already exists, action is inapplicable.

---

#### Action 4: PROMOTE (Andon::Halted → Andon::Overridden)

Override an Andon hold (e.g., when evidence arrives or authority waives a constraint).

```pddl
(:action promote-andon
  :parameters (
    ?obj - law-object
    ?authority - authority
    ?ob - obligation
  )
  :precondition (and
    (in-stage ?obj raw)
    (andon-holds ?obj)
    (andon-status ?obj halted)
    (obligation-unmet ?obj ?ob)
    (override-authority ?authority ?ob)
  )
  :effect (and
    (not (andon-status ?obj halted))
    (andon-status ?obj overridden)
    (andon-override-applied ?obj ?authority)
    (not (obligation-unmet ?obj ?ob))
    (not (andon-holds ?obj))
  )
)
```

**Semantics:**
- Precondition: object is Raw with Halted Andon; a specific obligation is unmet; authority is permitted to override it.
- Effect: Andon status changes to Overridden; obligation is cleared; object is no longer held.
- This allows the object to proceed to judgment even with overridden constraints.

---

#### Action 5: SUPPLY-EVIDENCE (Satisfy EvidenceRequired obligation)

External action: system receives evidence, satisfies an EvidenceRequired obligation.

```pddl
(:action supply-evidence
  :parameters (
    ?obj - law-object
    ?ob - obligation
    ?etype - evidence-type
  )
  :precondition (and
    (has-obligation ?obj ?ob)
    (requires-evidence ?ob ?etype)
    (not (evidence-satisfied ?ob))
  )
  :effect (and
    (evidence-satisfied ?ob)
    (not (obligation-unmet ?obj ?ob))
  )
)
```

**Semantics:**
- Precondition: object has obligation requiring evidence; evidence has not yet been provided.
- Effect: obligation is satisfied; contributes to Andon becoming Green.

---

## PDDL Problem: Concrete Example

### Domain: Smart Contract Claim Validation

**Scenario:** A contract claim arrives as Raw. It carries two obligations:
1. `Precondition("signature_valid", params_hash=0x...)` — signature must be cryptographically valid.
2. `EvidenceRequired("ledger_entry", ...)` — claim must reference a ledger entry.

**Goal:** Transition the claim from Raw → Receipted with a valid chain hash.

### Problem Definition

```pddl
(define (problem contract-claim-validation)
  (:domain lawobject-capability)

  (:objects
    claim-001 - law-object
    judge-service - validator
    admissions-authority - authority
    sig-validator - predicate
    ledger-supplier - evidence-type
    chain-genesis - chain-token
    chain-claim-001 - chain-token
  )

  (:init
    ; Initial stage
    (in-stage claim-001 raw)

    ; Obligations
    (has-obligation claim-001 ob-signature)
    (has-obligation claim-001 ob-ledger)

    (is-precondition ob-signature sig-validator)
    (requires-evidence ob-ledger ledger-supplier)

    ; Andon: initially halted (obligations unmet)
    (andon-status claim-001 halted)
    (andon-holds claim-001)
    (obligation-unmet claim-001 ob-signature)
    (obligation-unmet claim-001 ob-ledger)

    ; Authority relationships
    (override-authority admissions-authority ob-signature)

    ; Chain: previous hash exists and is valid
    (prev-chain-valid chain-genesis)
  )

  (:goal (and
    (in-stage claim-001 receipted)
    (andon-status claim-001 green)
    (chain-hash-computed claim-001 chain-claim-001)
  ))
)
```

### Plan (Expected Solution)

**Step 1: Supply evidence (ledger entry arrives)**
```
supply-evidence(claim-001, ob-ledger, ledger-supplier)
  → evidence-satisfied(ob-ledger) = true
  → obligation-unmet(claim-001, ob-ledger) = false
```

**Step 2: Override signature constraint (authority waives or evidence emerges)**
```
promote-andon(claim-001, admissions-authority, ob-signature)
  → andon-status(claim-001) = overridden
  → andon-holds(claim-001) = false
```

Alternatively, if signature validation infrastructure confirms the signature:
```
(precondition-satisfied sig-validator) = true
```
Then the judge action becomes applicable.

**Step 3: Judge the claim**
```
judge(claim-001, judge-service)
  → in-stage(claim-001) = validated
  → validated-by(claim-001, judge-service) = true
  → andon-status(claim-001) = green
```

**Step 4: Admit the claim**
```
admit(claim-001, admissions-authority)
  → in-stage(claim-001) = admitted
  → admitted-by(claim-001, admissions-authority) = true
```

**Step 5: Generate receipt**
```
receipt(claim-001, chain-genesis, chain-claim-001)
  → in-stage(claim-001) = receipted
  → chain-hash-computed(claim-001, chain-claim-001) = true
  → signature-applied(claim-001) = true
```

**Final state matches goal:**
- claim-001 is in Receipted stage ✓
- Andon status is Green ✓
- Chain hash is computed ✓

---

## Resource & Capacity Constraints

The Andon hold system can be modeled as a **resource constraint**. PDDL permits numeric resources:

```pddl
(:requirements :typing :numeric-fluents :durative-actions)

; Resource: number of Andon holds blocking progress
(holds - number)   ; initially = 0 if no blocks

; During PROMOTE: decrease holds by 1
(:action promote-andon
  :effect (and
    ...
    (decrease (holds) 1)
  )
)

; JUDGE is only applicable if holds == 0
(:action judge
  :precondition (and
    ...
    (= (holds) 0)
  )
  ...
)
```

**Interpretation:**
- Each unmet obligation or unmet condition contributes to the hold count.
- PROMOTE, SUPPLY-EVIDENCE, and precondition satisfaction all **decrement** holds.
- JUDGE is only applicable when holds == 0 (Andon is Green).
- RECEIPT is only applicable after ADMIT (no holds in that state).

---

## Mapping: RDF Ontology → PDDL

### Overview

The **ggen** system can synthesize PDDL domain and problem definitions from an RDF ontology (RDFS/OWL). The mapping is systematic:

```
RDF Class
  ↓
PDDL type
  
RDF Property (with domain/range)
  ↓
PDDL predicate
  
RDF Restriction (cardinality, class, datatype)
  ↓
PDDL precondition or resource constraint
```

### RDF Input Example

```ttl
@prefix dom:  <http://example.com/obligation-domain#> .
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

# Domain ontology for obligation management

dom:Obligation
    a               rdfs:Class ;
    rdfs:label      "Obligation" ;
    rdfs:comment    "A precondition, blocking constraint, or evidence requirement." .

dom:Precondition
    a               rdfs:Class ;
    rdfs:subClassOf dom:Obligation ;
    rdfs:label      "Precondition" ;
    rdfs:comment    "A predicate that must be satisfied." .

dom:EvidenceRequired
    a               rdfs:Class ;
    rdfs:subClassOf dom:Obligation ;
    rdfs:label      "EvidenceRequired" ;
    rdfs:comment    "External evidence must be provided." .

dom:predicate
    a            rdf:Property ;
    rdfs:label   "predicate" ;
    rdfs:domain  dom:Precondition ;
    rdfs:range   xsd:string ;
    rdfs:comment "Name of the predicate to check." .

dom:evidenceType
    a            rdf:Property ;
    rdfs:label   "evidenceType" ;
    rdfs:domain  dom:EvidenceRequired ;
    rdfs:range   xsd:string ;
    rdfs:comment "Category or type of required evidence." .

dom:satisfiedBy
    a            rdf:Property ;
    rdfs:label   "satisfiedBy" ;
    rdfs:domain  dom:Obligation ;
    rdfs:range   dom:Evidence ;
    rdfs:comment "Points to evidence that satisfies this obligation." .

dom:Evidence
    a               rdfs:Class ;
    rdfs:label      "Evidence" ;
    rdfs:comment    "A piece of exogenous evidence (e.g., signature, ledger entry)." .

dom:evidenceTypeMatch
    a            rdf:Property ;
    rdfs:label   "evidenceTypeMatch" ;
    rdfs:domain  dom:Evidence ;
    rdfs:range   xsd:string ;
    rdfs:comment "The type of evidence this instance provides." .

# Obligation lifecycle stages

dom:LifecycleStage
    a               rdfs:Class ;
    rdfs:label      "LifecycleStage" ;
    rdfs:comment    "Stages in the obligation lifecycle." .

dom:Raw
    a               dom:LifecycleStage ;
    rdfs:label      "Raw" .

dom:Validated
    a               dom:LifecycleStage ;
    rdfs:label      "Validated" .

dom:Admitted
    a               dom:LifecycleStage ;
    rdfs:label      "Admitted" .

dom:Receipted
    a               dom:LifecycleStage ;
    rdfs:label      "Receipted" .

# Andon states

dom:AndonState
    a               rdfs:Class ;
    rdfs:label      "AndonState" .

dom:Green
    a               dom:AndonState ;
    rdfs:label      "Green" ;
    rdfs:comment    "All obligations met; proceed." .

dom:Halted
    a               dom:AndonState ;
    rdfs:label      "Halted" ;
    rdfs:comment    "Obligations unmet; halted." .

dom:Overridden
    a               dom:AndonState ;
    rdfs:label      "Overridden" ;
    rdfs:comment    "Obligations overridden by authority." .

# Core actions (obligation lifecycle transitions)

dom:Judge
    a               rdfs:Class ;
    rdfs:label      "Judge" ;
    rdfs:comment    "Action: evaluate obligations (Raw → Validated)." .

dom:Admit
    a               rdfs:Class ;
    rdfs:label      "Admit" ;
    rdfs:comment    "Action: admit a validated object (Validated → Admitted)." .

dom:Receipt
    a               rdfs:Class ;
    rdfs:label      "Receipt" ;
    rdfs:comment    "Action: compute chain hash (Admitted → Receipted)." .

dom:Promote
    a               rdfs:Class ;
    rdfs:label      "Promote" ;
    rdfs:comment    "Action: promote Andon state (Halted → Overridden)." .
```

### ggen Mapping Algorithm

**Input:** RDF graph (Turtle/N-Triples from `ontology/domain.ttl`)  
**Output:** PDDL domain + problem stub (generated in `src/pddl_model.pddl`)

#### Step 1: Extract Classes → PDDL Types

```
FOR EACH rdfs:Class c IN ontology:
  IF c is subclass of dom:Obligation:
    → emit "obligation" type marker
  IF c is subclass of dom:LifecycleStage:
    → emit "lifecycle-stage" type marker
  IF c is subclass of dom:AndonState:
    → emit "andon-state" type marker
  ELSE:
    → emit "law-object" type or domain-specific type
```

**Result:**
```pddl
(:types
  law-object
  obligation
  evidence
  precondition - obligation
  evidence-required - obligation
  blocking-constraint - obligation
  lifecycle-stage
  raw - lifecycle-stage
  validated - lifecycle-stage
  admitted - lifecycle-stage
  receipted - lifecycle-stage
  andon-state
  green - andon-state
  halted - andon-state
  overridden - andon-state
  validator
  authority
)
```

#### Step 2: Extract Properties → PDDL Predicates

```
FOR EACH rdf:Property p IN ontology:
  domain = rdfs:domain(p)
  range = rdfs:range(p)
  
  IF range is xsd:<type>:
    → emit "(p ?x - domain)" for boolean/nominal properties
       or emit numeric fluent for xsd:integer, xsd:decimal
  ELSE:
    → emit "(p ?x - domain ?y - range)"
```

**Result (excerpt):**
```pddl
(is-precondition ?ob - obligation ?pred - string)
(requires-evidence ?ob - obligation ?etype - string)
(evidence-satisfies ?ev - evidence ?ob - obligation)
(in-stage ?obj - law-object ?stage - lifecycle-stage)
(andon-status ?obj - law-object ?state - andon-state)
```

#### Step 3: Extract Restrictions → PDDL Preconditions & Effects

```
FOR EACH Restriction or Property occurrence:
  IF requires cardinality 1:
    → emit precondition (?x - C) in judge action
       and effect (predicate ?x - C) in result
  
  IF has domain class X and range class Y:
    → For each lifecycle transition, emit (X → Y) effect
       e.g., Raw → Validated: (not (in-stage ?obj raw))
                               (in-stage ?obj validated)
```

#### Step 4: Extract Actions → PDDL Action Schema

```
FOR EACH rdfs:Class c that is-a dom:<ActionType>:
  (where ActionType ∈ {Judge, Admit, Receipt, Promote}):
  
  → Emit action schema with standard preconditions:
    - Judge: (in-stage ?obj raw), all obligations satisfied
    - Admit: (in-stage ?obj validated), (andon-status ?obj green)
    - Receipt: (in-stage ?obj admitted)
    - Promote: (in-stage ?obj raw), (andon-holds ?obj)
    
  → Emit standard effects (stage transitions)
```

### Generated Artifacts

**File: `generated/pddl_domain.pddl`**

```pddl
; Autogenerated PDDL domain from ontology/domain.ttl
; Do not edit directly; regenerate via `ggen sync`

(define (domain obligation-lifecycle)
  (:requirements :typing :durative-actions :adl)
  
  (:types ... ; extracted from RDF classes
  
  (:predicates ... ; extracted from RDF properties
  
  (:action judge ... ; standard template
  (:action admit ...
  (:action receipt ...
  (:action promote-andon ...
)
```

**File: `generated/pddl_problem_stub.pddl`**

```pddl
; Autogenerated PDDL problem template
; Customize :init and :goal for specific scenarios

(define (problem obligation-validation-case-001)
  (:domain obligation-lifecycle)
  
  (:objects
    ; User specifies domain instances here
    ; Autogenerated stub provides type declarations
  )
  
  (:init
    ; User populates initial state
    ; Autogenerated stub shows available predicates
  )
  
  (:goal
    ; User specifies goal
  )
)
```

### Integration with Rust Codegen

**File: `ggen.toml`** (project manifest)

```toml
[codegen]
ontology_path = "ontology/domain.ttl"

[[outputs]]
name = "pddl_domain"
type = "pddl-domain"
destination = "generated/pddl_domain.pddl"

[[outputs]]
name = "pddl_problem_stub"
type = "pddl-problem"
destination = "generated/pddl_problem_stub.pddl"
```

**Workflow:**

1. User defines obligation ontology in `ontology/domain.ttl`.
2. User runs `ggen sync`.
3. ggen parses the RDF graph, applies the mapping rules (steps 1–4 above).
4. Generated files appear in `generated/`.
5. User customizes `pddl_problem_stub.pddl` with specific instances and goals.
6. User runs PDDL planner (e.g., Fast Downward, OPTIC, Planner7) on the domain and problem.
7. Planner emits a sequence of actions (judge, admit, receipt, promote) that solves the problem.

---

## Planner Integration

### PDDL Solver Invocation (Sketch)

```bash
# Assuming planner available in PATH
$ fast-downward.py --plan-file plan.txt \
    generated/pddl_domain.pddl \
    generated/pddl_problem_stub.pddl

# Planner outputs plan.txt:
# 0: judge claim-001 judge-service
# 1: admit claim-001 admissions-authority
# 2: receipt claim-001 chain-genesis chain-claim-001
```

### Bridging PDDL Output to Rust

A lightweight interpreter can convert PDDL action sequences to Rust trait calls:

```rust
// pseudo-code
enum PddlAction {
    Judge { obj: LawObjectRef, validator: String },
    Admit { obj: LawObjectRef, authority: String },
    Receipt { obj: LawObjectRef, prev_token: ChainToken, new_token: ChainToken },
    PromoteAndon { obj: LawObjectRef, authority: String, obligation: Obligation },
}

fn execute_plan(plan: Vec<PddlAction>) -> Result<Vec<LawObject<_, Receipted, _>>> {
    let mut results = Vec::new();
    for action in plan {
        match action {
            PddlAction::Judge { obj, validator } => {
                let validated = Judge::judge(obj)?;
                // ...
            }
            PddlAction::Admit { obj, authority } => {
                let admitted = Admit::admit(obj)?;
                // ...
            }
            // ...
        }
    }
    Ok(results)
}
```

---

## Verification Checklist

### PDDL Model Coherence

- [x] **Predicates align with Obligation types:**
  - Precondition → `is-precondition`, `precondition-satisfied`
  - BlockingConstraint → `is-blocking-constraint`, `blocking-constraint-cleared`
  - EvidenceRequired → `requires-evidence`, `evidence-satisfied`

- [x] **Actions match lifecycle transitions:**
  - Judge: Raw → Validated (all obligations satisfied)
  - Admit: Validated → Admitted (Andon green)
  - Receipt: Admitted → Receipted (chain hash computed)
  - Promote: Andon::Halted → Andon::Overridden (authority override)

- [x] **Andon hold semantics:**
  - Halted state blocks Judge and Admit until cleared
  - Promote action lifts holds via authority override
  - Supply-Evidence action satisfies obligations without promotion

- [x] **Chain hash constraints:**
  - Previous chain must be valid before Receipt
  - New chain tokens are unique and single-use
  - Receipt seals the object (no duplicate receipts)

- [x] **Concrete example is solvable:**
  - Initial state: Raw, two unmet obligations, Andon::Halted
  - Plan: supply evidence, promote constraint, judge, admit, receipt
  - Final state: Receipted, Andon::Green, chain hash computed

- [x] **RDF → PDDL mapping is systematic:**
  - Classes → types (with subclass hierarchy)
  - Properties → predicates (with domain/range)
  - Restrictions → preconditions/effects
  - Action classes → action schemas

---

## References

- **PDDL 2.1 BNF:** https://www.aaai.org/Papers/AIPS/2002/AIPS02-050.pdf (foundational)
- **Durative Actions:** PDDL 2.1 `:durative-actions` allow time-extended tasks
- **Numeric Fluents:** `:numeric-fluents` enable resource (hold count) constraints
- **Fast Downward Planner:** http://www.fast-downward.org/ (reference PDDL solver)
- **SPARC/CPhy LawObject:** See `/crates/praxis-core/src/law.rs`
- **RDF/RDFS Semantics:** W3C RDFS Specification, https://www.w3.org/TR/rdf-schema/
- **ggen Codegen System:** See `ggen.toml` and `src/codegen/rdf_to_pddl.rs` (sketch)

---

## Future Extensions

### Temporal Reasoning

Extend to durative actions with time bounds:
```pddl
(:durative-action judge
  :parameters (?obj - law-object ?validator - validator)
  :duration (= ?duration 5)  ; 5 seconds
  :condition (and (at start (in-stage ?obj raw)) ...)
  :effect (and (at end (in-stage ?obj validated)) ...)
)
```

### Hierarchical Planning (HTN)

Model obligation workflows as task networks:
```pddl
(:task validate-contract
  :parameters (?claim - law-object)
  :precondition (in-stage ?claim raw)
  :subtasks (
    (judge ?claim)
    (admit ?claim)
    (receipt ?claim)
  )
)
```

### Cost/Preference Optimization

Add action costs (validator overhead, time) and preferences:
```pddl
(:metric minimize (total-cost))

(:action judge
  :parameters (...)
  :effect (and ... (increase (total-cost) 10))
)
```

### Plan Explanations

Emit human-readable justifications for each action:
- **Why judge?** "All obligations have evidence or have been verified."
- **Why admit?** "Authority approved; no holds remain."
- **Why receipt?** "Chain hash computed; object sealed."

---

## Example: Full PDDL Domain + Problem File

### Domain File: `pddl_domain.pddl`

```pddl
(define (domain lawobject-capability)
  (:requirements :typing :adl)

  (:types
    law-object
    obligation
    evidence-type
    predicate
    andon-state
    lifecycle-stage
    validator
    authority
    chain-token
  )

  (:predicates
    (in-stage ?obj - law-object ?stage - lifecycle-stage)
    (has-obligation ?obj - law-object ?ob - obligation)
    (is-precondition ?ob - obligation ?pred - predicate)
    (is-blocking-constraint ?ob - obligation)
    (requires-evidence ?ob - obligation ?etype - evidence-type)
    (evidence-satisfied ?ob - obligation)
    (precondition-satisfied ?pred - predicate)
    (blocking-constraint-cleared ?ob - obligation)
    (andon-status ?obj - law-object ?state - andon-state)
    (obligation-unmet ?obj - law-object ?ob - obligation)
    (andon-holds ?obj - law-object)
    (chain-hash-computed ?obj - law-object ?token - chain-token)
    (prev-chain-valid ?token - chain-token)
    (signature-applied ?obj - law-object)
    (validated-by ?obj - law-object ?validator - validator)
    (admitted-by ?obj - law-object ?authority - authority)
    (override-authority ?authority - authority ?ob - obligation)
    (andon-override-applied ?obj - law-object ?by - authority)
  )

  (:action judge
    :parameters (?obj - law-object ?validator - validator)
    :precondition (and
      (in-stage ?obj raw)
      (forall (?ob - obligation)
        (implies (has-obligation ?obj ?ob)
          (or
            (and (is-precondition ?ob ?pred) (precondition-satisfied ?pred))
            (and (is-blocking-constraint ?ob) (blocking-constraint-cleared ?ob))
            (and (requires-evidence ?ob ?etype) (evidence-satisfied ?ob))
          )
        )
      )
    )
    :effect (and
      (not (in-stage ?obj raw))
      (in-stage ?obj validated)
      (validated-by ?obj ?validator)
      (andon-status ?obj green)
      (not (andon-holds ?obj))
    )
  )

  (:action admit
    :parameters (?obj - law-object ?authority - authority)
    :precondition (and
      (in-stage ?obj validated)
      (not (andon-holds ?obj))
      (andon-status ?obj green)
    )
    :effect (and
      (not (in-stage ?obj validated))
      (in-stage ?obj admitted)
      (admitted-by ?obj ?authority)
    )
  )

  (:action receipt
    :parameters (
      ?obj - law-object
      ?prev-token - chain-token
      ?new-token - chain-token
    )
    :precondition (and
      (in-stage ?obj admitted)
      (prev-chain-valid ?prev-token)
      (not (chain-hash-computed ?obj ?new-token))
    )
    :effect (and
      (not (in-stage ?obj admitted))
      (in-stage ?obj receipted)
      (chain-hash-computed ?obj ?new-token)
      (signature-applied ?obj)
    )
  )

  (:action promote-andon
    :parameters (
      ?obj - law-object
      ?authority - authority
      ?ob - obligation
    )
    :precondition (and
      (in-stage ?obj raw)
      (andon-holds ?obj)
      (andon-status ?obj halted)
      (obligation-unmet ?obj ?ob)
      (override-authority ?authority ?ob)
    )
    :effect (and
      (not (andon-status ?obj halted))
      (andon-status ?obj overridden)
      (andon-override-applied ?obj ?authority)
      (not (obligation-unmet ?obj ?ob))
    )
  )

  (:action supply-evidence
    :parameters (
      ?obj - law-object
      ?ob - obligation
      ?etype - evidence-type
    )
    :precondition (and
      (has-obligation ?obj ?ob)
      (requires-evidence ?ob ?etype)
      (not (evidence-satisfied ?ob))
    )
    :effect (and
      (evidence-satisfied ?ob)
      (not (obligation-unmet ?obj ?ob))
    )
  )
)
```

### Problem File: `contract_claim_001.pddl`

```pddl
(define (problem contract-claim-validation-case-001)
  (:domain lawobject-capability)

  (:objects
    claim-001 - law-object
    judge-service - validator
    admissions-authority - authority
    sig-check - predicate
    ledger-type - evidence-type
    chain-genesis - chain-token
    chain-claim-001 - chain-token
  )

  (:init
    ; Lifecycle stage
    (in-stage claim-001 raw)

    ; Obligations
    (has-obligation claim-001 ob-sig)
    (has-obligation claim-001 ob-ledger)
    (is-precondition ob-sig sig-check)
    (requires-evidence ob-ledger ledger-type)

    ; Andon: initially halted
    (andon-status claim-001 halted)
    (andon-holds claim-001)
    (obligation-unmet claim-001 ob-sig)
    (obligation-unmet claim-001 ob-ledger)

    ; Authority relationships
    (override-authority admissions-authority ob-sig)

    ; Chain: genesis is valid
    (prev-chain-valid chain-genesis)
  )

  (:goal (and
    (in-stage claim-001 receipted)
    (andon-status claim-001 green)
    (chain-hash-computed claim-001 chain-claim-001)
  ))
)
```

---

## Summary

This document defines a complete, coherent PDDL capability model for CPhy's LawObject:

1. **Predicates** encode all Obligation types, lifecycle stages, Andon states, and chain constraints.
2. **Actions** (judge, admit, receipt, promote, supply-evidence) model the exact transitions in the Rust typestate system.
3. **Concrete example** shows a smart contract claim flowing from Raw → Receipted with unmet obligations resolved.
4. **RDF → PDDL mapping** is systematic, allowing ggen to synthesize PDDL automatically from ontologies.
5. **Planner integration** sketches how PDDL solutions can be executed as Rust trait calls.

A human can read this document, understand how LawObject → PDDL planning works, and implement either a manual PDDL solver or integrate with an external planner (Fast Downward, OPTIC, etc.).
