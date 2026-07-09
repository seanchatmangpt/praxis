# Vacuous Generation Defect

**Summary**: A prior ggen run emitted 33 assert-free tests; generation templates must carry
unconditional assertion text plus a post-hoc gate.

**Source evidence**: This session's review of a prior ggen output: 33 generated test functions
contained no assertions and passed vacuously.

**Why it matters**: Vacuous tests inflate coverage and pass counts while proving nothing —
the exact self-graded-victory failure mode the doctrine forbids (property P8).

**Future instruction**: Every test-generation template must include an unconditional assertion
in its body, and a post-generation gate must scan emitted tests and refuse any function
without an assertion.
