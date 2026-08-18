# PROJ-811: Fix broken `crates/cng` workspace dependency chain

**Status**: BLOCKED — needs human decision
**Dependencies**: none
**Blocks**: PROJ-812, PROJ-813, PROJ-814, PROJ-816 (every ticket in this milestone that needs a
passing `just` gate)

## Scope

`just fmt-check` (and therefore `check`, `clippy`, `test`, `verify-all`, `test-changed`) fails
before running any check-specific logic:

```
cargo fmt --all --check
`cargo metadata` exited with an error: error: failed to load manifest for workspace member `/Users/sac/praxis/crates/cng`
referenced by workspace at `/Users/sac/praxis/Cargo.toml`

Caused by:
  failed to load manifest for dependency `multifractal-workflow`
Caused by:
  failed to load manifest for dependency `my-conforming-project`
Caused by:
  failed to load manifest for dependency `rust-fable-testbed`
Caused by:
  failed to load manifest for dependency `ggen-core`
Caused by:
  failed to read `/Users/sac/ggen/crates/ggen-core/Cargo.toml`
Caused by:
  No such file or directory (os error 2)

error: recipe `fmt-check` failed on line 279 with exit code 1
```

Dependency chain: `crates/cng/Cargo.toml` → `multifractal-workflow` → `my-conforming-project` →
`rust-fable-testbed` → `ggen-core`, and `ggen-core`'s path dependency resolves to
`/Users/sac/ggen/crates/ggen-core/Cargo.toml` — a path outside this repo that does not exist on
this machine.

## Why this is reserved for human sign-off, not decide-and-proceed

Two structurally different fixes are both plausible and have different consequences:

1. **`/Users/sac/ggen` is a missing sibling checkout** that's supposed to exist alongside
   `/Users/sac/praxis` (the way `just standing`'s sibling-checkout convenience path already
   assumes `../cargo-cicd/plugins/cargo-cicd-kit/standing-pack` may or may not be present). If
   so, the fix is restoring/cloning that checkout, not touching any `Cargo.toml`.
2. **The path reference itself is stale** and `ggen-core`/`my-conforming-project`/
   `rust-fable-testbed` should point somewhere in-repo (or the dependency should be removed
   entirely if it's dead weight from an earlier port). If so, the fix is a real edit to
   `crates/cng`'s dependency graph — which needs to be verified against what `my-conforming-project`
   and `rust-fable-testbed` actually are before any path is repointed.

This is exactly the "genuinely underdetermined product law" class in
`.claude/rules/autonomous-escalation-policy.md` — the repo's own topology doesn't say which of
the two is correct, and guessing wrong either wastes a clone or silently breaks a real
dependency edge.

## Verification plan (once unblocked)

```
just fmt-check
just check
just test-changed
```
All three must show real PASS output (not summarized) before any downstream ticket in this
milestone commits its change.
