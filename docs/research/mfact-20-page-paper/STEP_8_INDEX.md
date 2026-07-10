# Step 8: Paper Wiring — Index & Preparation Guide

**Date**: 2026-07-07  
**Status**: Preparation phase complete; ready for Step 7 (build verification)  
**Scope**: D1 token-replay correspondence only  
**Integration Target**: combinatorial_maximalism.tex

---

## Deliverables Summary

This step produces three files in `/Users/sac/mfact/paper/`:

| File | Lines | Purpose | Type | Status |
|---|---|---|---|---|
| **PAPER_SECTIONS_DRAFT.md** | 375 | Hand-authored paper sections + auto-render templates | Markdown | Ready |
| **PROSE_LINT_RULES_CORRESPONDENCE.md** | 418 | Lint rules for prose anti-patterns (8 rules, grep-based) | Markdown | Ready |
| **STEP_8_INDEX.md** | This file | Workflow integration guide | Markdown | Reference |

---

## Files Inside PAPER_SECTIONS_DRAFT.md

The draft contains 8 sections ready for combinatorial_maximalism.tex integration:

### 1. Related Work Paragraph (§1, ~300 words)

**Content**: Framed within three precedent branches:
- Vertical (proof depth): Feit-Thompson theorem formalization in Coq (35,000 lines of tactics)
- Horizontal (project scale): Aeneas Rust-to-Lean extraction (arXiv:2206.07185)
- Operational middle layer: OCEL 2.0 process mining + Lean kernel integration

**Citations**:
- Gonthier, Werner (2013): Coq formalization of FLT
- Ho, Protzenko (2022): Aeneas, arXiv:2206.07185
- Protzenko et al. (2024): Charon, arXiv:2410.18042
- Van der Aalst et al. (2023): OCEL 2.0, arXiv:2403.01975

**Status**: Hand-authored; ready for copy-paste to combinatorial_maximalism.tex  
**Revision**: Awaiting user approval on citation accuracy and framing

### 2. Crown Claim Paragraph (§2, ~250 words)

**Content**: The core novelty defended:
- "Standing cannot originate in generation"
- Authority originates exclusively in **admission** (receipt chain)
- Specimen case: D1 token-replay correspondence
- Three falsifiers: (1) lake build exit nonzero, (2) chain validation fails, (3) obligations mismatch

**Design Principle**: Standing = receipt(A, O*, C*, P*, T*, F*) with cryptographic seal  
**Scope Guard**: D1 only; D2-D5 untouched

**Status**: Hand-authored; awaiting user approval for verbatim adoption or revision  
**Revision**: Replace placeholder with approved crown claim if different

### 3. Novelty Claim + Falsifier Checklist (§3, ~600 words)

**Content**:
- **§3.1**: What we DON'T claim (FLT scale, universal extraction, liveness, etc.)
- **§3.2**: Prior art table (OCEL 2.0, SMT, Isabelle, Aeneas, Charon) with scoping
- **§3.3**: Four falsifiable claims with exact falsifiers and evidence paths
- **§3.4**: Scope boundary (D1 only; D2-D5 explicitly out of scope)

**Four Falsifiable Claims**:
1. Extraction validity: Falsified if `lake check` fails
2. Receipt chain integrity: Falsified if single-byte mutation passes validation
3. Conformance witnessed: Falsified if replay fitness ≠ original fitness
4. Standing from admission: Falsified if we claim standing without lake build exit 0

**Status**: Hand-authored; ready for copy-paste  
**Revision**: None needed unless user wants to adjust falsifiers

### 4. Measurable Claims (§4, ~400 words)

**Content**: Raw numbers for D1 metrics, structured in three tables:
- **§4.1 Extraction Cost Curve**: Source lines, event count, generated Lean, build time
- **§4.2 Object-Centric Mining**: Discovered model, conformance %, precision %, generalization %
- **§4.3 Ledger Legibility**: Chain depth, receipt size, validation time, tamper latency, audit time

**Placeholder Format**: `TBD` marks indicate Step 7 fill-in points  
**Generation Path**: build_verif.py extracts metrics from receipts + lake output

**Status**: Structure hand-authored; values TBD for Step 7  
**Revision**: No changes needed; Step 7 populates `TBD` slots

### 5. correspondence_status.tex Template (§5, ~200 lines LaTeX)

**Content**: Auto-rendered LaTeX table + evidence details:
- Status table: Obligation | Status | Evidence | Receipt Hash | Timestamp
- Subsections: Token-replay, Extraction, Lake Build, Chain Validity
- Placeholders: `<!-- AUTO: ... -->` format for build_verif.py substitution

**Rendering Workflow**:
1. Step 7 (build_verif.py) computes all metrics
2. build_verif.py populates template via string substitution
3. Step 8 (paper-render) includes `\input{correspondence_status.tex}` in combinatorial_maximalism.tex

**Status**: Template ready; values populated by Step 7  
**Revision**: Adjust placeholder names if build_verif.py variable names differ

### 6. Hand-Authored vs. Auto-Rendered (§6, table)

**Content**: Stability matrix showing which sections are frozen (hand-authored) vs. generated (re-rendered on every build).

| Section | Type | Stability |
|---|---|---|
| Related Work | Hand-authored | Frozen |
| Crown Claim | Hand-authored | Frozen |
| Novelty + Falsifiers | Hand-authored | Frozen |
| Measurable Claims | Structure hand-authored + placeholders | Partially frozen |
| correspondence_status.tex | Auto-rendered from receipts | Dynamic |

**Status**: Reference only; no user action needed

### 7. Next Steps (§7, checklist)

**Pre-Step 8 checklist**:
1. User review and approval of Related Work, Crown Claim, Novelty
2. Verify citation keys in .bib file
3. Validate placeholder names in correspondence_status.tex
4. Confirm D2-D5 scope boundaries
5. Test falsifiers against verification pipeline

**Status**: Checklist provided; user to execute

### 8. Metadata (§8, table)

**Content**: Document tracking info (date, scope, mathematical foundation, standing definition).

**Status**: Reference only; no user action needed

---

## Files Inside PROSE_LINT_RULES_CORRESPONDENCE.md

Eight lint rules enforce prose discipline for the correspondence sections:

### Rule 1: Forbid "Aeneas proves"
- **Pattern**: `Aeneas\s+(proves|verified|checked|certified)`
- **Correction**: Use "Aeneas extracts" + "Lean proves"
- **Severity**: Error (CI gate: exit 1)

### Rule 2: Forbid Unscoped "verified"
- **Pattern**: Bare `verified` or `proven` without mechanism (via, through, using, by)
- **Correction**: State the mechanism (lake build exit 0, chain hash recompute, syntax check)
- **Severity**: Error

### Rule 3: Require "D1 only" Scope Guard
- **Pattern**: Result or claim without D1/specimen/this-work qualifier within 15 words
- **Correction**: Add scope prefix "D1..." or suffix "...in this specimen"
- **Severity**: Warning (non-fatal; manual review recommended)

### Rule 4: Forbid "automatically" or "without proof"
- **Pattern**: `automatically\s+(extract|verif|prove|check|generate)` or `without\s+(proof|verif)`
- **Correction**: Use "deterministically", "via mechanical rule", "without formal semantics"
- **Severity**: Error

### Rule 5: Require "receipt chain" Specificity
- **Pattern**: Bare `chain` or `hash` in verification context
- **Correction**: Prefix with "receipt", "chain", "content", "payload", "BLAKE3"
- **Severity**: Error

### Rule 6: Forbid Claims Without Falsifiers
- **Pattern**: "We claim" or "We prove" without "if X then falsified" within 2 sentences
- **Correction**: State falsifier after each claim
- **Severity**: Error

### Rule 7: Forbid "proof" for Non-Formal-Proof
- **Pattern**: `prove|proof|proven` without formal context (Lean, lake, kernel, formal)
- **Correction**: Use "evidence", "witness", or qualify with "formal proof via lake"
- **Severity**: Error

### Rule 8: Forbid Unscoped Totality Adverbs
- **Pattern**: `entirely|completely|fully|absolutely|always|never` without scope (D1, specimen, in-this-work)
- **Correction**: Add scope or caveat (e.g., "For D1..." or "...mutation and truncation; wholesale deletion out of scope")
- **Severity**: Error

---

## Integration Workflow: Steps 7 → 8 → 9

### Step 7: Build Verification (build_verif.py)

**Input**: Receipts, lake build output, hook instrumentation  
**Process**: Compute metrics, generate correspondence status data  
**Output**: JSON metrics file + correspondence status variables

**Script Entry Point**:
```bash
python tools/build_verif.py \
  --render-tex \
  --output-dir target/mfact/paper/ \
  --receipts-dir target/praxis-synthesis/receipts/ \
  --lake-build-output target/mfact/build.log
```

### Step 8: Paper Wiring (this step)

**Input**: PAPER_SECTIONS_DRAFT.md, PROSE_LINT_RULES_CORRESPONDENCE.md  
**Process**: Merge sections into combinatorial_maximalism.tex; substitute auto-rendered components; lint prose  
**Output**: combinatorial_maximalism.tex with correspondence sections + correspondence_status.tex included

**Integration Checklist**:
1. Run `just prose-lint` to verify hand-authored sections (exit 0 required)
2. Include `\input{target/mfact/paper/correspondence_status.tex}` in combinatorial_maximalism.tex after §4
3. Copy Related Work (§1) and Crown Claim (§2) into combinatorial_maximalism.tex Related Work section
4. Copy Novelty + Falsifiers (§3) into combinatorial_maximalism.tex Background/Novelty section
5. Copy Measurable Claims (§4) as standalone section before correspondence_status.tex
6. Commit combinatorial_maximalism.tex with all four sections integrated

**Exit Criteria**:
- All prose-lint checks pass (8 rules, exit 0)
- LaTeX compilation succeeds (`latexmk -pdf combinatorial_maximalism.tex` exit 0)
- PDF output contains all four sections + auto-rendered correspondence_status table

### Step 9: Paper Finalization (post-wiring)

**Input**: Compiled combinatorial_maximalism.pdf  
**Process**: Final review, submit to arXiv  
**Output**: Published paper

---

## Key Integration Points

### For combinatorial_maximalism.tex

#### Related Work (§1) location
Insert §1 text into the Related Work section of combinatorial_maximalism.tex, after existing related work and before Contribution section.

**LaTeX stub**:
```latex
\section{Related Work}
% Existing prior work ...

% D1 Correspondence section (from PAPER_SECTIONS_DRAFT.md §1)
\subsection{Correspondence and Proof Extraction}

<<COPY §1 Related Work paragraph>>

```

#### Crown Claim (§2) location
Insert §2 into the Introduction or Background section, *before* Contributions. Establishes what the paper proves and falsifies.

**LaTeX stub**:
```latex
\subsection{The Crown Claim: Standing from Admission}

<<COPY §2 Crown Claim paragraph>>
```

#### Novelty (§3) location
Insert §3 into the Contributions or Background section, detailing what prior work does not claim and what falsifiers apply.

**LaTeX stub**:
```latex
\section{Novelty and Scope}

<<COPY §3.1, §3.2, §3.3, §3.4>>
```

#### Measurable Claims (§4) location
Insert §4 as a standalone section before Results/Evaluation.

**LaTeX stub**:
```latex
\section{Measurable Claims and Metrics}

<<COPY §4.1, §4.2, §4.3 (with TBD placeholders filled by Step 7)>>
```

#### correspondence_status.tex location
Include via `\input{}` after Evaluation / before Conclusion.

**LaTeX stub**:
```latex
\section{D1 Correspondence Status}

\input{target/mfact/paper/correspondence_status.tex}
```

### For build_verif.py

The script must populate these template variables (from correspondence_status.tex):

```python
template_vars = {
    'status_1': 'PROVEN' | 'EXTRACTED' | 'STATED' | 'DECLARED' | 'FAILED',
    'status_color_1': 'green' | 'orange' | 'red',
    'evidence_1': f'Extracted {count} events; lake build {exit_code}',
    'receipt_hash_1': sha256(admission_record),
    'timestamp_1': admission_record['timestamp'],
    'hook_count': int,
    'ocel_event_count': int,
    'extraction_status': 'OK' | 'FAILED',
    'lake_exit_code': int (0 = success),
    'lake_build_time': float,
    'extracted_loc': int,
    'lake_check_status': 'PASS' | 'FAIL',
    'receipt_chain_depth': int,
    'fitness_original': float (target: 1.0),
    'fitness_replay': float (target: 1.0),
    'chain_validation_status': 'VALID' | 'TAMPERED' | 'TRUNCATED',
    'tamper_detection_status': 'DETECTED' | 'MISSED',
    # ... (additional 8-12 variables per subsection)
}
```

See correspondence_status.tex template for full placeholder list.

---

## Quick Reference: File Locations

| File | Path | Lines | Audience | Action |
|---|---|---|---|---|
| Paper Sections Draft | `/Users/sac/mfact/paper/PAPER_SECTIONS_DRAFT.md` | 375 | Author | Review + approve |
| Prose Lint Rules | `/Users/sac/mfact/paper/PROSE_LINT_RULES_CORRESPONDENCE.md` | 418 | Maintainer | Implement in justfile |
| This Index | `/Users/sac/mfact/paper/STEP_8_INDEX.md` | This file | Developer | Reference |
| combinatorial_maximalism.tex | `/Users/sac/mfact/paper/combinatorial_maximalism.tex` | ~36K | LaTeX | Integrate sections |
| correspondence_status.tex | `/Users/sac/mfact/paper/correspondence_status.tex` | Generated | LaTeX | Auto-rendered by Step 7 |

---

## Common Pitfalls & Solutions

### Pitfall 1: TBD placeholders in combinatorial_maximalism.tex before Step 7 runs

**Problem**: If you copy Measurable Claims (§4) into combinatorial_maximalism.tex before Step 7, all values are "TBD". The PDF renders but is incomplete.

**Solution**: Do not integrate §4 into combinatorial_maximalism.tex until Step 7 (build_verif.py) has populated the TBD slots. Either:
- Leave §4 in PAPER_SECTIONS_DRAFT.md as a staging area; merge only after Step 7 populates values.
- Or, include §4 in combinatorial_maximalism.tex but wrap it in a LaTeX comment: `% \input{measurable_claims.tex}` until Step 7 is done.

### Pitfall 2: correspondence_status.tex template variables mismatch

**Problem**: build_verif.py populates variables named `status_1`, but the template expects `status_token_replay`. LaTeX rendering fails with "undefined command".

**Solution**: Before Step 7 runs, verify that variable names in correspondence_status.tex match the variable names build_verif.py will produce. Keep a shared list (e.g., in a VARIABLES.md file) to sync the two.

### Pitfall 3: Prose lint check passes but human reviewer spots a violation

**Problem**: A new variation of "Aeneas proves" slips through (e.g., "Aeneas has proven" instead of "Aeneas proves").

**Solution**: Update the regex pattern in PROSE_LINT_RULES_CORRESPONDENCE.md and re-run `just prose-lint`. Add the new pattern to the justfile recipe.

### Pitfall 4: LaTeX \input{} path is relative; PDF build fails

**Problem**: correspondence_status.tex is at `target/mfact/paper/correspondence_status.tex`, but combinatorial_maximalism.tex is at `target/mfact/paper/combinatorial_maximalism.tex`. Relative path `\input{correspondence_status.tex}` works, but if combinatorial_maximalism.tex is moved, the path breaks.

**Solution**: Use absolute path or consistent relative path. In combinatorial_maximalism.tex: `\input{./correspondence_status.tex}` (same directory).

---

## Next Actions

### Immediate (User)
1. Review PAPER_SECTIONS_DRAFT.md §1-2 for accuracy and tone.
2. Approve or revise the Crown Claim (§2) — this is the falsifiable core.
3. Verify citation keys; cross-check .bib file for missing arXiv entries.

### Before Step 7
1. Set up build_verif.py with template variable names matching correspondence_status.tex.
2. Add a VARIABLES.md file listing which template variables build_verif.py will populate.

### Step 7 (Build Verification)
1. Run build_verif.py to populate metrics and generate correspondence_status.tex.
2. Verify the generated .tex file has no undefined variables.

### Step 8 (Paper Wiring, this step finalized)
1. Merge PAPER_SECTIONS_DRAFT.md §1-4 into combinatorial_maximalism.tex (following integration checklist).
2. Run `just prose-lint` to verify no prose violations.
3. Run `latexmk -pdf combinatorial_maximalism.tex` to compile and verify LaTeX output.
4. Commit combinatorial_maximalism.tex with correspondence sections integrated.

### Step 9 (Final Review & Submission)
1. Final PDF review; check all correspondence sections render correctly.
2. Submit to arXiv / venues.

---

## References

- **Mathematical Foundation**: `/Users/sac/praxis/docs/chatman-equation-phd-thesis.md` (Chatman Equation, three-pole model A ≅ O ≅ L)
- **Research Prior Work**: `/Users/sac/praxis/research/post_chatman_research.md` (FLT, Aeneas, OCEL, post-Chatman formulation)
- **v26.7.4 Context**: `/Users/sac/praxis/docs/jira/v26.7.4/tickets/index.md` (PROJ-301..306, DoD gate)
- **Standing Index**: `/Users/sac/praxis/target/praxis-standing/standing.json` (run `just standing` to generate)

---

**End of Step 8 Preparation**
