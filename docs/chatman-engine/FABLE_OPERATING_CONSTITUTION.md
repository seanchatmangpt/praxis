# Claude Fable 5 Operating Constitution — Chatman Engine v26.7.9

Operating doctrine for long-horizon model-driven runs on this engine. The auditor's verdict is
the only status that matters.

## North Star

The gap-closing bet: every run exists to close the measured gap between what the graph asserts
and what the code, receipts, and auditor can independently verify. Work that does not narrow
that gap — or that widens it with unverified claims — is out of scope by definition.

## Absolute Doctrine

1. **RDF authority** — the RDF graph is the single source of truth. Code, docs, and reports are
   projections of it, never the other way around.
2. **RDFTriple8 is projection-only** — RDFTriple8 is a derived, bounded projection of the graph.
   It never originates facts and is never treated as authoritative storage.
3. **N3 quarantine** — N3 is disabled by default and quarantined. It may only run behind an
   explicit opt-in, never as a default or fallback dialect.
4. **Agents are witnesses** — agents observe and attest; they cannot override profiles,
   admission tables, or graph authority. Witness output is evidence, not authority.
5. **Typed refusals** — every failure is a typed `Refusal` variant. No panics, no silent
   defaults, no untyped error strings.
6. **No self-graded victory** — the agent that did the work may not declare it done. Only an
   independent auditor examining artifacts and receipts can issue a verdict.
7. **Three verdicts** — the only terminal statuses are ADMITTED_DRY_RUN_PUBLISHABLE, PARTIAL,
   and REFUSED. Nothing else counts as a status.

## Exploration Standing (Two-Label Model)

Exploration is unlabeled by design. The explore phase emits unadmitted candidate mass, not
classified truth. The explorer must not be required to distinguish real APIs from fictional
APIs, implemented behavior from desired behavior, or valid requirements from dead branches
during generation.

All explore-phase output enters the system with one envelope status: UNADMITTED_CANDIDATE.
This is not a hallucination label. It is a standing label.

The exploit/admission phase performs stratification. Stratification assigns content labels:
VERIFIED, DRIFTED, STUB, DEAD, HALLUCINATED, MINE, ENGINE-OWNS, UNPROVEN, REFUSED.
"Hallucination" is not an input type. It is a post-audit stratum.

The defect is not hallucination inside exploration. The defect is unstratified candidate mass
crossing into admitted status. Therefore: maximize exploratory coverage; enforce a hard
admission boundary.

Explorers generate. Falsifiers mutate. Exploiters stratify and repair. Auditors assign
standing. No explorer self-label is authoritative. No builder self-grade is authoritative.
No candidate becomes evidence without verification.

Exploration is allowed to be wrong. Admission is not allowed to be vague.

Applied ledger: /Users/sac/chicago-tdd-tools/DFLSS_CAPABILITY_MATRIX.md
(strat(C) = V ∪ M ∪ D ∪ S ∪ R ∪ U over 94 rows, 2026-07-09).

## Fable Runtime Rules

- **Act, don't plan, when informed** — once the work order and constraints are known, execute.
  Planning documents are not progress.
- **Pause only for** destructive operations, irreversible actions, or scope changes. Everything
  else proceeds without asking.
- **Audit every claim against tool output** — a claim with no command output behind it, made in
  the same breath, is unverified and must be labeled as such.
- **Reporting vocabulary** — every item in a report is exactly one of: verified (command +
  output cited), not-verified (stated plainly), or skipped (with reason).

## Work Order

The 19-step sequence, in order:

1. North star
2. TTL
3. Docs/atlas
4. Repo audit
5. Schemas
6. Fixtures
7. Generated cases
8. Falsification
9. Harnesses
10. Initial failures
11. Minimal implementation
12. fmt
13. Tests
14. Static gates
15. Duplicate scan
16. Replay checks
17. Reports
18. Independent auditor
19. Verdict

## Chicago TDD Rule

External fixtures come first — before any implementation exists. Tests use real collaborators;
mocks are permitted only at nondeterministic boundaries (wall clock, network, randomness). No
testing of private implementation details: tests exercise the public surface against fixtures.

## Anti-Fake Requirement

Reject, on sight, any implementation exhibiting:

- return-0 / return-true / return-ok stubs
- constant receipt, constant hash, or constant route
- ignored inputs (output independent of arguments)
- silent clamp or silent fallback
- witness output treated as authority
- any-string-accepted-as-receipt (no structural validation)

## The 8 Anti-Hardcode Properties (P1–P8)

- **P1** — Output varies when inputs vary (no constant outputs across distinct inputs).
- **P2** — Every declared input is load-bearing (perturbing it changes some output).
- **P3** — Receipts replay: recomputing from the same material yields byte-identical receipts.
- **P4** — Receipts falsify: perturbing any receipt input changes the receipt.
- **P5** — Bounds are enforced, not clamped: out-of-range input yields a typed refusal.
- **P6** — Unknown vocabulary is refused by name, never silently skipped.
- **P7** — Routing decisions derive from the graph/profile, never from hardcoded branches.
- **P8** — No test passes vacuously: every generated test carries an unconditional assertion.

## Required Falsification Pairs

Every subsystem ships positive/negative pairs. The required negative cases:

- **Receipt replay**: graph mismatch, profile mismatch, symbol-table mismatch, admission-table
  mismatch, route mismatch.
- **Triple8**: 257 terms, unknown term, profile mismatch, more than 8 constraints.
- **Admission**: missing required bit, forbidden bit set, wrong table length, constant zero
  mask.
- **Routing**: N3 as default, N3 actuation, SPARQL escalation, unsupported dialect.
- **Hooks**: constellation violation, missing receipt, invalid receipt, unadmitted OCEL.
- **Agents**: profile override attempt, authority claim, disabled breed, nondeterministic
  behavior without a receipt.
- **Tape**: duplicate `Pddl8Tape`, duplicate `PowlTape`.
- **Static**: duplicate `ProcessReceipt`, broad allows, forbidden tokens, silent fallback,
  N3 default.

## Canonical Type Boundary

Eight types are forbidden as standalone definitions inside praxis-graphlaw; they must be
imported from their canonical owning crates, never redefined locally. A duplicate definition of
any of these is a static-gate failure, not a style note.

## Progress Claim Rule

No progress claim without a command run this session and its output cited alongside the claim.
Prior-agent summaries, README text, and code comments are hearsay until re-verified.

## Victory Conditions

**ADMITTED_DRY_RUN_PUBLISHABLE** requires all of:

- [ ] All 19 work-order steps executed in order, each with cited evidence
- [ ] All falsification pairs present and passing (positive passes, negative refuses)
- [ ] All 8 anti-hardcode properties (P1–P8) demonstrated
- [ ] Zero anti-fake patterns in the shipped surface
- [ ] Receipts replay byte-identically; perturbation changes them
- [ ] Static gates green: no duplicate canonical types, no forbidden tokens, no broad allows,
      no silent fallback, no N3 default
- [ ] fmt, tests, and duplicate scan all green with output cited
- [ ] Reports written to the canonical paths
- [ ] Independent auditor has examined artifacts and issued the verdict — not the implementer

**PARTIAL** — some gates green, gaps named explicitly with file:line or missing-artifact
citations. **REFUSED** — a doctrine violation, unsupported construct, or failed falsification
that cannot be fixed within scope; refusal is typed and cited.

## Effort Ladder

- **high** — the default for all engine work.
- **xhigh** — CENG-410-FINAL, falsification design, receipt-replay verification, victory audit.
- **medium** — docs-only changes.

## Report Paths

Canonical reports live in `docs/chatman-engine/`:

- `chicago_tdd_acceptance_surface.md` and `chicago_tdd_final_report.md` — as currently produced
  by workflow wf_255e0807.

Per-run artifacts land in `docs/chatman-engine/build/`:

- `fable_run_report.md`
- `chicago_tdd_initial_failures.md`
- `fable_victory_audit.md`

## API Notes

- Model id: `claude-fable-5`.
- Adaptive thinking is always on; thinking-disabled is unsupported.
- `stop_reason: "refusal"` arrives as HTTP 200 and must be handled as refusal semantics, not as
  a transport error.
- 30-day retention, no ZDR — a sensitive-material governance decision is required before
  sending sensitive material.

## See Also

- [North Star](NORTH_STAR.md)
- [Definition of Done](DEFINITION_OF_DONE.md)
