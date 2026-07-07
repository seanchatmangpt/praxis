# GraphLaw Judgment Model

Real rules and real numbers from Lane 2's SHACL/ShEx/N3/Datalog authoring
plus Lane 6's claim-promotion re-run. Source files:
`case-study/graphlaw_judgment.ttl`, `case-study/shapes/*.shacl.ttl`,
`case-study/shex/case-study.shex`, `case-study/rules/{judgment.n3,readiness.dl.n3}`,
`src/bin/case_study_judge.rs`.

## SHACL (Criterion02)

Four shape files, all targeting real classes asserted in the merged graph:

| file | target class | constrains |
|---|---|---|
| `standing-envelope.shacl.ttl` | `praxis:StandingEnvelope` | `generatedAtUtc` (1, dateTime), `hasEvidence` (>=1), `hasVerdict` (>=1) |
| `case-study.shacl.ttl` | `praxis:CaseStudy` | `hasScope` (1), `hasEvidence`/`recordsExecution`/`validatesProcess` (>=0) |
| `evidence-ref.shacl.ttl` | `praxis:EvidenceRef` | `path` (1, string), `hash` (1, string, pattern `^sha256:[0-9a-f]{64}$`) |
| `final-verdict.shacl.ttl` | `praxis:FinalVerdict` | `status`/`hasScope` (1, string), `hasEvidence` (>=1), `generatedAtUtc` (1, dateTime) |

Real run (`cargo run --bin case_study_judge`, Lane 6): **all 4 conform, 0
violations** (`case-study/shacl-report.json`).

## ShEx (Criterion03)

`case-study/shex/case-study.shex` — 7 shapes (`CaseStudyShape`,
`StandingEnvelopeShape`, `EvidenceBundleShape`, `GraphLawJudgmentShape`,
`ProcessValidationShape`, `FinalVerdictShape`, `PromotedClaimShape`).

Real run: **conforms: true, 0 failures** (`case-study/shex-report.json`).

Lane 6 finding: `CaseStudyShape` constrains `praxis:hasEvidence` to
`@praxis:StandingEnvelopeShape +` — every value of that specific predicate
on the case-study subject must itself look like a standing envelope
(`generatedAtUtc`/`hasEvidence`/`hasVerdict`). When promoting Criteria
10/11/13/14/15's supporting evidence, Lane 6 initially reused
`praxis:hasEvidence` for those refs too and got a real ShEx conformance
failure (`predicate ... hasEvidence value violated its shape`) — correctly
caught by the schema. Fixed by introducing a distinct
`praxis:citesEvidence` predicate for that non-StandingEnvelope-shaped
supporting evidence, since `CaseStudyShape` is not `CLOSED` (extra
predicates are permitted).

## N3 (Criterion04)

`case-study/rules/judgment.n3` derives, as `rdf:type` class memberships
(not a shared `hasVerdict` predicate — see the file's own header for the
stratification-safety reason): `EvidenceComplete`, `StandingInputsValid`,
`ProcessEvidenceValid`, `ClientDisplayValid`, `HasBlockingSideEffect` /
`ExternalSideEffectsSeparated`, and the 3 mutually-exclusive verdict
classes (`ProductionReadyForDeclaredScope`,
`PilotReadyWithExternalSideEffects`, `NotReadyWithReasons`). One `=> false.`
denial rule fires if any `praxis:PromotedClaim` lacks `>=1
praxis:hasEvidence`.

Real run: **0 denials**.

## Datalog (Criterion05)

`case-study/rules/readiness.dl.n3` — transitive closure over
`depends_on`/`blocks`, `requires`/`satisfied` → `hasUnsatisfiedDependency`,
`critical` → `HasTerminalBlocker`, `ready`/`not_ready`,
`claim_promoted`/`claim_demoted`, `external_side_effect`/`non_blocking`/
`BlockingSideEffect`. The unsatisfied-dependency COUNT is computed via the
Rust-only `add_rule_with_aggregate` API (no N3 text-syntax aggregate exists
in this crate).

Real run (post Lane 6 promotion): **`unsatisfied_dependency_count: 0`**
(all 15 criteria in `praxis:requires` are also in `praxis:satisfied`; see
`CLAIM_PROMOTION_TABLE.md`).

## Real bug found and fixed by Lane 6: `verdict_present` never actually read the derived verdict

`src/bin/case_study_judge.rs`'s original `verdict_present` function:

```rust
fn verdict_present(store: &TripleStore, verdict_class: &str, subject_local: &str) -> Result<bool, Refusal> {
    let q = format!("SELECT ?cs WHERE {{ <{NS}{subject_local}> a <{NS}{verdict_class}> }}");
    Ok(!query_first_col(store, &q)?.is_empty())
}
```

The WHERE clause is fully ground (both subject and object are concrete
IRIs) — `?cs` never appears in the pattern and is never bound.
`query_first_col` extracts the FIRST BOUND BINDING per solution row; a row
with zero bindings (since the projected variable never appears in the
pattern) yields `row.into_iter().next() == None`, which `query_first_col`
silently filters out. The net effect: **`verdict_present` returned `false`
unconditionally, for every verdict class, on every call** — the `for v in
VERDICTS { ... }` loop in `run()` never matched anything, and `verdict`
always fell through to its `.unwrap_or("NotReadyWithReasons")` default.

This was never caught because:

- The hardcoded fallback string happened to equal the correct answer for
  every case-study state that existed before Lane 6's evidence promotion
  (the graph never actually reached `ProductionReadyForDeclaredScope`
  before Criteria 6-15 had real evidence, so "always print
  NotReadyWithReasons" looked like an honest, evidence-driven result).
- The existing test suite's own `present()` helper (in the same file's
  `#[cfg(test)] mod tests`) checks `!rows.is_empty()` on the raw
  solution-ROW COUNT directly, not through `query_first_col`'s
  binding-extraction — so it never exercised this exact bug, even though
  it uses a structurally similar ground-pattern query.

Confirmed via direct instrumentation (temporarily added, then removed) that
`verdict_present` returned `false` for all 3 verdict classes even though
`content_to_string()`'s full dump of the same, unchanged store showed
`ProductionReadyForDeclaredScope` genuinely present as a type triple on the
case-study subject at that exact point in the run.

**Second, related finding (disclosed, not fixed in this lane): the
mutually-exclusive verdict classes are not actually mutually exclusive in
practice.** Even after the query fix above, `graphlaw_derived.ttl`'s fully
materialized dump shows the case-study subject typed as BOTH
`NotReadyWithReasons` AND `ProductionReadyForDeclaredScope` simultaneously
— contradicting the file's own header claim that these are "mutually
exclusive by construction." Root cause: `run()` calls `store.materialize()`
in 3 separate stages (pass 1 before the SHACL/ShEx/denial gate facts are
injected, pass 2 after injection, pass 3 after the aggregate rule is
added). In pass 1, none of `StandingInputsValid`'s prerequisites exist yet,
so `ProductionReadyForDeclaredScope`'s body fails — which makes
`NotReadyWithReasons`'s negated body (`not
{ProductionReadyForDeclaredScope} not {PilotReadyWithExternalSideEffects}`)
match, and it gets asserted as a fact. This engine's forward-chaining
`materialize()` is purely additive (rules only ever add triples, never
retract one that was legitimately derivable at an earlier point but whose
justification no longer holds) — so once gate facts land in pass 2 and
`ProductionReadyForDeclaredScope` newly becomes derivable, the stale pass-1
`NotReadyWithReasons` fact is never retracted; both coexist from then on.
The reported verdict is still CORRECT despite this, only because
`VERDICTS` is checked in a fixed priority order
(`ProductionReadyForDeclaredScope` first) and the fixed `verdict_present`
now genuinely finds it — but the underlying graph is not the clean,
single-verdict graph the design intends, and a hypothetical future reader
of `graphlaw_derived.ttl` directly (bypassing `case_study_judge`'s own
priority-ordered verdict selection) would see an apparent contradiction.
Not fixed in this lane: correctly fixing it requires re-ordering when the
gate facts are computed relative to the first `materialize()` call (SHACL/
ShEx target only base, non-derived classes here, so this looks feasible),
which is a real change to `run()`'s control flow beyond a single-function
query fix and warrants its own dedicated test pass rather than a same-lane
opportunistic edit. Flagged for Lane 7 audit / a follow-up ticket.

**Fixed** (the query bug) by mirroring the working `case_study_subjects` query shape a few
lines below in the same file (`SELECT ?s WHERE { ?s a <class> }` — variable
SUBJECT, ground object, so the projected variable is actually bound), then
checking membership of the target subject's local name in the returned
rows:

```rust
fn verdict_present(store: &TripleStore, verdict_class: &str, subject_local: &str) -> Result<bool, Refusal> {
    let q = format!("SELECT ?s WHERE {{ ?s a <{NS}{verdict_class}> }}");
    let rows = query_first_col(store, &q)?;
    Ok(rows.iter().any(|s| s.contains(subject_local)))
}
```

Added a regression test, `verdict_present_finds_a_genuinely_asserted_class`,
that directly exercises the fixed function (not the pre-existing
`present()` helper, which never had the bug). `cargo test --bin
case_study_judge`: **6/6 passed** (5 pre-existing + 1 new). No other
`praxis-graphlaw` source changed; `cargo test -p praxis-graphlaw` and
`cargo test --bin ocel_process_validate` stay green (147+ / 8 passed,
unaffected — this was a bug in the judge binary's own query string, not the
underlying SPARQL engine).

## Real verdict after the fix + claim promotion

```
GRAPHLAW_JUDGED_PRODUCTION_READY_FOR_SCOPE
```

`raw_verdict_fact: "ProductionReadyForDeclaredScope"`,
`unsatisfied_dependency_count: 0`, `denials: []`, all 4 SHACL shapes
conform, ShEx conforms. Confirmed deterministic across 4 independent runs:
`graph_hash` = `blake3:4e1843d2cf5dfc8b12e2ad30e72329ce58a77d1b8c6f7ac255101bec399a6efa`
every time (`case-study/final_graphlaw_verdict.json`).
