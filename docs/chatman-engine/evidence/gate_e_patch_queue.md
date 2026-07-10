# Gate E Patch Queue

## Applied this session (file edits only, not yet re-verified by cargo)
- `src/shacl/shacl_test.rs:28,58` — `vec![...]` → `[...]` array literal (clippy `useless_vec`);
  content unchanged, only the container type.
- `src/parser_edge_cases_test.rs:85,100,115,164-168` — removed tautological
  `assert!(store.len() >= 0, ...)` (usize is always >= 0; these never asserted anything),
  replaced with `let _store = TripleStore::from(ttl);` — the real assertion in each of these
  4 tests was always "construction completes without panicking," which is preserved
  unchanged; no coverage was removed, only the vacuous clippy-flagged comparison.

## Full clippy scan result (2026-07-09, supersedes the tail-truncated scan)
`cargo clippy -p praxis-graphlaw --all-targets -- -D warnings > /tmp/clippy_full.txt` was
run to completion: **63 errors across 26 files, zero of them in `src/chatman/` or
`tests/chatman_*`**. Sampled offending lines predate commit `2dd4f04` (2026-07-07/08
commits in hooks/, owlrl/, shacl/, sparql/, reasoner/, queryengine/, tripleindex.rs,
bindings.rs). Per the closure-run decision, Gate E is Chatman-scoped: these 63 are
documented preexisting repository debt (named exclusion in `DEFINITION_OF_DONE.md`),
not Gate E failures. The Chatman surface passes clippy today.

## Formatting policy (fixed, not to be re-litigated per file)
- Running `cargo fmt` (the automated formatter, no hand-editing) is allowed on any file,
  including the ggen-generated `chatman_acceptance_*.rs` files that show fmt-check
  failures — **all 8 of them** (admission, agents, hooks, receipts, replay, routing,
  static, triple8; 16 diff sites total per `cargo fmt -p praxis-graphlaw --check`), not
  the 4 previously claimed — this is mechanical whitespace normalization, not a
  hand-edit to generated logic.
- Do NOT hand-edit generated file content/logic. If `cargo fmt` output disagrees with what
  the ggen template produces (i.e. every regeneration re-introduces the diff), the template
  itself needs a formatting fix — flag that separately, do not silently re-run fmt every cycle.

## Mutation/coverage tooling availability — CONFIRMED INSTALLED (corrects earlier assumption)
`cargo-nextest` 0.9.137, `cargo-mutants`, `cargo-llvm-cov` 0.8.5, and `dylint`/`cargo-dylint`
6.0.1 are all present on PATH. The repo's own `justfile` already has the exact recipes for
this: `chatman-verify` (tests + static gates + diagram atlas, prefers nextest), `chatman-quality`
(mutation via `cargo mutants --file 'src/chatman/*'`, coverage via `cargo llvm-cov nextest
--fail-under-lines 85`, `cargo dylint --all --workspace`), and `chatman-sync-verify` (double
ggen sync idempotence, Gate D's determinism check). Use these recipes directly instead of
hand-composing cargo invocations — they're already scoped correctly to `src/chatman/*`.
