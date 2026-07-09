# Falsification Pairs Are Mandatory

**Summary**: Every positive test ships with negative counterparts that must refuse; a green
suite without falsification cases proves nothing.

**Source evidence**: Required Falsification Pairs section of
`docs/chatman-engine/FABLE_OPERATING_CONSTITUTION.md` (receipt replay, triple8, admission,
routing, hooks, agents, tape, static).

**Why it matters**: Happy-path-only suites pass over constant receipts, ignored inputs, and
silent fallbacks — exactly the anti-fake patterns the doctrine forbids.

**Future instruction**: For each subsystem touched, enumerate its required negative cases from
the constitution and land them alongside the positive tests; a missing pair blocks the verdict.
