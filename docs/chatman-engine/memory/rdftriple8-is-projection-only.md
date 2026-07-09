# RDFTriple8 Is Projection-Only

**Summary**: RDFTriple8 is a bounded projection derived from the RDF graph; it never
originates facts and is never authoritative.

**Source evidence**: Absolute Doctrine items 1–2 in
`docs/chatman-engine/FABLE_OPERATING_CONSTITUTION.md`; triple8 falsification pairs (257 terms,
unknown term, profile mismatch, >8 constraints).

**Why it matters**: Treating the projection as storage lets facts bypass graph authority and
its admission gates, breaking receipt replay.

**Future instruction**: Write to the graph, project to RDFTriple8. Any code path that mutates
or originates data at the triple8 layer is a doctrine violation.
