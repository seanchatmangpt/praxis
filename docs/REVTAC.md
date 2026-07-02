# RevTAC v0 — the revenue mission language

RevTAC ("Revenue Task-And-Constraint") is the layer where **operators author
missions, not PDDL**. It sits one level above the Revenue Physics substrate the
prior Genesis phase built (`propose revenue` → PDDL goal → `plan solve` →
`law admit` → receipt), in the same spirit ORTAC+ sits above a raw
task/action grammar: a small declarative document names an intent and its
lawful scope, and RevTAC *compiles* it down to a proposer invocation plus a
planner goal atom.

Everything RevTAC emits is **observation (O), not authority (O\*)** (AR-9). A
compiled mission is a ranked set of proposals, a planner goal, and a reachable-
revenue ceiling — none of which grant permission. They must still pass
`law judge` / `law admit` before any effect. RevTAC adds vocabulary, not power.

## The verb

```
propose mission --payload '<json>'
```

The payload carries the observed snapshot plus the mission, inline or by file:

```json
{ "state": { "accounts": [ … ] }, "mission": { … } }        // inline mission
{ "state": { "accounts": [ … ] }, "mission_file": "m.toml" } // mission from disk
```

Exactly one of `mission` / `mission_file` must be present. A `mission_file`
ending in `.toml` is parsed as TOML; anything else is JSON (or auto-detected by
a leading `{`).

## The mission format (JSON or TOML)

```json
{
  "mission": "close-q3",
  "constraints": {
    "min_evidence": ["legal_approved", "security_review_done"],
    "exclude_accounts": ["acct-legal-gap"]
  },
  "objective": "crates/praxis-proposer/revenue_objective.json"
}
```

| field | meaning |
|-------|---------|
| `mission` | Free-form intent name. Echoed into the compiled output so a downstream receipt can bind *which mission* produced a proposal. |
| `constraints.min_evidence` | Accounts are in scope only if they carry **all** of these evidence flags. Names must be real `Account` evidence fields — `legal_approved`, `security_review_done`, `exec_sponsor`. An unknown name is a hard error (never silently ignored). |
| `constraints.exclude_accounts` | Account ids removed from scope before proposing. |
| `objective` | Either a **path** to a domain-authored objective JSON file, or an **inline** objective object. RevTAC never invents the objective (Non-goal 1); a mission with no objective is a hard error. |

`constraints` may be omitted entirely (every account is then in scope). The
same document is accepted verbatim as TOML:

```toml
mission = "close-q3"
objective = "crates/praxis-proposer/revenue_objective.json"

[constraints]
min_evidence = ["legal_approved", "security_review_done"]
exclude_accounts = ["acct-legal-gap"]
```

TOML and JSON missions compile to **byte-identical** output (proven by the
`toml_and_json_missions_compile_identically` test in `src/revtac.rs`).

## What "compile" produces

```jsonc
{
  "status": "compiled",                 // or "no_lawful_candidates" when scope is empty
  "mission": "close-q3",
  "objective": { "name": …, "version": … },
  "constraints": { "min_evidence": […], "exclude_accounts": […] },
  "accounts_considered": 1,             // accounts left after filtering
  "accounts_dropped": [                 // every drop, with a reason
    { "id": "acct-legal-gap", "reason": "missing_min_evidence: [\"legal_approved\"]" }
  ],
  "planner_goal": "(stage acct-apex closed-won)",  // top proposal's PDDL atom, ready for `plan solve`
  "top_proposal_hash": "81393deaf9b84ced…",        // binds the compiled goal to the substrate receipt
  "proposals": [ … ],                   // full ranked proposal list (each with pddl_goal + proposal_hash)
  "mrr": { "max_reachable_revenue_cents": 5000000, … }  // reachable-revenue ceiling for THIS scope
}
```

`planner_goal` is the single line you splice into a `plan solve` problem's
`(:goal …)` block over `ontology/revenue.pddl` — the exact seam the prior phase
proved reachable. `mrr` is the Maximum Reachable Revenue (see below) computed
over the *constrained* scope, so a mission also answers "how much revenue does
this mission's scope put in reach?"

---

## Worked example 1 — a focused close mission (TOML, with a file)

**Mission** (`close-q3.toml`): only pursue accounts that already have legal +
security sign-off, and explicitly drop the low-value fresh lead.

```toml
mission = "close-q3"
objective = "crates/praxis-proposer/revenue_objective.json"

[constraints]
min_evidence = ["legal_approved", "security_review_done"]
exclude_accounts = ["acct-fresh"]
```

**State**: `acct-apex` (proposal, full evidence, $50k), `acct-legal-gap`
(qualified, no legal, $30k), `acct-fresh` (lead, no evidence, $10k).

```bash
praxis propose mission --payload '{
  "state": {"accounts": [
    {"id":"acct-apex","stage":"proposal","amount_cents":5000000,"security_review_done":true,"legal_approved":true,"exec_sponsor":true,"days_in_stage":20},
    {"id":"acct-legal-gap","stage":"qualified","amount_cents":3000000,"security_review_done":true,"legal_approved":false,"exec_sponsor":true,"days_in_stage":40},
    {"id":"acct-fresh","stage":"lead","amount_cents":1000000,"security_review_done":false,"legal_approved":false,"exec_sponsor":false,"days_in_stage":90}
  ]},
  "mission_file": "close-q3.toml"
}'
```

**Compiles to**:

- `accounts_considered: 1` — `acct-legal-gap` is dropped
  (`missing_min_evidence: ["legal_approved"]`), `acct-fresh` is dropped
  (`excluded_by_mission`).
- `planner_goal: "(stage acct-apex closed-won)"` — the top-ranked proposal.
- `top_proposal_hash: "81393deaf9b84ced…"`.
- `mrr.max_reachable_revenue_cents: 5000000` — only apex's $50k is reachable
  under this mission's scope.

The `planner_goal` line drops straight into `ontology/revenue.pddl`'s
`(:goal …)` block and is solvable by `plan solve` (classical), yielding the
`advance-gated` → `close` plan the prior phase pinned.

## Worked example 2 — an over-constrained mission is a domain "no", not an error

**Mission** (inline JSON): require an executive sponsor *and* exclude the only
account that has one — a lawful scope that turns out to be empty.

```bash
praxis propose mission --payload '{
  "state": {"accounts": [
    {"id":"acct-apex","stage":"proposal","amount_cents":5000000,"security_review_done":true,"legal_approved":true,"exec_sponsor":true,"days_in_stage":20},
    {"id":"acct-legal-gap","stage":"qualified","amount_cents":3000000,"security_review_done":true,"legal_approved":false,"exec_sponsor":true,"days_in_stage":40}
  ]},
  "mission": {
    "mission": "impossible-q3",
    "constraints": { "min_evidence": ["exec_sponsor"], "exclude_accounts": ["acct-apex", "acct-legal-gap"] },
    "objective": "crates/praxis-proposer/revenue_objective.json"
  }
}'
```

**Compiles to**:

- `status: "no_lawful_candidates"` — an `Ok` result, matching the CLI's
  "domain denial is `Ok`" convention. An empty scope is a *finding*, not a
  crash.
- `planner_goal: null`, `top_proposal_hash: null`.
- `accounts_dropped` names both accounts with their reasons; `mrr` is `0`.

Contrast the two hard-error cases RevTAC *does* reject up front, because they
are authoring mistakes rather than domain findings:

- an unknown `min_evidence` flag (e.g. `"legel_aproved"`) →
  `unknown evidence flag 'legel_aproved' in min_evidence`;
- no `objective` supplied → the system never invents one (Non-goal 1).

---

## Relationship to the substrate

```
        RevTAC mission  (this layer — operators author here)
              │  compile_mission()
              ▼
   objective  +  constrained RevenueState        MRR ceiling for the scope
              │  Proposer::propose()                    ▲
              ▼                                          │
   ranked proposals ──► planner_goal ──► plan solve ──► law admit ──► receipt
        (O, not O*)        (top atom)     (ontology/revenue.pddl)     (binds proposal_hash)
```

RevTAC is deliberately thin: it filters scope, resolves the authored
objective, and hands the substrate a goal. It owns no admission authority and
invents no values — it is the words an operator uses to point the physics at a
quarter.
