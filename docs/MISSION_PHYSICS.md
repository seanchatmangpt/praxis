# Mission Physics — one substrate, many institutions

*Genesis Day 6, phase 2. Companion to `src/mission.rs`, `src/verbs/mission.rs`,
and the proof test `tests/two_domains.rs`.*

## The claim

A **mission** is one small, declarative request an operator makes of the
substrate: *given what we observe, what is the highest-value lawful move, and
can we reach it?* Mission Physics is the layer that answers it — and it is
**domain-independent**. The revenue pipeline and the church-operations pipeline
are not two programs that resemble each other; they are the **same generic
function** ([`mission::run_pipeline`](../src/mission.rs)) instantiated at two
[`Pack`]s. Exactly three things differ between a revenue mission and a church
mission:

| what differs | revenue | church |
|---|---|---|
| **ontology** (`Pack` impl over a proposer `Domain`) | accounts, pipeline stages, evidence: `legal_approved` / `security_review_done` / `exec_sponsor` | people, assimilation stages, evidence: `welcomed` / `followed_up` / `in_small_group` / `care_assigned` |
| **authored objective** (Non-goal 1: never invented) | `revenue_objective.json` | `church_objective.json` |
| **observed state** | a `RevenueState` snapshot | a `ChurchState` snapshot |

Everything else — the *code path* — is identical: the proposer, the scorer, the
ranker, the blake3 hasher, the PDDL planner adapter, the `law judge`/`law admit`
admission gate, and the `law receipt` chain. Written once, reused verbatim.
`tests/two_domains.rs` makes this literal: one assertion loop, two packs, the
same substrate functions named in the body.

## The mission language (RevTAC, generalized)

```
mission run     --pack <revenue|church> --objective <path> --state <path> [--mission <name>] [--ts-ns <ns>]
mission ceiling --pack <revenue|church> --state <path>
```

`mission run` compiles a mission down to the substrate invocation and drives it
end to end:

```
observe → propose → goal → plan solve → law judge / law admit → law receipt
```

`mission ceiling` computes the pack's **Maximum Reachable objective** — MRR
generalized (see below). Both emit observations (O), never authority (O\*): the
compiled goal and its receipt bind an admitted proposal's `proposal_hash`, but
the authority lives in the admission gate the pipe calls, never in the mission
document (AR-9).

## Same structure, side by side

The two invocations are the same command with a different `--pack`, objective,
and state. The transcripts have the **identical shape** (`tests/two_domains.rs`
asserts the step-key sets are equal).

### Revenue

```console
$ praxis mission run \
    --pack revenue \
    --objective crates/praxis-proposer/revenue_objective.json \
    --state revenue_state.json \
    --mission close-q3
```

```jsonc
{
  "pack": "revenue",
  "step_2_top_goal": {
    "goal": "(stage acct-apex closed-won)",
    "target_id": "acct-apex",
    "target_stage": "closed-won",
    "score": 5012000.0
  },
  "step_3_plan": {
    "plan": [
      "advance-gated(acct-apex,proposal,procurement)",
      "close(acct-apex,procurement,closed-won)"
    ]
  },
  "step_4_admissions": [
    { "action": "advance-gated(...)", "required_evidence": ["legal_approved","security_review_done"],                  "admit_status": "admitted" },
    { "action": "close(...)",         "required_evidence": ["legal_approved","security_review_done","exec_sponsor"],   "admit_status": "admitted" }
  ],
  "step_5_receipt": { "binds_proposal_hash": "…", "chain_hash": "…" }
}
```

### Church

```console
$ praxis mission run \
    --pack church \
    --objective crates/praxis-proposer/church_objective.json \
    --state church_state.json \
    --mission connect-newcomers
```

```jsonc
{
  "pack": "church",
  "step_2_top_goal": {
    "goal": "(stage visitor-apex leading)",
    "target_id": "visitor-apex",
    "target_stage": "leading",
    "score": 4200.0
  },
  "step_3_plan": {
    "plan": [
      "advance-to-serving(visitor-apex,connected,serving)",
      "advance-to-leading(visitor-apex,serving,leading)"
    ]
  },
  "step_4_admissions": [
    { "action": "advance-to-serving(...)", "required_evidence": ["welcomed","followed_up","in_small_group"],                "admit_status": "admitted" },
    { "action": "advance-to-leading(...)", "required_evidence": ["welcomed","followed_up","in_small_group","care_assigned"], "admit_status": "admitted" }
  ],
  "step_5_receipt": { "binds_proposal_hash": "…", "chain_hash": "…" }
}
```

Read the two transcripts together: same keys, same steps, same admission field
shape. Only the ontology's nouns changed — accounts became people, stages
became assimilation tiers, dollar evidence became hospitality evidence. The
substrate did not change; it was never told which institution it was serving.

## The Maximum Reachable objective (MRR, generalized)

`mission ceiling` answers a single question with a single number: *if every
entity that could lawfully advance did, how much realized mission value would
exist?* It is the ceiling the ranked proposals chase, expressed as a physical
fact about the state rather than a scored preference — so it is
**objective-independent** (the authored weights change *ranking*, never the
ceiling) and it **respects the evidence gates** (it only maximizes over lawful
forward targets).

- **Revenue** → Maximum Reachable Revenue. Ceiling fluent: `realized_revenue`.
  The generic `mission::ceiling::<RevenueDomain>` reproduces the bespoke
  `praxis_proposer::maximum_reachable_revenue` headline numbers **exactly**
  (`tests/two_domains.rs::revenue_ceiling_equals_bespoke_mrr`).
- **Church** → the ceiling of people the welcome team can lawfully connect and
  care for. Ceiling fluents: `people_connected`, `care_completion_rate`. Cost
  and process fluents (`volunteer_capacity_used`, follow-up timeliness) are
  excluded: the ceiling bounds *value*, not *score*.

```console
$ praxis mission ceiling --pack revenue --state revenue_state.json
{ "pack": "revenue", "ceiling_fluents": ["realized_revenue"],
  "max_reachable_value": 5500000.0, "already_realized_value": 500000.0,
  "opportunity_value": 5000000.0, "utilization": 0.0909…,
  "entities": [ … { "id": "acct-legal-gap", "max_reachable_value": 0.0,
                    "blocked_on": ["legal_approved"] } … ] }

$ praxis mission ceiling --pack church --state church_state.json
{ "pack": "church", "ceiling_fluents": ["people_connected","care_completion_rate"],
  "max_reachable_value": 11.0, "already_realized_value": 9.0,
  "opportunity_value": 2.0, "utilization": 0.818…,
  "entities": [ … { "id": "visitor-fresh", "max_reachable_value": 0.0,
                    "blocked_on": ["welcomed","followed_up","in_small_group","care_assigned"] } … ] }
```

In both packs, an entity that can realize nothing is attributed to the exact
evidence it is missing — the same `blocked_on` shape, computed by the same
generic algebra. Stripping evidence lowers the ceiling in both (proven for
revenue by `removing legal_approved`, for church by
`church_ceiling_respects_evidence_gates`).

## Adding a third institution

The entire cost of a new institution on this substrate is:

1. a `praxis_proposer::engine::Domain` impl (ordered stages, an evidence gate,
   the mission fluents) — the ontology;
2. an authored objective JSON over those fluents — the weights (never invented);
3. a hand-authored PDDL8 domain for the same stages; and
4. a `Pack` impl in `src/mission.rs` wiring the PDDL/admission surface (domain
   text, problem projection, per-stage required evidence, per-entity evidence).

No proposer, scorer, ranker, hasher, planner, admission, or receipt code is
written. `mission::run_pipeline::<NewPack>` and `mission::ceiling::<NewPack>`
already exist. That absence *is* the doctrine: the substrate is
domain-independent; only the ontology and the authored objective change.

## Boundary position (AR-9), unchanged across packs

Every proposal a mission emits is an observation (O), not authority (O\*). It
must pass `law judge` / `law admit` before any effect, and the receipt binds
back to the admitted proposal's `proposal_hash` so *which* proposal was admitted
stays provable — in every institution. The seam invariant
`mission::evidence_gate_agrees::<P>` proves the proposer's lawfulness pre-filter
and the admission gate never disagree, for every entity × stage, in every pack.
A church proposal is a suggestion for a human on the welcome team to weigh —
never an instruction, and never authority over a person.
