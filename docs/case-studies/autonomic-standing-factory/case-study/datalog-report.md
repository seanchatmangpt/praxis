# Datalog readiness closure report

Status: FINAL for this lane's run. Real numbers from
`final_graphlaw_verdict.json`:

- `unsatisfied_dependency_count`: **10** (computed via
  `TripleStore::add_rule_with_aggregate` with a `Count` accumulator
  grouped by the case-study subject over `praxis:hasUnsatisfiedDependency`
  — Rust-API aggregate, not text syntax, per this file's own header) —
  matches the 10 unsatisfied criteria (`Criterion06`-`Criterion15` minus
  the 5 satisfied ones tracked in `graphlaw_judgment.ttl`'s
  acceptance-criteria block: criteria 1-5 satisfied, 6-15 unsatisfied).
- `praxis:HasTerminalBlocker` derived on the case-study subject (several
  of the 10 unsatisfied criteria — `Criterion06`/`07`/`08`/`09`/`12`/`15`
  — are marked `praxis:critical true`), so `praxis:NoTerminalBlockers` did
  NOT derive this run — consistent with `NotReadyWithReasons` being the
  correct verdict.
- `claim_promoted`/`claim_demoted`, `external_side_effect`/
  `non_blocking`, and the `depends_on`/`blocks` transitive-closure rules
  all loaded and stratified correctly as part of the same combined rule
  set that produced the 22 derived triples reported in `n3-report.md`.

---

## Preliminary stage (superseded by the numbers above)

Status: PRELIMINARY — see `shacl-report.md`'s header note. Superseded by
`graphlaw_judgment_report.md` once `src/bin/case_study_judge.rs` runs.

## What was checked at this stage

`case-study/rules/readiness.dl.n3` loads and stratifies successfully
together with `judgment.n3` (same engine, same `add_rules` call site —
see that file's own header comment: "Datalog here is the same
forward-chaining engine as judgment.n3"). Confirmed by the same
`materialize()` run recorded in `n3-report.md`:

- The `requires`/`satisfied`/`critical` closure over the 15 acceptance
  criteria (`graphlaw_judgment.ttl`'s "Acceptance-criteria dependency
  model" section) correctly derived `praxis:HasTerminalBlocker` on the
  case-study subject, since criteria 6, 7, 8, 9, 12, and 15 are marked
  `praxis:critical true` and are unsatisfied (Lanes 3-7 have not produced
  their evidence yet).
- The transitive-closure rules over `praxis:depends_on` and
  `praxis:blocks` loaded and stratified correctly (no cyclic-negation
  rejection), mirroring this crate's own
  `{?a in ?b. ?b in ?c} => {?a in ?c}` pattern
  (`crates/praxis-graphlaw/src/lib_test.rs::test_transitive_rule`) — no
  facts using these two predicates exist yet in the seed graph, so they
  produced zero derivations this run (present for when a real
  dependency/blocking chain between artifacts is recorded).
- `claim_promoted`/`claim_demoted`: the seed graph's one promoted claim
  derived `praxis:claim_promoted "true"` (it has real evidence) and did
  NOT derive `praxis:claim_demoted` (no claim lacks evidence in this run).
- `external_side_effect`/`non_blocking`: the one recorded external side
  effect (`praxis:ExternalSideEffectPlaceholder`, `nonBlocking true`)
  derived `praxis:external_side_effect "true"` and `praxis:non_blocking
  "true"`, and did NOT derive `praxis:BlockingSideEffect`.

## Aggregation (COUNT) — honest scope note

This file's `.n3` TEXT syntax has no aggregate literal at all —
`praxis_graphlaw`'s N3 parser only ever produces ordinary
`Rule{body,head}` structs from text; `AggregateFunction`/`Aggregate` are
Rust-API-only concepts consumed via `TripleStore::add_rule_with_aggregate`
(see `crates/praxis-graphlaw/src/rule.rs` and `src/lib.rs`). The
unsatisfied-dependency-count fact the control spec asks for is therefore
added programmatically inside `src/bin/case_study_judge.rs`
(`unsatisfied_dependency_count_rule`), not as text in this file — see
`graphlaw_judgment_report.md` for the real count once that binary runs.
