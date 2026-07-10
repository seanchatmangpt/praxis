# Prose Lint Rules: Correspondence Chapters

**Purpose**: Prevent common prose anti-patterns when authoring the correspondence sections of the paper. Each rule is a forbid pattern (what not to write) with a corresponding grep pattern to detect violations and a suggested replacement.

**Enforcement**: Run `just prose-lint` to execute all checks. Violations cause a non-zero exit code; the CI gate requires exit 0.

---

## Rule 1: Forbid "Aeneas proves"

**Rationale**: Aeneas is an *extraction tool*, not a prover. Saying "Aeneas proves X" implies that Aeneas generated the proof of X, which is false. Aeneas extracts Rust code to Lean; Lean's kernel proves (or fails to prove) properties of the extracted code.

**Forbid Pattern**: Any phrase matching `Aeneas\s+(proves|verified|checked|certified)`.

**Grep Command**:
```bash
grep -n "Aeneas\s\+\(proves\|verified\|checked\|certified\)" combinatorial_maximalism.tex PAPER_SECTIONS_DRAFT.md
```

**Violation Example**:
```latex
Aeneas proves that the Rust code is memory-safe.
```

**Corrected Examples**:
```latex
Aeneas extracts Rust code to Lean, and Lean's kernel proves memory safety.

Aeneas extracts the Rust code; we then use Lean's kernel to prove the correspondence.

The correspondence witness is extracted by Aeneas into Lean 4 syntax.
```

**Exception**: If the phrase is in a direct quote from Aeneas documentation or a cited paper, wrap it in `\texttt{...}` and cite the source inline.

---

## Rule 2: Forbid Unscoped "verified"

**Rationale**: The word "verified" is ambiguous—it could mean type-checked, runtime-verified, formally proven, or just "declared and not contradicted." Every use must specify the *mechanism* of verification. In the correspondence context, "verified" means "re-admitted by Lean 4's kernel" or "chain hash re-computed correctly" or "lake build exit 0". Bare "verified" is theater.

**Forbid Pattern**: `\b(verified|verified|proven)\b` followed by something other than `by`, `via`, `through`, `using`, or `when` within 2 words. Also forbid bare `is verified` or `was verified` without a mechanism clause.

**Grep Command**:
```bash
grep -nE "([^.]*)\b(verified|proven)\b([^.]*?)(\s|\.)" combinatorial_maximalism.tex PAPER_SECTIONS_DRAFT.md | \
  grep -v "via\|through\|using\|by\|when\|because" | \
  head -20
```

**Violation Examples**:
```latex
The correspondence is verified.

The extraction was verified by Aeneas.  % (violates Rule 1 + Rule 2)

Our approach verifies conformance.

The log is verified to conform.
```

**Corrected Examples**:
```latex
The correspondence is verified by re-admitting the extracted code through Lean 4's kernel (lake build exit 0).

The extraction is valid when syntax-checked via Lean 4's parser.

Conformance is witnessed via receipt chain re-validation (chain_hash recompute passes).

The log conforms to the POWL model when token-replay fitness = 1.0.
```

---

## Rule 3: Require "D1 only" Scope Guard

**Rationale**: The paper claims apply only to D1 token-replay correspondence. Any statement that could be read as a general claim (across D2-D5, or for all correspondences, or for process mining in general) must include an explicit scope qualifier: "D1 only", "in this specimen", "for the token-replay target", or "in this work".

**Forbid Pattern**: Claim or result statement without at least one of: `"D1"`, `"specimen"`, `"this work"`, `"token-replay"`, `"this correspondence"` appearing within 15 words before the period.

**Grep Command**:
```bash
grep -nE "^[^.]*\." combinatorial_maximalism.tex PAPER_SECTIONS_DRAFT.md | \
  grep -v "D1\|specimen\|this work\|token-replay\|this correspondence\|this apparatus" | \
  grep -E "extract|correspond|admission|proof|verif" | \
  head -20
```

**Violation Examples**:
```latex
Extraction produces a Lean 4 program that lake compiles successfully.
% Missing D1 scope guard; sounds like a general claim.

The receipt chain proves conformance.
% Could imply this holds for all correspondences, but we only show it for D1.

Process-mined OCEL logs enable formal verification.
% Overstates the claim; we show this for one specific log.
```

**Corrected Examples**:
```latex
D1 extraction produces a Lean 4 program that lake compiles successfully.

In this specimen (D1), the receipt chain proves conformance by witnessing token-replay fitness 1.0.

For the D1 token-replay target, process-mined OCEL logs are re-admitted through Lean 4's kernel.

Our approach witnesses the D1 correspondence via a receipt chain whose integrity is validated by replay.
```

**Exception**: When discussing prior work (Related Work section), scope guards are not required for other people's claims (e.g., "Aeneas extracts Rust to Lean" is understood to be Aeneas's claim, not ours). Use scope guards for statements about this work's results only.

---

## Rule 4: Forbid "automatically" or "without proof"

**Rationale**: In the extraction and proof context, "automatically" and "without proof" are false claims. Automatic extraction is still extraction (requires code); "without proof" means "without re-admitting through the kernel", which is exactly what our work avoids. Use specific language: "deterministically", "without human annotation", "via mechanical rule", "without formal semantics" (if the source is unverified).

**Forbid Pattern**: `automatically\s+(extract|verif|prove|check|generate)` or `without\s+(proof|verif|checking)`.

**Grep Command**:
```bash
grep -n "automatically\|without proof\|without verif\|without check" combinatorial_maximalism.tex PAPER_SECTIONS_DRAFT.md | head -20
```

**Violation Examples**:
```latex
Aeneas automatically extracts Rust to Lean without proof.
% "without proof" is wrong; the extracted code still needs Lean to prove it.

Our tool automatically verifies conformance.
% "automatically" hides the lake build step; be specific.

The correspondence is established without formal reasoning.
% Suggests we're not doing formal verification, which we are.
```

**Corrected Examples**:
```latex
Aeneas mechanically extracts Rust to Lean via a deterministic translation; Lean's kernel then re-admits the code.

Our tool deterministically computes token-replay fitness and witnesses it via a receipt chain.

The D1 correspondence is established via formal verification: we re-admit the extracted code through Lean 4, and lake build exit 0 is the proof.

The mechanism requires no human annotation of the event log.
```

---

## Rule 5: Require "receipt chain" Specificity

**Rationale**: "Chain" alone is ambiguous (could mean git, proof chain, supply chain, etc.). In the correspondence context, always say "receipt chain" or "chain hash" or "chain linkage", never bare "chain". Similarly, "hash" alone is vague; say "chain hash", "payload hash", "content hash", or specify which digest is meant.

**Forbid Pattern**: `\bchain\b` or `\bhash\b` appearing alone (not preceded by "receipt", "chain", "content", "payload", "BLAKE3", etc.) in sentences about verification or admission.

**Grep Command**:
```bash
grep -nE "\s(chain|hash)\s" combinatorial_maximalism.tex PAPER_SECTIONS_DRAFT.md | \
  grep -v "receipt chain\|chain hash\|content hash\|payload hash\|BLAKE3\|chain linkage\|chain integrity" | \
  head -20
```

**Violation Examples**:
```latex
The chain is tamper-evident.
% Ambiguous; which chain?

A hash commits the payload.
% Which hash? Use content hash or chain hash.

Validation recomputes the chain.
% Unclear; receipt chain recomputes?
```

**Corrected Examples**:
```latex
The receipt chain is tamper-evident: any mutation in a payload hash or chain hash is detected on recompute.

A BLAKE3 content hash commits the payload; a chain hash commits the content hash plus the previous chain hash.

Validation recomputes the chain hash for each receipt and compares against the stored value.
```

---

## Rule 6: Forbid "Claims" Without Falsifiers

**Rationale**: Every falsifiable claim in the paper (Sections 3.3-3.4) must state what would prove it false. Bare claims like "X is true" without naming how you'd know if X were false are not falsifiable. This rule applies to all primary results in the correspondence sections.

**Forbid Pattern**: Sentences containing `claim` or `our approach` or `we show` or `we prove`, not followed (within 2 sentences) by a phrase matching "if\s+.*\s+(fail|reject|exit nonzero|error|contradict)".

**Grep Command**:
```bash
awk '/^(Claim|We claim|Our claim|We show|We prove|our approach)/ \
     { getline; getline; if (!/if\s|fail|reject|exit|error|contradict/) \
       print NR": " $0 }' combinatorial_maximalism.tex PAPER_SECTIONS_DRAFT.md | head -20
```

**Violation Example**:
```latex
Claim 1: Extraction produces valid Lean 4 code.
% No falsifier stated. What would make this false?
```

**Corrected Example**:
```latex
Claim 1: Extraction produces valid Lean 4 code.
Falsifier: If the extracted .lean file fails syntax check or type-check in lake, extraction is invalid.
Evidence: lake check D1_extracted.lean exits 0; type-checker passes with no errors.
```

---

## Rule 7: Forbid "Proof" for Non-Formal-Proof

**Rationale**: "Proof" in mathematics means a formal, deductive argument verified by a proof assistant (Lean, Coq, Isabelle, etc.) or by human formal reasoning. In an engineering context, we have "evidence" (artifacts, receipts, test results), not "proofs". We prove *via* formal verification when lake build exits 0, but we don't "prove" conformance just by running an unverified script. This rule prevents inflating engineering evidence to the level of formal proof without qualification.

**Forbid Pattern**: `\b(prove|proof|proven)\b` in a sentence about the system's behavior or correctness, unless the same sentence or immediately adjacent sentence mentions "Lean", "lake", "kernel", "formal", or "theorem".

**Grep Command**:
```bash
grep -nE "prove|proof|proven" combinatorial_maximalism.tex PAPER_SECTIONS_DRAFT.md | \
  grep -v "Lean\|lake\|kernel\|formal\|theorem\|re-admit\|lake build" | \
  head -20
```

**Violation Examples**:
```latex
We prove that the event log is correct.
% No formal verification mentioned; this is too strong.

The correspondence proof is the receipt chain.
% Receipt chain is evidence, not a mathematical proof object.
```

**Corrected Examples**:
```latex
We verify (via re-admission through Lean 4's kernel) that the event log conforms to the process model.

The receipt chain is the *evidence* of conformance; *formal proof* of conformance is witnessed by lake build exit 0.

The correspondence is proved (in Lean 4 terms) when the extracted code compiles under the kernel.
```

---

## Rule 8: Forbid Adverbs of Completeness Without Caveats

**Rationale**: Adverbs like "entirely", "completely", "fully", "absolutely", "always", "never" are totality claims. In an engineering paper with finite scope (D1 only), these must be scoped or caveated. "We completely verify the correspondence" overstates if we only verify D1.

**Forbid Pattern**: `entirely|completely|fully|absolutely|always|never` not followed within 5 words by a scope qualifier like "D1", "specimen", "in this work", "for correspondence X", or a mechanism like "via lake build".

**Grep Command**:
```bash
grep -nE "entirely|completely|fully|absolutely|always|never" combinatorial_maximalism.tex PAPER_SECTIONS_DRAFT.md | \
  grep -v "D1\|specimen\|this work\|correspondence\|lake build\|in this apparatus" | \
  head -20
```

**Violation Examples**:
```latex
We completely verify the event log.
% Overstates; we only verify D1 and only via token-replay, not all properties.

The receipt chain never loses integrity.
% Too strong; we prove it detects tampering, but don't address wholesale deletion.
```

**Corrected Examples**:
```latex
We verify the D1 event log via token-replay against the POWL model.

The receipt chain detects mutation and truncation (intra-chain integrity); wholesale deletion is out of scope (PR-16, cross-repository anchoring).

For the D1 specimen, re-admission through Lean 4's kernel completely validates the correspondence (lake build exit 0 is necessary and sufficient for standing).
```

---

## Implementation: just prose-lint Recipe

**File**: `justfile`

```makefile
prose-lint:
    @echo "Running prose-lint checks on correspondence sections..."
    
    # Rule 1: Forbid "Aeneas proves"
    @if grep -n "Aeneas\s\+\(proves\|verified\|checked\|certified\)" target/mfact/paper/combinatorial_maximalism.tex target/mfact/paper/PAPER_SECTIONS_DRAFT.md 2>/dev/null; then \
        echo "ERROR: Rule 1 violation - 'Aeneas proves' detected (use 'Aeneas extracts' or 'Lean proves')."; \
        exit 1; \
    fi
    
    # Rule 2: Forbid unscoped "verified"
    @if grep -nE "([^.]*)\b(verified|proven)\b([^.]*?)(\s|\.)" target/mfact/paper/combinatorial_maximalism.tex 2>/dev/null | \
        grep -v "via\|through\|using\|by\|when\|because"; then \
        echo "ERROR: Rule 2 violation - unscoped 'verified' or 'proven' detected (specify the mechanism)."; \
        exit 1; \
    fi
    
    # Rule 3: Require "D1 only" scope guard (check main results sections)
    @echo "Checking Rule 3: D1 scope guards..."
    @if grep -E "^[^.]*\.$" target/mfact/paper/PAPER_SECTIONS_DRAFT.md | \
        grep -v "D1\|specimen\|this work\|token-replay\|this correspondence" | \
        grep -E "extract|correspond|admission|proof|verif" | wc -l | grep -v "^0"; then \
        echo "WARNING: Rule 3 - possible unscoped claims found (review manually)."; \
    fi
    
    # Rule 4: Forbid "automatically" or "without proof"
    @if grep -n "automatically.*\(extract\|verif\|prove\|check\|generate\)\|without.*\(proof\|verif\|check\)" \
        target/mfact/paper/combinatorial_maximalism.tex target/mfact/paper/PAPER_SECTIONS_DRAFT.md 2>/dev/null; then \
        echo "ERROR: Rule 4 violation - 'automatically' or 'without proof' detected (be specific)."; \
        exit 1; \
    fi
    
    # Rule 5: Require "receipt chain" specificity
    @if grep -E "\s(chain|hash)\s" target/mfact/paper/combinatorial_maximalism.tex 2>/dev/null | \
        grep -v "receipt chain\|chain hash\|content hash\|payload hash\|BLAKE3\|chain linkage\|chain integrity"; then \
        echo "ERROR: Rule 5 violation - bare 'chain' or 'hash' detected (specify which)."; \
        exit 1; \
    fi
    
    # Rule 6: Verify falsifiers are stated (manual check)
    @echo "Checking Rule 6: Falsifiers stated..."
    @if grep -E "^Claim|^Claim:" target/mfact/paper/PAPER_SECTIONS_DRAFT.md | wc -l | grep -v "^0" > /dev/null; then \
        if ! grep -c "Falsifier:" target/mfact/paper/PAPER_SECTIONS_DRAFT.md | grep -q -E "[0-9]{1,}"; then \
            echo "ERROR: Rule 6 - claims stated but no falsifiers found."; \
            exit 1; \
        fi; \
    fi
    
    # Rule 7: Forbid "proof" without formal context
    @if grep -E "\b(prove|proof|proven)\b" target/mfact/paper/combinatorial_maximalism.tex 2>/dev/null | \
        grep -v "Lean\|lake\|kernel\|formal\|theorem\|re-admit"; then \
        echo "ERROR: Rule 7 violation - 'proof' used without formal context (use 'evidence', 'witness', etc.)."; \
        exit 1; \
    fi
    
    # Rule 8: Forbid totality adverbs without caveats
    @if grep -E "entirely|completely|fully|absolutely|always|never" \
        target/mfact/paper/combinatorial_maximalism.tex target/mfact/paper/PAPER_SECTIONS_DRAFT.md 2>/dev/null | \
        grep -v "D1\|specimen\|this work\|correspondence\|lake build\|in this apparatus"; then \
        echo "ERROR: Rule 8 violation - unscoped totality claim (add scope: D1, specimen, etc.)."; \
        exit 1; \
    fi
    
    @echo "✓ All prose-lint checks passed."
```

**Execution**:
```bash
just prose-lint
# Exit 0 if all rules pass, nonzero if any violation detected.
# CI gate: require exit 0 before merging paper branches.
```

---

## Rule Metadata

| Rule | Severity | Check Type | Auto-Fix Possible | Rationale |
|---|---|---|---|---|
| 1 (Aeneas proves) | Error | Regex | Yes (find-replace) | Aeneas is a tool, not a prover |
| 2 (unscoped verified) | Error | Regex + semantic | Partial | Must specify *how* verification happened |
| 3 (D1 scope guard) | Warning | Heuristic | No (human review) | Prevents generalization beyond specimen |
| 4 (automatically/without proof) | Error | Regex | Yes | Hides the actual mechanism |
| 5 (receipt chain specificity) | Error | Regex | Yes | Reduces ambiguity |
| 6 (falsifiers stated) | Error | Heuristic | No | Ensures paper is falsifiable |
| 7 (proof without formal context) | Error | Regex | Partial | Prevents inflating engineering to formal proof |
| 8 (totality adverbs) | Error | Regex | Partial | Prevents overstating finite scope |

---

## Future Additions (Post-D1)

When D2-D5 are added to the paper:

- **Rule 9**: Forbid claims about D2-D5 correspondence without explicit citation to their section numbers.
- **Rule 10**: When citing D1 results in D2-D5 sections, require "cf. §X (D1)" to indicate that the result came from D1, not the target section.

---

## Debugging Lint Violations

If a lint check fails, the recipe exits with code 1 and prints the offending lines. To debug:

```bash
# Find all lines matching Rule 2 (unscoped verified)
grep -nE "([^.]*)\b(verified|proven)\b([^.]*?)(\s|\.)" \
  target/mfact/paper/combinatorial_maximalism.tex target/mfact/paper/PAPER_SECTIONS_DRAFT.md | \
  grep -v "via\|through\|using\|by\|when\|because"

# Review the lines, then fix them according to the rule.
# Re-run: just prose-lint
```

---

## Notes for Reviewers

When reviewing paper sections against these rules:

1. **Rules 1-5, 7-8** are mechanical (grep-based); automated CI gates are enforced.
2. **Rules 3, 6** require human judgment; marked as "Heuristic" in the metadata table. The corresponding `just prose-lint` checks emit WARNING (non-fatal) for these rules. Reviewers should spot-check the warnings manually.
3. **All rules use conservative patterns** to minimize false positives. Some violations may slip through if phrased in unexpected ways. Manual proof-reading is still required.

---

## Maintenance

**File**: `/Users/sac/praxis/mfact/paper/PROSE_LINT_RULES_CORRESPONDENCE.md`  
**Last Updated**: 2026-07-07  
**Version**: 1.0  
**Scope**: D1 token-replay correspondence paper sections  

To update rules: edit this file and the `just prose-lint` recipe in `justfile` in tandem. Keep rule descriptions and grep patterns in sync.
