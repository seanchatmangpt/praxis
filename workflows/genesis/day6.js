export const meta = {
  name: 'genesis-day-6',
  description: 'Genesis Day 6: Mission Physics beyond revenue — second domain pack (church ops), two institutions one substrate',
  phases: [
    { title: 'Gate', detail: 'wait for Day 5 receipt + quiescence' },
    { title: 'ChurchPack', detail: 'church-operations domain pack: ontology, obligations, objective' },
    { title: 'TwoDomains', detail: 'prove domain-independence: both run on one law' },
    { title: 'Receipt', detail: 'Day 6 receipt' },
  ],
}

phase('Gate')
const gate = await agent(
  'Quiescence gate for Genesis Day 6 on /Users/sac/praxis. Poll every 120s (cap 100 min): DAY_5_RECEIPT.md exists with chain hash, no praxis builds, git stable twice. Survey: git log --oneline -8, DAY_5_RECEIPT.md, how the revenue domain pack is structured (ontology/revenue.ttl, revenue-domain.pddl, revenue_objective.json, propose ops, RevTAC) — the church pack must mirror this structure. Compact.',
  { label: 'day6-gate', effort: 'low' }
)

phase('ChurchPack')
const church = await agent(
  'Genesis Day 6, phase 1: the church-operations domain pack — Mission Physics beyond revenue, in /Users/sac/praxis. Gate + revenue-pack structure: ' + JSON.stringify(gate).slice(0, 2500) +
  ' The doctrine claim to PROVE: the substrate (proposer + planner + admission + receipt) is domain-independent; only the ontology and the AUTHORED objective function change. Church operations have mission variables that are NOT revenue: attendance, volunteer_hours, welcome_completed, care_completed, prayer_requests_closed, outreach_touches, giving, participation, discipleship_stage. This is grounded in real service (ZOE Church welcome team — treat respectfully, this is operational discretization to make sure no one who came for help gets lost, NOT reduction of the spiritual). ' +
  'Mirror the revenue pack EXACTLY in structure so the parallelism is the proof: (1) ontology/church.ttl — a pdl:-vocabulary domain of Person/Visitor states (FirstTime -> Returning -> Connected -> Serving -> Leading, an ordered stage enum like revenue stages) with evidence flags (welcomed, followed_up, in_small_group, care_assigned). Author it PDDL8-safe (positive-precondition actions only) so it solves. (2) A church-domain.pddl (or manufacture via mfg) with evidence-gated stage-advance actions (advance-to-connected requires welcomed AND followed_up etc.). (3) church_objective.json — AUTHORED weights over church fluents (people_connected, care_completion_rate, volunteer_capacity_used, first_time_followup_within_48h) — the values are domain-authored data, the algebra is the same as revenue (deny_unknown_fields loader, fixed fluent vocab). (4) Extend the proposer (or add a generic domain-pack loader if the proposer was revenue-hardcoded — refactor toward a Domain trait if needed, but keep revenue passing) so `propose --pack church` ranks proposals over church state with rationales. Tests: a visitor missing followed_up is never proposed past Returning (SAME lawfulness-gate mechanism as revenue, proving reuse), objective sensitivity, ranking determinism. cargo build/test --all-features green. Report: church ontology summary, whether the proposer needed a Domain-trait refactor (and that revenue still passes), sample ranked church proposals.',
  { label: 'church-pack', effort: 'high' }
)

phase('TwoDomains')
const twodomains = await agent(
  'Genesis Day 6, phase 2: prove two institutions run on one substrate, in /Users/sac/praxis. Prior: ' + JSON.stringify(church).slice(0, 2000) +
  ' (1) A single integration test tests/two_domains.rs that runs the IDENTICAL pipeline code path (propose -> goal -> plan solve -> admit -> receipt) over BOTH the revenue pack and the church pack, asserting: the substrate functions called are the same (the only inputs that differ are ontology + objective + state), both produce valid receipt chains, both enforce their evidence gates via the same admission mechanism. This test IS the domain-independence proof — structure it so a reader sees one loop, two packs. (2) A `mission` verb generalizing RevTAC to any pack: `mission run --pack <revenue|church> --objective <path> --state <path>` compiling a mission to the proposer+planner invocation — one mission language above the substrate for all domains (document in docs/MISSION_PHYSICS.md with a revenue example AND a church example side by side, showing identical structure). (3) Maximum Reachable value generalized: `mission ceiling --pack <p>` computing the pack\'s Maximum Reachable objective (MRR generalized to church = max reachable people_connected/care under evidence constraints). Tests: mission verb works for both packs, ceiling respects evidence gates in both. cargo build/test --all-features green. Report: the two-domains test structure, the mission-language examples, and confirmation that the same substrate functions serve both.',
  { label: 'two-domains', effort: 'high' }
)

phase('Receipt')
const receipt = await agent(
  'Genesis Day 6 closer for /Users/sac/praxis. Reports: ' + JSON.stringify(church).slice(0, 1200) + ' | ' + JSON.stringify(twodomains).slice(0, 1500) +
  ' Write docs/genesis/DAY_6_RECEIPT.md: church pack summary, the two-domains proof (one substrate, two institutions, both receipted — the day\'s thesis), the generalized mission language, any Domain-trait refactor noted. Chain: prev = MANIFEST_DAY_5.json hash, write MANIFEST_DAY_6.json. Update GENESIS.md Day 6 row. Full test sweep tail. Commit ("feat(genesis): day 6 — church domain pack, two institutions one substrate") and push. Return receipt + Day 7 inheritance.',
  { label: 'day6-receipt', effort: 'medium' }
)

return { gate, church, twodomains, receipt }
