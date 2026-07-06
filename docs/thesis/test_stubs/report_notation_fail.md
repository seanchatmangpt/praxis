# E2E LaTeX Test Report

- **Target File:** `docs/thesis/test_stubs/stub_notation_fail.tex`
- **Timestamp:** `2026-07-04 23:54:24 UTC`

## Summary Table

| Step | Status | Issues Found | Details |
| --- | --- | --- | --- |
| Compilation | PASS | 0 | Compilation succeeded. Command: pdflatex -interaction=nonstopmode -output-directory=docs/thesis/build docs/thesis/test_stubs/stub_notation_fail.tex |
| Structural & Content | PASS | 0 | 0 mismatches, 0 hype, 0 overclaims |
| Notation Canon | FAIL | 1 | 1 violations |

## 1. Compilation & Logs

**Result:** Compilation succeeded. Command: pdflatex -interaction=nonstopmode -output-directory=docs/thesis/build docs/thesis/test_stubs/stub_notation_fail.tex

No warnings or errors found in compilation logs.

## 2. Structural & Content Audit

- ✅ All theorem-like environments have matching proofs.
- ✅ No hype words found.
- ✅ No overclaim words found.

## 3. Notation Canon Audit

### Notation Canon Violations:
- ❌ Notation Violation: Calligraphic/Macro symbol '\mathcal{O}' at line 10 is used outside the 6 allowed Chatman Equations.
Context: Here we use the symbol $\mathcal{O}$ in a random sentence which is a notation violation.

## Verdict

🔴 **FAILED** (Notation Canon failure, Exit Code 4)
