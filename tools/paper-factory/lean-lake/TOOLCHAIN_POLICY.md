# Lean/Lake Toolchain & Dependency Pinning Policy

v1 — 2026-07-12

## Scope

This document states `tools/paper-factory/lean-lake`'s explicit dependency-reproducibility
posture, per Lake's documented best practices (manifest commit, tagged ecosystem deps,
overrides only where audited, toolchain-pin discipline). It complements the repo-wide
FIX-FORWARD-ONLY / determinism-first policy in `/Users/sac/praxis/CLAUDE.md`.

## Findings (this session)

### 1. `lake-manifest.json` is gitignored — PARTIAL gap

`.gitignore` (line 5) excludes `lake-manifest.json`:

```
.lake/
lake-manifest.json
```

Verified: `git check-ignore -v tools/paper-factory/lean-lake/lake-manifest.json` returns
`tools/paper-factory/lean-lake/.gitignore:5:lake-manifest.json` (exit 0 = ignored). The file
is neither tracked nor staged (`git log -- lake-manifest.json` is empty, `git status --porcelain`
shows nothing for it because ignored files are hidden by default).

Lake's own documentation states the manifest should normally be committed, since it is the
thing that pins every transitive dependency to an exact resolved `rev` — the `lakefile.lean`
`require` line alone only pins the direct `mathlib` dependency; the six inherited deps
(`plausible`, `LeanSearchClient`, `importGraph`, `aesop`, `Qq`, `batteries`, `Cli`) are pinned
only inside the manifest, not in this project's `lakefile.lean`. Without committing the
manifest, a fresh `git clone` + `lake build` on another machine or at another commit could
resolve different transitive revs than what's currently on disk, even though `lakefile.lean`
is unchanged.

This is a real reproducibility gap. I have **not** added or committed the file — that decision
is the user's call, consistent with the task instructions.

### 2. Direct dependency pinning — ALIVE

`lakefile.lean`'s only `require` line pins `mathlib` to tag `v4.31.0` (not `main`):

```lean
require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "v4.31.0"
```

The manifest resolves this to a concrete commit
(`rev: fabf563a7c95a166b8d7b6efca11c8b4dc9d911f`, `inputRev: "v4.31.0"`) — good, this is a
release-tag pin, not a floating branch pin.

### 3. Transitive dependency pinning — ALIVE, with one caveat

All six inherited deps in `lake-manifest.json` (`plausible`, `LeanSearchClient`,
`importGraph`, `aesop`, `Qq`, `batteries`, `Cli`) show `"inherited": true`, meaning their
pins come from Mathlib's own manifest at the resolved Mathlib commit, not from a floating
resolution against this project. Each has a concrete `rev` (40-char SHA) recorded — so the
*resolved* state is fully pinned even though `inputRev` for most of them reads `main` or
`master` (that's the branch name Mathlib's lakefile requested at the time Mathlib's own
manifest was generated, not a live floating reference from this project's perspective).

Caveat: `inputRev` values of `main`/`master` mean that if this project's manifest is ever
regenerated without `--keep-toolchain`/pinned inputs (i.e. via `lake update`), those
transitive deps will re-resolve against whatever `main`/`master` point to at that time,
which could silently pull in newer, non-audited transitive code even if `mathlib`'s own tag
doesn't move. This is inherent to how Mathlib structures its own deps and is not something
this project can unilaterally re-pin without maintaining its own fork/override — see
Section 4.

### 4. `lean-toolchain` vs. Mathlib rev compatibility — ALIVE, confirmed

`lean-toolchain` reads `leanprover/lean4:v4.31.0`. `lakefile.lean`'s inline comment states this
was deliberately chosen to match Mathlib's `v4.31.0` tag exactly, avoiding a second toolchain
fetch. This session's read of both files confirms the comment is still accurate: the
`require mathlib ... @ "v4.31.0"` line matches the `lean-toolchain` version. No drift found.

### 5. Package overrides / `packagesDir` — no gap found, no override added

No overrides are configured (`lakefile.lean` has no `override` blocks), and none are needed
right now. The Rust side of this repo avoids crates.io forks per
`praxis-toolchain-deps.md` (upstream-only policy, forks removed 2026-07-09) — the equivalent
Lean-side gap would be depending on unofficial forks of Mathlib or its sub-dependencies. That
gap does not currently exist: every dependency in the manifest points at the canonical
`leanprover-community`/`leanprover` GitHub orgs. No speculative override is recommended.

### 6. Build state at end of session — UNVERIFIED (concurrency, not a dependency issue)

`lake build` was run once as instructed. It failed with:

```
error: no such file or directory (error code: 4294967294)
failed to load header from .../mathlib/.lake/build/ir/Mathlib/Data/List/Defs.setup.json:
  offset 0: unexpected end of input
```

`ps aux` at the time showed multiple concurrent `lake build` processes (PIDs 83738, 83739,
84750, 84752, plus their child `lean` invocations) already running against the same
`.lake/build` cache directory from other agents/sessions on this corpus. The truncated
`.setup.json` files are a live write-write race on shared build-cache files, not a
manifest/dependency-pin defect. Per task instructions I did not retry aggressively or run
`lake update`; this finding is BLOCKED on those concurrent builds finishing, not on anything
this audit should fix.

## Decision: toolchain-pinning discipline

**Recommendation: always pass `--keep-toolchain` on `lake update`; never allow a bare
`lake update` to move `lean-toolchain` automatically.**

Reasoning:

- This repo's controlling invariant is FIX-FORWARD-ONLY, determinism-first
  (`/Users/sac/praxis/CLAUDE.md`): "same inputs → byte-identical receipts," no unannounced
  version drift. A bare `lake update` can silently bump `lean-toolchain` to whatever Mathlib's
  current `main` branch (not the pinned `v4.31.0` tag) requires, which would also force a
  second, larger toolchain download and could shift proof term elaboration behavior between
  Lean versions — the definition of "algorithmic surprise" this repo's discipline rules
  forbid.
- The `lakefile.lean` comment already records *why* `v4.31.0` was chosen (to match Mathlib's
  tag and avoid a second toolchain fetch) — an unpinned `lake update` would silently discard
  that reasoning the next time someone runs it.
- `lake-manifest.json`'s `"fixedToolchain": false` means Lake is not currently enforcing a
  toolchain lock at the manifest level — the only enforcement is human/process discipline
  (this document) plus the tracked `lean-toolchain` file. `--keep-toolchain` is the concrete
  mechanism that makes that discipline binding rather than aspirational.
- No concrete reason was found in this session to prefer auto-tracking `main` (no
  currently-blocked feature that depends on a newer Mathlib, no toolchain EOL pressure) — so
  the deterministic default wins by this repo's own tie-breaking rule (Invariant 5:
  "Deterministic under fixed seed").

## Action items (not performed this session — reviewer/owner call)

1. Decide whether to commit `lake-manifest.json` (remove it from `.gitignore` line 5). This
   is the single concrete gap found; committing it is the only change needed to close it.
2. If/when `lake update` is next run, pass `--keep-toolchain` explicitly and diff the
   resulting `lake-manifest.json` rev changes before committing.

## See also

- `/Users/sac/praxis/CLAUDE.md` — repo-wide invariants, FIX-FORWARD-ONLY policy
- `.claude/projects/-Users-sac-praxis/memory/praxis-toolchain-deps.md` — Rust-side upstream-only
  dependency precedent this document mirrors on the Lean side
