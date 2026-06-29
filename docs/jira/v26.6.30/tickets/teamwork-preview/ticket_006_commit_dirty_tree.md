# Ticket: Commit Dirty Working Tree Before Publish

## Title
Commit Dirty Working Tree Before Publish

## Description
The praxis workspace currently has 33+ modified tracked files (from agent-generated modules, lints, examples, and Cargo.lock updates). Before publishing v26.6.30 to crates.io, all uncommitted changes must be committed to `main`. This is a **release blocker** — `cargo publish --dry-run` will fail on a dirty working tree unless `--allow-dirty` is used, and clean publishes are required for production releases.

Modified files include:
- `Cargo.lock` (dependency lock updates)
- `crates/praxis-retrofit/src/` (agent-generated modules and corrections)
- `crates/chatman-common/src/chain.rs` (integration updates)
- `.github/workflows/ci.yml` (CI/CD setup)
- Root `Cargo.toml` (workspace configuration)
- Various `examples/` and config files

## Acceptance Criteria
- **All Modified Files Staged and Committed**: `git add -A` from `/Users/sac/praxis` followed by a single commit with message following praxis convention (e.g., `"chore(release): finalize v26.6.30 release artifacts and integration"`)
- **Clean Working Tree**: `git status` from `/Users/sac/praxis` shows:
  ```
  On branch main
  Your branch is ahead of 'origin/main' by 1 commit.
  nothing to commit, working tree clean
  ```
- **Publish Dry-Run Success**: All three crates pass `cargo publish --dry-run`:
  ```bash
  cargo publish --dry-run -p chatman-common
  cargo publish --dry-run -p praxis-retrofit
  cargo publish --dry-run
  ```
  All three must complete with exit code 0 (expected message: "aborting upload due to dry run").

## Dependencies
Depends on: **PRAXIS-005** (version bumps must be complete before committing).

## Verification Mechanism
Execute the following verification steps from `/Users/sac/praxis`:
1. Check git status:
   ```bash
   git status
   ```
   Must show clean working tree.

2. Verify the commit is on main:
   ```bash
   git log --oneline -1
   ```
   Must show the release commit.

3. Run dry-run publishes:
   ```bash
   cargo publish --dry-run -p chatman-common
   cargo publish --dry-run -p praxis-retrofit
   cargo publish --dry-run
   ```
   All three must exit 0.
