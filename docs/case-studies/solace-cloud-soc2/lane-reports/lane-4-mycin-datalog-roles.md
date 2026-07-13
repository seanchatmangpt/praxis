# Lane 4 — Mycin/Datalog Standing-Role Layer

Status: DONE (Stage 2 addition).

## Commands run this session

```
just cng-test-lib-isolated soc2-2 bench::roles -- --nocapture
```

Result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 177 filtered out`.

## Design

`bench::roles` already carries a category→role→action Mycin table (`role_rules`) used generically
by every `CATEGORIES` entry, and a real praxis-graphlaw Datalog engine (`derive_roles_datalog`,
over `rules/bench-roles.dl`) that a roster-declared role must agree with (mismatch is a typed
`HardcodingSuspicion` refusal) — this is the existing agreement mechanism this lane extends,
rather than a new one. Since `role_rules`'s premise is a single `category=X` fact (inherently 1:1
per category), it cannot itself select among 5 distinct SOC2 standing roles for one category. A
second, responsibility-keyed Mycin sub-table (`soc2_role_rules`, `infer_soc2_standing_role`) was
added instead:

| Responsibility | Standing role | Lawful next action | Mycin CF (role rule × action rule) |
|---|---|---|---|
| `control-design-and-evidence` | control-owner | document-control-design-and-attach-evidence | 0.95 × 0.9 = 0.855 |
| `readiness-and-oe-testing` | internal-audit-lead | execute-readiness-assessment-and-oe-testing | 0.95 × 0.9 = 0.855 |
| `scoping-and-bundle-coordination` | compliance-program-manager | coordinate-scope-and-assemble-evidence-bundle | 0.95 × 0.9 = 0.855 |
| `exception-remediation` | remediation-engineer | implement-remediation-for-identified-exception | 0.95 × 0.9 = 0.855 |
| `evidence-chain-of-custody` | evidence-custodian | maintain-evidence-chain-of-custody | 0.95 × 0.9 = 0.855 |

Certainty factors mirror the existing `role_rules`/`role_of`+`action_of` convention exactly (0.95
direct classification, 0.9 one-hop inference), combined via the real Shortliffe-Buchanan
certainty-factor engine (`wasm4pm_cognition::breeds::production_rules::Mycin`), not hand-picked.

The generic per-artifact classification for the `soc2-audit` bench category (used when an
artifact is classified only by its `CATEGORIES` value, not a specific SOC2 responsibility) routes
to `auditor`, mirroring `compliance-check` — added to `role_rules`'s existing flat table.

## Datalog parity

`rules/bench-roles.dl`'s existing `{?w :declaredRole ?r}=>{?w :derivedRole ?r}` identity rule is
role-name-agnostic and needed no change; 5 new obligation rules were added, one per SOC2 standing
role, with obligation text copied verbatim from the Mycin table above. `derive_roles_datalog` was
extended with a new `obligations: BTreeMap<String, String>` field (additive; existing callers
unaffected) so a test can assert full TEXT parity, not just role-identity parity.

`roles_test.rs::soc2_standing_roles_mycin_and_datalog_agree`: for each of the 5 responsibilities,
asserts (a) Mycin infers the expected `next=<action>` conclusion, (b) a small 5-worker Datalog
roster (declared with the 5 SOC2 role names) derives without contradiction, and (c) the
Datalog-derived `:obligation` atom for each worker equals Mycin's `next=<action>` text exactly.

## Evidence paths

- `crates/cng/src/bench/roles.rs` (`soc2_role_rules`, `infer_soc2_standing_role`,
  `DatalogRoles::obligations`)
- `crates/cng/rules/bench-roles.dl` (5 new obligation rules)
- `crates/cng/src/bench/roles_test.rs`
  (`soc2_standing_roles_mycin_and_datalog_agree`)
