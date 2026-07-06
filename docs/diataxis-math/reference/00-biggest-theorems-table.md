# Reference: the biggest theorems, exact citations and verification status

Every claim on this page is sourced from a real file: the theorem statement's `.tex`
location, and (where attempted) the exact Lean file and the corresponding line in one of
the two receipts files:

- **Bare-core lane**: `tools/paper-factory/lean-pilot/formalization_receipts.jsonl`
- **Mathlib lane**: `tools/paper-factory/lean-lake/mathlib_migration_receipts.jsonl`

"Verified" means the Lean 4 kernel (`lake env lean <file>` or bare `lean <file>`)
accepted the file with exit code 0 — not that an agent or a person merely claimed it did.
"Unformalized" means a translation attempt was made (up to 3 tries) and a real compiler
error remains on file. "Blocked" means the statement was never attempted because a
dependency it needs wasn't itself verified. "Excluded" means it was never attempted by
design (see the entry's note).

| Theorem | One-sentence claim | Paper / line | Bare-core lane | Mathlib lane |
|---|---|---|---|---|
| `thm:rice` | No algorithm can decide any non-trivial *meaning*-based property of an observation. | `00_foundations.tex:183` | verified — `lean-pilot/thm_rice.lean` | verified — `lean-lake/.../thm_rice.lean`, cites Mathlib's own `ComputablePred.rice₂` directly, 0 new axioms |
| `thm:mono` | Adding obligations can only shrink the admitted set, never grow it. | `01_admission_algebra.tex:318` | blocked (dependency unverified) | verified |
| `thm:total` | The 13-scenario-to-7-category classification is total and compiler-certified. | `01_admission_algebra.tex:419` | verified | unformalized (3 attempts, real Lean errors) |
| `thm:freehom` | Pipeline denial is order- and repetition-independent (a free-monoid homomorphism into a semilattice). | `01_admission_algebra.tex:557` | verified | verified |
| `thm:faithful` | Any tamper to a committed field changes the terminal chain hash, unless the hash function itself is broken. | `02_receipt_cryptography.tex:387` | not attempted (excluded by design — no verified BLAKE3/cryptographic-hardness framework exists in Lean/Mathlib to cite; faking one would be meaningless) | excluded, same reason |
| `thm:conservation` | Every actuated artifact traces to exactly one admitted cause; no orphaned or double-counted actions. | `02_receipt_cryptography.tex:445` | blocked | blocked |
| `thm:localize` | Tampering is caught at the exact record it occurred at; the honest prefix is never implicated. | `02_receipt_cryptography.tex:522` | blocked | blocked |
| `thm:bounded-ground` | The number of ground actions is a fixed, finite bound independent of the goal. | `03_planning_geometry.tex:189` | verified | unformalized (3 attempts — missing `Finset.sum_le_sum`/`smul_eq_mul` in this Mathlib snapshot) |
| `thm:farkas` | Farkas' lemma certifies plan-unreachability with a short, checkable, order-independent certificate. | `03_planning_geometry.tex:410` | blocked | unformalized (missing `PointedCone.FG.isClosed` in this Mathlib snapshot) — **not machine-verified in either lane** |
| `thm:lang-correct` | Converting a workflow between two internal representations preserves the exact set of valid step-sequences at every prefix length. | `03_planning_geometry.tex:678` | verified | verified (first attempt) |
| `thm:kill` | A staged validator kills a mutant iff it's real, and always at the correct stage. | `04_projection_and_scale.tex:424` | verified | verified |
| `thm:branchless` | Checking 64 agents' 8-condition admission is 7 branchless AND instructions total. | `04_projection_and_scale.tex:547` | verified | verified |
| `thm:swar-verify` | The branchless bit-trick sweep computes the same result as the naive per-agent check. | `04_projection_and_scale.tex:596` | verified | verified, axioms used: `propext`, `Classical.choice`, `Quot.sound` (Lean's own 3 standard foundational axioms — nothing corpus-specific) |

## How to reproduce any row of this table yourself

```sh
# Bare-core lane (no Mathlib dependency, fast)
cd tools/paper-factory/lean-pilot
lean thm_rice.lean   # substitute the sanitized label, e.g. thm_bounded_ground.lean

# Mathlib lane (first run is slow -- see tutorials/00-verify-a-theorem-yourself.md)
cd tools/paper-factory/lean-lake
lake env lean Praxis/Corpus/thm_rice.lean
```

Exit code `0` and no output means the Lean kernel accepted the file — you have now
independently reproduced one row of this table on your own machine, without trusting this
document, a receipts file, or any prior claim.
