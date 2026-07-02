# Vision 2030 — PRD / Architecture Requirements Document

**Product:** Praxis Capability Physics
**Status:** Draft v1 (2026-07-01)
**Companion documents:** `CPHY_ROADMAP.md` (phased delivery), `CONCEPTS_CATALOG.md` (pattern provenance), the Vision 2030 working-backwards press release (this PRD's north star), `docs/PDDL_CAPABILITY_MODEL.md`.

---

## 1. Problem statement

Organizations adopting AI agents and automation face a proof gap: they can show *what* an automated system did (logs) but not that *every action was lawful when it executed* — judged against explicit obligations, admitted under a named policy, and recorded in a form a third party can independently re-verify. Logs are assertions by the system about itself; regulators, auditors, and boards increasingly need evidence that survives a hostile reviewer.

Existing answers fail in one of two ways:

- **Workflow engines / approval chains** enforce order by convention. Nothing prevents code from skipping a step; the guarantee is "we usually call things in order."
- **LLM guardrails** are advisory. The model is asked to follow policy; there is no artifact proving it did, and no refusal semantics when it can't.

**The product thesis:** treat every action as an object that must pass a compile-time-enforced lifecycle — **Raw → Validated → Admitted → Receipted** — with cryptographic receipts chaining each transition, structured refusals when obligations are unmet, and replay verification proving the executed trace matches receipted intent.

## 2. Who it's for

| Persona | Today's pain | What Vision 2030 gives them |
|---|---|---|
| Compliance-bound automation team (fintech, health) | Manual review as the only enforcement; audit prep is archaeology | Admission gates + receipt chain = audit trail generated as a side effect of running |
| Agent-platform builder | Agents act through tool calls with no accountability layer | MCP membrane: every tool call judged/admitted/receipted before effect |
| Revenue/mission operations lead | Policy lives in docs and heads; violations found after the fact | Obligations as executable predicates; refusals with named categories, not silent failures |
| The praxis project itself (dogfood) | Integration decisions made and forgotten | The DfCm frontier matrix: every capability × socket combination admitted or refused with recorded reason |

**Explicit non-persona:** anyone hoping the system will *discover* their policy or decide *what is worth doing*. See Non-goals.

## 3. Product requirements

### P0 — the lawful execution core (largely built as of 2026-07)

- **PR-1 Typestate lifecycle.** `LawObject<Payload, Stage, Law>` with sealed stages; receipting an un-admitted object must be a compile error, not a runtime check. *(Shipped: praxis-core.)*
- **PR-2 Obligation model.** Preconditions, blocking constraints, evidence requirements as first-class hashable values; unmet obligations halt (Andon) rather than error, preserving inspectable state. *(Shipped.)*
- **PR-3 Payload-bound receipts.** Chain hash must commit to the payload, previous hash, and caller-supplied metadata (instruction id, activity, timestamp); identical inputs → identical hash; any perturbation → different hash. *(Shipped + regression-tested this cycle.)*
- **PR-4 Structured refusal.** Every denial carries a category (Identity / Capacity / Topology / Temporal / Lifecycle / Authorization / Prerequisites / Reserved) and a scenario; refusals are outputs, not exceptions. *(Shipped: `praxis-core/refusal.rs` — 8-bucket `RefusalCategory` + `RefusalScenario`, exhaustive-match totality tests; `Andon::Halted` carries `refusals`; verified by running `law judge` on a blocking-constraint payload: output includes `refusal_categories: ["lifecycle"]` and the halted object, exit `Ok`.)*
- **PR-5 Persistence + replay.** Receipts persist append-only; validation recomputes chains (tamper detection) and token-replays lifecycle traces against a POWL model producing conformance metrics; validation of ~100 records in <5ms. *(Shipped, one open defect: `receipt issue` appends `ReceiptRecord`s to `receipts/receipts.jsonl`; `receipt validate --dir` runs the 5-stage pipeline (schema → chain_recompute → chain_linkage → monotonic → token_replay) — verified: a one-hex-digit flip in a copied store is rejected at `chain_recompute` while the original validates clean, and a reversed store is rejected at linkage/monotonic; `receipt replay` reports per-record fitness 1.0 on a lawful trace. Bench `benches/receipt_validate.rs` exists; the <5ms number is asserted there, not re-verified in this reconcile. Open defect (receipted): under the `lsp`/`andon` features, affidavit's same-named verbs leak into the `receipt` noun via lsp-max and can shadow praxis's `show`/`replay`/`export-ocel` — clap-noun-verb's registry is last-write-wins; the walkthrough preflight detects and refuses this.)*
- **PR-6 Signing.** Ed25519 over chain hashes, fail-closed, key via env/file; signature is self-contained (verifiable without out-of-band key exchange). *(Shipped: verified by running `law receipt` under `PRAXIS_SIGNING_KEY` — identical input → identical chain hash, signature present, `law verify-signature` returns `status: "valid"`; with no key set, `receipt issue` fails closed with "no signing key available".)*

### P1 — the planning + manufacturing layer

- **PR-7 Bounded planning.** Deterministic forward-search planning (STRIPS8/temporal) over admitted action spaces, with lexicographic cost arbitration where `admitted` dominates all other cost dimensions. Refusal (infeasible) is a deterministic, receipted outcome. *(Shipped: `plan route|solve|analyze|execute|lawobject` — verified by solving the manufactured lawobject domain (golden 5-step plan), the shipped `ontology/revenue.pddl` (combined-file split + classical solve), and by module tests covering refusal determinism and plan-chain determinism.)*
- **PR-8 Ontology → representation manufacturing.** A domain ontology (Turtle) deterministically projects to PDDL domain/problem text and machine-readable facts; emission is byte-deterministic and round-trips through the parser; bounds (arity ≤ 8 etc.) enforced in code. *(Shipped: `mfg pddl|facts|validate` over `ontology/lawobject.ttl` — verified: two consecutive `mfg pddl` runs are byte-identical, and the emitted domain/problem solves to the golden plan.)*
- **PR-9 Config admission.** The system's own configuration passes through the same discipline: layered TOML admitted via strict unknown-field-rejecting loader, provenance bound in a witness hash, gated by evidence verdicts. *(Shipped: `config show|witness|validate` — verified witness hash emission and env-layer override. Note: the env override prefix is `PRAXIS_CONFIG__*`, not `PRAXIS_*` — strict unknown-field rejection would otherwise veto the operational `PRAXIS_SIGNING_KEY` the config itself names; regression-tested in `tests/config_admission.rs`.)*
- **PR-10 Capability membrane (MCP).** All lifecycle operations exposed as MCP tools sharing one implementation with the CLI (zero drift); pure results cached under law-object keys (capability version, policy digest); denials cached, errors never. *(Partial → landing: as of this reconcile the `mcp_lawobject_server` tools have been rewired to call the shared `ops::*_payload` functions (the zero-drift requirement) and the tool-result cache module landed, but the lane's final `--features mcp` build/test sweep was still running when this document was reconciled — treat as Shipped only once that sweep is green and committed.)*

### P2 — the accountability meta-layer

- **PR-11 Frontier receipting (Vision 2030's namesake).** Every integration decision — adopted or refused — is a cell in a coverage matrix (DfCmMatrix) with expected/actual standing and stated reasons; the matrix is a build artifact and a test. Silent omission is a defect class.
- **PR-12 Evidence emission.** Release artifacts carry an `[evidence]` block binding receipts, trustworthiness scoring, and standards claims, generated by tooling not prose.
- **PR-13 Doctor.** One command reporting holistic system health: build state, config witness, frontier coverage, receipts store, tool availability.

### P3 — 2027+ (sequenced, not yet designed in detail)

- **PR-14 Proposer layer.** Given an admitted current state and a *domain-supplied* objective function with numeric fluents, propose and rank candidate goal states for the planner. First domain: revenue operations (fluents already numeric). **Hard requirement: the objective function is user/domain-authored; the system never invents values.** *(Shipped ahead of sequence, Genesis Day 1, first-domain scope: `crates/praxis-proposer` (workspace member, feature `proposer`) + `propose revenue|goal` verbs. Verified end-to-end: `RevenueState::from_admitted` observes an admitted law object's payload; the objective is loaded from authored JSON only (omitting it is a hard error citing Non-goal 1); `propose goal` emits a `(stage <acct> <stage>)` atom that splices into `ontology/revenue.pddl`'s `(:goal ...)` block and `plan solve` finds an admitted plan for it (`propose_goal_feeds_plan_solve` drives the real `solve_payload`). Every proposal carries rationale lines and a blake3 `proposal_hash`. Still P3 in ambition: one domain, linear scoring algebra only.)* The pre-ship framing — everything else decides whether a chosen transition is lawful, nothing decides what is worth choosing — is now answered for exactly one authored domain.
- **PR-15 Domain packs.** Reference ontologies + obligation sets + objective functions for revenue ops, clinical coordination, church operations — the ORTAC+ pattern (operator-native vocabulary compiling to PDDL) per domain.
- **PR-16 Distributed supervision.** OTP-style supervision, checkpoint/recovery via replay, cross-node receipt chaining (roadmap Phase 5).
- **PR-17 Promotion gates.** Multi-gate promotion to standing credentials (automated replay audit → compliance check → human guardian), per roadmap Phase 6 and the BreedStanding ladder.

## 4. Non-goals (binding)

1. **No value discovery.** The system does not infer, learn, or decide objective functions, policies, or obligations. It enforces what a human authored. Marketing or docs claiming otherwise are defects.
2. **No physical enforcement claims.** Admission is software-binding, not physics-binding. Anything bypassing the membrane is out of scope; we say so in the FAQ and we say so here. No claim of the form "this prevents X" where X can route around the process boundary.
3. **No LLM in the critical admission path.** Models may propose (outside the boundary); admission, receipting, and replay are deterministic code. (Same rule mcpp adopted.)
4. **No general workflow-engine ambitions.** We do not compete on connector count, UI builders, or human-task orchestration.
5. **No unlicensed or unauditable dependencies in the trust core.** The refusal register is normative: stpnt (no license), clnrm-core (footprint), open-ontologies (footprint), affidavit (incompatible chain rule) remain design references, not dependencies, unless their stated blocker is resolved upstream.

## 5. Architecture requirements & decisions (ADR summary)

### AR-1 Typestate over runtime checks — *decided, shipped*
Lifecycle stages are phantom types with a sealed trait. Rationale: illegal transitions become compile errors; the guarantee survives refactoring in a way runtime asserts don't. Consequence: stage transitions need crate-internal helpers; consumers can't forge stages.

### AR-2 Single ops core, thin adapters — *decided*
All pure payload logic lives in one library module (`ops`); CLI verbs and MCP tools are thin wrappers over identical functions. Rationale: the membrane and the CLI must never drift; there is exactly one implementation of "judge."

### AR-3 Chain rule: BLAKE3 over prev ‖ fixed-layout frame — *decided (bcinr-compatible)*
We adopt bcinr-powl-receipt's frame chaining (raw 32-byte prev + 99-byte LE frame), with the payload committed via obj_refs carrying the full payload hash. Affidavit's hex+canonical-JSON rule was evaluated and refused (incompatible); its *verifier design* (staged CheckOutcome/Verdict, sealed receipt carrier) was ported instead. Consequence: emission and recomputation must share one frame-construction function so replay can never drift from emit.

### AR-4 Refusal as data — *decided*
Domain denials return `Ok` with structured refusal JSON (status, category, scenario, unmet obligations); only malformed input is an error. Rationale: a refusal is a lawful, cacheable, receiptable outcome of the system working correctly.

### AR-5 Deterministic bounded planners over optimal ones — *decided for now*
BFS/greedy temporal search over STRIPS8, bounded depth, no heuristics. Rationale: determinism and refusal-on-infeasible matter more than optimality at current domain sizes; the cost vector scores post-hoc. Revisit when a domain pack exceeds the 64-op bound (escape hatch: external solver e.g. Fast Downward behind the same admission boundary).

### AR-6 Direct emission over templating for law-bound text — *decided*
PDDL emission is Rust string-building, not Tera: the PDDL8 bounds are invariants that must be enforced in code. Tera remains the projection surface for non-law artifacts (docs, facts).

### AR-7 Path-dependency constellation with committed-state discipline — *decided*
Sibling crates are path deps pinned to committed states; a `[patch.crates-io]` against a dirty tree is a defect (fixed this cycle). Every praxis commit must build from HEAD with sibling checkouts. Consequence: cross-repo changes land upstream-first.

### AR-8 The frontier matrix is a test — *decided*
Coverage of the integration space is asserted in CI (`tests/frontier_matrix.rs`): every enumerated combination is admitted (and verified working) or impossible (with reason). Rationale: combinatorial maximalism is only honest if refusals are receipted artifacts.

### AR-9 Proposer sits outside the admission boundary — *decided, shipped (Genesis Day 1)*
Proposals are untrusted observations (O, not O*): a proposed goal state passes Rice quarantine and admission like any other input. The proposer may be heuristic or model-backed precisely *because* it has no authority. As shipped: `propose` verbs document this in their output contract, `Proposal::proposal_hash` exists so an admission receipt can bind back to exactly which proposal was admitted, and the proposer crate depends on praxis-core only to *observe* admitted payloads (`RevenueState::from_admitted`) — it cannot construct, judge, admit, or receipt a law object.

## 6. Success metrics

| Metric | 2026 exit | 2027 | 2030 |
|---|---|---|---|
| Frontier matrix pass rate (refusals counted as passes when expected) | 1.0 | 1.0 sustained | 1.0 sustained |
| Receipt validation latency (100 records) | <5ms | <5ms | <5ms at 10k records |
| CLI/MCP behavioral drift (same input, differing output) | 0 (shared ops) | 0 | 0 |
| Domains with authored packs (ontology + obligations + objective) | 1 (lawobject self-test) | 2 (+ revenue) | ≥4 |
| External adopters running admission-gated automation | 0 | 1 design partner | regulated-industry reference customer |
| Claims in docs exceeding mechanism (audited per release) | 0 | 0 | 0 |

That last row is deliberate: this product category dies by overclaiming. "Docs exceed mechanism" is tracked as a defect with the same severity as a failing test.

## 7. Risks

1. **Governance without a proposer is a hard sell** — buyers want "what should we do," we ship "prove what you did was lawful." Mitigation: PR-14/15 sequencing; the revenue domain pack is the wedge.
2. **Constellation fragility** — ~8 sibling repos, CalVer, no semver contracts, one machine. Mitigation: AR-7 discipline, fresh-HEAD build verification in CI, publish-to-registry for the trust core when it stabilizes.
3. **Receipt theater** — a perfect chain over wrong obligations proves nothing about value. Mitigation: keep the claim precise (integrity, not virtue) in every artifact; promotion gates (PR-17) add human judgment where value is asserted.
4. **Two typestate systems** (praxis-core LawObject vs root Evidence types) invite drift. Mitigation: converge on LawObject; deprecate the parallel surface by 2027.
5. **prolog8/wasm4pm churn** (NAF semantics, StratifiedNegation enforcement) can silently change admission behavior. Mitigation: canary tests pinned to observed kernel behavior; version repoints are explicit commits.

## 8. Release criteria (Vision 2030 v1 / the press-release moment)

Status column verified by running the verbs on 2026-07-01 (Genesis Day 1
reconcile) and re-verified 2026-07-02 (Genesis Day 7 release); every claim
below was exercised, not inferred. Day-7 delta: the `dod matrix` frontier lane
landed (criterion 3 FAIL → PASS).

1. Full workspace builds and tests green from a fresh HEAD clone with sibling checkouts.
   **PARTIAL → landing.** `cargo build --all-features` and `cargo test --workspace --all-features` are green in the working tree, now **deterministically** (Day 8 fixed the one `--all-features` test race: two independent mutexes guarding the process-global `PRAXIS_SIGNING_KEY` were unified into one shared lock in `src/ops.rs`, so the receipt and receipt-noun test groups can no longer flip the key under one another — confirmed exit 0). The Day-1…6 integration is being committed as part of the Day-8 quiescence pass; the *fresh-HEAD clone* form of the claim becomes verifiable once that commit lands — re-verify after.
2. Every noun in the press release exists and does what the release says: `law`, `plan`, `receipt`, `mfg`, `config`, `frontier/dod`, `doctor`.
   **PASS (one open defect).** `law`, `plan`, `receipt`, `mfg`, `config` exist and behave as documented (plus `propose`, beyond the list); **`dod matrix` exists** (`src/verbs/dod.rs`, main-binary noun, delegating to `src/frontier.rs`) and produces the frontier report; **`doctor` exists and is functional** (`src/verbs/doctor.rs`, dispatched via the `doctor → check` alias in `src/main.rs`) — `doctor check --format json` returns holistic health (build/config-witness/frontier coverage/receipts store/tool availability/feature flags), verified Day 8. Open defect on `receipt` (tracked, deferred to Day 9): dependency verb leakage can shadow `show`/`replay`/`export-ocel` under `lsp`/`andon` (see PR-5).
3. `target/frontier-report.json` shows pass rate 1.0 with the refusal register fully receipted.
   **PASS** — `dod matrix` (main-binary noun, `src/verbs/dod.rs` → `src/frontier.rs`, built on `wasm4pm_compat::dfcm`) writes `target/frontier-report.json`: 19 capability-source cells, `coverage == 1.0`, `pass_rate == 1.0`, zero failures. 10 Admitted (each carrying the socket it landed in) + 9 Impossible (each refused with reason + salvage — the refusal register is now *data* in the cells, not prose, closing the PR-11 silent-omission gap). Asserted by `tests/frontier_matrix.rs` (6 tests green, 2026-07-02).
4. The FAQ's negative claims (no policy discovery, no physical enforcement, no LLM in the admission path) are each verifiable against the codebase by inspection.
   **PASS** — no value discovery: `ObjectiveFunction` deserializes authored JSON only, and `propose` without an authored objective is a hard error citing Non-goal 1; no physical enforcement claims found in shipped docs; no LLM anywhere in the judge/admit/receipt/validate path (deterministic Rust, prolog8 kernel included).
5. One end-to-end demo: ontology → manufactured PDDL → plan → policy-gated execution → signed receipt → tamper-detected on mutation → replay conformance 1.0 — as a single scripted walkthrough a skeptical reviewer can run.
   **PARTIAL → re-verify.** `scripts/walkthrough.sh` steps 1–5 (manufacture determinism, plan + chain, refuse/admit/signed receipt, one-byte tamper caught at chain_recompute, replay fitness 1.0 + disorder rejected at linkage) each demonstrated against the built binary; `dod matrix` is built, so step 6a's frontier check passes (pass_rate 1.0 including the receipted refusal register). **`doctor` is now built and functional**, removing the last missing verb the preflight probes — so the one prior gap is closed. The single-command end-to-end form should be re-run against HEAD after the Day-8 commit to confirm the preflight clears every probed verb (note: the preflight still detects and refuses the `receipt`-noun shadowing under `lsp`/`andon`, Day-9 defect, by design).
