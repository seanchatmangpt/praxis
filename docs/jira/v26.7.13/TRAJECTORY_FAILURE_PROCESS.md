# Trajectory Failure Process — Wire-phase-0 Module

Last Updated: <!-- fill in today's actual date -->

`crates/multifractal-workflow/src/trajectory_failure_process.rs` (1468 lines) implements the
t_err/t_lock/t_obs failure-trajectory framework from Zhao, Li, Li, Zhao, Barr, Sarro & Ye,
"Failure as a Process: An Anatomy of CLI Coding Agent Trajectories" (arXiv:2607.09510), and
re-grounds it in this repository's own `git log` history rather than in a live agent trace. It
is a standalone Wire-phase-0 analysis module: no production code calls it, it carries no
`V12-0XX` ticket, and it is **not a crown-witness edge** in the v26.7.12 architecture atlas.
This document states what the module does, what it deliberately does not yet do, and the exact
numbers its own worked example computes — none of the figures below are invented; each is
either read directly from the module's test assertions or independently re-derived from `git
log` this session.

## Quick Reference

- [What This Implements](#what-this-implements)
- [Wire-phase Status: What This Does NOT Do](#wire-phase-status-what-this-does-not-do)
- [Worked Example: The F18→F19 Case Study](#worked-example-the-f18f19-case-study)
- [Known Staleness](#known-staleness)
- [Disclosed Limitations](#disclosed-limitations)
- [Verification Performed](#verification-performed)
- [See Also](#see-also)

## What This Implements

The paper frames every failing agent run around three timestamps:

- **t_err** — the decisive, root-cause error.
- **t_lock** — the point after which no correct recovery is observed.
- **t_obs** — the first observable failure signal.

Two derived intervals follow: `fix_window = t_lock - t_err` (the time recovery was still
possible) and `observability_lag = t_obs - t_lock` (the time the failure was real but
invisible). The paper's headline finding (its F2/F11) is that a fix window usually exists and
usually goes unused — the gap between successful and failed trajectories is whether the agent
*acts* on an observed error signal, not whether one occurs.

This module re-grounds that framework in git history: a **trajectory** is the ordered slice of
this repo's own commit history for one **claim** (e.g. "this call-graph edge is a `REAL_EDGE`"),
zero or more later commits that build on that claim without re-checking it, and an eventual
independent verdict — a fix commit, or an external audit that has not yet produced one — that
confirms or falsifies it. `t_err` is the commit whose claim the verdict falsifies; `t_lock` is
the last commit (among caller-supplied dependents) before the verdict landed; `t_obs` is the
verdict event itself.

## Wire-phase Status: What This Does NOT Do

This is **Wire-phase-0**: the module compiles, has its own test suite, and works correctly
against real repository data, but it is not connected to anything that runs it automatically.
Specifically, independently re-verified this session (not merely taken from the module's own
doc comments):

- `grep -rn "trajectory_failure_process\|FailureTrajectory\|TrajectoryRefusal"
  crates/multifractal-workflow/src/crown_local.rs
  crates/multifractal-workflow/src/crown_external.rs`
  returns **no matches**. Neither crown-witness driver calls into this module.
- The only reference outside the module's own file is `crates/multifractal-workflow/src/lib.rs:100`
  (`pub mod trajectory_failure_process;`), which is required to include the file in the crate
  tree and does not constitute wiring.
- `git status --porcelain -- docs/jira/v26.7.12/CROWN_STATUS.md` shows the module never writes
  to that file; it only *cites* its existing text in doc comments and test fixtures.
- The module carries no `V12-0XX` ticket and asserts no crown-witness edge of its own. It is
  data-flow analysis tooling for auditing existing claims, not a claim generator.
- No CLI command, bench, or scheduled job invokes it. Running it means running
  `cargo test --lib trajectory_failure_process` directly.

Future work (undone, disclosed rather than implied) would be: a caller that re-runs this
analysis automatically when a `CROWN_STATUS.md` verdict changes, and a ticketed home for it in
a milestone atlas if it graduates past Wire-phase-0.

## Worked Example: The F18→F19 Case Study

The module's own `f18_f19_case_study` test (bottom of the file) runs
`FailureTrajectory::compute_failure_window` over real, `git log`-sourced commit records for this
repo's actual F18→F19 overclaim and asserts these exact values:

| Quantity | Value |
|---|---|
| t_err commit | `eeca952a` (2026-07-12T15:44:25-07:00, unix `1783896265`) |
| t_lock commit | `66cb59b1` (2026-07-12T17:08:50-07:00, unix `1783901330`) |
| t_obs (external observation) | unix `1783925118` (2026-07-12T23:45:18-07:00) |
| fix_window_commit_count | 8 |
| fix_window_seconds | 5065 (1h 24m 25s) |
| observability_lag_seconds | 23788 (6h 36m 28s) |
| observability_lag_commit_count | 11 |
| recovery_opportunities | 0 |

The underlying claim: `crown_local.rs:572-580` calls `resolve_hook_for_action(&run.hook_pack_turtle,
&ground_action, &mut hook_ledger)` immediately after `dispatch_local_execution_via_broker`
succeeds and binds `broker_receipt` — real control sequencing (the `?` gates whether the call
runs at all) with zero data threading (none of `BrokerReceipt`'s seven public fields is read
between the two calls). Commit `eeca952a` wired this; two minutes later, `77da318b` classified
the edge `REAL_EDGE` in `CROWN_STATUS.md`. Eight further commits built the rest of the LOCAL
crown-witness chain on that unverified claim before an independent re-audit (task `wqv5aaz7u`,
agent "mid") flagged it as an overclaim. `recovery_opportunities = 0` because no commit after
`66cb59b1` touches `crown_local.rs` again before the audit — every later commit in that window
only documents the separately-wired EXTERNAL witness tail.

## Known Staleness

Independently re-verified this session: the module's doc comment (lines 85-89) and the
`f18_f19_case_study` test (lines 1374-1379) both assert "no fix commit exists for the F18→F19
overclaim yet" and use `VerdictSource::External` with the re-audit's own timestamp
(`1783925118`) rather than `VerdictSource::Commit`. That was true when the module was authored,
but it is no longer true: `git log -1` on `docs/jira/v26.7.12/CROWN_STATUS.md` now shows commit
`8a66ea62028180f4f139eb402e4cdee83f87d0be` (2026-07-13T00:14:09-07:00, unix `1783926849`),
`"docs(v26.7.12): correct F18->F19 and F21->F25 from REAL_EDGE to PARTIAL_REAL_EDGE"` — a real
fix commit for exactly this claim, landing 1731 seconds (28m 51s) after the External
observation instant the test hardcodes. If the module's own worked example were re-captured
today and anchored on that fix commit instead of the external observation, the observability
lag would be 25519 seconds (`1783926849 - 1783901330`), not the 23788 seconds currently
asserted. This is disclosed here rather than silently patched into the module, because fixing
it requires re-running the module's documented `git log` capture command and updating its test
fixtures — a code change outside this documentation task's scope.

## Disclosed Limitations

Carried over from the module's own doc comments, verified as accurate by direct reading:

- **Verdicts are never inferred.** `git log` has no ground truth of correctness; a verdict
  (`Overclaim` vs. confirmed) must come from an external audit, never from commit text alone.
- **Dependencies are never inferred.** Which commits "built on" an unverified claim is
  caller-supplied (`AssumedDependency`), established by direct source reading, not a
  same-file/same-keyword heuristic.
- **Timestamps are self-reported and rewritable** (`git commit --amend`/`rebase` changes author
  dates) — weaker evidence than this repo's BLAKE3 receipt invariant, never presented as
  tamper-proof.
- **t_lock selection is a judgment call**, computed mechanically once the caller supplies the
  dependency list, but which commits belong on that list is a human/audit decision.
- **`StageDataThreading` field lists are curated by direct source reading**, not extracted by an
  AST parser (no `syn`-based extractor exists in this module).
- **No wall clock anywhere in this module.** Every timestamp is caller-supplied: either
  `git log`'s own `%aI`/`%at` fields or an externally-recorded observation instant passed as a
  literal. Grep confirms zero `SystemTime::now()`/`Instant::now()` calls in the file.

## Verification Performed

This session, independently of the module's own claims:

- `wc -l` confirmed 1468 lines.
- `grep -rn "trajectory_failure_process\|FailureTrajectory\|TrajectoryRefusal"
  crates/multifractal-workflow/src/crown_local.rs
  crates/multifractal-workflow/src/crown_external.rs`
  returned no matches, confirming no production caller wires this module in.
- `grep -n "trajectory_failure_process" crates/multifractal-workflow/src/lib.rs` showed exactly
  one hit — the `pub mod` declaration required for compilation, not a call site.
- `git log -1 --format='%H%x09%aI%x09%s' -- docs/jira/v26.7.12/CROWN_STATUS.md` and `git log -1
  --format='%H %aI %s'` both resolved to commit `8a66ea62`, confirming the fix-commit staleness
  noted above.
- The unix timestamp of `8a66ea62` (`1783926849`) was independently recomputed from its ISO
  author date and cross-checked against the 1731-second and 25519-second deltas cited above.

Earlier sessions additionally ran an isolated `cargo check`, an isolated `cargo test` (fresh
target dir, 19/19 passing including `f18_f19_case_study`), and a grep sweep for
`unwrap`/`expect`/`panic!`/`SystemTime`/`Instant::now` outside the `#[cfg(test)]` boundary
(none found in production code); those results are not re-run here and should not be treated as
refreshed by this document.

## See Also

- `crates/multifractal-workflow/src/trajectory_failure_process.rs` — the module itself
- `docs/jira/v26.7.12/CROWN_STATUS.md` — the crown-witness atlas this module cites but does not
  write to
- `docs/jira/v26.7.12/PRD.md` — crown witness definitions (F18, F19, and related edges)
- `.claude/rules/no-overclaiming.md` — the status vocabulary this document follows
- `.claude/rules/autonomous-escalation-policy.md` — crown-frontier commit trailer convention
  (not applicable here: this document does not change any crown-witness edge)
