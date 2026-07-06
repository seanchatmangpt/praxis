# Axiom census — Mathlib-lane Lean corpus (v26.7.6)

Source: `praxis-l4 no-sorry --root tools/paper-factory/lean-lake/Praxis`,
run 2026-07-06 against the tree that `lake build` kernel-accepts (826 jobs,
exit 0, same date). Every finding below is an `axiom` declaration; zero
`sorry`/`admit` findings anywhere under `Praxis/`. This closes
ARXIV_READINESS.md Sec. 11 blocker 3 (axiom census): the verified count
measures kernel acceptance, and these are the per-file axiom counts the
paper must report alongside it. The tree holds 184 files in
`Praxis/Corpus/` and 4 in `Praxis/Mathlib/`.

| File | axiom declarations |
|---|---|
| `Praxis/Corpus/ax_cr.lean` | 4 |
| `Praxis/Corpus/ax_obs.lean` | 1 |
| `Praxis/Corpus/ax_refusal.lean` | 2 |
| `Praxis/Corpus/con_commit.lean` | 1 |
| `Praxis/Corpus/con_fablechain.lean` | 1 |
| `Praxis/Corpus/con_merklecell.lean` | 1 |
| `Praxis/Corpus/con_xorf.lean` | 1 |
| `Praxis/Corpus/def_body.lean` | 2 |
| `Praxis/Corpus/def_contentaddr.lean` | 1 |
| `Praxis/Corpus/def_obsauth.lean` | 4 |
| `Praxis/Corpus/def_receipt.lean` | 1 |
| `Praxis/Corpus/def_residual.lean` | 2 |
| `Praxis/Corpus/def_sandbox.lean` | 5 |
| `Praxis/Corpus/def_vizgap.lean` | 1 |
| `Praxis/Corpus/def_walframe.lean` | 1 |
| `Praxis/Corpus/est_bit_supply.lean` | 2 |
| `Praxis/Corpus/est_comp_supply.lean` | 18 |
| `Praxis/Corpus/lineage_armstrong.lean` | 4 |
| `Praxis/Corpus/meas_recoveryinvisible.lean` | 1 |
| `Praxis/Corpus/prop_boundary.lean` | 1 |
| `Praxis/Corpus/prop_intauth.lean` | 2 |
| `Praxis/Corpus/prop_topology.lean` | 1 |
| `Praxis/Corpus/ref_curve.lean` | 2 |
| `Praxis/Corpus/refusal_noknhkpath.lean` | 2 |
| `Praxis/Corpus/refusal_nomeasuredticks.lean` | 2 |
| `Praxis/Corpus/refusal_simpleoneforone.lean` | 3 |
| `Praxis/Mathlib/DefReceipt.lean` | 2 |
| `Praxis/Mathlib/ObsSimEquivalence.lean` | 3 |
| **total** | **71** across 28 files |

160 of 188 files declare no axioms. The dominant
contributor is `Praxis/Corpus/est_comp_supply.lean` (18):
computational-supply estimates with no Mathlib formalization surface.
Regenerate with the command above.
