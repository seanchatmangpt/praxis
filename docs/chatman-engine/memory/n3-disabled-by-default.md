# N3 Disabled by Default

**Summary**: N3 is quarantined — never a default, never a fallback, never an actuation path;
explicit opt-in only.

**Source evidence**: Absolute Doctrine item 3 and routing falsification pairs (N3-default,
N3-actuation) in `docs/chatman-engine/FABLE_OPERATING_CONSTITUTION.md`.

**Why it matters**: N3 rule evaluation has known cubic scaling behavior and a broader semantic
surface than the closed profiles; defaulting to it silently trades determinism bounds away.

**Future instruction**: Routing must refuse N3 unless explicitly enabled; add the N3-default
and N3-actuation negative tests to any routing change.
