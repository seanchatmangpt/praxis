export const meta = {
  name: 'genesis-day-7',
  description: 'Genesis Day 7: the release — version, changelog, publish, Book Three from receipts, close the seven-day chain',
  phases: [
    { title: 'Gate', detail: 'wait for Day 6 receipt + quiescence' },
    { title: 'Release', detail: 'version, changelog, frontier matrix final, publish lawful subset' },
    { title: 'BookThree', detail: 'The First Week, written FROM the receipts not memory' },
    { title: 'CloseChain', detail: 'Day 7 receipt commits Day 1 hash — the week is one object' },
  ],
}

phase('Gate')
const gate = await agent(
  'Quiescence gate for Genesis Day 7 on /Users/sac/praxis. Poll every 120s (cap 100 min): DAY_6_RECEIPT.md exists with chain hash, no praxis builds, git stable twice. Survey broadly for the release: git log --oneline -30 (the week\'s commits), ls docs/genesis/ (all manifests + receipts day 1-6), current version in Cargo.toml, whether the frontier matrix verb (dod matrix / frontier) works, and read all six DAY_N_RECEIPT.md chain hashes so the closer can verify the chain. Compact but include every day\'s manifest hash.',
  { label: 'day7-gate', effort: 'low' }
)

phase('Release')
const release = await agent(
  'Genesis Day 7, phase 1: the release, in /Users/sac/praxis (and constellation). Gate + week survey: ' + JSON.stringify(gate).slice(0, 3000) +
  ' (1) FRONTIER MATRIX FINAL: run the frontier/dod matrix verb; verify every capability source explored this session is a cell — including the ones surveyed after the matrix was first built: unibit, dteam, bytestar, unrdf, open-ontologies corpus, agent8, powl2-decompose, pddl-index. Each is Admitted (with the socket it landed in) or Impossible (refused with reason + salvage): unibit (dirty-tree/154-files — harvest admit4 ported, not dep), dteam (INSA coupling — bitmask_replay differential-only), bytestar (C stubs/dormant — design ported), unrdf (Node runtime — hooks/mu semantics ported), stpnt (no license — taxonomy ported), etc. Assert pass_rate 1.0 with refusals receipted; write target/frontier-report.json. If the matrix verb lacks rows for the new sources, add them (small edit to build_frontier_matrix). (2) VERSION + CHANGELOG: bump praxis to a Genesis release version (CalVer, e.g. 26.7.1 or a dated tag — match the constellation norm), generate/update CHANGELOG.md from the week\'s conventional commits (git log since Day 1), git tag the release. (3) PUBLISH: for each repo with a remote and committed clean state, push; for each PUBLISHABLE crate (registry-resolvable deps, licensed) run cargo publish --dry-run then real publish if a token exists — otherwise receipt BLOCKED(reason). Zero silent rows: every crate published or refused-with-reason (reuse the Day 1 publication-matrix approach). (4) Update docs/VISION_2030_PRD.md release-criteria table to final PASS/PARTIAL/FAIL against verified reality. cargo build/test --workspace --all-features + clippy green. Report: frontier matrix final numbers, version/tag, publication table (zero silent rows), release-criteria table.',
  { label: 'release', effort: 'high' }
)

phase('BookThree')
const bookthree = await agent(
  'Genesis Day 7, phase 2: write Book Three of the Primer — "The First Week" — FROM THE RECEIPTS, not from memory. Repo /Users/sac/praxis. Release report: ' + JSON.stringify(release).slice(0, 2000) +
  ' Read the primary sources: all of docs/genesis/DAY_1..7 receipts and MANIFEST_DAY_*.json, git log --oneline for the week, target/frontier-report.json, and the two prior books (docs/fiction/THE_FIRST_RECEIPT.md, docs/fiction/BOOK_TWO_THE_LAW_OF_WORK.md) for voice continuity. Write docs/fiction/BOOK_THREE_THE_FIRST_WEEK.md in the established mythic-technical-nonfiction voice, with the same discipline as Book One: every narrative beat backed by a real artifact, with executable/verifiable sidebars (the receipts, the manifest hashes, the commits, the frontier cells). Structure: a prologue framing the week as one auditable object (the chained manifest), then one chapter per Genesis day narrating what was actually built (pulled from that day\'s receipt — NOT invented; if a day had a gap or a refusal, the chapter says so, because a book written from receipts inherits their honesty), and a coda on the closed chain. This is the payoff of the whole doctrine: the book is not memoir (memory is unreliable, O not O*), it is HISTORY DERIVED FROM RECEIPTS (O*) — say that explicitly. Cross-reference the Chatman Equation paper (/Users/sac/knhk/docs/papers/reference/the_chatman_equation_fortune5_v1.2.0.pdf, Nov 2025) as the doctrine\'s prior publication and bytestar as its C-era prehistory. Include a receipts appendix listing every manifest hash and the chain linking them. Report the chapter list and confirm every claim traces to a receipt.',
  { label: 'book-three', effort: 'high' }
)

phase('CloseChain')
const closechain = await agent(
  'Genesis Day 7 closer — seal the week as one object. Repo /Users/sac/praxis. Reports: ' + JSON.stringify(release).slice(0, 1500) + ' | ' + JSON.stringify(bookthree).slice(0, 1000) +
  ' Write docs/genesis/DAY_7_RECEIPT.md AND the closing artifact docs/genesis/GENESIS_SEAL.json: (1) The Day 7 receipt: what the release shipped, Book Three pointer, final frontier numbers. (2) THE CHAIN CLOSURE — read every MANIFEST_DAY_1..7.json, verify each day\'s prev_day_hash equals the prior day\'s manifest_hash (report any break honestly — a broken chain is a receipted finding, not something to hide), and compute a GENESIS_SEAL: a single hash over the ordered list of all seven manifest hashes, written to GENESIS_SEAL.json with {days: [{day, manifest_hash}], seal_hash, algorithm}. This seal is the week-as-one-object — Day 7\'s receipt provably commits Day 1\'s hash. (3) Update docs/GENESIS.md: all seven rows done, add a final row with the seal hash. (4) Verify the whole thing builds and tests green one final time (cargo build/test --workspace --all-features, tail). Commit ("feat(genesis): day 7 — release, Book Three, chain sealed") and push with the release tag. Return: the GENESIS_SEAL contents, chain-verification result (each link checked), the final test summary, and an honest closing assessment — what the seven days actually delivered vs the GENESIS.md program, what remains, and whether the chain is intact.',
  { label: 'day7-close', effort: 'high' }
)

return { gate, release, bookthree, closechain }
