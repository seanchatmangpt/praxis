# PROJ-617 — Release closure: RELEASE_CONTROL.md final statuses + DoD sign-off

Status: DONE (delivered: `RELEASE_CONTROL.md` Sec. 8 closure table with consolidated-build
evidence, `docs/releases/v26.7.10/DOD_SIGNOFF.md` 15-clause sign-off, `DOD_EVIDENCE_MAP.md`
query-name sweep, `DEFINITION_OF_DONE.md` Sec. 4/14/15 refresh; `V26_7_10_PRODUCTION_READY`
claimed scoped only — loopback-real, MOCKED-HUMAN, no live network)

## Summary

Final release-doc pass: update `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8 with final
statuses and per-ticket verification evidence lines (command + output, per Sec. 1's rule —
no row upgrades without fresh evidence), and produce a clause-by-clause sign-off against
`DEFINITION_OF_DONE.md`, including the honest loopback-vs-network boundary for external
dispatch (loopback mechanism ALIVE; live third-party endpoints UNVERIFIED and out of scope)
and MOCKED-HUMAN labeling for synthesized human consequences.

## Acceptance criteria

1. Every PROJ-606..622 row in `RELEASE_CONTROL.md` Sec. 8 carries a final no-overclaiming
   status and cites its verification command + output.
2. Every DoD clause (Sec. 1-15) re-statused against the evidence; no clause flips to ALIVE
   without a cited command run in the closing session.
3. Loopback/network boundary and MOCKED-HUMAN labels stated wherever external dispatch or
   human consequences are claimed.
4. `V26_7_10_PRODUCTION_READY` claimed only if PROJ-622's marker conjunction derives true.

## Verification

Doc-only ticket; verification is that every status line in `RELEASE_CONTROL.md` Sec. 8 and
`DEFINITION_OF_DONE.md` resolves to a cited command + output from the closing session.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md`
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 1 and Sec. 8
