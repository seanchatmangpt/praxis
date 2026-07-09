# Agents Cannot Override Profiles

**Summary**: Agents are witnesses; they may attest but never override profiles, claim
authority, act while disabled, or act nondeterministically without a receipt.

**Source evidence**: Absolute Doctrine item 4 and the agents falsification pairs in
`docs/chatman-engine/FABLE_OPERATING_CONSTITUTION.md`.

**Why it matters**: If an agent can override a profile, graph authority is fiction — any
witness becomes a writer and admission gates are bypassable.

**Future instruction**: Agent surfaces expose attest-only APIs. Add negative tests for
override, authority claim, disabled breed, and receipt-less nondeterminism.
