# mathlib_migration_receipts.jsonl — status provenance

`status` reflects `Praxis.lean`'s actual import closure as built by `lake build` on
2026-07-12 (826 jobs, build succeeded), regenerated mechanically by cross-referencing
every label's source file in `Praxis/Corpus/`, `Praxis/Mathlib/`, `Praxis/Milestone/`
against the `.olean` artifacts actually produced under `.lake/build/lib/lean/Praxis/`.
Files outside the closure — present as `.lean` source but never reached by
`Praxis.lean`'s `import` graph, hence no `.olean` emitted — are `not_in_build_closure`,
not "unverified-but-someday-true".

## Status vocabulary

- `verified` — file compiles and is reachable from `Praxis.lean` (has a `.olean`).
- `not_in_build_closure` — file exists under `Praxis/Corpus/` (or `Mathlib/`/`Milestone/`)
  but is not imported (directly or transitively) by `Praxis.lean`, so `lake build` never
  compiles it and no `.olean` is produced. There is no compile error to report for these
  (`last_error` is `null`), because the file is never fed to the compiler at all.
- `unformalized`, `blocked`, `excluded` — unchanged from the prior audit; untouched by
  this pass.

## Label → file mapping

Labels use `kind:name` (e.g. `thm:farkas`); the corresponding source file replaces `:`
and `-` with `_` (e.g. `thm_farkas.lean` under `Praxis/Corpus/`). Confirmed against all
185 `Praxis/Corpus/*.lean` files, 4 `Praxis/Mathlib/*.lean` files, and 10
`Praxis/Milestone/V26711/*.lean` files.

## 2026-07-12 correction

Prior audit (this session) found 82 of 185 `Praxis/Corpus/*.lean` files never compiled
by `lake build` (unreachable from `Praxis.lean`'s import graph). Of those 82:

- 77 were marked `"status": "verified"` in error — corrected to `not_in_build_closure`.
- 5 were already correctly marked `"status": "unformalized"` — left untouched.

`Praxis/Mathlib/` (4/4 files) and `Praxis/Milestone/V26711/` (10/10 files) are fully
inside the build closure; no changes were needed there.

18 labels in the receipt file (`cor:antibody`, `cor:assembled`, `cor:cert`, `cor:cost`,
`cor:noncomp`, `cor:orphan`, `def:fitnessdistance`, `prop:attnconservation`,
`prop:dictcost`, `prop:embed`, `prop:mapcommit`, `prop:section`,
`prop:selfauditedscale`, `thm:conservation`, `thm:faithful`, `thm:localize`,
`thm:scope`, `thm:unitfitness`) have no corresponding `.lean` file anywhere under
`Praxis/` at all. None of them were marked `verified` (17 `blocked`, 1 `excluded`), so
they were out of scope for this correction and were left untouched.

Net result: 202 entries total (unchanged count — no entries added or removed), 178
`verified` before this pass, 101 `verified` after, 77 changed to
`not_in_build_closure`.
