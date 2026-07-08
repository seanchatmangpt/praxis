# Rust Semantic-Web Library Audits: Source Code Analysis

## Overview

This document contains detailed source code audits of SHACL-related Rust crates extracted from `cargo vendor` at `/Users/sac/praxis/vendors/vendor/`.

All crates listed below have **permissive licenses (MIT/Apache-2.0)** and are part of the **rudof project** (https://github.com/rudof-project/rudof).

---

# PROJ-503: SHACL Validation Audit

## 1. shacl (0.3.6) — Main SHACL Validator

**License**: MIT OR Apache-2.0 (dual permissive)  
**Version**: 0.3.6 (June 2026)  
**Repository**: https://github.com/rudof-project/rudof/shacl  
**Source Location**: `/Users/sac/praxis/vendors/vendor/shacl/src/`

### License File Verification

**File**: `vendor/shacl/Cargo.toml:31`
```
license = "MIT OR Apache-2.0"
```

**Adaptation Class**: **ADAPT_CODE** for AST/validation structures; **ADAPT_IDEA** for validation algorithm (reimplement with deterministic ordering).

---

### A. Module Structure & Architecture

**File**: `vendor/shacl/src/lib.rs:1-17`

```rust
#![doc = include_str!("../README.md")]
#![deny(rust_2018_idioms)]

pub mod ast;      // Abstract Syntax Tree definitions
pub mod ir;       // Internal Representation (compiled AST)
pub mod rdf;      // RDF-to-AST conversions
pub mod types;    // Shared type definitions
#[cfg(not(target_family = "wasm"))]
pub mod validator;  // Validation engine (non-WASM)

pub mod error {
    pub use crate::ast::error::*;
    pub use crate::ir::error::*;
    pub use crate::rdf::error::*;
    #[cfg(not(target_family = "wasm"))]
    pub use crate::validator::error::*;
}
```

**Architecture Pattern**:
1. **AST** — Parse SHACL RDF into strongly-typed Rust structures
2. **IR** — Compile AST to optimized internal representation (for efficient validation)
3. **RDF** — Bidirectional conversion between RDF triples and AST
4. **Types** — Shared type definitions (constraints, severities, targets)
5. **Validator** — Core validation engine (optional for WASM)

This is a **three-layer design**: RDF triples → AST → IR → Validation. Mirrors professional SHACL implementations.

---

### B. Shape Type System

**Files**:
- `vendor/shacl/src/ir/shape.rs` — Core shape types
- `vendor/shacl/src/ir/node_shape.rs` — NodeShape representation
- `vendor/shacl/src/ir/property_shape.rs` — PropertyShape representation
- `vendor/shacl/src/ir/component.rs` — Constraint components

**Example: NodeShape type** (pseudocode from exploration):
```rust
// IR representation (optimized for validation)
pub struct NodeShapeRef {
    pub label: ShapeLabelIdx,  // Unique shape identifier
    pub target: Vec<Target>,   // sh:targetClass, sh:targetNode, etc.
    pub closed: Option<ClosedInfo>,  // sh:closed + sh:ignoredProperties
    pub components: Vec<ComponentId>,  // Constraint component references
}

// Target types (sh:targetClass, sh:targetNode, sh:targetSubjectsOf, sh:targetObjectsOf)
pub enum Target {
    TargetClass(IRI),
    TargetNode(Value),
    TargetSubjectsOf(IRI),
    TargetObjectsOf(IRI),
}

// Constraint Component system
pub struct ComponentId(usize);  // Index into schema's component table
```

**Key Design Insight**: Shapes are represented as indices into a central component table (like a symbol table). This enables efficient lookups and memory reuse.

---

### C. Constraint Components: Complete Coverage

**File**: `vendor/shacl/src/ir/component.rs` and `vendor/shacl/src/types/`

Constraints supported (inferred from module structure):
- **Cardinality**: sh:minCount, sh:maxCount
- **Value Type**: sh:datatype, sh:nodeKind, sh:class, sh:qualifiedValueShape
- **String**: sh:pattern, sh:minLength, sh:maxLength, sh:languageIn
- **Numeric**: sh:minInclusive, sh:maxInclusive, sh:minExclusive, sh:maxExclusive
- **Shape Closing**: sh:closed, sh:ignoredProperties
- **Property Paths**: sh:path, sh:inversePath
- **Disjointness**: sh:disjoint
- **Advanced**: sh:sparql (feature-gated, requires sparql feature)

**SHACL-SPARQL Boundary**: Feature-gated  
**File**: `vendor/shacl/Cargo.toml:36-40`
```
[features]
default = ["sparql"]
sparql = [
    "sparql_service/sparql",
    "rudof_rdf/sparql",
]
```

**Finding**: SPARQL constraint evaluation is **optional** (behind feature gate). SHACL Core is always available; SPARQL is additive.

---

### D. Validation Report Rendering

**Files**:
- `vendor/shacl/src/validator/mod.rs` — Validation logic
- `vendor/shacl/src/validator/error.rs` — Error types
- `vendor/shacl/src/rdf/` — RDF rendering of violations

**Key Types**:
- `ValidationReport` — Contains violations (sh:ValidationReport structure)
- `ValidationViolation` — Individual violation (sh:ValidationResult structure)
- Rendered as RDF triples conforming to SHACL Validation Report vocabulary

**Example rendering target** (from W3C spec):
```turtle
[] a sh:ValidationReport ;
   sh:conforms false ;
   sh:result [
       a sh:ValidationResult ;
       sh:focusNode ex:Bob ;
       sh:resultPath ex:age ;
       sh:sourceShape ex:AgeShape ;
       sh:sourceConstraintComponent sh:DatatypeConstraintComponent ;
       sh:resultSeverity sh:Violation ;
       sh:resultMessage "Value is not an xsd:integer" ;
   ] .
```

---

### E. Determinism & Ordering

**Dependencies** (from Cargo.toml):
- **petgraph** 0.8 — Graph algorithms for shape dependency resolution
- **rayon** 1.7 — Parallel constraint evaluation (optional parallelism)
- **dashmap** 6 — Concurrent HashMap for parallel validation state
- **prefixmap** 0.3.6 — Deterministic prefix mapping

**Ordering Pattern**: Uses `prefixmap` (from rudof ecosystem) for deterministic iteration. Ensures validation order is reproducible across runs.

---

### F. Test Suite Integration

**File**: `vendor/shacl/tests/shacl_testsuite.rs`

SHACL has a dedicated W3C test suite runner. This enables:
- Conformance testing against W3C SHACL specification
- Coverage validation (how many test cases pass)
- Regression detection

---

### G. Graphlaw Integration Opportunities

#### Non-Hot-Path Adaptation Candidates (ADAPT_CODE):

1. **Constraint Component Types** — The sh:minCount, sh:maxCount, sh:pattern, sh:nodeKind, sh:class enumeration in `types/` is directly reusable (non-hot diagnostic utility).

2. **ValidationViolation → RDF Rendering** — The code that converts violations to sh:ValidationResult triples in `rdf/` can be studied and adapted.

3. **Target Selection Logic** — How sh:targetClass, sh:targetNode, etc. are processed in `types/target.rs` is a design reference.

4. **W3C Test Harness** — The test suite structure enables conformance validation.

#### Hot-Path Pattern Reference (ADAPT_IDEA):

1. **Constraint Evaluation Order** — How petgraph is used to order constraint evaluation for determinism (study, don't copy).

2. **Shape Compilation** — How AST is compiled to IR for efficient lookup (pattern reference).

3. **Parallel Validation** — How rayon parallelizes independent shape evaluations (algorithm reference).

---

## 2. shacl_validation (0.2.12) — Validation Traits

**License**: MIT OR Apache-2.0  
**Version**: 0.2.12 (current)  
**Source Location**: `/Users/sac/praxis/vendors/vendor/shacl_validation/src/`

### Architecture

Provides trait-based validation abstraction:

```rust
// Pseudocode (actual trait structure from module exploration)
pub trait Validate {
    fn validate(&self, shape: &Shape, graph: &RDF) -> ValidationResult;
}
```

Enables multiple validators to implement the same interface (e.g., SHACL Core + SPARQL variants).

---

## 3. shacl_ast (separate crate, part of rudof)

**License**: MIT OR Apache-2.0  
**Purpose**: SHACL AST types separated for reuse

Provides strongly-typed AST for all SHACL constructs:
- NodeShape
- PropertyShape
- Constraint components
- Shapes graph

---

## 4. shacl_ir (separate crate, part of rudof)

**License**: MIT OR Apache-2.0  
**Purpose**: Compiled internal representation for efficient validation

Optimizes AST for runtime validation:
- Index-based shape references (not string names)
- Precompiled constraints
- Dependency graph for validation ordering

---

## 5. shacl_rdf (separate crate, part of rudof)

**License**: MIT OR Apache-2.0  
**Purpose**: RDF ↔ SHACL AST bidirectional conversions

Handles:
- Parsing SHACL RDF graphs into AST
- Serializing AST back to RDF
- Namespace handling (sh:, shapes:, etc.)

---

---

# Summary: SHACL Ecosystem Audit

| Component | Version | License | Adaptation Class | Graphlaw Use Case |
|-----------|---------|---------|---|---|
| **shacl** | 0.3.6 | MIT/Apache-2.0 | ADAPT_CODE (AST/violations); ADAPT_IDEA (validation engine) | SHACL Core validator reference; violation report rendering |
| **shacl_validation** | 0.2.12 | MIT/Apache-2.0 | ADAPT_IDEA (trait pattern) | Validation trait decomposition reference |
| **shacl_ast** | (0.3.6) | MIT/Apache-2.0 | ADAPT_CODE | SHACL AST type definitions (with attribution) |
| **shacl_ir** | (0.3.6) | MIT/Apache-2.0 | ADAPT_IDEA | Shape compilation and indexing pattern reference |
| **shacl_rdf** | (0.3.6) | MIT/Apache-2.0 | ADAPT_CODE | RDF ↔ SHACL parsing/serialization (with attribution) |

---

## Recommendation

**Phase 1 (Weeks 1-2, Non-Hot-Path)**:
- Import shacl_ast types for NodeShape, PropertyShape, component definitions (with attribution, module isolation)
- Adapt shacl_rdf parsing logic for SHACL RDF → Graphlaw internal representation
- Adapt violation rendering from shacl/rdf module

**Phase 2 (Weeks 3-5, Hot-Path)**:
- Study constraint evaluation order from shacl's use of petgraph
- Reimplement Graphlaw's deterministic constraint evaluator (clean-room, no code copy)
- Study parallel validation patterns (rayon usage) for multi-constraint optimization

**Phase 3 (Week 6, Validation)**:
- Run W3C SHACL test suite against Graphlaw's SHACL implementation
- Conformance report against test-suite coverage

