# PROJ-415: Implement CompiledShape Compilation
**Title:** Populate `CompiledShape` targets, constraints, property shapes, and required-property mask at parse time
**Type:** Feature / Debt
**Target:** `/Users/sac/praxis` (module: `crates/praxis-graphlaw/src/shacl/model.rs`)
**Status:** OPEN

## Description
`ShapesGraph::compile_shape` (`crates/praxis-graphlaw/src/shacl/model.rs:245-256`) builds
`CompiledShape` values with five placeholder fields:

1. `targets: Vec::new()` — TODO(PROJ-415): populate from target declarations
2. `constraints: Vec::new()` — TODO(PROJ-415): populate from constraint declarations
3. `property_shapes: Vec::new()` — TODO(PROJ-415): populate from `sh:property`
4. `required_properties_mask` hardcoded to `PropertyMask(0)` — should scan `sh:minCount`
   constraints in property shapes
5. `PropertyMask` 64-property limit (`model.rs:50`) — shapes with >64 properties require a
   `Vec<u64>` extension (currently documented as future work)

Only `iri`, `closed`, and `allowed_predicates` are real. Any consumer relying on the other
fields silently observes empty data (Invariant 1 risk: placeholder success).

## Implementation Spec
1. In `compile_shape`, populate:
   - `targets` from `sh:targetNode` / `sh:targetClass` / `sh:targetSubjectsOf` /
     `sh:targetObjectsOf` declarations, as `CompiledTarget { target_value, target_type }`.
   - `constraints` from constraint declarations on the shape, sorted by `CostClass`
     (Cardinality first, Recursive last) per the `CompiledShape.constraints` doc comment.
   - `property_shapes` by recursively compiling each `sh:property` object.
   - `required_properties_mask` by scanning `sh:minCount >= 1` constraints in property
     shapes, indexing bits by property position in the compiled shape's properties.
2. Handle the >64-property case explicitly: either extend `PropertyMask` to `Vec<u64>` or
   return a typed refusal at compile time — no silent truncation.
3. Complexity stays O(|shapes| * |properties|) at parse time as documented on
   `compile_all_shapes`.

## Acceptance Criteria
- [ ] All five `CompiledShape` fields are populated from the shapes graph (no `Vec::new()`
      TODO placeholders, no hardcoded `PropertyMask(0)`).
- [ ] Tests exercise each field: targets of all four types, constraints in CostClass order,
      recursive property shapes, and a required-property mask with mixed `sh:minCount`.
- [ ] Shapes with >64 properties are handled deterministically (extension or typed refusal),
      with a test.
- [ ] No panics, unwraps, or silent defaults in new code.
