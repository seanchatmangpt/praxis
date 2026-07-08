# Graphlaw Semantic Profile Doctrine

**Version**: v26.7.8  
**Status**: STANDING  
**Authority**: Architecture Decision

---

## Core Principle

**Graphlaw handles the daily 80% of semantic dialects inside a small deterministic standing core and refuses the heavy 20% by name.**

Graphlaw is **not** a universal semantic-web engine. It is a **standing engine with bounded semantic profiles**.

---

## Architecture

```
Small core (fast, deterministic, auditable)
↓
Daily-use 80% of each dialect (implemented)
↓
Heavy 20% of each dialect (refused explicitly)
↑
Boundary requests (out-of-band heavy profiles)
```

---

## The Doctrine

> Graphlaw implements bounded daily-use profiles of semantic dialects. The system supports the common 80% of OWL RL, SHACL, ShEx, N3, and Datalog needed for ordinary ontology organization, validation, derivation, denial, hook eligibility, and standing manufacture. The remaining 20% is not silently supported. It is refused, marked UNSUPPORTED, or routed to a heavy external profile outside the hot path.

---

## Daily-Use Profiles (80%)

| Dialect | Core Profile | Features |
|---------|---|---|
| **OWL RL** | GL-Daily-OWL-RL-v0 | subclass, subproperty, domain, range, equivalent class/property, inverse, symmetric, bounded transitive |
| **SHACL** | GL-Daily-SHACL-Core-v0 | NodeShape, PropertyShape, class, datatype, min/max count, node kind, pattern, target class/node/subjects/objects |
| **ShEx** | GL-Daily-ShEx-v0 | basic shape maps, node constraints, property constraints, cardinality, datatype, closed shape subset |
| **N3** | GL-Daily-N3-v0 | bounded forward rules, denials, simple built-ins, proof/denial traces |
| **Datalog** | GL-Daily-Datalog-v0 | stratified positive/negation-safe rules, semi-naive materialization, bounded rule counts |
| **Hooks** | GL-Daily-Hooks-v0 | deterministic triggers, graph deltas, diagnostics, boundary requests, receipts, replay |

---

## Heavy Features (20% — Refused)

| Dialect | Unsupported Features | Reason |
|---------|---|---|
| **OWL RL** | Unrestricted `owl:sameAs`, complex OWL DL restrictions, property chains, cardinality reasoning | Equivalence closure explosion; outside hot-path bounds |
| **SHACL** | SHACL-SPARQL, remote validation, arbitrary JS/functions, advanced recursive shapes (unless profiled) | SPARQL execution outside deterministic boundary |
| **ShEx** | Full recursive/semantic-action-heavy ShEx, external functions | Unbounded recursion incompatible with receipt stability |
| **N3** | Unbounded built-ins, network/file/shell built-ins, complex backward search | Side effects not allowed in semantic derivation |
| **Datalog** | Unstratified recursion, unsafe negation, unbounded programs | Receipt stability requires stratification |
| **Hooks** | Direct shell/network actuation, unreceipted side effects, unsupported dialect calls | All actuation must be boundary-routed with receipts |

---

## Error Model: Explicit Refusal

**Rule**: Unsupported features do **not** silently promote. They produce typed, doctrinal refusals.

### Example: SHACL-SPARQL

```
ERROR: UNSUPPORTED_DIALECT_FEATURE

feature: sh:sparql
profile: GL-Daily-SHACL-Core-v0
reason: SHACL-SPARQL is outside the bounded hot-path profile.
supported_alternative: use SHACL Core constraints or route through an explicit heavy-profile adapter.
standing: NOT_ADMITTED
```

### Example: OWL Equivalence Explosion

```
ERROR: UNSUPPORTED_DIALECT_FEATURE

feature: unrestricted owl:sameAs
profile: GL-Daily-OWL-RL-v0
reason: unrestricted sameAs closure can cause equivalence explosion and is outside the bounded daily profile.
supported_alternative: declare profile-gated equivalence or use explicit owl:equivalentClass / owl:equivalentProperty.
standing: NOT_ADMITTED
```

### Example: N3 Network Built-in

```
ERROR: UNSUPPORTED_DIALECT_FEATURE

feature: network built-in
profile: GL-Daily-N3-v0
reason: network/file/shell effects are not allowed inside Graphlaw semantic derivation.
supported_alternative: emit a boundary request artifact.
standing: REFUSED
```

---

## Implementation Rule

```
If feature in daily-use 80%:
    ✅ implement in Graphlaw core
    ✅ benchmark for hot-path
    ✅ receipt/replay test
    ✅ positive + negative fixtures
    ✅ document in profile spec

If feature in heavy 20%:
    ❌ do not silently approximate
    ❌ do not ignore
    ✅ refuse by name
    ✅ explain supported alternative
    ✅ optionally emit heavy-profile boundary request
    ✅ mark standing as UNSUPPORTED_BY_DESIGN
```

---

## Why 80/20 > Full Support

**Full support trap**:
```
more features → larger dependency graph → slower builds
→ bigger binary → harder replay → fuzzier standing
→ more hidden trust → weaker position
```

**80/20 strategy**:
```
fewer features → explicit boundary → fast hot path
→ small core → stable receipts → clean refusal surface
→ better standing → stronger position
```

---

## Standing Ledger

```
SEMANTIC_PROFILE_STRATEGY = EIGHTY_TWENTY
FULL_SEMANTIC_WEB_ENGINE = REFUSED
DAILY_DIALECT_CORE = SUPPORTED
HEAVY_DIALECT_FEATURES = UNSUPPORTED_BY_DEFAULT
HEAVY_PROFILE_ADAPTER = OPTIONAL_BOUNDARY_REQUEST
SILENT_APPROXIMATION = REFUSED
PROFILE_REFUSAL_ERROR = TYPED_EXPLICIT
RECEIPT_STABILITY = REQUIRED
DETERMINISTIC_EVALUATION = REQUIRED
```

---

## Relation to Invariants

This doctrine reinforces praxis CLAUDE.md invariants:

1. **No panics/silent defaults** ← Explicit refusal instead of silent approximation
2. **Receipts computed, never asserted** ← Deterministic evaluation required
3. **No wall clock in hash/receipt** ← Deterministic evaluation required
4. **Closed vocabularies refused by name** ← Profile boundary enforced
5. **Smallest diff, reuse first** ← Daily profiles prioritize reuse over completeness
6. **Dependency freeze** ← Heavy profiles out-of-band, not in core

---

## References

- **PROJ-501**: OWL RL daily profile (reasonable audit)
- **PROJ-503**: SHACL daily profile (shacl ecosystem audit)
- **PROJ-502**: ShEx daily profile (rudof audit)
- **PROJ-504**: N3 daily profile (N3 audit)
- **PROJ-401**: Quick-win optimizations (IRI interning, IR compilation, deterministic ordering)
- **PROJ-505**: horned-owl (study patterns, refuse full OWL 2 DL)

---

## The Deep Point

You do not win by saying:

> "Graphlaw supports every OWL, SHACL, ShEx, N3, and SPARQL feature."

You win by saying:

> "Graphlaw supports the bounded daily subset that produces standing, and refuses everything else clearly."

That is better engineering and better doctrine.
