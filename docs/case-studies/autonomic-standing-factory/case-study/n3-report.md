# N3 judgment rules report

Status: FINAL for this lane's run. Real numbers from
`final_graphlaw_verdict.json` (merged graph: seed +
`target/praxis-standing/standing.ttl`, all 3 materialize() passes
combined, all 4 SHACL shapes + ShEx conforming, 0 denials):

- `derived_triple_count`: 22
- `denials`: `[]`
- `raw_verdict_fact`: `NotReadyWithReasons`
- `verdict`: `GRAPHLAW_JUDGED_NOT_READY_WITH_RECEIPTED_REASONS`

This is the correct, honest verdict: `praxis:StandingInputsValid` DID
derive this run (all 3 gate facts — `ShaclShapesConform`,
`ShexSchemaConforms`, `NoDenialsFound` — were injected as true, and
`EvidenceComplete` derived from Lane 1's real evidence), but
`praxis:ProcessEvidenceValid` and `praxis:ClientDisplayValid` did not
(Lanes 3-4-5's OCEL/wasm4pm/client artifacts are still the
`praxis:NotYetProduced` placeholders), and `praxis:NoTerminalBlockers`
did not derive either (10 of 15 acceptance criteria are unsatisfied per
`readiness.dl.n3`'s closure, several marked critical) — so
`ProductionReadyForDeclaredScope` and `PilotReadyWithExternalSideEffects`
both correctly failed to derive, and `NotReadyWithReasons` is the one
verdict class SPARQL found on the case-study subject.

---

## Preliminary stage (superseded by the numbers above)

Status: PRELIMINARY — see `shacl-report.md`'s header note. Superseded by
`graphlaw_judgment_report.md` once `src/bin/case_study_judge.rs` runs.

## What was checked at this stage

`case-study/rules/judgment.n3` loads via `TripleStore::load_rules` without
a stratification or safety error, combined in the same rule set as
`case-study/rules/readiness.dl.n3` (both loaded into one store, per
`TripleStore::add_rules`'s "re-stratify the combined rule set on every
load" behavior).

Real `materialize()` run against `graphlaw_judgment.ttl` alone (seed graph
only, no `standing.ttl` merge, no bin-injected SHACL/ShEx/denial gate
facts yet):

- 20 new triples derived
- `check_denials()` returned zero denials — the seed graph's one promoted
  claim (`praxis:ClaimCanonicalStandingEvidenceExists`) carries real
  evidence, so the `=> false.` denial rule correctly does not fire
- Verdict facts found via direct SPARQL, one class per verdict:
  - `praxis:ProductionReadyForDeclaredScope`: 0 rows
  - `praxis:PilotReadyWithExternalSideEffects`: 0 rows
  - `praxis:NotReadyWithReasons`: 1 row (the case-study subject)
  - `praxis:HasTerminalBlocker`: 1 row (criteria 6-15 are unsatisfied and
    several are marked `praxis:critical true` — Lanes 3-7 have not run)
  - `praxis:NoTerminalBlockers`: 0 rows
  - `praxis:EvidenceComplete`: 1 row (Lane 1's real standing evidence)
  - `praxis:StandingInputsValid`: 0 rows (the `ShaclShapesConform` /
    `ShexSchemaConforms` / `NoDenialsFound` gate facts are injected by the
    judge bin only after it actually runs those checks — they are not yet
    present in the seed graph alone)

This is the expected, honest state given only Lanes 1-2 have run:
`NotReadyWithReasons` is the only verdict fact derivable right now, and it
derives from real unsatisfied-dependency structure, not a hardcoded
string. See `graphlaw_judgment_report.md` for the same numbers computed
inside the actual judge bin against the FULL merged graph (seed +
`target/praxis-standing/standing.ttl`) plus the injected gate facts.

## Design rationale

Verdict facts are asserted as `rdf:type` class memberships
(`?cs a praxis:<Verdict>`), never through a shared literal-valued
predicate — see `judgment.n3`'s own header comment for why: this crate's
Datalog stratifier (`crates/praxis-graphlaw/src/datalog.rs::relation_of`)
groups ordinary triples by predicate but `rdf:type` triples by object
class (the standard Datalog-over-RDF heuristic). Three mutually-exclusive
verdicts sharing one predicate would collapse into a single stratification
relation and make the negation between them an unstratifiable self-loop;
typing each verdict as its own class avoids that entirely.

The `ExternalSideEffectsSeparated` rule needed one extra derivation step
(`HasBlockingSideEffect`) rather than negating an unbound blocking side
effect directly — the engine's safety check requires every variable in a
negated literal to be bound by some positive literal in the same rule, so
"no side effect of this case study is blocking" is expressed as
"not (∃ a side effect derived as HasBlockingSideEffect for this ?cs)",
with `?cs` the shared, positively-bound variable.
