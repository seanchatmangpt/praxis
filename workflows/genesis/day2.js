export const meta = {
  name: 'genesis-day-2',
  description: 'Genesis Day 2: Revenue Physics end-to-end — propose→plan→admit→receipt live, RevTAC v0, MRR computed',
  phases: [
    { title: 'Gate', detail: 'wait for Day 1 receipt + quiescence' },
    { title: 'Pipe', detail: 'propose→plan→admit→receipt end-to-end' },
    { title: 'RevTAC', detail: 'revenue mission language + MRR' },
    { title: 'Receipt', detail: 'Day 2 receipt chained to Day 1' },
  ],
}

phase('Gate')
const gate = await agent(
  'Quiescence gate for Genesis Day 2 on /Users/sac/praxis. Poll every 120s (sleep 120, cap 100 min): require (a) /Users/sac/praxis/docs/genesis/DAY_1_RECEIPT.md exists and contains a manifest hash (not just the skeleton markers), (b) no cargo/rustc processes building praxis, (c) git log stable across two polls. On timeout return GATE_TIMEOUT with evidence. On pass, survey: git log --oneline -10, contents of docs/genesis/DAY_1_RECEIPT.md (full), ls src/verbs/, cargo run -- propose --help and plan --help and receipt --help outputs (build first if needed), and whether crates/praxis-proposer is a workspace member. Compact factual report.',
  { label: 'day2-gate', effort: 'low' }
)

phase('Pipe')
const pipe = await agent(
  'Genesis Day 2, phase 1: make the Revenue Physics pipe real end-to-end in /Users/sac/praxis. Read docs/GENESIS.md Day 2 row, docs/VISION_2030_PRD.md (PR-14/AR-9), and the gate survey: ' + JSON.stringify(gate).slice(0, 3000) +
  ' Deliver ONE command (a just recipe `just revenue-demo` plus the underlying verbs) that demonstrates: (1) a RevenueState fixture (3-5 accounts with mixed evidence flags) -> `propose revenue` ranked proposals with rationales and proposal_hash; (2) top proposal -> `propose goal` PDDL goal atoms; (3) a PDDL8-safe revenue domain (author ontology/revenue-domain.pddl or manufacture it from ontology/revenue.ttl via the mfg lane if that is feasible — check mfg pddl first; hand-author if mfg vocab does not fit, receipting the gap) where actions are evidence-gated stage advances (advance-to-procurement requires security-review + legal-approved predicates etc.) -> `plan solve` produces an action sequence reaching the proposed goal; (4) each plan action passes `law judge`/`law admit` with the evidence obligations (the SAME evidence flags — proposal lawfulness pre-filter and admission gate must agree; add a test asserting an account missing legal_approved is BOTH never proposed past Proposal AND refused by admit if forced); (5) `law receipt` with the proposal_hash embedded in the payload so the receipt chain binds back to which proposal was admitted (AR-9 closure). ' +
  'Wire it as a new integration test (tests/revenue_pipe.rs) that runs the whole chain in-process via the ops functions, deterministic (fixed ts_ns), asserting the final chain_hash is stable. Fix any seams between the lanes you find broken (field-name mismatches between propose goal output and plan solve input etc.) — that IS the work. cargo build/test until green. Report the one-command demo transcript and test results.',
  { label: 'revenue-pipe', effort: 'high' }
)

phase('RevTAC')
const revtac = await agent(
  'Genesis Day 2, phase 2: RevTAC v0 + Maximum Reachable Revenue, in /Users/sac/praxis. Prior phase report: ' + JSON.stringify(pipe).slice(0, 2500) +
  ' Deliver: (1) RevTAC v0 — revenue operators author MISSIONS, not PDDL: extend the propose ops with a mission input format (JSON or TOML: {mission: "close-q3", constraints: {min_evidence: [...], exclude_accounts: [...]}, objective: path-to-objective.json}) that compiles to the proposer invocation + planner goal — one layer of ORTAC+-style mission language above the substrate; document the format in docs/REVTAC.md with 2 worked examples. (2) Maximum Reachable Revenue: a `propose mrr` verb/op computing, for a RevenueState + objective + evidence constraints, the maximum realizable revenue over ALL lawful proposal combinations (bounded enumeration — accounts are independent so this is a sum of per-account maxima; document the boundedness argument), plus Revenue Utilization (actual closed / MRR) and Revenue Opportunity (gap). Output all three as chunk-sized numbers with per-account attribution. Tests: MRR is invariant to proposal ordering, respects evidence gates (removing legal_approved from an account lowers MRR by exactly that account\'s contribution), utilization in [0,1]. (3) Validate praxis receipt records against open-ontologies\' canonical receipt SHACL: read /Users/sac/open-ontologies/ontology/shared-receipt-shapes.ttl (sr:SharedReceiptV1), and add a test (feature ggen, using ggen_graph::prelude::validate_shacl) that maps a praxis ReceiptRecord to the sr: vocabulary and validates — if the mapping reveals fields praxis lacks (duration_ms, conformance dims), add them to ReceiptRecord or receipt the mismatch honestly in the test\'s doc comment. cargo build/test --all-features until green. Report the RevTAC format, MRR numbers for the fixture, and SHACL validation result.',
  { label: 'revtac-mrr', effort: 'high' }
)

phase('Receipt')
const receipt = await agent(
  'Genesis Day 2 closer for /Users/sac/praxis. Reports — pipe: ' + JSON.stringify(pipe).slice(0, 1500) + ' revtac: ' + JSON.stringify(revtac).slice(0, 1500) +
  ' Write docs/genesis/DAY_2_RECEIPT.md: What Landed (verify by running `just revenue-demo` or the equivalent yourself), the MRR/utilization numbers, SHACL validation outcome, refusals/gaps with reasons, and Chain: prev = the manifest_hash from docs/genesis/MANIFEST_DAY_1.json (read it), this day = hash over a canonical JSON of {day:2, repos HEAD hashes, prev_day_hash} written to docs/genesis/MANIFEST_DAY_2.json (same algorithm as Day 1 — read how Day 1 computed it and match exactly). Update docs/GENESIS.md Day 2 row to done. Run cargo test --workspace --all-features, record tail summary. Commit ("feat(genesis): day 2 — revenue physics end-to-end + RevTAC v0 + MRR") and push. Return the receipt content + what Day 3 inherits.',
  { label: 'day2-receipt', effort: 'medium' }
)

return { gate, pipe, revtac, receipt }
