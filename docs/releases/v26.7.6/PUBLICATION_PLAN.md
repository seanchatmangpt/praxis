# Publication Plan — Praxis v26.7.6 "After Neon"

Target: arXiv submission (cs.SE primary; cs.LO cross-list) of "Praxis
v26.7.6 After Neon: Verified AI Engineering Infrastructure for Manufacturing
Standing in AI-Generated Technical Work". Prerequisite readiness assessment:
`docs/releases/v26.7.6/ARXIV_READINESS.md`. Each step has an exit gate; no
step is skipped by declaring it done.

## Step 1 — Close the evidence gaps (in-repo)

- Run `just verify-all`; capture output into RELEASE_CONTROL.md Sec. 8
  (closes exit criterion 1).
- Run receipt-chain verification (`crates/ggen/tests/receipt_chain_e2e.rs`
  path); record receipt rows in RELEASE_CONTROL.md Sec. 7 (criterion 7).
- Land the one-command full-loop demo with byte-identical receipts across
  two runs (criterion 3) — this backs the paper's determinism claim.
- Axiom census over `tools/paper-factory/lean-lake/Praxis/Corpus/*.lean`
  (grep `axiom` per file, emit a table); required by ARXIV_READINESS.md
  Sec. 6 and ADVERSARIAL_REVIEW.md challenge 1.

Exit gate: RELEASE_CONTROL.md criteria 1, 3, 7 green with recorded outputs.

## Step 2 — Assemble the paper from existing corpus

- Source of truth stays RDF: extend
  `docs/thesis/math_manufacturing/rdf/thesis.ttl` (or a sibling TTL) with
  the systems-paper sections; render via `ggen sync` + doc-pack, as the
  thesis already does (`docs/thesis/math_manufacturing/thesis.tex` header).
- Import: abstract and claim-boundary table from ARXIV_READINESS.md Sec. 1–2;
  threats-to-validity from ADVERSARIAL_REVIEW.md; figures/tables per
  ARXIV_READINESS.md Sec. 8; keep Part-II economics out entirely or inside a
  marked Speculation environment.
- Add missing citations (POWL, PDDL, BLAKE3, SHACL/ShEx, N3) to the existing
  bibliography (Lean 4/mathlib/AlphaGeometry/autoformalization anchors
  already present in `thesis.tex`).

Exit gate: PDF builds from regenerated LaTeX; every quantitative claim in
the PDF resolves to a path/test/receipt.

## Step 3 — Internal adversarial pass

- Re-run the ADVERSARIAL_REVIEW.md matrix against the draft: any challenge
  still OPEN must appear in the paper's limitations, verbatim in substance.
- Number check: partition sums (219 = 179+4+18+17+1-adjustments; 202-label
  Mathlib lane 178 verified + 6 unformalized + remainder) recomputed from
  `mathlib_migration_receipts.jsonl` and `rdf/thesis.ttl`, not copied — the
  recorded 218-vs-219 incident (`rdf/thesis.ttl:63-64`) is the reason this
  step exists.

Exit gate: zero unbacked claims; limitations section covers all OPEN rows.

## Step 4 — Public artifact bundle

- Bundle: `docs/thesis/math_manufacturing/rdf/thesis.ttl`,
  `tools/paper-factory/lean-lake/` (sources + `mathlib_migration_receipts.jsonl`),
  `examples/v26_7_6_after_neon/`, receipt chain snapshot, and a replay
  script (cold-start Lake rebuild + `ggen sync` + full-loop demo).
- Host: public git repo or Zenodo deposit (DOI preferred for the artifact
  citation). Decision and license (code: MIT/Apache-2.0 dual is the Rust
  norm; text: CC-BY-4.0) recorded here when made — currently OPEN.
- Verify the bundle on a second machine; record that run's receipts.

Exit gate: a third party can replay the Lean verdicts and the factory demo
from the bundle alone.

## Step 5 — Submit

- arXiv metadata: title as above; abstract from ARXIV_READINESS.md Sec. 1;
  artifact DOI in the abstract's final sentence.
- Post-submission: record the arXiv ID and submission receipt in
  RELEASE_CONTROL.md Sec. 10; open a follow-up ticket for venue targeting
  (workshop/conference) — out of scope for this plan.

## Sequencing and dependencies

Step 1 blocks 2 (determinism claim) and 4 (replay script). Steps 2 and the
axiom census can run in parallel. Step 3 blocks 5. Nothing here depends on
FORTUNE5/DEPLOYMENT work; publication and pilot tracks are independent
except for shared Step-1 gates.

## Risks

- Full-loop determinism (criterion 3) may surface nondeterminism (map
  ordering, path iteration) — fix-forward in the factory, never by relaxing
  the byte-identical bar.
- Cold-start Mathlib rebuild is heavy (prebuilt cache dependency, commit
  `dab70b7`); the replay script must pin the cache source or accept long
  build times.
- The private→public repo transition must exclude unrelated dirty files and
  any leaked-token history (see memory census flag); publish the bundle,
  not the working repo.
