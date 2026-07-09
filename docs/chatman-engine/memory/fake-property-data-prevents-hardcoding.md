# Fake Property Data Prevents Hardcoding

**Summary**: Property-varied (generated) test data defeats implementations that hardcode
outputs for known fixtures.

**Source evidence**: Anti-Hardcode Properties P1–P8 and work-order step 7 (generated cases) in
`docs/chatman-engine/FABLE_OPERATING_CONSTITUTION.md`.

**Why it matters**: A stub returning the one expected constant passes a single-fixture test;
it cannot pass when inputs vary and every input must be load-bearing (P1, P2).

**Future instruction**: For every fixture-based test, add generated cases that perturb each
input and assert the output changes accordingly.
