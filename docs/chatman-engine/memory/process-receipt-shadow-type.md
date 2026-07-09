# ProcessReceipt Shadow Type

**Summary**: wasm4pm-compat has NO `ProcessReceipt`; wrap `ReceiptEnvelope`/`Digest` instead —
the digest is carried, not computed, at that boundary.

**Source evidence**: Repo audit of the wasm4pm-compat surface during the Chatman Engine
workflow; static gate "duplicate ProcessReceipt" in the constitution's falsification pairs.

**Why it matters**: Defining a local `ProcessReceipt` shadow type creates a duplicate canonical
type and invites recomputing a digest that must be carried verbatim from its source.

**Future instruction**: Never define `ProcessReceipt` in compat layers. Wrap the canonical
`ReceiptEnvelope`/`Digest` types and pass digests through unchanged.
