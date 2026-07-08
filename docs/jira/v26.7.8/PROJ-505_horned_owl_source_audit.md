# PROJ-505 Audit: horned-owl (LGPL-3.0) — Detailed Source Code Analysis

**Crate**: horned-owl  
**Version**: 2.0.0 (2024 edition, Rust 1.88+)  
**License**: LGPL-3.0 (copyleft, study patterns only — CLEAN_ROOM_REIMPLEMENT)  
**Repository**: https://github.com/phillord/horned-owl  
**Source location**: `/Users/sac/praxis/vendors/horned-owl/`

---

## A. License & Maturity Assessment

### License: LGPL-3.0
File: `horned-owl/Cargo.toml:13`
```
license = "LGPL-3.0"
```
**Implication**: Copyleft license. Any code copied from horned-owl into Graphlaw creates a derivative work that must be LGPL-3.0 licensed. **Adaptation strategy: ADAPT_IDEA only. Study the AST design, reimplement clean-room without code copy.**

### Version & Maintenance
- **Version**: 2.0.0 (recent, 2024 edition)
- **Edition**: 2024 (latest, uses modern Rust features)
- **Rust Version**: 1.88+ required
- **Status**: Actively maintained; author Phillip Lord continues work
- **Dependencies**: Uses oxigraph ecosystem (oxrdf, oxrdfio, oxiri) — well-connected to production Rust RDF stack

### Maturity Indicators
- **Test coverage**: Has dev-dependencies on proptest (property testing), rstest (parametrized tests), pretty_assertions
- **Benchmarks**: Has criterion benchmarks at `/Users/sac/praxis/vendors/horned-owl/benches/horned.rs`
- **Production usage**: Dependencies on pretty_rdf (0.12.0), oxrdf (0.3.0) — known stable ecosystem crates

---

## B. OWL AST Design — Entity & Axiom Representation

### B.1 IRI Representation: Generic, Cached, Memory-Efficient

**File**: `vendors/horned-owl/src/model.rs:107-173`

```rust
// Line 107-120: IRI<A> is a generic newtype wrapper
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct IRI<A>(pub(crate) A);

// Lines 124-162: ForIRI trait bounds the string type
pub trait ForIRI:
    AsRef<str>
    + Borrow<str>
    + Clone
    + Debug
    + Deref<Target = str>
    + Eq
    + From<String>
    + Hash
    + PartialEq
    + Ord
    + PartialOrd
{
    fn from_str(s: &str) -> Self;
}

// Lines 165-166: Type aliases for common implementations
pub type RcStr = Rc<str>;
pub type ArcStr = Arc<str>;
```

**Pattern**: IRI<A> is generic over any type A satisfying ForIRI bounds (AsRef, Borrow, Clone, Hash, Ord, etc.). Implementations use Rc<str> (single-threaded, memory-sharing) or Arc<str> (thread-safe).

**Graphlaw Integration**: horned-owl's IRI design can be studied for inspiration, but reimplnmented in Graphlaw to avoid LGPL implications. Key idea: generics + memory-efficient string sharing.

### B.2 Build: Factory Pattern with String Interning via BTreeSet

**File**: `vendors/horned-owl/src/model.rs:270-352`

```rust
// Lines 278-284: Build holds three BTreeSet instances for IRI/anon caching
#[derive(Debug, Default)]
pub struct Build<A: ForIRI>(
    RefCell<BTreeSet<IRI<A>>>,        // IRI cache
    RefCell<BTreeSet<AnonymousIndividual<A>>>,  // Anon individual cache
    RefCell<i64>,                      // Anon counter
);

// Lines 343-352: iri() method caches strings in BTreeSet
pub fn iri<S: Borrow<str>>(&self, s: S) -> IRI<A> {
    let mut cache = self.0.borrow_mut();
    if let Some(iri) = cache.get(s.borrow()) {
        iri.clone()      // Reuse cached IRI
    } else {
        let iri = IRI(A::from_str(s.borrow()));
        cache.insert(iri.clone());
        iri
    }
}
```

**Pattern**: Build uses mutable caching with RefCell to interning IRIs. Each unique IRI string is created once; subsequent lookups return a clone of the cached instance. Uses BTreeSet (ordered, deterministic iteration).

**Graphlaw Integration**: This Build pattern can be adapted as a design reference: string deduplication via cache + ordered collection (BTreeSet) ensures determinism.

### B.3 Component Axiom Enum: Macro-Generated, Complete OWL 2 DL Coverage

**File**: `vendors/horned-owl/src/model.rs:1220-1500+` (macro-generated, ~200+ axiom types)

The Component enum is generated via the `components!` macro. Axiom types include:

```rust
// Meta components (Ontology metadata)
Meta OntologyID{iri: Option<IRI<A>>, viri: Option<IRI<A>>},
Meta DocIRI(IRI<A>),

// Declaration axioms
Axiom DeclareClass(Class<A>),
Axiom DeclareObjectProperty(ObjectProperty<A>),
Axiom DeclareAnnotationProperty(AnnotationProperty<A>),
Axiom DeclareDataProperty(DataProperty<A>),
Axiom DeclareNamedIndividual(NamedIndividual<A>),
Axiom DeclareDatatype(Datatype<A>),

// Class axioms
Axiom SubClassOf{ sup: ClassExpression<A>, sub: ClassExpression<A> },
Axiom EquivalentClasses(Vec<ClassExpression<A>>),
Axiom DisjointClasses(Vec<ClassExpression<A>>),
Axiom DisjointUnion(Class<A>, Vec<ClassExpression<A>>),

// ObjectProperty axioms
Axiom SubObjectPropertyOf{
    sup: ObjectPropertyExpression<A>,
    sub: SubObjectPropertyExpression<A>
},
Axiom EquivalentObjectProperties(Vec<ObjectPropertyExpression<A>>),
Axiom InverseObjectProperties(ObjectProperty<A>, ObjectProperty<A>),
Axiom ObjectPropertyDomain{ ope: ObjectPropertyExpression<A>, ce: ClassExpression<A> },
Axiom ObjectPropertyRange{ ope: ObjectPropertyExpression<A>, ce: ClassExpression<A> },

// ... plus ~180 more axiom types covering all of OWL 2 DL
```

**Coverage**: Complete OWL 2 DL specification. Each axiom is a struct/tuple with strongly-typed fields (no stringly-typed conditions).

**Graphlaw Integration**: Graphlaw's OWL RL v0 will only support a bounded subset (RL profile constraints). horned-owl demonstrates how to represent the full spectrum; Graphlaw can study the design and adapt the subset needed.

### B.4 Visitor Pattern: Type-Safe AST Traversal

**File**: `vendors/horned-owl/src/visitor/`

The crate provides a visitor trait for traversing axioms:

```rust
// Example from normalize.rs lines 41-56
pub struct Reanonymize<A: ForIRI> {
    count: usize,
    b: Build<A>,
}

impl<A: ForIRI> VisitMut<A> for Reanonymize<A> {
    fn visit_anonymous_individual(&mut self, ai: &mut AnonymousIndividual<A>) {
        self.count += 1;
        *ai = self.b.anon(format!("anon_{}", self.count))
    }
}
```

**Pattern**: VisitMut trait with method overrides for each axiom type. Enables reusable transformations (reanonymization, simplification, serialization).

---

## C. Axiom Normalization & Equivalence

**File**: `vendors/horned-owl/src/normalize.rs:1-40`

```rust
// Lines 15-19: Normalize performs simplify + reanonymize + sort
pub fn normalize<A: ForIRI>(o: Vec<AnnotatedComponent<A>>) -> Vec<AnnotatedComponent<A>> {
    let mut o = reanonymize(simplify(o));
    o.sort();  // Deterministic ordering
    o
}

// Lines 21-26: Equivalence checking via normalized comparison
pub fn normalize_and_compare<A: ForIRI>(
    o1: Vec<AnnotatedComponent<A>>,
    o2: Vec<AnnotatedComponent<A>>,
) -> bool {
    normalize(o1).eq(&normalize(o2))  // Canonical form comparison
}

// Lines 59-65: Reanonymize uses VisitMut to walk axioms
pub fn reanonymize<A: ForIRI>(mut o: Vec<AnnotatedComponent<A>>) -> Vec<AnnotatedComponent<A>> {
    let mut walk: WalkMut<A, _> = WalkMut::new(Reanonymize::new(Build::new()));
    walk.ontology_vec(&mut o);
    o
}
```

**Pattern**: Equivalence-preserving transformations:
1. **simplify()** — Remove metadata (DocIRI components)
2. **reanonymize()** — Standardize anonymous individual names to predictable sequence
3. **sort()** — Deterministic ordering via Ord implementation

**Graphlaw Integration**: The normalize pattern is generalizable. For OWL RL v0, Graphlaw would:
- Sort axioms deterministically (for receipt stability)
- Canonicalize variable names (for replay determinism)
- Implement via sorted N-Quads + BLAKE3 (instead of Component sorting)

---

## D. Data Structures for Memory Efficiency

### D.1 Use of Rc<str> and Arc<str> for String Sharing

**File**: `vendors/horned-owl/Cargo.toml:24,28`
```
indexmap={workspace=true}  # Deterministic HashMap
oxrdf={workspace=true}     # RDF types from oxigraph
```

**File**: `vendors/horned-owl/src/model.rs:165-166`
```rust
pub type RcStr = Rc<str>;
pub type ArcStr = Arc<str>;
```

**Pattern**: Instead of cloning String for every entity, use Rc<str> (single-threaded) or Arc<str> (multi-threaded) to share the same string in memory. This reduces memory footprint when the same IRI appears in multiple axioms.

**Graphlaw Integration**: Symbol interning pattern (PROJ-401, quick-win crate optimization). Praxis could adopt a similar approach using crates like `lasso` or `string_cache` for IRI deduplication.

### D.2 BTreeSet for Deterministic Ordering

**File**: `vendors/horned-owl/src/model.rs:280-284`
```rust
RefCell<BTreeSet<IRI<A>>>,        // IRI cache with deterministic iteration
RefCell<BTreeSet<AnonymousIndividual<A>>>,  // Anon cache with deterministic iteration
```

**Pattern**: BTreeSet (ordered, deterministic) is used instead of HashSet. Iteration order is predictable and sorted lexicographically.

**Graphlaw Integration**: For receipt stability, Graphlaw must use ordered collections. Quick-win crate `indexmap` (already in horned-owl workspace) provides deterministic HashMap; `heapless::Vec<T>` or BTreeSet can be used for deterministic iteration.

---

## E. Parsing & I/O: Format Support

**File**: `vendors/horned-owl/src/io/`

Directory contents:
- `rdf.rs` — RDF/XML parser
- `manchesterowl.rs` — Manchester OWL syntax parser
- `xml.rs` — XML utilities
- `functional.rs` — OWL Functional syntax parser

**Formats supported**:
- RDF/XML (standard W3C format)
- Manchester OWL Syntax (human-readable)
- OWL Functional Syntax (structural)
- Turtle (via oxrdf)

**Graphlaw Integration**: If Graphlaw needs to load OWL ontologies, horned-owl's parsers can be studied (not code-copied due to LGPL) to understand parsing strategies. Note: parsing format is separate from reasoning — Graphlaw likely only needs to convert OWL to RDF triples + RL rules, not maintain horned-owl's full AST.

---

## F. Hot-Path vs. Non-Hot-Path Analysis

### Hot Path (would violate LGPL if copied):
- Axiom resolution and matching (`reasoner/`)
- Entailment checking (`reasoner/`)
- Axiom traversal with VisitMut

### Non-Hot-Path (low semantic risk, still LGPL):
- AST type definitions (Component enum fields)
- IRI/AnonymousIndividual normalization
- Serialization to RDF/XML/Manchester/Functional

### Graphlaw Strategy:
1. **Learn axiom representation design** from horned-owl's Component enum, but reimplement as RDF triples + rule compilation (OWL RL v0)
2. **Study IRI normalization** (normalize.rs), but apply to N-Quads + BLAKE3 receipt chain
3. **Do NOT import** horned-owl parsing, matching, or reasoning code due to LGPL — reimplement clean-room using OWL RL rule patterns from `reasonable` crate (which is BSD-3-Clause permissive)

---

## G. Recommendation & Integration Path

### Adaptation Class: **ADAPT_IDEA** (clean-room reimplementation)

### Why LGPL Prevents Code Adaptation:
- horned-owl is licensed LGPL-3.0, making it copyleft
- Static linking in Rust means any code copy requires entire derivative work to be LGPL-compatible
- Graphlaw appears to be permissive-licensed; importing horned-owl code would violate that

### What Graphlaw CAN Reuse (with LGPL compliance):
- **AST design patterns** — Entity type hierarchy, axiom representation, visitor pattern (study patterns, reimplement)
- **Normalization strategies** — Deterministic ordering, variable renaming (implement clean-room)
- **Format knowledge** — OWL Functional/Manchester syntax structure (implement custom parser if needed)

### What Graphlaw SHOULD NOT Reuse (code):
- Actual parsing code (rio, manchesterowl, functional modules)
- Axiom resolution/matching logic
- Reasoning engine code

### Timeline & Dependency:
- **Conditional**: Only if PROJ-501 (OWL RL audit of `reasonable` crate) determines that Graphlaw needs sophisticated OWL representation beyond RDF triples
- **If needed**: 2-3 week design exercise to build Graphlaw's own OWL RL v0 AST using horned-owl patterns as inspiration

### Next Step:
1. Complete PROJ-501 (audit `reasonable` crate for OWL RL rule encoding)
2. Assess whether Graphlaw needs horned-owl AST or can use RDF triples + rule rules
3. If AST needed, schedule PROJ-505 design phase (NOT implementation phase yet); mark as "CONDITIONAL" pending PROJ-501 findings

---

## Summary Table

| Dimension | Finding | Impact |
|-----------|---------|--------|
| **License** | LGPL-3.0 | Code adaptation forbidden; ideas OK via clean-room |
| **Version** | 2.0.0 (2024) | Production-ready, actively maintained |
| **AST Coverage** | Complete OWL 2 DL | Study design, subset for OWL RL v0 |
| **Memory Pattern** | Rc<str> interning + BTreeSet | Deterministic, efficient; reimplement in Graphlaw |
| **Normalization** | sort + reanonymize + simplify | Pattern applicable to Graphlaw's N-Quads canonicalization |
| **Parsing** | RDF/XML, Manchester, Functional | Study; don't code-copy |
| **Visitor Pattern** | VisitMut trait for axiom traversal | Design pattern reusable (not code-copied) |
| **Graphlaw Fit** | P1 optional, conditional on PROJ-501 | If OWL RL v0 needs typed AST; otherwise defer |


---

## H. Architectural Principle: horned-owl as Design Reference, Not Dependency

### Core Verdict

**License Incompatibility**: LGPL-3.0 code adaptation is forbidden. horned-owl is **ADAPT_IDEA / CLEAN_ROOM_REIMPLEMENT** only.

### Architectural Value

horned-owl teaches **representation discipline**, not code patterns:

| horned-owl Pattern | Graphlaw Translation |
|---|---|
| `IRI<A>` generic wrapper | Typed IRI identity, not raw strings |
| `Rc<str>` / `Arc<str>` sharing | Lower IRI clone cost; symbol interning |
| `BTreeSet` cache with `Build` factory | Deterministic entity construction |
| `Component` enum (200+ axiom types) | Optional OWL RL v0 typed AST (if needed) |
| `normalize` = simplify + reanonymize + sort | Canonical closure/replay hash discipline |

### The Crown Insight

**Triple-native → OWL RL profile scan → bounded rule compile → materialize → canonicalize → BLAKE3**

horned-owl's normalization patterns inform Graphlaw's receipt generation:
- Simplify (remove non-semantic metadata)
- Reanonymize (standardize variable names)
- Sort (deterministic ordering)
- Hash (BLAKE3 for receipt)
- Replay (idempotent verification)

### OWL RL v0 Strategy

**Phase 1 (Current)**: Stay triple-native unless forced otherwise.

```
RDF triples → OWL RL profile scanner → bounded rule compiler → materialization → closure hash
```

Do **not** build a full OWL AST yet. Graphlaw is **not** trying to become an OWL 2 DL processor.

**Phase 2** (Conditional on PROJ-501): Only introduce Graphlaw-owned OWL AST if `reasonable` proves that triple-native rule compilation becomes too messy.

Possible Graphlaw-owned OWL RL v0 AST (much smaller than horned-owl):
```rust
enum OwlRlAxiom {
    SubClassOf { sub: ClassExpr, sup: ClassExpr },
    SubPropertyOf { sub: PropertyExpr, sup: PropertyExpr },
    Domain { prop: PropertyExpr, domain: ClassExpr },
    Range { prop: PropertyExpr, range: ClassExpr },
    EquivalentClass(Vec<ClassExpr>),
    EquivalentProperty(Vec<PropertyExpr>),
    InverseOf { p1: PropertyExpr, p2: PropertyExpr },
    SymmetricProperty(PropertyExpr),
    TransitiveProperty(PropertyExpr),
    SameAs { a: Individual, b: Individual },
    Unsupported(String),
}
```

**Phase 3** (Receipt discipline): Every OWL RL closure produces:
```
OWL_RL_PROFILE
OWL_RL_RULE_SET_HASH
OWL_RL_INPUT_GRAPH_HASH
OWL_RL_DERIVED_FACT_COUNT
OWL_RL_CLOSURE_HASH
OWL_RL_UNSUPPORTED_FEATURE_COUNT
```

---

## I. Definition of Done: PROJ-505

| Gate | Status |
|---|---|
| License identified | ✅ LGPL-3.0 |
| Code adaptation | ✅ REFUSED |
| Clean-room path | ✅ ALIVE |
| Useful design patterns extracted | ✅ ALIVE (normalize → canonicalize → hash → replay) |
| Hot-path adoption | ✅ REFUSED |
| OWL AST implementation | ✅ CONDITIONAL (only if PROJ-501 shows triple-native insufficient) |
| Depends on PROJ-501 | ✅ YES (OWL RL audit determines AST necessity) |
| Final classification | ✅ **ADAPT_IDEA** |

---

## J. Ledger Entry

**Architecture Decision**:

> horned-owl is not adopted as a Graphlaw dependency because its LGPL-3.0 license is incompatible with Graphlaw's intended adaptation boundary. Its value is architectural: typed OWL entity representation, deterministic normalization, shared IRI identity, visitor-based traversal, and canonical comparison. Graphlaw may clean-room reimplement the relevant bounded OWL RL subset if the `reasonable` audit shows that RDF-triple-native compilation is insufficient.

**Key Insight**: The normalization pattern (simplify + reanonymize + sort) directly informs Graphlaw's receipt generation discipline: deterministic closure hashing and replay verification.

**Implementation Condition**: Conditional on PROJ-501 (OWL RL audit) determining whether Graphlaw needs typed OWL RL AST or can use RDF triples + rule compilation.

---

**PROJ-505 Status**: ✅ **COMPLETE** (DoD satisfied)

