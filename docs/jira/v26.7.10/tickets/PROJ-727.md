# PROJ-727 — Distributed evidence — OBS_KINDS, OCEL construct, markers

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: E (multi-engine execution).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

New OBS_KINDS: EngineStarted/EngineQuiesced, RemoteDispatchSent, RemoteConsequenceReceived,
LedgerAppended, ResumeVerified, SharedMemoryCrossing, DirectEngineBypass. New
ocel-remote-engine CONSTRUCT stem. Marker queries realized as `DISTRIBUTED_MARKER_MAP`
(`crates/cng/src/bench/workday.rs:148-174`): `SHARED_MEMORY_CROSSINGS_ZERO`,
`DIRECT_ENGINE_BYPASSES_ZERO`, `REMOTE_WORKFLOWS_ACKNOWLEDGED`, `REMOTE_WORKFLOWS_COMPLETED`,
`REPLAY_DIVERGENCES_ZERO`, `ARAZZO_WORKFLOWS_DISPATCHED`, `MULTI_ENGINE_EXECUTION_PROVEN`,
`ENGINE_INSTANCES_PROVEN`, `ARAZZO_INTER_ENGINE_WORKFLOW_PROVEN` (names reconciled to on-disk
identifiers at PROJ-743 — the original bare-form names above are historical scope text, not
the shipped identifiers), plus the `LLM_CALLS_ZERO` family and the planning set (DoD §16).
WorkdayReport graph/telemetry twin counters + EvidenceGateFailed gates. Includes the SPARQL
rule minting DirectEngineBypass for any admitted consequence lacking a matching
RemoteDispatchSent/ledger entry (DoD §14 item 2). Gate: G16 inputs.

## Evidence (this session)

`crates/cng/queries/ocel-remote-engine.construct.rq`, `DISTRIBUTED_MARKER_MAP`
(`workday.rs:148-174`) on disk. `isolation_falsifier_hostile_graph_is_refuted_by_markers`
(`cng_multi_engine.rs:279`), plus the marker assertions at `cng_multi_engine.rs:232-236`
(`MULTI_ENGINE_EXECUTION_PROVEN`, `ARAZZO_INTER_ENGINE_WORKFLOW_PROVEN`,
`DIRECT_ENGINE_BYPASSES_ZERO`, `SHARED_MEMORY_CROSSINGS_ZERO`, `REMOTE_WORKFLOWS_COMPLETED`
all true) — part of the `cargo test -p cng --features bench --test cng_multi_engine --
--test-threads=1` run, 6/6 passed, this session.
