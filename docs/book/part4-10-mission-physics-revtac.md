# Mission Physics & RevTAC

This chapter covers two documents that describe the same seam from opposite
sides: `docs/MISSION_PHYSICS.md` describes the domain-independent substrate,
and `docs/REVTAC.md` describes the authoring language layered on top of the
revenue instantiation of that substrate. Read together, they show how one
generic pipeline function gets reused verbatim across institutions that share
nothing in their subject matter.

## The substrate: one function, two institutions

The core claim of Mission Physics is that a "mission" — an operator's
declarative request for the highest-value lawful next move — is answered by a
single generic function, `mission::run_pipeline`, instantiated over a `Pack`
type (`docs/MISSION_PHYSICS.md:13`). The repository verifies this with
`src/mission.rs`, where `run_pipeline` is a generic function over a `Pack`
type parameter:

```
pub fn run_pipeline<P: Pack>(
```

confirmed at `src/mission.rs:258`. A second generic entry point,
`pub fn ceiling<P: Pack>(state: &P::State) -> Value`, exists at
`src/mission.rs:411`, and a third function bearing directly on the
substrate's central lawfulness claim, `pub fn evidence_gate_agrees<P: Pack>`,
sits at `src/mission.rs:159`.

According to `docs/MISSION_PHYSICS.md:17-21`, exactly three things vary
between a revenue mission and a church mission: the `Pack`'s ontology
(accounts and pipeline-stage evidence like `legal_approved` for revenue, vs.
people and assimilation-stage evidence like `welcomed` for church), the
authored objective JSON, and the observed state snapshot. Everything else —
proposer, scorer, ranker, blake3 hasher, PDDL planner adapter, the admission
gate, and the receipt chain — is one code path reused verbatim
(`docs/MISSION_PHYSICS.md:23-25`). The document points to `tests/two_domains.rs`
as the proof: one assertion loop run against two packs, checking that the
step-key sets produced by each are equal (`docs/MISSION_PHYSICS.md:26-27`,
`docs/MISSION_PHYSICS.md:52-53`). That file exists in the repository at
`tests/two_domains.rs`.

## The mission language

Both packs are driven by the same two-verb command surface
(`docs/MISSION_PHYSICS.md:31-34`):

```
mission run     --pack <revenue|church> --objective <path> --state <path> [--mission <name>] [--ts-ns <ns>]
mission ceiling --pack <revenue|church> --state <path>
```

`mission run` compiles a mission down to a substrate invocation and executes
the full pipeline: observe → propose → goal → plan solve → law
judge/law admit → law receipt (`docs/MISSION_PHYSICS.md:36-41`). `mission
ceiling` computes the pack's Maximum Reachable objective, a generalization of
Maximum Reachable Revenue (MRR) covered below. Both commands emit
observations (O), never authority (O\*) — the document is explicit that the
authority lives in the admission gate the pipeline calls, never in the
mission document itself, citing architecture rule AR-9
(`docs/MISSION_PHYSICS.md:44-47`).

The document walks the two invocations side by side. A revenue mission run
against `revenue_objective.json` and a `RevenueState` snapshot produces a
transcript with keys `step_2_top_goal`, `step_3_plan`, `step_4_admissions`,
and `step_5_receipt` (`docs/MISSION_PHYSICS.md:65-86`). A church mission run
against `church_objective.json` and a `ChurchState` snapshot produces a
transcript with the identical key shape — same steps, same admission-field
structure — but with people and assimilation stages in place of accounts and
pipeline stages, and hospitality evidence (`welcomed`, `followed_up`,
`in_small_group`, `care_assigned`) in place of dollar evidence
(`docs/MISSION_PHYSICS.md:98-124`). The document's own framing: "the
substrate did not change; it was never told which institution it was
serving" (`docs/MISSION_PHYSICS.md:124`).

## RevTAC: the authoring layer above the revenue pack

`docs/REVTAC.md` describes the layer immediately above the revenue
instantiation of this substrate. RevTAC ("Revenue Task-And-Constraint") is
where an operator authors a mission document instead of writing PDDL
directly (`docs/REVTAC.md:3-9`). It sits above the prior "Revenue Physics"
phase's pipeline (`propose revenue` → PDDL goal → `plan solve` → `law admit`
→ receipt) and compiles a small declarative document down to a proposer
invocation plus a planner goal atom. The document is explicit about
provenance discipline here too: everything RevTAC emits is observation, not
authority, under AR-9 — a compiled mission is a ranked proposal set, a
planner goal, and a reachable-revenue ceiling, none of which grant permission
on their own (`docs/REVTAC.md:11-14`).

The verb is `propose mission --payload '<json>'` (`docs/REVTAC.md:16-19`).
The payload always carries an observed state plus a mission, either inline or
loaded from a file — exactly one of `mission` / `mission_file` must be
present, and a `.toml`-suffixed file is parsed as TOML while anything else is
parsed as JSON or auto-detected by a leading `{` (`docs/REVTAC.md:21-31`).

A mission document has three parts (`docs/REVTAC.md:33-51`):

| field | meaning |
|-------|---------|
| `mission` | free-form intent name, echoed into the compiled output so a downstream receipt can bind which mission produced a proposal |
| `constraints.min_evidence` / `constraints.exclude_accounts` | scope filters — accounts must carry every listed real `Account` evidence field, or are dropped outright by id; an unknown evidence-flag name is a hard error, never silently ignored |
| `objective` | a path to an authored objective JSON file, or an inline objective object — RevTAC never invents this; a mission with no objective is a hard error |

The document claims TOML and JSON missions compile to byte-identical output,
naming a specific test: `toml_and_json_missions_compile_identically` in
`src/revtac.rs` (`docs/REVTAC.md:65-66`). Checking the source directly, the
function that implements the compilation itself is at
`src/revtac.rs:183` (`pub fn compile_mission(mission: &Mission, state:
&RevenueState) -> Result<Value, String>`), and the named test function exists
at `src/revtac.rs:276` (`fn toml_and_json_missions_compile_identically`).
Both citations in the doc check out against the file.

## What compilation produces

A compiled mission is a JSON document reporting: `status` (`"compiled"` or
`"no_lawful_candidates"` when scope is empty), the echoed `mission` name and
`objective`, the applied `constraints`, `accounts_considered`,
`accounts_dropped` (each with a reason string), the top proposal's PDDL atom
as `planner_goal`, a `top_proposal_hash` binding the compiled goal back to the
substrate receipt, the full ranked `proposals` list, and an `mrr` block
giving the reachable-revenue ceiling for the constrained scope
(`docs/REVTAC.md:70-91`). `planner_goal` is stated to be the exact line an
operator splices into a `plan solve` problem's `(:goal …)` block over
`ontology/revenue.pddl` (`docs/REVTAC.md:87-88`).

The document works through two examples. In the first
(`docs/REVTAC.md:95-135`), a mission constrained to `min_evidence:
["legal_approved", "security_review_done"]` and `exclude_accounts:
["acct-fresh"]` reduces a three-account state to one considered account
(`acct-apex`), with the other two accounts named and reasoned in
`accounts_dropped`, and an `mrr.max_reachable_revenue_cents` of 5,000,000 —
only the surviving account's value.

In the second (`docs/REVTAC.md:137-169`), a mission requires
`exec_sponsor` evidence while simultaneously excluding the only account that
has that evidence — a lawful but self-defeating scope. The document is
careful to distinguish this from an error: `status: "no_lawful_candidates"`
is an `Ok` result (matching what it calls the CLI's "domain denial is `Ok`"
convention), with `planner_goal` and `top_proposal_hash` both `null` and
`mrr` at zero (`docs/REVTAC.md:156-162`). This is contrasted with two cases
RevTAC does treat as hard, up-front errors because they are authoring
mistakes rather than domain findings: an unrecognized `min_evidence` flag
name, and a mission with no `objective` at all (`docs/REVTAC.md:164-169`).

## The Maximum Reachable objective (MRR, generalized)

Back in Mission Physics, `mission ceiling` generalizes Maximum Reachable
Revenue into a pack-independent computation: "if every entity that could
lawfully advance did, how much realized mission value would exist?"
(`docs/MISSION_PHYSICS.md:128-130`). The document states two properties: the
ceiling is objective-independent (authored weights change ranking, never the
ceiling itself), and it respects evidence gates — it only maximizes over
targets that are lawfully reachable (`docs/MISSION_PHYSICS.md:131-134`).

For revenue, the ceiling fluent is `realized_revenue`, and the document
claims the generic `mission::ceiling::<RevenueDomain>` reproduces the
bespoke `praxis_proposer::maximum_reachable_revenue` headline numbers
exactly, citing `tests/two_domains.rs::revenue_ceiling_equals_bespoke_mrr`
(`docs/MISSION_PHYSICS.md:136-139`). For church, the ceiling fluents are
`people_connected` and `care_completion_rate`; cost/process fluents such as
`volunteer_capacity_used` are explicitly excluded from the ceiling because it
bounds value, not score (`docs/MISSION_PHYSICS.md:140-143`). Sample output
for both packs shows the same shape: `max_reachable_value`,
`already_realized_value`, `opportunity_value`, `utilization`, and a per-entity
list where blocked entities carry a `blocked_on` list of the exact missing
evidence (`docs/MISSION_PHYSICS.md:145-158`).

## The cost of a third institution

Mission Physics closes with what it frames as the real payoff of the
substrate/ontology split: adding a third institution costs exactly four
things — a `Domain` impl (ordered stages, an evidence gate, mission fluents),
an authored objective JSON, a hand-authored PDDL8 domain for the same stages,
and a `Pack` impl in `src/mission.rs` wiring the PDDL/admission surface
(`docs/MISSION_PHYSICS.md:169-176`). No proposer, scorer, ranker, hasher,
planner, admission, or receipt code needs to be written, because
`mission::run_pipeline::<NewPack>` and `mission::ceiling::<NewPack>` already
exist as generic functions (`docs/MISSION_PHYSICS.md:178-181`).

## The boundary that holds across every pack

Both documents converge on the same invariant, stated in Mission Physics as
AR-9: every proposal a mission emits is an observation, never authority. It
must pass `law judge` / `law admit` before any effect, and the receipt binds
back to the admitted proposal's `proposal_hash` so which proposal was
admitted stays provable, in every institution
(`docs/MISSION_PHYSICS.md:185-190`). The document names a specific seam
invariant, `mission::evidence_gate_agrees::<P>`, which it says proves the
proposer's lawfulness pre-filter and the admission gate never disagree, for
every entity × stage, in every pack (`docs/MISSION_PHYSICS.md:189-190`) — a
function confirmed present at `src/mission.rs:159`. The chapter's closing
line in the source document is worth preserving verbatim: "A church proposal
is a suggestion for a human on the welcome team to weigh — never an
instruction, and never authority over a person" (`docs/MISSION_PHYSICS.md:191-192`).
