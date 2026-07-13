# v26.7.13 — Product Requirements Document

Status: DRAFT, tied to `RELEASE_CONTROL.md` (single control surface). Every claim in this
document cites a file, test, or receipt in this repository. Rows without evidence are marked
PLANNED or UNKNOWN, never asserted. This release reconciles eight independently shipped
surfaces (Themes A–H below) against the repo's no-overclaiming vocabulary
(`.claude/rules/no-overclaiming.md`) — see `## Claims Reconciliation` below. v26.7.12 has no
release doc of its own; `docs/jira/v26.7.12/CROWN_STATUS.md` is cited directly rather than
duplicated here. `docs/jira/v26.7.13/TRAJECTORY_FAILURE_PROCESS.md` is a separate,
already-existing artifact this document links to rather than collides with.

## Claims Reconciliation

Every shipped-work claim for v26.7.13 is reconciled below against this repository's
evidentiary vocabulary (`.claude/rules/no-overclaiming.md`). This table is the single source
of truth for claim status; narrative sections elsewhere in this document must not assert a
status stronger than the row below. Status vocabulary: **ALIVE** (verified, executes, cited
test/receipt passes), **PARTIAL** (real but narrower than the claim — gap named explicitly),
**PLANNED** (roadmap/ticket only, no code path), **UNKNOWN** (not yet investigated to a
verdict), **MOCKED** (a stand-in exists where the claim implies the real thing).

| # | Claim | Status | Scope / caveat | Evidence | Ticket |
|---|---|---|---|---|---|
| 1 | Theme A — Crown-witness composition continuation (LOCAL/EXTERNAL) | PARTIAL | LOCAL 9/11 `REAL_EDGE` + 2 `PARTIAL_REAL_EDGE` (`F08→F09`, `F18→F19`); EXTERNAL `MISSING_EDGE_COUNT`=0 but `F10→F12` `PARTIAL_REAL_EDGE`; union 20 REAL / 3 PARTIAL of 23 distinct edges. `LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` = **false**, `EXTERNAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` = **false**, `OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` = **false** — stated as-is, never rounded up. Repair R8 (`F08→F09`, `F18→F19`, `F10→F12`) unstarted. | `docs/jira/v26.7.12/CROWN_STATUS.md` (executive-summary table); LOCAL driver `crown_local::drive_local_witness_prefix` (commits `3322bf2d`, `d60f2036`, `eeca952a`, `66d8732e`, `0815680a`, `217dc37d`, `66cb59b1`); EXTERNAL tail `crown_external::drive_external_witness_tail` (`4ce20102` `F16→F18`, `1e1ce976` `F18→F20`) | R8 (v26.7.12 repair backlog, unstarted) |
| 2 | Theme B — ~40 production-issue fixes (determinism, typed refusals, mutex poisoning, security/injection) | ALIVE per fix | Count is approximate (~40 across the milestone), not an exact commit tally. This row cites a representative, non-exhaustive sample spanning the four named categories; each cited commit was independently verified this session per its own commit message, not re-verified here. | mutex poisoning: `0767bcc2`, `35d991ca` (recover from poisoned `GLOBAL_ENCODER`/registry locks), `aee200c7` (`r2s_operator` poison regression test); determinism: `967ad485` (DSTREAM deletion order), `4de48d54` (SPARQL `GROUP BY` row order), `f08b4e41` (`sh:closed`/ShEx `CLOSED` order), `89ba964c` (SHACL report order), `bba62242` (empty-ruleset false-positive cycle); security/injection: `035bf0ec`, `f789c6db` (refuse `"""` before Turtle embedding), `4b2774ba` (validate `engine_id`, block path relocation); typed refusals: `46dcee68` (`RSPBuilder::build`), `d01d2904` (CSprite panic), `06ee9f81` (`kh:window` underflow), `213ebcbe` (`substitute_triple_with_bindings`), `64048f43` (`Parser::parse` bounds-check) | UNTRACKED (informal swarm findings #3–#33, no single PROJ ticket covers this bucket) |
| 3 | Theme C — ggen 26.7.13 (`extra_ontologies`, `lock:bool`, pack migrations, GGEN_PARITY.md) | ALIVE | `extra_ontologies` mechanism + first consumer (`togaf-adm-pack`) ALIVE; `lock:bool` opt-out ALIVE; version bump to 26.7.13 + clean sync ALIVE — all three superseding `docs/GGEN_PARITY.md`'s own BLOCKED/PARTIAL disclosure, which reflected an earlier, since-resolved session state. `GGEN_PARITY.md`'s disclosed `EXPECTED_FACTORY_HEAD` pinning risk remains open (Sec. 12, register item). | `4f1c4428` (`lock:bool`; `pack_e2e` 12/12, `ggen_toml_schema_match` 5/5, `graph_config_test` 8/8, `ggen_toml_semantic_validation` 7/7); `5639123b` (version bump, clean `ggen sync run`, `ggen 26.7.13` installed live, idempotent second run, adversarial fail-closed tamper check restored); `89279221` (3-pack `extra_ontologies` migration, sync clean and idempotent); `6342e2d6` (`docs/GGEN_PARITY.md`, 5-agent survey verdict doc) | #102–#105, #111 |
| 4 | Theme D — TOGAF ADM increment 1 | ALIVE, scoped to increment 1 only | Increment 1 (togaf-adm-pack + phase fixtures + full-cycle projection test) only; increments 2 and 3 are zero-commit PLANNED (Sec. 12). `togaf.rs`'s own module doc stamps `(v26.7.13)` though only increment 1 has landed — the ticket status below is authoritative, not the doc-comment stamp. | `c0e7cd71` (TOGAF ADM full-lifecycle increment 1 + ggen `extra_ontologies` unions; `ggen-test-isolated togaf1 pack_e2e` 10/10 incl. 2 new, `graph_config_test` 8/8, chicago `test!` 4 new, cng lib 173/173); `67106c39` (module-doc private-namespace undercount fix, dogfood w6a5iv25g) | #99 |
| 5 | Theme E — SOC2 testbed stages 1–4 + Arclight rescale + hook-actuation tests | PARTIAL, explicitly not a compliance claim | This is a workflow-domain-modeling exercise, not a SOC2 attestation of any system, including this repo — see Doctrine Sec. 6 for the compliance-overclaim fence. Stage 2 disclosed a concurrent-edit collision against the live shared tree (176/3 vs. 179/0 isolated). Stage 3's 3 full-lib failures were pre-existing (Arclight-naming drift), not a Stage-3 regression, resolved by the reconciliation commit. | Stage 1 `756a2584` (`soc2` filter 4/0, full lib 177/0); Stage 2 `3909f89d` (`bench::soc2` 5p, `bench::roles` 2p, `bench::hooks` 5p; isolated 179/0, live tree 176/3 — 3 unrelated pre-existing fixed forward same commit); Fortune-5 rescale `f8c9e3dd`; Stage 3 `fd504676` (`soc2_growth` filter 5/0, full lib 181/3 at Stage-3 `HEAD`); Arclight reconciliation `8f461232` (isolated `bench::soc2` 10/0, full lib suite **184/0**, background task `b02woqay0`); fence doc `2a9f90a1`; stale-claims correction `7b6a08e0` (dogfood wl8n77q65) | #107–#119 |
| 6 | Theme F — `materialize` `catch_unwind` hardening | ALIVE, scoped | Hardens a panic path with no currently-demonstrated production trigger — proactive, not a reproduced-crash fix. The follow-on `verdicts`-not-restored bug (a real, demonstrated gap) is fixed in the same theme's second commit. `Binding::len()`'s HashMap-iteration-order column-length issue (`bindings.rs:22`) was found but reachability via real query-constructed bindings is unconfirmed (open disclosure item). | `70f2faba` (`catch_unwind` rollback hardening; full `--lib` suite isolated 428/0, 7 ignored); `bf982815` (restores `verdicts` in both rollback arms + `strip_comments` escaped-quote fix; `materialize_clears_verdicts_on_rollback_not_just_triple_index` 1/0; `strip_comments` 4/0; full `--lib` 433/0, 7 ignored) | #120, #121 |
| 7 | Theme G — gap-closure coverage commits | ALIVE, narrow scope | Test/disclosure coverage only, no new features. 6-commit fixer-candidate batch: two close reachability gaps with new tests, one discloses a target as structurally untestable rather than fabricating coverage, one retracts a stale doc claim rather than leaving it uncorrected. | `d0372c67` (`ProfileSymbolTable::project_quad` round-trip); `d969119a` (`consult_breed`/`into_authority` under `--all-features`); `74b63c63` (`ReplayMismatch::ReplayRefused` in `chatman_receipts_chain`); `f5b59a56` (F04 `IoRefused` disclosed structurally untestable); `3e582bf9` (F07 `ShexValidatorError` reachability closed; `ShaclValidatorError` disclosed open); `581db137` (retract stale bracket-workaround claim in `project_residue` doc) | #106 |
| 8 | Theme H — arazzo Erlang fixes | ALIVE, scoped to finding #13's siblings | Covers finding #13's OTP-runner broker-dispatch, OTP-runner reaction-dispatch, and AtomVM transition siblings. A fourth, structurally identical sibling (`loop_waiting_for_io/2`) is deliberately left unwrapped as confirmed dead code, not overlooked. Task #85's originally suspected `arazzo_runner_broker_test` hang is NOT supported by the repo's own test record and is closed as unsupported-by-record, not fixed. `admit_return/3` remains a disclosed, un-fixed production dead end (broker-path completions never re-enter `air_core:transition`). | `2966330f` (`apply_transition/4` broker-dispatch catch; `erlang-test-workflow` 5 tests/1 pre-existing unrelated failure); `f22c1db4` (AtomVM transition catch; `erlang-test-atomvm-workflow` 3/0, `erlang-test-atomvm-differential` 4/0); `3d37cf24` (`handle_reaction/3` catch-all; 5 tests/1 pre-existing unrelated failure); `df634bb8` (3 `dispatch_event` coverage gaps, dogfood w6a5iv25g) | finding #13 (informal); #85 (closed-as-unsupported) |

## 1. Product summary

v26.7.13 is a hardening-and-disclosure release across eight independently shipped surfaces
(Themes A–H above): a continuation of the v26.7.12 crown-witness repair with both contiguity
booleans corrected down to their honest **false** value; a representative basket of ~40
production-issue fixes spanning determinism, typed refusals, mutex poisoning, and
security/injection; `ggen` reaching 26.7.13 parity on its `extra_ontologies` and `lock:bool`
mechanisms; TOGAF ADM increment 1; a SOC2 audit-engagement case study (stages 1–4, explicitly
not a compliance claim); `TripleStore::materialize` panic-rollback hardening; a 6-commit
gap-closure coverage batch; and four Erlang crash-safety fixes in `arazzo_runner`/
`arazzo_atomvm`. It ships no new externally-facing product surface. Every claim below is
scoped to the file, commit, or test that grounds it — see `## Claims Reconciliation`.

## 2. Narrative frame

This release is a repair-and-disclosure pass, not a new-capability launch. Its anchor artifact
is `docs/jira/v26.7.12/CROWN_STATUS.md`: an earlier session incorrectly marked the LOCAL crown
witness `true`; two rounds of independent re-audit corrected it to **false**, with the honest
count (9/11 `REAL_EDGE`, 2 `PARTIAL_REAL_EDGE`) carried forward rather than rounded. The other
seven themes follow the same discipline — a mock, a stale claim, or an unfixed dead end is
named as such (Theme E's SOC2 fence, Theme H's `admit_return/3` dead end, Theme F's disclosed
`Binding::len()` gap) rather than silently omitted from the record.

## 3. Customer problem

Systems accumulate small, real defects — panics on malformed input, non-deterministic
iteration order, unguarded mutex poisoning, injection surfaces in string-embedding paths —
faster than any single audit pass catches them. The "customer" for this release is this
repo's own future maintainer and auditor sessions, who need a disclosed, cited basket of
fixes (Theme B) and a corrected crown-witness status (Theme A) rather than a silent diff or a
rounded-up headline number.

## 4. Product position

**Eight independently verified surfaces — explicitly not a unified new capability.** Six
things are out of scope for this closure and must be disclosed before any confidence-building
narrative: TOGAF increments 2 and 3, the ratified Rust-only forward architecture (canonical
`ArchitectureSnapshot` carrier, `SearchOutcome` algebra, differential promotion gate,
`PlanWitness`/`plancheck`), crown-witness repair R8, the `Binding::len()` HashMap-order
column-length issue, 67 pre-existing `cng` clippy findings, and the pinned
`EXPECTED_FACTORY_HEAD` risk in `run-evidence-pass.mjs`. See Sec. 12.

## 5. Core equation

Chatman Engine remains the concrete realization of μ in the Chatman Equation
`A = μ(O*)`, `R = receipt(A)` (`docs/CHATMAN_EQUATION.md`) — `O*` the admitted observation
graph, `μ` the lawful manufacturing transformation, `A` a standing-bearing artifact, `R` a
receipt proving consequence. No v26.7.13 theme modifies this equation or the S1–S6 engine
pipeline; Themes A, E, F, and G touch machinery adjacent to μ's inputs and receipts (crown
witnesses, hook actuation, `materialize` rollback, coverage of receipt-adjacent replay paths)
without changing the equation itself.

## 6. Doctrine

**SOC2 compliance-overclaim fence (verbatim, non-negotiable):** the SOC2 audit-engagement
case study (Theme E) uses only the vocabulary *evidenced*, *exception identified*,
*remediation applied*, *evidence bundle assembled* — never *compliant*, *passed the audit*,
or *SOC2-ready*. The fence is structural, not just prose: the `soc2-audit-pack` PDDL domain
(`packs/soc2-audit-pack/`) has no action anywhere whose effect predicate is `(compliant ?x)`
or `(opinion-issued ?x)`; the only terminal goal atom across all 10 phases is
`(evidence-bundle-complete ?x)`. `Stage 1`'s `verify_no_compliance_or_opinion_effects()`
(`crates/cng/src/bench/soc2.rs`) makes this a mechanical check, not a doc comment — two
adversarial mutants renaming an effect predicate to `audit-compliant`/`auditor-opinion-issued`
both refused typed (`CNG_R05 UnsupportedConstruct`) rather than silently passing
(`docs/SOC2_TESTBED.md`). This same discipline governs every claim in this document per
`.claude/rules/no-overclaiming.md`: forbidden bare phrases ("substantially complete," "should
work," "production-ready" unscoped) do not appear anywhere in this release's docs without a
command-and-output tied to the claim in the same breath. Crown-witness contiguity booleans are
stated exactly as computed (Theme A) — never rounded toward `true`.

## 7. Primary release goal

Land and disclose all eight themes with cited commit and test evidence, close the two
dogfood-audit findings from workflow `wl1quccmk` (Theme F), and close task #85 as
unsupported-by-record (Theme H) — while leaving the ratified Rust-only forward architecture
(canonical carrier, `SearchOutcome` algebra, differential promotion gate, `PlanWitness`)
entirely PLANNED, with kernel-level proof authority explicitly DEFERRED/EXCLUDED per the
ratified design. No forward-architecture item in this release claims ALIVE.

## 8. MVP definition

The MVP is the eight themes, each independently gated:

1. **Theme A — Crown-witness repair continuation.** PARTIAL. Both contiguity booleans false;
   LOCAL improved from a prior 8/11 to 9/11 `REAL_EDGE`; EXTERNAL's `MISSING_EDGE_COUNT`
   reached 0 but `F10→F12` stays `PARTIAL_REAL_EDGE`.
2. **Theme B — Production-issue fix basket.** ALIVE per fix, ~40 commits across four
   categories; representative sample cited, not exhaustively enumerated.
3. **Theme C — ggen 26.7.13 parity.** ALIVE; `extra_ontologies`, `lock:bool`, and the version
   bump to 26.7.13 all committed and live-verified, superseding `GGEN_PARITY.md`'s own
   BLOCKED/PARTIAL disclosure of an earlier session state.
4. **Theme D — TOGAF ADM increment 1.** ALIVE, scoped to increment 1; increments 2/3 PLANNED.
5. **Theme E — SOC2 testbed stages 1–4.** PARTIAL, fenced as non-compliance-claim per Sec. 6.
6. **Theme F — `materialize` hardening.** ALIVE; both the proactive `catch_unwind` wrap and
   the demonstrated `verdicts`-restoration bug are fixed and regression-tested.
7. **Theme G — Gap-closure coverage batch.** ALIVE, narrow scope (6 commits, coverage/
   disclosure only).
8. **Theme H — Arazzo Erlang crash-safety fixes.** ALIVE, scoped to finding #13's three
   siblings plus one coverage commit; task #85 closed as unsupported-by-record.

## 9. Personas

- **Founder-operator.** Needs `docs/jira/v26.7.12/CROWN_STATUS.md` and this document's Claims
  Reconciliation table as single audit artifacts rather than re-deriving standing by hand.
- **AI agent / future session.** Consumes the eight-theme Claims table to avoid re-litigating
  already-settled disclosures (e.g., re-suspecting the arazzo broker-test hang task #85
  already closed as unsupported-by-record).
- **Adversarial reviewer.** Served directly by Theme A's re-audit history and Theme F's
  dogfood-audit-driven fixes (`wl1quccmk`) — both corrected an earlier session's overclaim
  rather than accepting a first-pass self-report.
- **SOC2 case-study reader / compliance stakeholder.** Must be told, not left to discover,
  that Theme E is a workflow-domain-modeling exercise, not a SOC2 attestation of this repo,
  `crates/cng`, or any other system (Sec. 6, `docs/SOC2_TESTBED.md`).

## 10. Functional requirements

| # | Requirement | Evidence surface |
|---|---|---|
| F1 | Crown-witness repair continuation with honest LOCAL/EXTERNAL boolean disclosure | `docs/jira/v26.7.12/CROWN_STATUS.md`; `crown_local.rs`, `crown_external.rs` |
| F2 | Production-issue fix basket (determinism, typed refusals, mutex poisoning, injection) | representative commits, Claims row 2 |
| F3 | ggen `extra_ontologies` union mechanism + `lock:bool` opt-out + 26.7.13 version bump | `crates/ggen/src/config.rs`; `4f1c4428`, `5639123b`, `89279221` |
| F4 | TOGAF ADM increment 1: `togaf-adm-pack` + phase fixtures + full-cycle projection test | `c0e7cd71`; `packs/togaf-adm-pack/` |
| F5 | SOC2 audit-engagement testbed stages 1–4 with structural compliance-overclaim fence | `crates/cng/src/bench/soc2.rs`, `soc2_growth.rs`; `docs/SOC2_TESTBED.md` |
| F6 | `TripleStore::materialize` checkpoint rollback survives both typed errors and panics | `crates/praxis-graphlaw/src/*` (materialize); `70f2faba`, `bf982815` |
| F7 | Gap-closure coverage batch closing named reachability gaps or disclosing them untestable | Claims row 7 commits |
| F8 | Arazzo Erlang crash-safety: broker-dispatch, reaction-dispatch, AtomVM transition catches | `apps/arazzo_runner/`, `apps/arazzo_atomvm/`; Claims row 8 commits |

## 11. Non-functional requirements

1. **Determinism.** Theme B's determinism fixes (DSTREAM, SPARQL `GROUP BY`, `sh:closed`/ShEx
   `CLOSED`, SHACL report order) each carry their own before/after ordering fix, not a
   headline benchmark claim.
2. **Typed refusal completeness.** Theme B's refusal-taxonomy fixes replace `.unwrap()`/panic
   sites with named `Refusal`/`CngRefusal`/Erlang-tuple variants; Theme H's Erlang fixes use
   the equivalent catch-and-disclose idiom translated to untyped message passing.
3. **No wall clock in receipt/hash paths.** Unchanged by this release; no theme touches
   `SystemTime`/`Instant::now` inside chatman receipt or hash code.
4. **Panic-safe rollback.** Theme F: `TripleStore::materialize`'s checkpoint restore now
   covers both the typed-`Err` arm and a `catch_unwind`-wrapped panic arm, and both arms now
   clear `verdicts` (the field the original rollback comment overclaimed as restored).
5. **Compliance-overclaim fence.** See Doctrine Sec. 6, verbatim and non-negotiable —
   duplicated here only by reference, not restated, to avoid drift between the two sections.

## 12. Out of scope

1. **TOGAF ADM increments 2 and 3** — tickets #100 (`ea-adm` bench category + roles +
   `meridian-adm` bundle) and #101 (F09 recursion + crown witness + v26.7.13 docs). Zero
   commits against either ticket this release; both PLANNED.
2. **The ratified Rust-only forward architecture** — canonical `ArchitectureSnapshot`
   carrier, truthful `SearchOutcome` algebra, dependency-footprinted semantic caching,
   Datalog specialization behind a differential promotion gate, `TraceEq`-guarded search
   reduction, six-obligation cross-slice composition, `PlanWitness`/`plancheck` verifier.
   Every item is PLANNED or UNKNOWN per the ARD, which this document does not itself detail;
   kernel-level proof authority is explicitly DEFERRED/EXCLUDED by the ratified design.
3. **Crown-witness repair R8** — `F08→F09`, `F18→F19`, `F10→F12` remain `PARTIAL_REAL_EDGE`;
   repair unstarted (Claims row 1).
4. **`Binding::len()` HashMap-iteration-order column-length issue** (`bindings.rs:22`) — found
   during Theme F's investigation, reachability via real query-constructed bindings
   unconfirmed; open, not fixed this release.
5. **67 pre-existing `cng` clippy findings** (workday/measurement/`otel_*`/runner +
   `jira_routes` formatting) — pre-existing debt, untouched by this release.
6. **`EXPECTED_FACTORY_HEAD` pinning risk** in
   `clients/autonomic-platform/tests/run-evidence-pass.mjs` — disclosed by
   `docs/GGEN_PARITY.md`, still pinned against a chain head that Theme C's clean sync has
   since moved; not re-pinned this release.
7. **tier2/tier3 `knowledge_hooks_e2e` residual failures** — `test_b6_multi_strata_evaluation`
   is fixed by Theme F's `strip_comments` correction, but a separate hook/rule-interleaving
   gap remains (hook-derived facts not visible to N3 rules within the same `materialize()`
   call); tier3's three named failures (`test_c3_construct_empty_no_receipt`,
   `test_c3_datalog_construct_delta_cascade`, `test_c3_threshold_count_window_concurrency`)
   are named and confirmed unrelated to Theme F's fix, root causes not investigated this
   release.
8. **Mutation testing and line coverage** — not run this release; no claim made either way.

## 13. Day-one finish plan

1. Investigate the tier2 hook/rule-interleaving gap (Sec. 12 item 7) — a multi-file change
   touching the stratifier and/or hook-evaluation loop placement, deliberately deferred out
   of Theme F's scope.
2. Start crown-witness repair R8 against the three named `PARTIAL_REAL_EDGE` edges, or record
   why it remains deprioritized relative to other v26.7.13 work.
3. Decide whether to open TOGAF increments 2 (#100) and 3 (#101), or explicitly re-park them
   for a later milestone.
4. Re-pin `EXPECTED_FACTORY_HEAD` in `run-evidence-pass.mjs` against the post-Theme-C chain
   head, or document why the frozen v26.7.6 evidence harness is out of scope for that update.
5. Triage the 67 pre-existing `cng` clippy findings into fix-forward commits or an explicit
   deferred-debt ticket.

## 14. Acceptance criteria

Each row's status is the literal Claims Reconciliation disposition, not a paraphrase.

| # | Criterion | Proof required | Status |
|---|---|---|---|
| 1 | Theme A — crown-witness repair continuation | `docs/jira/v26.7.12/CROWN_STATUS.md` booleans, both false | PARTIAL |
| 2 | Theme B — production-issue fix basket | representative commits verified per own message | ALIVE per fix |
| 3 | Theme C — ggen 26.7.13 parity | `4f1c4428`, `5639123b`, `89279221` live-verified | ALIVE |
| 4 | Theme D — TOGAF ADM increment 1 | `c0e7cd71`, 10/10 + 8/8 + 4 new + 173/173 | ALIVE, scoped |
| 5 | Theme E — SOC2 testbed stages 1–4 | Stage commits + fence check, `docs/SOC2_TESTBED.md` | PARTIAL, fenced |
| 6 | Theme F — `materialize` hardening | `70f2faba` 428/0, `bf982815` 433/0 | ALIVE |
| 7 | Theme G — gap-closure coverage batch | 6 commits, Claims row 7 | ALIVE, narrow |
| 8 | Theme H — arazzo Erlang fixes | 4 commits, Claims row 8; #85 closed-as-unsupported | ALIVE, scoped |
| 9 | Forward architecture / TOGAF increments 2–3 | zero commits against #100/#101 | PLANNED/GAP — does not invalidate rows 1–8 |

Verdict: eight themes independently ALIVE or PARTIAL as scoped above, with all forward-looking
and unstarted work (R8, TOGAF increments 2/3, the ratified Rust-only architecture) held to
PLANNED — no row in this document rounds up.
