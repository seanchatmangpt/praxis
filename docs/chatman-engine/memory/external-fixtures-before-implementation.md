# External Fixtures Before Implementation

**Summary**: Fixtures are authored as external files before any implementation code exists.

**Source evidence**: Chicago TDD Rule in `docs/chatman-engine/FABLE_OPERATING_CONSTITUTION.md`;
work-order step 6 (fixtures) precedes step 11 (minimal implementation).

**Why it matters**: Fixtures written after the implementation tend to encode the
implementation's behavior — including its bugs — instead of the specification.

**Future instruction**: Land fixture files (TTL, JSON, expected receipts) and the harnesses
that read them before writing the code under test. Initial failures are expected and recorded.
