# arXiv submission package — Praxis v26.7.6 "After Neon" (dry-run, NOT submitted)

Assembled 2026-07-06 by the v26.7.6 verification gate. This is a dry-run
package: nothing has been uploaded to arXiv. See
`docs/releases/v26.7.6/ARXIV_READINESS.md` for open blockers to actual
submission (public artifact bundle, axiom census).

## Contents

| File | SHA-256 | Role |
|---|---|---|
| `arxiv-submission.tar.gz` | `67e0725f3c1299ca1b0a66f7b1e64fc4816612e1ece941fdb0185727875ad767` | the upload candidate (28 KB) |
| `src/thesis.tex` | `8f8d7d8a60fa2fd3d04968778bef269f6bd17014e4616edbf79d32c13f5e836a` | master file (`\documentclass[11pt]{report}`); bibliography is inline `thebibliography` — no `.bbl` needed |
| `src/generated/chapters_body.tex` | `4d1f5498da513c43fe811ace40e2ed97782194cf344cc575aee7e6600ef02665` | chapter body, ggen-rendered from `docs/thesis/math_manufacturing/rdf/thesis.ttl` (source of truth is the TTL; regenerate via `docs/thesis/math_manufacturing/regenerate.sh`) |

No figures: the source contains no `\includegraphics` (grep over both files,
2026-07-06). No `.bbl`: citations are an inline `thebibliography` block
(`thesis.tex:75-107`).

## Build verification

- Repo build: `latexmk -pdf -interaction=nonstopmode -halt-on-error thesis.tex`
  in `docs/thesis/math_manufacturing/` — exit 0, 30 pages, output
  `thesis.pdf` SHA-256
  `c46c43be320850093e755f495464adf5b6c41ccbf734cd368bbb745c8d82f0b7`.
  Only warning: benign font-shape substitution (`OMS/cmtt/m/n` undefined).
- Standalone build: the exact file set in `src/` was copied to a clean
  directory and built with the same latexmk command — exit 0. The tarball is
  self-contained.

## Submission command (operator, after blockers close)

Upload `arxiv-submission.tar.gz` at https://arxiv.org/submit (category
cs.LO or cs.SE cross cs.AI, per ARXIV_READINESS.md). Do not submit until
ARXIV_READINESS.md Sec. 11 blockers 2-3 (public artifact bundle, axiom
census) are closed.
