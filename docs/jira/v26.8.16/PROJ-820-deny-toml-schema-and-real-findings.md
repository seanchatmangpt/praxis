# PROJ-820: `deny.toml` schema was broken (config-parse error), masking real cargo-deny findings

**Status**: PARTIAL -- schema fixed (commit `05460620`) and all 28 license
rejections resolved (commit `7626aacd`); 6 wildcard-dependency findings and
6 vulnerability advisories remain open, needing real per-item decisions.

## Scope

Discovered while investigating why PR #17's `deny` CI check failed after a
clean rebase onto current `main` (which itself has a fully green `just
check`): `deny.toml`'s `[advisories]` section still used the OLD
`cargo-deny` schema (`vulnerability = "deny"`, `unmaintained = "warn"`,
`notice = "warn"`) against the CURRENTLY INSTALLED `cargo-deny`, whose
schema removed the `vulnerability`/`notice` keys entirely and changed
`unmaintained` from a warn-level string to a scope enum (`all` / `workspace`
/ `transitive` / `none`). The tool refused to even parse the config:

```
error[unexpected-value]: expected ["all", "workspace", "transitive", "none"]
   ┌─ ./deny.toml:15:17
15 │ unmaintained = "warn"
```

This means `cargo deny check` / `just deny` has been hard-failing at the
config-parse step -- before evaluating a single real advisory, ban, or
license rule -- on every PR and on `main`'s own CI, for an unknown but
apparently long period (confirmed present on `main` before any of this
session's changes).

## Fix 1: schema (DONE, commit `05460620`)

Matched `bcinr`'s own `deny.toml` (same ecosystem, already migrated):
`unmaintained = "workspace"`, `vulnerability`/`notice` keys removed.
Config now parses.

## Fix 2: license allow-list (DONE, commit `7626aacd`)

With the config parsing, `cargo deny check licenses` ran to completion for
the first time and rejected 28 real dependencies across 5 previously-
unencountered license identifiers (`Unicode-3.0`, `Zlib`, `MPL-2.0`,
`BSL-1.0`, `CDLA-Permissive-2.0`) plus `BUSL-1.1` (`prolog8`,
`wasm4pm-cognition` -- a load-bearing transitive dependency pair). All six
were added to `allow`; `BUSL-1.1` propagates a decision `bcinr`'s own
`deny.toml` already made and documented for these exact two crates, not a
fresh policy call. `cargo deny check licenses` now reports `licenses ok`.

## Remaining: [bans] wildcard-dependency findings (NOT fixed)

`cargo deny check bans` reports 6 `error[wildcard]` findings: `agent8`,
`my-conforming-project` (6 occurrences), `praxis-core` (4), `praxis-graphlaw`
(1), `praxis-proposer` (1), `rust-fable-testbed` (1) are consumed as path
dependencies with NO `version` field by roughly 15 different `Cargo.toml`
files across this workspace and its sibling repos (`bcinr`, `wasm4pm`,
`affidavit`, `ggen`, `chicago-tdd-tools`, `lsp-max` all have entries like
this). `allow-wildcard-paths = true` no longer exempts these under the
current `cargo-deny`: "does not apply to public crates as crates.io
disallows path dependencies" -- i.e. any crate without `publish = false`
that's depended on via a versionless `path = "..."` now fails this check
regardless of the wildcard-paths allowance.

**Why not fixed here**: the mechanical fix (add `, version = "X.Y.Z"` to
each unversioned entry) is straightforward, but which exact version string
is correct is NOT straightforward -- this repo's existing, ALREADY-VERSIONED
path deps are inconsistent about whether the pinned version matches the
target crate's current on-disk `[package].version` exactly, or an older,
deliberately-frozen value (e.g. several existing entries pin
`wasm4pm-compat` to `26.6.29` while the crate's on-disk version is `26.8.7`
today). Picking the wrong convention for ~35 new version strings across
~15 files, several of them in sibling repos this session does not own,
risks silently changing publish-time SemVer requirements rather than just
satisfying a lint. This needs either a human call on which convention
applies, or a careful per-crate check of whether each target crate's
version has genuinely moved since its LAST intentionally-pinned reference
elsewhere in the same ecosystem.

## Remaining: [advisories] vulnerability findings (NOT fixed)

`cargo deny check advisories` reports 6 `error[vulnerability]` findings
across 4 distinct RUSTSEC IDs:

- `RUSTSEC-2026-0189` -- `rmcp` 0.11.0, DNS rebinding vulnerability in the
  Streamable HTTP server transport (loopback/private-network exposure via
  a malicious page). Fix: upgrade to `rmcp >= 1.4.0`. This is a REAL,
  actionable, low-risk upgrade (not a transitive-only dependency) -- the
  most concerning of the four and the one most worth acting on first.
- `RUSTSEC-2026-0194` / `RUSTSEC-2026-0195` -- `quick-xml` 0.36.2/0.37.5,
  transitive via `oxrdfxml`/`sparesults` -> `oxigraph` -> `ggen-graph`/
  `praxis-graphlaw`/`lsp-max`/etc. `bcinr`'s own `deny.toml` already
  documents these exact two IDs as transitive-only and not fixable from
  that workspace's own `Cargo.toml`, with an `ignore` list entry and a
  cross-reference to `.cargo/audit.toml`'s rationale. Praxis likely needs
  the same `ignore` entries (not yet added) rather than an upgrade, since
  the dependency is pulled in the same way.
- `RUSTSEC-2026-0258` -- not yet read; needs its own advisory-text lookup
  before deciding ignore vs. upgrade vs. something else.

**Why not fixed here**: upgrading `rmcp` and/or adding `ignore` entries for
the quick-xml pair are both real, scoped, low-risk moves that a follow-up
pass should make quickly -- but doing so without reading `RUSTSEC-2026-0258`
first, and without verifying an `rmcp` upgrade doesn't break anything that
depends on its current API surface, would be exactly the kind of rushed
fix this session's own `.claude/rules/_core/absolute.md` "Fence" discipline
warns against.

## Verification plan (once fully fixed)

```
cargo deny check
```
must report `advisories ok, bans ok, licenses ok, sources ok` (or each
non-ok category must carry an explicit, reasoned `ignore`/`allow` entry,
not a silent pass).

## See Also

- `/Users/sac/bcinr/deny.toml` -- the same-ecosystem sibling repo whose
  already-migrated schema and license allow-list this fix propagated from,
  including its own documented `BUSL-1.1` and quick-xml-ignore precedents
- Commits `05460620`, `7626aacd`
