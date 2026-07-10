# PROJ-411..417 Reconciliation

Chatman Engine v26.7.9 closure (Gate F verdict: `ADMITTED_DRY_RUN_PUBLISHABLE`,
signed 2026-07-10T01:23:35Z — see `docs/chatman-engine/chicago_tdd_final_report.md`).

Individual ticket specs remain at `docs/jira/v26.7.8/tickets/PROJ-41{1..7}.md`.
This doc reconciles their in-file status headers against the Gate F verdict.

| Ticket | Title | In-file status | Gate F disposition |
|---|---|---|---|
| PROJ-411 | CE-ABI Boundary and Typed Refusals | IN PROGRESS | In scope, PASS (Gate B) |
| PROJ-412 | Unify praxis-graphlaw with bcinr-pddl/bcinr-powl | IN PROGRESS | In scope, PASS (Gate B) |
| PROJ-413 | ChatmanEngine Core Pipeline Orchestration | IN PROGRESS | In scope, PASS (Gate B) |
| PROJ-414 | BLAKE3 Receipt Evidence Generation | IN PROGRESS | In scope, PASS (Gate D) |
| PROJ-415 | CompiledShape Compilation | OPEN | Explicitly excluded from Gate F scope |
| PROJ-416 | Wire Pattern-4 Canonical Renders to BLAKE3 Receipt Hashing | OPEN | Explicitly excluded from Gate F scope |
| PROJ-417 | Surface Status::HashMismatch in verify_replay | OPEN | Explicitly excluded from Gate F scope |

PROJ-411..414 are the authoritative Chatman Engine v26.7.9 substrate/ABI/hot-path/
receipt work admitted by Gate F. PROJ-415..417 remain OPEN debt tickets, carried
forward unchanged — their "IN PROGRESS" vs "OPEN" in-file headers were not
updated as part of this closure pass and should not be read as reflecting Gate
F's verdict on them; Gate F names them explicitly as out-of-scope exclusions,
not as failed or deferred work items.

No code or ticket-file changes are made by this reconciliation; it is a
paperwork cross-reference only.
