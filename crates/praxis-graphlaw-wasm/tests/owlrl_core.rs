//! Comprehensive OWL RL rule tests for the WASM bridge.
//!
//! Tests all 11 supported OWL RL rules (subClassOf transitivity, type propagation,
//! subPropertyOf transitivity, property assertion propagation, domain, range,
//! equivalentClass, equivalentProperty, inverseOf, symmetricProperty, transitiveProperty)
//! plus 5 unsupported/refused features (sameAs, propertyChainAxiom, cardinality,
//! complex class expressions, imports).
//!
//! Each test:
//! 1. Creates minimal TTL demonstrating the rule/feature
//! 2. Provides non-empty profile_ttl to enable OWL RL materialization
//! 3. Calls validate_all_core
//! 4. Asserts the expected status (Admitted for supported, Refused/Unsupported for unsupported)
//! 5. For supported rules, verifies triples_out > 0 (derivation occurred)
//!
//! # Rule References
//!
//! Supported rules are implemented in `crates/praxis-graphlaw/src/owlrl.rs`:
//! - rule_subclass_transitive (line 239)
//! - rule_subclass_type_propagation (line 271)
//! - rule_subproperty_transitive (line 303)
//! - rule_subproperty_assertion_propagation (line 335)
//! - rule_domain (line 367)
//! - rule_range (line 399)
//! - rules_equivalent_class (line 431)
//! - rules_equivalent_property (line 471)
//! - rule_inverse_of (line 515)
//! - rule_symmetric_property (line 547)
//! - rule_transitive_property (line 579)
//!
//! Unsupported features:
//! - SameAs (line 117): External boundary required, unrestricted closure
//! - PropertyChainAxiom (line 122): Unsupported, requires forward-chaining
//! - Cardinality (line 125): Unsupported, requires constraint solving
//! - ComplexClassExpression (line 129): Unsupported, requires class-expr evaluation
//! - Imports (line 134): Unsupported, requires remote ontology loading

use praxis_graphlaw_wasm::{core::validate_all_core, dto::Status};

// ============================================================================
// Helper: Minimal profile TTL to enable OWL RL
// ============================================================================

/// Creates a minimal profile TTL that enables OWL RL materialization.
/// Any non-empty profile_ttl triggers materialize_owlrl() in the bridge.
fn minimal_profile() -> &'static str {
    r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        # Minimal profile to enable OWL RL
        <http://example/profile> a sh:NodeShape .
    "#
}

// ============================================================================
// Test 1: OWL RL Subclass Transitivity
// ============================================================================

/// Test RDFS subClassOf transitivity: A subClassOf B, B subClassOf C => A subClassOf C
///
/// This rule implements the transitive closure of rdfs:subClassOf, deriving
/// that if A is a subclass of B and B is a subclass of C, then A is a subclass of C.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new subClassOf triple derived)
#[test]
fn test_owlrl_subclass_transitivity() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:A rdfs:subClassOf ex:B .
        ex:B rdfs:subClassOf ex:C .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL subclass transitivity should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Subclass transitivity should derive new triples (ex:A rdfs:subClassOf ex:C)"
    );
}

// ============================================================================
// Test 2: OWL RL Subclass Type Propagation
// ============================================================================

/// Test RDFS type propagation: x rdf:type A, A rdfs:subClassOf B => x rdf:type B
///
/// When an instance is typed as A, and A is a subclass of B, the instance
/// is automatically typed as B.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new rdf:type triple derived)
#[test]
fn test_owlrl_subclass_type_propagation() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:person1 rdf:type ex:Student .
        ex:Student rdfs:subClassOf ex:Person .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL type propagation should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Type propagation should derive new triples (ex:person1 rdf:type ex:Person)"
    );
}

// ============================================================================
// Test 3: OWL RL Subproperty Transitivity
// ============================================================================

/// Test RDFS subPropertyOf transitivity: p subPropertyOf q, q subPropertyOf r => p subPropertyOf r
///
/// Implements the transitive closure of rdfs:subPropertyOf, deriving that if
/// property p is a subproperty of q and q is a subproperty of r, then p is a subproperty of r.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new subPropertyOf triple derived)
#[test]
fn test_owlrl_subproperty_transitivity() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:hasChild rdfs:subPropertyOf ex:hasDescendant .
        ex:hasDescendant rdfs:subPropertyOf ex:hasRelative .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL subproperty transitivity should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Subproperty transitivity should derive new triples (ex:hasChild rdfs:subPropertyOf ex:hasRelative)"
    );
}

// ============================================================================
// Test 4: OWL RL Subproperty Assertion Propagation
// ============================================================================

/// Test RDFS property propagation: x p y, p rdfs:subPropertyOf q => x q y
///
/// When an assertion uses property p, and p is a subproperty of q, the
/// assertion also holds for property q.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new property assertion derived)
#[test]
fn test_owlrl_subproperty_assertion_propagation() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:alice ex:hasChild ex:bob .
        ex:hasChild rdfs:subPropertyOf ex:hasRelative .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL property propagation should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Property propagation should derive new triples (ex:alice ex:hasRelative ex:bob)"
    );
}

// ============================================================================
// Test 5: OWL RL Domain Rule
// ============================================================================

/// Test RDFS domain rule: x p y, p rdfs:domain C => x rdf:type C
///
/// If a property p has domain C, then any subject using p must be of type C.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new rdf:type triple derived)
#[test]
fn test_owlrl_domain() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:alice ex:hasAge "25" .
        ex:hasAge rdfs:domain ex:Person .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL domain rule should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Domain rule should derive new triples (ex:alice rdf:type ex:Person)"
    );
}

// ============================================================================
// Test 6: OWL RL Range Rule
// ============================================================================

/// Test RDFS range rule: x p y, p rdfs:range C => y rdf:type C
///
/// If a property p has range C, then any object using p must be of type C.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new rdf:type triple derived)
#[test]
fn test_owlrl_range() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:alice ex:knows ex:bob .
        ex:knows rdfs:range ex:Person .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL range rule should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Range rule should derive new triples (ex:bob rdf:type ex:Person)"
    );
}

// ============================================================================
// Test 7: OWL RL Equivalent Class (Forward Direction)
// ============================================================================

/// Test OWL equivalentClass forward direction: A owl:equivalentClass B, x rdf:type A => x rdf:type B
///
/// owl:equivalentClass creates a bidirectional relationship. This test verifies
/// the forward direction: typing an instance as A implies typing it as B.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new rdf:type triple derived)
#[test]
fn test_owlrl_equivalent_class_forward() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        ex:Author owl:equivalentClass ex:Creator .
        ex:person1 rdf:type ex:Author .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL equivalent class should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Equivalent class should derive new triples (ex:person1 rdf:type ex:Creator)"
    );
}

// ============================================================================
// Test 8: OWL RL Equivalent Class (Reverse Direction)
// ============================================================================

/// Test OWL equivalentClass reverse direction: A owl:equivalentClass B, x rdf:type B => x rdf:type A
///
/// This test verifies the reverse direction: typing an instance as B implies typing it as A.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new rdf:type triple derived)
#[test]
fn test_owlrl_equivalent_class_reverse() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        ex:Author owl:equivalentClass ex:Creator .
        ex:person1 rdf:type ex:Creator .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL equivalent class should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Equivalent class reverse should derive new triples (ex:person1 rdf:type ex:Author)"
    );
}

// ============================================================================
// Test 9: OWL RL Equivalent Property (Forward Direction)
// ============================================================================

/// Test OWL equivalentProperty forward direction: p owl:equivalentProperty q, x p y => x q y
///
/// owl:equivalentProperty creates a bidirectional relationship. This test verifies
/// the forward direction: using property p implies using property q.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new property assertion derived)
#[test]
fn test_owlrl_equivalent_property_forward() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        ex:hasAuthor owl:equivalentProperty ex:creator .
        ex:book1 ex:hasAuthor ex:person1 .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL equivalent property should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Equivalent property should derive new triples (ex:book1 ex:creator ex:person1)"
    );
}

// ============================================================================
// Test 10: OWL RL Equivalent Property (Reverse Direction)
// ============================================================================

/// Test OWL equivalentProperty reverse direction: p owl:equivalentProperty q, x q y => x p y
///
/// This test verifies the reverse direction: using property q implies using property p.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new property assertion derived)
#[test]
fn test_owlrl_equivalent_property_reverse() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        ex:hasAuthor owl:equivalentProperty ex:creator .
        ex:book1 ex:creator ex:person1 .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL equivalent property should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Equivalent property reverse should derive new triples (ex:book1 ex:hasAuthor ex:person1)"
    );
}

// ============================================================================
// Test 11: OWL RL Inverse Of
// ============================================================================

/// Test OWL inverseOf rule: p owl:inverseOf q, x p y => y q x
///
/// If property p is the inverse of property q, then an assertion using p
/// implies the inverse assertion using q.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new inverse property assertion derived)
#[test]
fn test_owlrl_inverse_of() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        ex:parent owl:inverseOf ex:child .
        ex:alice ex:parent ex:bob .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL inverse of should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Inverse of should derive new triples (ex:bob ex:child ex:alice)"
    );
}

// ============================================================================
// Test 12: OWL RL Symmetric Property
// ============================================================================

/// Test OWL SymmetricProperty rule: p rdf:type owl:SymmetricProperty, x p y => y p x
///
/// If property p is symmetric, then any assertion using p implies the
/// symmetric assertion.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new symmetric assertion derived)
#[test]
fn test_owlrl_symmetric_property() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        ex:knows rdf:type owl:SymmetricProperty .
        ex:alice ex:knows ex:bob .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL symmetric property should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Symmetric property should derive new triples (ex:bob ex:knows ex:alice)"
    );
}

// ============================================================================
// Test 13: OWL RL Transitive Property
// ============================================================================

/// Test OWL TransitiveProperty rule: p rdf:type owl:TransitiveProperty, x p y, y p z => x p z
///
/// If property p is transitive, then a chain of assertions using p implies
/// a direct assertion.
///
/// Assertions:
/// - OWL_RL dialect status == Status::Admitted
/// - triples_out > 0 (new transitive closure triple derived)
#[test]
fn test_owlrl_transitive_property() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        ex:ancestorOf rdf:type owl:TransitiveProperty .
        ex:alice ex:ancestorOf ex:bob .
        ex:bob ex:ancestorOf ex:charlie .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    assert_eq!(
        owlrl_dialect.status,
        Status::Admitted,
        "OWL RL transitive property should be admitted"
    );
    assert!(
        owlrl_dialect.triples_out > 0,
        "Transitive property should derive new triples (ex:alice ex:ancestorOf ex:charlie)"
    );
}

// ============================================================================
// Test 14: OWL RL sameAs (External Boundary / Unsupported)
// ============================================================================

/// Test OWL sameAs feature (unsupported in daily profile).
///
/// owl:sameAs defines entity equivalence. The daily profile marks this as
/// ExternalBoundaryRequired because unrestricted sameAs closure is outside
/// the bounded daily profile (equivalence merging is a later profile).
///
/// Assertions:
/// - OWL_RL dialect status should be Refused or indicate unsupported
/// - detail should mention external boundary or sameAs limitation
#[test]
fn test_owlrl_sameas_external_boundary() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        ex:Person1 owl:sameAs ex:Person2 .
        ex:Person2 owl:sameAs ex:Person3 .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    // sameAs is marked as ExternalBoundaryRequired, so status should be Refused
    assert_eq!(
        owlrl_dialect.status,
        Status::Refused,
        "OWL RL sameAs should be refused as external boundary required"
    );
    assert!(
        owlrl_dialect.detail.contains("sameAs") || owlrl_dialect.detail.contains("external"),
        "Detail should mention sameAs or external boundary. Got: {}",
        owlrl_dialect.detail
    );
}

// ============================================================================
// Test 15: OWL RL Property Chain Axiom (Unsupported)
// ============================================================================

/// Test OWL propertyChainAxiom feature (unsupported).
///
/// owl:propertyChainAxiom defines a chain of properties that imply another property.
/// This is unsupported in the daily profile because it requires forward-chaining
/// schema computation not supported in daily profile.
///
/// Assertions:
/// - OWL_RL dialect status should be Refused
/// - detail should mention propertyChainAxiom
#[test]
fn test_owlrl_property_chain_axiom_unsupported() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        ex:transitiveChain rdf:type owl:ObjectProperty ;
            owl:propertyChainAxiom (ex:prop1 ex:prop2) .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    // propertyChainAxiom is unsupported
    assert_eq!(
        owlrl_dialect.status,
        Status::Refused,
        "OWL RL propertyChainAxiom should be refused as unsupported"
    );
    assert!(
        owlrl_dialect.detail.contains("propertyChainAxiom")
            || owlrl_dialect.detail.contains("chain"),
        "Detail should mention propertyChainAxiom. Got: {}",
        owlrl_dialect.detail
    );
}

// ============================================================================
// Test 16: OWL RL Cardinality Constraint (Unsupported)
// ============================================================================

/// Test OWL cardinality constraint features (unsupported).
///
/// Cardinality constraints (owl:cardinality, owl:minCardinality, owl:maxCardinality)
/// require constraint solving outside the daily profile scope.
///
/// Assertions:
/// - OWL_RL dialect status should be Refused
/// - detail should mention cardinality
#[test]
fn test_owlrl_cardinality_unsupported() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        ex:Person a owl:Class ;
            owl:minCardinality 1 ;
            owl:maxCardinality 1 .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    // Cardinality is unsupported
    assert_eq!(
        owlrl_dialect.status,
        Status::Refused,
        "OWL RL cardinality should be refused as unsupported"
    );
    assert!(
        owlrl_dialect.detail.contains("cardinality"),
        "Detail should mention cardinality. Got: {}",
        owlrl_dialect.detail
    );
}

// ============================================================================
// Test 17: OWL RL Complex Class Expression (Unsupported)
// ============================================================================

/// Test OWL complex class expression features (unsupported).
///
/// Complex class expressions (owl:unionOf, owl:intersectionOf, owl:oneOf, restrictions)
/// require class-expression evaluation not supported in the daily profile.
///
/// Assertions:
/// - OWL_RL dialect status should be Refused
/// - detail should mention unionOf/intersectionOf/oneOf
#[test]
fn test_owlrl_complex_class_expr_unsupported() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        ex:Container a rdfs:Class ;
            owl:unionOf (ex:Box ex:Bag ex:Crate) .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    // Complex class expressions are unsupported
    assert_eq!(
        owlrl_dialect.status,
        Status::Refused,
        "OWL RL complex class expressions should be refused as unsupported"
    );
    assert!(
        owlrl_dialect.detail.contains("unionOf")
            || owlrl_dialect.detail.contains("intersectionOf")
            || owlrl_dialect.detail.contains("oneOf")
            || owlrl_dialect.detail.contains("class expression"),
        "Detail should mention unionOf/intersectionOf/oneOf. Got: {}",
        owlrl_dialect.detail
    );
}

// ============================================================================
// Test 18: OWL RL Imports (Unsupported)
// ============================================================================

/// Test OWL imports feature (unsupported).
///
/// owl:imports requires remote ontology loading and is outside the bounded
/// daily profile scope.
///
/// Assertions:
/// - OWL_RL dialect status should be Refused
/// - detail should mention imports
#[test]
fn test_owlrl_imports_unsupported() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .

        <http://example.org/ontology> owl:imports <http://example.org/external-ontology> .
    "#;

    let result = validate_all_core(ttl, minimal_profile(), "", "", "");
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();
    let owlrl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "OWL_RL")
        .expect("OWL_RL dialect should be present");

    // Imports is unsupported
    assert_eq!(
        owlrl_dialect.status,
        Status::Refused,
        "OWL RL imports should be refused as unsupported"
    );
    assert!(
        owlrl_dialect.detail.contains("imports"),
        "Detail should mention imports. Got: {}",
        owlrl_dialect.detail
    );
}
