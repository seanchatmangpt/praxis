# PROJ-821: CI pipeline had accumulated several independent, pre-existing breaks

**Status**: PARTIAL -- 5 real, independent CI-blocking bugs found and fixed
this session (none caused by the PRs whose failures surfaced them); one
large remaining item (doc warnings promoted to errors) and one out-of-repo
item (sibling `wasm4pm` fmt drift) left open.

## Context

Discovered while investigating why PR #17 (2-file CI-config change) and PR
#18 (1-file docs-only change) both failed nearly every CI check after a
clean rebase onto `main`. Each failure was root-caused individually before
being dismissed as "pre-existing" -- per this repo's own no-overclaiming
discipline, "probably environmental" is not evidence; a command+output is.

## Fixed this session (all on `main` directly, all root-caused with real logs)

1. **`deny.toml` schema** (`05460620`) -- `unmaintained = "warn"` used a
   removed `cargo-deny` schema shape; the tool refused to parse the config
   at all, so `deny` failed before evaluating any rule. Fixed to match
   `bcinr`'s already-migrated `deny.toml`.
2. **`deny.toml` license allow-list** (`7626aacd`) -- with the config
   parsing, 28 real dependencies across 5 license identifiers were
   rejected for the first time. Added them (`Unicode-3.0`, `Zlib`,
   `MPL-2.0`, `BSL-1.0`, `CDLA-Permissive-2.0`, `BUSL-1.1` -- the last
   propagating an already-made, documented `bcinr` decision).
3. **`ggen-code-modernization.yml`'s `cargo install --package`**
   (`379f151e`) -- `--package` is not a valid `cargo install` flag on the
   installed cargo; this exact fix already existed on
   `agent/praxis-dfcm-brce-reconciler` (commit `5c2f22b0`) but was never
   merged to `main`. Applied the same proven fix directly.
4. **`dtolnay/rust-toolchain@1.100`** (`d6d959bc`) -- Dependabot's
   minor-and-patch group mis-bumped this action's Rust-VERSION-selector ref
   (not a normal action release tag) from `@1.82` to a Rust version that
   does not exist (`1.100.0`, confirmed 404 from static.rust-lang.org).
   Reverted to `@1.82` (matches every crate's declared `rust-version`);
   added a `dependabot.yml` ignore rule for this one dependency so the
   same mis-bump can't recur silently.
5. **`dod`/`mcp_server` bin-name collisions** (`066efecd`) -- the root
   package and `crates/ggen` each declare `[[bin]]` entries with the same
   two names; `cargo doc --workspace` refuses outright (same output path).
   Marked `crates/ggen`'s two colliding bins `doc = false` per cargo's own
   suggested remediation.

Each of the five was verified independently (`cargo deny check`, `cargo
doc --workspace`, reading the exact CI job log) before being called fixed,
not assumed from the commit message alone.

## Remaining, NOT fixed here

- **`doc` job's `RUSTDOCFLAGS=-D warnings`** -- with the bin-collision error
  gone, `cargo doc --workspace --no-deps --all-features` now completes but
  emits roughly 116 warnings across several crates (`my-conforming-project`:
  22, `multifractal-workflow`: 93, plus at least one broken intra-doc link
  in `crown-local-cli.rs`'s `[\`BrokerReceipt\`]` reference). CI's `doc` job
  promotes every one of these to a hard error. This is a genuinely large,
  separate cleanup (broken rustdoc links, missing doc comments, etc. across
  many files) -- out of scope for a CI-infrastructure investigation pass.
- **`fmt` job failing inside the sibling `wasm4pm` checkout** -- CI's
  ecosystem-materialization step clones `wasm4pm` fresh and `cargo fmt
  --check` runs across it too; the CI-cloned copy has real formatting
  drift (`tests/ocel_v2.rs`, `crates/wasm4pm-cognition/...`, others). This
  is a different repository's own formatting state -- not something this
  session has standing to fix unilaterally.
- **`typos`, `security-audit` job failures** -- observed failing on both
  PRs but not yet individually root-caused; may be more instances of the
  same "CI environment drifted, was never caught because an earlier job
  always failed first" pattern, or may be genuinely new findings like
  PROJ-820's advisories. Needs the same per-check log read the five fixed
  items above got, not assumed to be the same class without evidence.

## Verification plan (once fully resolved)

Every CI check on a trivial, content-free PR (e.g. a single comment-only
diff) should pass. Until then, `deny`/`doc`/`fmt`/`typos`/`security-audit`
failing is NOT evidence against a given PR's own content -- confirm via
the same "does `main` itself pass this check" test this session used
throughout before attributing a failure to a PR's diff.

## See Also

- `docs/jira/v26.8.16/PROJ-820-deny-toml-schema-and-real-findings.md` --
  the `deny.toml` half of this investigation, with its own remaining
  wildcard-dependency and vulnerability-advisory items
- Commits `05460620`, `7626aacd`, `379f151e`, `d6d959bc`, `066efecd`
