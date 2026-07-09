# Hooks Require Receipts

**Summary**: Every hook execution must carry a valid receipt; missing, invalid, or unadmitted
(OCEL) material is refused.

**Source evidence**: Hooks falsification pairs (constellation, missing receipt, invalid
receipt, unadmitted OCEL) in `docs/chatman-engine/FABLE_OPERATING_CONSTITUTION.md`.

**Why it matters**: A hook without a receipt is an unauditable side effect — it breaks replay
and lets unadmitted events enter the graph.

**Future instruction**: Hook code paths must validate receipt structure and admission before
acting; ship the four negative tests with any hook change.
