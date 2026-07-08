/**
 * Fixture loader for sample Turtle files.
 */

export const sampleTurtle = `# Sample Turtle fixture for the Praxis Playground

@prefix ex: <http://example.com/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

# Define a Person class and instances
ex:Person a rdfs:Class ;
  rdfs:label "Person" ;
  rdfs:comment "A human being" .

ex:John
  a ex:Person ;
  foaf:name "John Doe" ;
  foaf:age 30 ;
  foaf:email "john@example.com" ;
  foaf:knows ex:Jane .

ex:Jane
  a ex:Person ;
  foaf:name "Jane Smith" ;
  foaf:age 28 ;
  foaf:email "jane@example.com" ;
  foaf:knows ex:John .

# Define a Project class
ex:Project a rdfs:Class ;
  rdfs:label "Project" ;
  rdfs:comment "A collaborative project" .

ex:ProjectA
  a ex:Project ;
  rdfs:label "Project Alpha" ;
  rdfs:comment "First research project" ;
  ex:hasMember ex:John, ex:Jane ;
  ex:startDate "2024-01-15"^^xsd:date ;
  ex:status ex:Active .

# Status enumeration
ex:Active a rdf:Property ;
  rdfs:label "Active" .
`;

/**
 * OWL RL profile pack for daily profile testing (v26.7.8).
 * Minimal OWL RL content with class hierarchies and property constraints.
 */
export const OWL_RL_PROFILE_TTL = `@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.com/> .

# Profile pack definition for OWL RL daily profile
<http://example.com/profiles/owlrl-daily> a sh:NodeShape ;
  rdfs:label "OWL RL Daily Profile" ;
  sh:targetClass <http://example.com/Profile> ;
  sh:property [
    sh:path <http://seanchatmangpt.github.io/praxis/kh#kind> ;
    sh:hasValue "owlrl" ;
    sh:minCount 1 ;
  ] .

# Class hierarchy for testing subClassOf
ex:Agent a rdfs:Class .
ex:Person rdfs:subClassOf ex:Agent .
ex:Organization rdfs:subClassOf ex:Agent .

# Property hierarchy for testing subPropertyOf
ex:knows a rdf:Property .
ex:familiarWith rdfs:subPropertyOf ex:knows .

# Domain and range constraints
ex:name rdfs:domain ex:Agent ;
  rdfs:range rdfs:Literal .

ex:hasMember rdfs:domain ex:Organization ;
  rdfs:range ex:Agent .

# Symmetric property for testing
ex:linkedTo a owl:SymmetricProperty ;
  rdfs:domain ex:Agent .

# Transitive property for testing
ex:ancestor a owl:TransitiveProperty ;
  rdfs:domain ex:Person ;
  rdfs:range ex:Person .
`;

/**
 * SHACL shapes for validation testing.
 * Covers sh:minCount, sh:maxCount, sh:datatype, sh:class, sh:pattern,
 * sh:nodeKind, sh:in, sh:closed, and logical operators (sh:and/sh:or).
 */
export const SHACL_SHAPES_TTL = `@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex: <http://example.com/> .

# Person shape with comprehensive constraints
<http://example.com/PersonShape> a sh:NodeShape ;
  rdfs:label "Person Validation Shape" ;
  sh:targetClass ex:Person ;
  sh:closed true ;
  sh:ignoredProperties ( rdf:type ) ;
  sh:property [
    sh:path ex:name ;
    sh:minCount 1 ;
    sh:maxCount 1 ;
    sh:datatype xsd:string ;
    sh:pattern "^[A-Za-z\\s]+$" ;
    sh:minLength 1 ;
    sh:maxLength 100 ;
  ] ,
  [
    sh:path ex:email ;
    sh:minCount 1 ;
    sh:maxCount 1 ;
    sh:datatype xsd:string ;
    sh:pattern "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$" ;
  ] ,
  [
    sh:path ex:age ;
    sh:datatype xsd:integer ;
    sh:minInclusive 0 ;
    sh:maxInclusive 150 ;
  ] ,
  [
    sh:path ex:status ;
    sh:in ( ex:active ex:inactive ex:suspended ) ;
  ] ,
  [
    sh:path ex:role ;
    sh:class ex:Role ;
    sh:nodeKind sh:IRI ;
  ] .

# Document shape with logical operators
<http://example.com/DocumentShape> a sh:NodeShape ;
  rdfs:label "Document Validation Shape" ;
  sh:targetClass ex:Document ;
  sh:property [
    sh:path ex:title ;
    sh:minCount 1 ;
    sh:datatype xsd:string ;
  ] ,
  [
    sh:path ex:content ;
    sh:minLength 10 ;
  ] ;
  sh:and (
    [
      sh:property [
        sh:path ex:author ;
        sh:minCount 1 ;
      ]
    ]
    [
      sh:property [
        sh:path ex:createdDate ;
        sh:datatype xsd:dateTime ;
      ]
    ]
  ) .

# Role shape
<http://example.com/RoleShape> a sh:NodeShape ;
  sh:targetClass ex:Role ;
  sh:property [
    sh:path rdfs:label ;
    sh:minCount 1 ;
    sh:datatype xsd:string ;
  ] .
`;

/**
 * ShEx schema in ShExC syntax for structural validation.
 * Defines shapes with node constraints, cardinality, and facets.
 */
export const SHEX_SCHEMA_SHEXC = `PREFIX ex: <http://example.com/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

# Person shape with required and optional properties
<http://example.com/PersonShape> {
  ex:name xsd:string ;
  ex:email xsd:string ;
  ex:age xsd:integer ? ;
  ex:phone xsd:string * ;
  ex:status [ex:active ex:inactive ex:suspended] ;
  ex:gender ["male" "female" "other"] ?
}

# Document shape with structured constraints
<http://example.com/DocumentShape> {
  ex:title xsd:string ;
  ex:content xsd:string+ ;
  ex:author <http://example.com/PersonShape> ;
  ex:createdDate xsd:dateTime ;
  ex:tags xsd:string *
}

# Role shape
<http://example.com/RoleShape> {
  rdf:label xsd:string
}
`;

/**
 * Hook pack using ONLY the hook: namespace alias (NOT kh:).
 * Tests the bridge's rewrite_hook_alias logic with 3 hooks:
 * - delta hook (default kind)
 * - threshold hook (k-based condition)
 * - count hook (occurrence-based)
 */
export const HOOKS_TTL = `@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.com/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Delta hook: fires on new facts matching pattern
<http://example.com/hooks/approval-delta> a hook:Hook ;
  rdfs:label "Approval Delta Hook" ;
  hook:on ex:RequestSubmitted ;
  hook:kind "delta" ;
  hook:priority 5 ;
  hook:effect "emit-delta" ;
  hook:name "approval-workflow" ;
  rdfs:comment "Emits approval events for newly submitted requests" .

# Threshold hook: fires when aggregate exceeds k
<http://example.com/hooks/escalation-threshold> a hook:Hook ;
  rdfs:label "Escalation Threshold Hook" ;
  hook:on ex:RequestSubmitted ;
  hook:kind "threshold" ;
  hook:k 50000 ;
  hook:op ">" ;
  hook:priority 10 ;
  hook:after <http://example.com/hooks/approval-delta> ;
  hook:effect "emit-delta" ;
  hook:reason "amount exceeds escalation threshold" ;
  hook:name "escalation-workflow" ;
  rdfs:comment "Escalates requests with amounts exceeding 50000" .

# Count hook: fires after N occurrences
<http://example.com/hooks/batch-count> a hook:Hook ;
  rdfs:label "Batch Count Hook" ;
  hook:on ex:RequestProcessed ;
  hook:kind "count" ;
  hook:k 10 ;
  hook:priority 3 ;
  hook:after <http://example.com/hooks/approval-delta> ;
  hook:effect "emit-delta" ;
  hook:name "batch-processor" ;
  rdfs:comment "Batches every 10 processed requests" .
`;

/**
 * Event data for testing hook triggers.
 * Contains RDF facts that match hook:on conditions in HOOKS_TTL.
 */
export const EVENT_TTL = `@prefix ex: <http://example.com/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Request submission events
ex:request-1 a ex:RequestSubmitted ;
  ex:id "REQ-001" ;
  ex:requester ex:alice ;
  ex:amount 35000 ;
  ex:status ex:pending ;
  ex:submittedAt "2024-01-15T10:30:00Z"^^xsd:dateTime .

ex:request-2 a ex:RequestSubmitted ;
  ex:id "REQ-002" ;
  ex:requester ex:bob ;
  ex:amount 75000 ;
  ex:status ex:pending ;
  ex:submittedAt "2024-01-15T11:00:00Z"^^xsd:dateTime .

ex:request-3 a ex:RequestSubmitted ;
  ex:id "REQ-003" ;
  ex:requester ex:charlie ;
  ex:amount 25000 ;
  ex:status ex:pending ;
  ex:submittedAt "2024-01-15T11:30:00Z"^^xsd:dateTime .

# Request processed events (for count hook testing)
ex:request-1-processed a ex:RequestProcessed ;
  ex:requestId "REQ-001" ;
  ex:processedAt "2024-01-15T12:00:00Z"^^xsd:dateTime ;
  ex:result ex:approved .

ex:request-2-processed a ex:RequestProcessed ;
  ex:requestId "REQ-002" ;
  ex:processedAt "2024-01-15T12:01:00Z"^^xsd:dateTime ;
  ex:result ex:escalated .
`;
