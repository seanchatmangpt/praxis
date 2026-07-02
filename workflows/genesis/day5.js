export const meta = {
  name: 'genesis-day-5',
  description: 'Genesis Day 5: manufacture at scale — ontology corpus ingest, Kourani decomposition, pddl-index, self-manufacture ratio',
  phases: [
    { title: 'Gate', detail: 'wait for Day 4 receipt + quiescence' },
    { title: 'Corpus+Kourani', detail: 'ingest open-ontologies vocab, WF-net->POWL2.0 decomposition' },
    { title: 'Index', detail: 'dictionary-encoded grounding (bytestar succinct primitives, Rust port)' },
    { title: 'Receipt', detail: 'Day 5 receipt + manufacture ratio' },
  ],
}

phase('Gate')
const gate = await agent(
  'Quiescence gate for Genesis Day 5 on /Users/sac/praxis. Poll every 120s (cap 100 min): DAY_4_RECEIPT.md exists with chain hash, no praxis builds, git stable twice. Survey: git log --oneline -8, DAY_4_RECEIPT.md, current mfg lane state (src/mfg.rs exists? cargo run --features ggen -- mfg --help), ls ontology/. Compact.',
  { label: 'day5-gate', effort: 'low' }
)

phase('Corpus+Kourani')
const corpus = await agent(
  'Genesis Day 5, phase 1: amplify manufacture + implement the one genuinely-missing algorithm (Kourani decomposition). Repo /Users/sac/praxis; gate: ' + JSON.stringify(gate).slice(0, 2000) +
  ' (1) CORPUS INGEST: the mfg lane currently manufactures from one ontology. Wire it to consume the reusable open-ontologies vocabularies as INPUTS (they are dependency-free TTL data — copy the needed ones into ontology/vendor/ with provenance comments, or read from /Users/sac/open-ontologies/ontology/ directly if a path is acceptable): shared-receipt-shapes.ttl, truex-ecosystem.ttl (obligation/refusal vocab -> map to praxis refusal.rs categories, add a test that every truex RefusalCondition has a praxis RefusalCategory), mcpp-proof-chain.ttl (Admitted/Refused/Partial verdict model). Manufacture Rust constants/enums from these where it reduces hand-maintenance, byte-deterministic, round-tripped. (2) KOURANI DECOMPOSITION — the missing Stage-1 (read /Users/sac/Documents/Papers/workflow/Hierarchical Decomposition of Separable Workflow-Nets .pdf if still accessible, else work from the design in prior context): implement WF-net -> POWL 2.0 recursive decomposition into choice-graph nodes (choice/cyclic logic) and partial-order nodes (concurrency), over bounded lanes. Put it in a new crate crates/powl2-decompose or a module, operating on a simple WF-net input (places/transitions/flow). SEPARABILITY IS THE ADMISSION PREDICATE: separable nets decompose (admitted); non-separable nets are refused with a receipted reason (a Rice boundary for process models — this is the doctrine payoff). Test against small hand-authored nets (a sequence, an XOR-choice, an AND-parallel, a loop, and one deliberately non-separable net that must be refused). Differential-check the round-trip: decompose then recompose (POWL->WF-net) preserves the language on the small cases. cargo build/test --all-features green. Report: ontologies ingested + refusal-mapping test result, decomposition API, and the separable/non-separable test outcomes.',
  { label: 'corpus-kourani', effort: 'high' }
)

phase('Index')
const index = await agent(
  'Genesis Day 5, phase 2: dictionary-encoded grounding (the qlever treatment), in /Users/sac/praxis. Prior: ' + JSON.stringify(corpus).slice(0, 2000) +
  ' Read /Users/sac/bytestar/bytecore/abi/tables.h (MPHF bs_mphf_read, LOUDS bs_louds_nav, XOR filter bs_xorf_maybe_has, bitset rank/select) as DESIGN reference — these are C ABI headers with partial stubs; do NOT depend on bytestar, port the design to Rust. Build crates/pddl-index (or a module): (1) A dictionary encoder interning predicate/object/type strings to compact u32 IDs (extend bcinr-powl-receipt ActivityTable pattern), (2) permutation-friendly sorted-ID storage for the init state and reachable facts, (3) grounding-as-join: instantiate action schemas against the init state via sorted-merge over ID space + XOR-filter membership pruning, materializing only actions the frontier touches — a lazy grounder. Wire it as an alternative grounding path behind the existing plan solve (feature-flag or auto-select for large domains), falling back to bcinr-pddl BFS for small ones. Benchmark: a synthetic domain with 10^3+ candidate groundings, showing the indexed grounder materializes << all of them; record the ratio and time vs naive. Differential-check: indexed grounding produces the SAME plan as naive grounding on the small shared corpus from Day 3 (correctness via agreement). (3) MANUFACTURE RATIO: count lines of code manufactured today (mfg outputs + Kourani-generated constants + any codegen) vs hand-written; record the ratio in prep for the receipt (the Day 5 target: the repo generates a meaningful fraction of itself). cargo build/test --all-features green. Report: index API, grounding ratio + bench, differential agreement, manufacture ratio.',
  { label: 'pddl-index', effort: 'high' }
)

phase('Receipt')
const receipt = await agent(
  'Genesis Day 5 closer for /Users/sac/praxis. Reports: ' + JSON.stringify(corpus).slice(0, 1200) + ' | ' + JSON.stringify(index).slice(0, 1500) +
  ' Write docs/genesis/DAY_5_RECEIPT.md: corpus ingested, Kourani decomposition + separability-as-admission result, indexed grounding ratio/bench, differential agreement, and the MANUFACTURE RATIO (generated vs hand-written LoC today). Chain: prev = MANIFEST_DAY_4.json hash, write MANIFEST_DAY_5.json. Update GENESIS.md Day 5 row. Full test sweep tail. Commit ("feat(genesis): day 5 — corpus ingest, Kourani decomposition, indexed grounding") and push. Return receipt + Day 6 inheritance.',
  { label: 'day5-receipt', effort: 'medium' }
)

return { gate, corpus, index, receipt }
