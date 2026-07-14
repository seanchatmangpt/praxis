# Trajectory Constraints Certification Pack

## Expected Outcomes

1. **SHACL Admission**:
   - `domain-valid.ttl` and `problem-valid.ttl` checked against `shapes.ttl` MUST return `AdmissionGranted`.
   - `problem-invalid-bounds.ttl` checked against `shapes.ttl` MUST return `AdmissionRefused` (violates length constraint bounds).

2. **Planner Capability Negotiation (Pddl31Ir)**:
   - When the admitted `Pddl31Ir` representation of `domain-valid.ttl` and `problem-valid.ttl` is passed to `bcinr_pddl`'s solver, it MUST be recognized as structurally valid.
   - If the solver does not yet implement state tracking for trajectory constraints, it MUST immediately return `Unsupported("trajectory_constraints")`.
   - If supported, the solver MUST return a valid plan or prove exhaustion, adhering to the trajectory constraint boundaries.

3. **Solver/Parsing Validation via IR**:
   - Passing `problem-invalid-syntax.ttl` into the solver MUST result in a parse/grounding error during IR translation due to the malformed `(always)` constraint syntax, even if it passed SHACL string length bounds.
   - Passing `domain-invalid-req.ttl` paired with a problem containing constraints MUST fail solver validation due to missing `:constraints` requirement.
