export const meta = {
  name: 'genesis-day-4',
  description: 'Genesis Day 4: the membrane ships — agent8 byte + env64/pulse64 wire ABI + fleet kernel + external-agent demo',
  phases: [
    { title: 'Gate', detail: 'wait for Day 3 receipt + quiescence' },
    { title: 'Agent8', detail: 'byte layout + wire ABI Rust port + fleet kernel' },
    { title: 'Membrane', detail: 'MCP server productized, external agent drives the pipe' },
    { title: 'Receipt', detail: 'Day 4 receipt chained to Day 3' },
  ],
}

phase('Gate')
const gate = await agent(
  'Quiescence gate for Genesis Day 4 on /Users/sac/praxis. Poll every 120s (cap 100 min): docs/genesis/DAY_3_RECEIPT.md exists with chain hash, no praxis builds, git stable twice. Survey on pass: git log --oneline -8, DAY_3_RECEIPT.md, current MCP tool list (read src/bin/mcp_lawobject_server.rs #[tool] fns), .mcp.json contents. Compact.',
  { label: 'day4-gate', effort: 'low' }
)

phase('Agent8')
const agent8 = await agent(
  'Genesis Day 4, phase 1: agent8 — the 8-bit agent projection + wire ABI, in /Users/sac/praxis. Gate: ' + JSON.stringify(gate).slice(0, 2000) +
  ' Background you must read first: /Users/sac/semantic_bit/src/status8.rs (Status8Field: OK/WARN/BLOCKED/UNKNOWN/SKIPPED/STALE/RECEIPTED/REPLAYABLE bits, const carries()/with(), Status8Receipt), /Users/sac/bytestar/bytecore/abi/envelope.h + pulse.h + README_ABI_IMPLEMENTATION.md (env64_t 64-byte ingress envelope: magic/ver/pb pattern-byte/budget/flags/priority 0-7/in_cid 128-bit/timestamp/seq/source/aux; pulse64_t observer pulse: in_cid/out_cid/receipt fragment 16B/ticks<=8/hops<=8), and /Users/sac/unibit/crates/unibit-kernel/src/lib.rs admit4/commit_masked (read-only reference — do NOT add unibit as a dependency, its tree is 154-files dirty; port the 3 needed const fns with a prior-art citation, ~40 lines). ' +
  'Build a new workspace crate crates/agent8: (1) AgentByte — an 8-bit agent projection newtype with named bit consts (ADMITTED, EVIDENCE_OK, WITHIN_BUDGET, AUTHORITY_BOUND, HEALTHY, CONFORMANT, RECEIPTED, REPLAYABLE), const fn with/carries/select (Grant iff a documented required mask is all-set), serde as u8, Display as 8-char flag string for the at-a-glance read. Cite semantic_bit as the generated-sibling prior art. (2) Env64 + Pulse64 — #[repr(C, align(64))] Rust ports of the bytestar ABI with compile-time size_of asserts == 64 (use the OcelCausalFrame assert idiom from bcinr-powl-receipt), const fn validate() (magic/version/bounds as mask compares, no branches on secret data), pb field documented as the AgentByte wire slot, and a bridge fn pulse64_from_receipt_record(record) -> Pulse64 mapping praxis ReceiptRecord fields (chain fragment = first 16 bytes of chain_hash, ticks/hops capped at 8). (3) Fleet kernel: Fleet { bytes: Vec<u64> } packing 8 agents/word; const fn sweep_admit(word, required_mask) -> denial word (ported admit4 shape); popcount stats (admitted/blocked/receipted counts across the fleet); update_from_pulse. Benchmark: sweep 10_000_000 simulated agents (1.25M words), assert and record time (target: single-digit ms). Tests: byte round-trip, ABI size/alignment asserts, validate rejects bad magic/version, fleet sweep correctness vs a naive loop (differential, per Day 3 doctrine), popcount stats. cargo build/test -p agent8 then workspace green. Report crate layout, bench number, and the exact required-mask semantics you chose.',
  { label: 'agent8', effort: 'high' }
)

phase('Membrane')
const membrane = await agent(
  'Genesis Day 4, phase 2: productize the membrane and prove an external agent can build through it. Repo /Users/sac/praxis. Prior: ' + JSON.stringify(agent8).slice(0, 2000) +
  ' (1) Extend src/bin/mcp_lawobject_server.rs with tools: propose_revenue + propose_goal (over the proposer ops), fleet_status (agent8: accept a fleet state, return popcount stats + per-agent byte flags — wire the agent8 crate in behind the mcp feature), and ensure the full tool list covers the Day 2 pipe: judge/admit/receipt/plan_solve/propose so the entire revenue demo is drivable through MCP alone. Keep the shared-ops single-source rule (tools call ops::, zero drift) and the cache policy from the earlier lane. (2) Maintain each connected session\'s AgentByte in ServerState: judge/admit/receipt outcomes update the byte (halted -> BLOCKED set, receipted -> RECEIPTED set, etc.); add a whoami tool returning the caller\'s current byte + flag string. This is the agent8 adapter: MCP lifecycle events -> resident byte. (3) THE DEMO (release-critical): write scripts/membrane_demo.sh that runs the server over stdio and drives the COMPLETE Day 2 revenue pipe through raw JSON-RPC (initialize, tools/list, then propose_revenue -> propose_goal -> plan_solve -> judge -> admit -> receipt), using jq/python for framing, asserting each response, ending with the receipt chain_hash and the session\'s final AgentByte showing RECEIPTED. This proves an external agent with ONLY membrane access completes a receipted mission — no repo access, no CLI. Make it a CI-runnable test if timing allows (assert_cmd spawning the server binary). (4) Update docs/WALKTHROUGH.md with a membrane section. cargo build/test --features mcp green; run the demo script and capture its transcript. Report tool list, demo transcript summary, and any honest gaps.',
  { label: 'membrane', effort: 'high' }
)

phase('Receipt')
const receipt = await agent(
  'Genesis Day 4 closer for /Users/sac/praxis. Reports: ' + JSON.stringify(agent8).slice(0, 1200) + ' | ' + JSON.stringify(membrane).slice(0, 1500) +
  ' Write docs/genesis/DAY_4_RECEIPT.md: agent8 crate summary + fleet sweep bench number, membrane tool list, the external-agent demo transcript (the proof artifact — include the final chain_hash and AgentByte), refusals/gaps. Chain: prev = MANIFEST_DAY_3.json hash, write MANIFEST_DAY_4.json. Update GENESIS.md Day 4 row. Full test sweep tail. Commit ("feat(genesis): day 4 — agent8 + wire ABI + membrane demo") and push. Return receipt + Day 5 inheritance.',
  { label: 'day4-receipt', effort: 'medium' }
)

return { gate, agent8, membrane, receipt }
