use crate::encoding::Encoder;
use crate::parser::Syntax;
/// Type definitions for compiled SHACL shapes
///
/// These types represent the intermediate representation of SHACL shapes,
/// ready for efficient evaluation.
use crate::tripleindex::TripleIndex;

/// CostClass ordering for constraint evaluation: evaluate cheaper constraints
/// (O(1) operations) before expensive ones (O(n) or O(n²) operations).
/// This enables early termination when a constraint fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CostClass {
    /// sh:minCount, sh:maxCount (cardinality check, O(1))
    Cardinality = 0,
    /// sh:nodeKind (type check, O(1))
    NodeKind = 1,
    /// sh:datatype (string comparison, O(1))
    Datatype = 2,
    /// sh:class (subclass lookup, O(closure))
    Class = 3,
    /// sh:path (graph traversal, O(graph))
    Path = 4,
    /// sh:pattern (string regex, O(string))
    Regex = 5,
    /// Recursive shape reference (O(depth) or O(graph))
    Recursive = 6,
}

/// A pre-compiled SHACL constraint, ready for evaluation.
/// Uses String IRIs (not SymbolId) to match current codebase conventions.
#[derive(Debug, Clone)]
pub struct CompiledConstraint {
    /// Cost class determines evaluation order
    pub cost_class: CostClass,
    /// The constraint predicate (e.g., sh:minCount, sh:class, sh:pattern)
    pub predicate: usize,
    /// The constraint value (e.g., class IRI, regex pattern)
    pub value: usize,
    /// Whether this constraint is deactivated (sh:deactivated)
    pub is_optional: bool,
}

/// Target selection for a SHACL shape
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    /// sh:targetNode: explicit focus node
    Node,
    /// sh:targetClass: all instances of a class
    Class,
    /// sh:targetSubjectsOf: all subjects of a property
    SubjectsOf,
    /// sh:targetObjectsOf: all objects of a property
    ObjectsOf,
}

/// Pre-compiled SHACL shape target
#[derive(Debug, Clone)]
pub struct CompiledTarget {
    /// The target value (node IRI, class IRI, or property IRI)
    pub target_value: usize,
    /// The type of target
    pub target_type: TargetType,
}

/// Pre-compiled SHACL shape representation
#[derive(Debug, Clone)]
pub struct CompiledShape {
    /// Shape IRI
    pub iri: usize,
    /// Target nodes/classes/properties
    pub targets: Vec<CompiledTarget>,
    /// Constraints, sorted by CostClass (Cardinality first, Recursive last)
    pub constraints: Vec<CompiledConstraint>,
    /// sh:closed (property whitelist enforcement)
    pub closed: bool,
    /// Property shapes (sh:property, recursion)
    pub property_shapes: Vec<CompiledShape>,
}

/// SHACL-SPARQL Dialect Boundary Decision (PROJ-407 Step 2)
///
/// Decision: CORE_ONLY (most conservative)
///
/// Rationale:
/// - SHACL-SPARQL constraints (sh:sparql, sh:select, sh:ask) are rejected at
///   shape load time. Shape validation is purely constraint-based, with no
///   SPARQL evaluation.
/// - Constraints: v26.7.8 threat model prioritizes smallest attack surface
///   and deterministic (not network-dependent) validation. SPARQL evaluation
///   introduces: (1) unbounded query complexity, (2) remote endpoint risk
///   (if federated), (3) non-determinism (variable query planning).
/// - Existing use cases: Graphlaw's production shapes do not rely on
///   SPARQL constraints; all observed use cases are expressible in SHACL
///   CORE (class, property, cardinality, datatype constraints).
///
/// If future use cases require SPARQL constraints, revisit this decision and
/// implement SPARQL_OPTIONAL (local queries only, no federation) in a
/// follow-up ticket.
pub const SHACL_SPARQL_BOUNDARY: &str = "CORE_ONLY";

/// SHACL shapes graph representation
pub struct ShapesGraph {
    pub raw_index: TripleIndex,
}

impl ShapesGraph {
    pub fn parse(shapes_str: &str) -> Result<Self, String> {
        let triples = crate::parser::Parser::parse_triples(shapes_str, Syntax::Turtle)?;
        let mut raw_index = TripleIndex::new();
        for triple in triples {
            raw_index.add(triple);
        }
        Ok(ShapesGraph { raw_index })
    }
}
