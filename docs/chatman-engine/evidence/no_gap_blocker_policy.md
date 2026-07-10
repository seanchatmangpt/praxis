# No-Gap Blocker Policy (this closure run only)

A gap is closable — must be fixed, not reported around — unless it requires:
- external credentials
- unavailable infrastructure
- destructive user approval (e.g. force-push, hard delete)
- a policy decision genuinely outside this repository

Explicitly NOT nonlocal blockers under this policy: missing snapshot baseline, missing OCEL
log, failing `cargo fmt`/`cargo clippy`, stale ticket status, stale standing index, unrun
gates, missing evidence files, missing local cargo subcommand tooling (installable via `cargo
install`). All of these must be fixed inside this repository before any terminal status is
written.

This policy is stricter than `DEFINITION_OF_DONE.md`'s own Gate F vocabulary, which permits a
`PARTIAL` verdict with named gaps. For this specific closure run, `PARTIAL` is not an accepted
terminal status — only `ADMITTED_DRY_RUN_PUBLISHABLE` or `REFUSED_WITH_NONLOCAL_BLOCKER`.
