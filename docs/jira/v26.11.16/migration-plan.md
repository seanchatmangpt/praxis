# praxis migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/praxis
- Remote: https://github.com/seanchatmangpt/praxis.git
- Branch: main@c78783b4 (upstream origin/main, in sync)
- Last commit: 2026-09-27 ("Extract GraphLaw into canonical repository")
- Commits: 628; size: 1071 MB (.git 573 MB)
- Languages/build: Rust workspace (Cargo.toml, Cargo.workspace.toml, justfile, deny.toml,
  cicd.toml),
  Erlang/AtomVM artifact, Python debug scripts; CLAUDE.md + AGENTS.md
- Nested repos: none

## Observed state
- Dirty: 4 (docs/BUILD_CACHING.md, docs/CHATMAN_EQUATION.md, docs/CORE_TEAM_DISCIPLINE.md,
  docs/book/part2-06-the-nine-crates.md), untracked: `crates/praxis-graphlaw/` and
  `docs/jira/v26.9.19/`.
- Ahead 0 / behind 0. Stash: 311. Local branches: 29, 25 unpushed (subagent-*, feat/*, perf/*,
  security/rotate-signing-keys-v26.9.24, security/key-exposure-record-v26.9.25).
- Build dirs: no `target/`. `data/validated_receipts` 4.5 MB.
- Root clutter: cargo.toml.backup, cargo.workspace.toml.backup, debug_test*, cng_tests.txt.
- Submodules: none.

## What it is
The seanchatmangpt Rust "house style" kit: scaffolding, lint, build and release conventions
derived from a survey of 18 repos, plus compliance gates and (formerly) GraphLaw, which was
extracted to its own repo in the last commit.

## Classification
ACTIVE (census) -> WARM: in sync, last commit Sept 2026, GraphLaw moved out. Keeping ACTIVE per
census is defensible; action is the same.

## Migration risk
- Untracked `crates/praxis-graphlaw/` (post-extraction leftover? DECISION: delete or keep).
- 25 unpushed branches incl. two `security/*` key-rotation branches (merged via PRs #19/#20,
  so likely safe) and 311 stashes.
- History contains previously committed-then-revoked signing keys (documented v26.9.25).
Rating: MEDIUM.

## Recommended action
COMMIT-PUSH

1. `git add docs && git commit -m "docs: v26.9.19 catalogue + doc edits" && git push origin main`.
2. DECISION: `crates/praxis-graphlaw/` — if identical to the new graphlaw repo, `rm -rf` it;
   else commit it on a branch.
3. `git push --all origin` after pruning `subagent-*` branches (`git branch --merged main`).
4. `tar czf /Volumes/archive/praxis-dotgit.tgz -C /Users/sac/praxis .git` (311 stashes).
5. New machine: clone fresh; nothing to clean (no target/).

## First commands on the new machine
`cargo check --workspace && cargo deny check` and `just --list`. Stated, not run.

## See Also
- ../v26.9.19/
- /Users/sac/docs/jira/v26.11.16/README.md
