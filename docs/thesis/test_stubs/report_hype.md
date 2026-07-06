# E2E LaTeX Test Report

- **Target File:** `docs/thesis/test_stubs/stub_hype.tex`
- **Timestamp:** `2026-07-04 23:54:24 UTC`

## Summary Table

| Step | Status | Issues Found | Details |
| --- | --- | --- | --- |
| Compilation | PASS | 0 | Compilation succeeded. Command: pdflatex -interaction=nonstopmode -output-directory=docs/thesis/build docs/thesis/test_stubs/stub_hype.tex |
| Structural & Content | FAIL | 2 | 0 mismatches, 1 hype, 1 overclaims |
| Notation Canon | PASS | 0 | 0 violations |

## 1. Compilation & Logs

**Result:** Compilation succeeded. Command: pdflatex -interaction=nonstopmode -output-directory=docs/thesis/build docs/thesis/test_stubs/stub_hype.tex

No warnings or errors found in compilation logs.

## 2. Structural & Content Audit

- ✅ All theorem-like environments have matching proofs.

### Hype Word Violations:
- ❌ Line 11: found 'revolutionary' in `This is a revolutionary theorem.`

### Overclaim Violations:
- ❌ Line 14: found 'trivially' in `The proof is trivially correct.`

## 3. Notation Canon Audit

- ✅ No notation canon violations found.

## Verdict

🔴 **FAILED** (Structural / Content failure, Exit Code 3)
