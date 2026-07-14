# lean-pilot — SUPERSEDED

**Status: superseded, never kernel-checked.** Do not cite this directory's `formalization_receipts.jsonl`
as evidence of proof. See `tools/paper-factory/lean-lake/Praxis/Corpus/` for the actual built Lake
project.

## What this directory is

~183 standalone `.lean` draft files, bare Lean 4 (no `import Mathlib`, no cross-file imports), with
no `lakefile.lean` and no `lean-toolchain`. There are zero `.olean` files anywhere in this directory
and nothing elsewhere in the repo imports these files. They have never been part of any build.

`formalization_receipts.jsonl` previously claimed 179/201 entries `"verified"` (18 `"blocked"`, 4
`"unformalized"`). That claim was never backed by a real Lean build — no lakefile existed to run
`lake build` against. This session (2026-07-12) corrected every entry's `status` field to
`"superseded_unverified"`.

## Relationship to lean-lake/Praxis/Corpus

176 of ~183 filenames in this directory have a same-named counterpart in
`tools/paper-factory/lean-lake/Praxis/Corpus/`. Diffing several matching pairs (`thm_mrr.lean`,
`ax_cr.lean`, `prop_fitness.lean`, `con_agent8.lean`) shows they are **not identical** — the
lean-lake versions are later reformalizations: they `import Mathlib.*` and `import Praxis.Corpus.*`
(cross-referencing other corpus modules), use Lake-project module paths, and in some cases restate
the theorem differently. The lean-pilot versions are earlier, self-contained, bare-Lean-4 drafts of
the same labels (`thm:mrr`, `ax:cr`, `prop:fitness`, `con:agent8`, ...).

This is consistent with lean-pilot being an earlier draft stage, superseded by the Lake-built corpus.
Because lean-lake owns an independent Lake project (lean-toolchain, lakefile.lean, `lake build`
target — see that directory, not touched by this session), lean-pilot was **not** turned into a
second parallel Lake project; that would just duplicate already-built content under a different,
never-verified name.

7 filenames exist only in lean-pilot with no lean-lake counterpart: `cor_assembled.lean`,
`def_fitnessdistance.lean`, `prop_attnconservation.lean`, `prop_dictcost.lean`, `prop_embed.lean`,
`prop_section.lean`, `thm_unitfitness.lean`. These are also unverified — never built, no lakefile —
and are not automatically superseded by name-match, but the directory as a whole is not real
verified work either way and needs a lakefile to make any claim about them.

12 filenames exist only in lean-lake with no lean-pilot draft: `ax_curve.lean`,
`ax_restartpolicy.lean`, `ax_verify.lean`, `cor_galois.lean`, `cor_mrrindep.lean`, `def_conf.lean`,
`def_polytope.lean`, `prop_fleet.lean`, `prop_payloadcommit.lean`, `prop_thermo3.lean`,
`thm_farkas.lean`, `thm_mono.lean` — these are lean-lake-only content, out of scope here.

## If you need the real, checked status of a label

Check `tools/paper-factory/lean-lake/Praxis/Corpus/` and that project's own build receipts, not this
directory's `formalization_receipts.jsonl`.

## Terminology note

The `.jsonl` files in this directory are self-reported status logs from formalization attempts
(`label`/`status`/`attempts`/`last_error` fields, no hash, no timestamp), not cryptographic
receipts; do not cite "verified" status here as proof without independently re-running `lake build`.
For a real BLAKE3 hash-chained receipt, use `crates/praxis-lean`'s `praxis-l4 verify` command
(schema v2, `crates/praxis-lean/src/receipt.rs`), which this file predates.
