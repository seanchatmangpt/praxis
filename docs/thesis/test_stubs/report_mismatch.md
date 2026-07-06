# E2E LaTeX Test Report

- **Target File:** `docs/thesis/test_stubs/stub_mismatch.tex`
- **Timestamp:** `2026-07-04 23:54:24 UTC`

## Summary Table

| Step | Status | Issues Found | Details |
| --- | --- | --- | --- |
| Compilation | PASS | 0 | Compilation succeeded. Command: pdflatex -interaction=nonstopmode -output-directory=docs/thesis/build docs/thesis/test_stubs/stub_mismatch.tex |
| Structural & Content | FAIL | 2 | 2 mismatches, 0 hype, 0 overclaims |
| Notation Canon | PASS | 0 | 0 violations |

## 1. Compilation & Logs

**Result:** Compilation succeeded. Command: pdflatex -interaction=nonstopmode -output-directory=docs/thesis/build docs/thesis/test_stubs/stub_mismatch.tex

No warnings or errors found in compilation logs.

## 2. Structural & Content Audit

### Theorem-Proof Structural Mismatches:
- ❌ Total environment count mismatch: 1 theorem-like environment(s) vs 0 proof(s).
- ❌ Theorem-like environment 'theorem' at line 10 has no matching proof at the end of the document.
- ✅ No hype words found.
- ✅ No overclaim words found.

## 3. Notation Canon Audit

- ✅ No notation canon violations found.

## Verdict

🔴 **FAILED** (Structural / Content failure, Exit Code 3)
