use crate::error::{LeanRefusal, Result};
use camino::{Utf8Path, Utf8PathBuf};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use walkdir::WalkDir;

/// Policy for refusing fake verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPolicy {
    pub forbid_sorry: bool,
    pub forbid_admit: bool,
    pub forbid_axiom: bool,
    pub allowed_axiom_prefixes: Vec<String>,
}

impl Default for AuditPolicy {
    fn default() -> Self {
        Self {
            forbid_sorry: true,
            // Off by default: Lean 4 has no `admit` tactic (that's a Coq
            // idiom the original scaffold carried over) -- `sorry` is Lean
            // 4's only real proof-hole. Running this policy against the
            // real 183-file corpus found 5 "admit" matches, all of them
            // the corpus's own `LifeHom` type's constructor literally named
            // `admit` (a real identifier, not a proof escape hatch), i.e.
            // false positives. Left available for callers who genuinely
            // want to forbid a project-specific `admit` convention.
            forbid_admit: false,
            forbid_axiom: true,
            allowed_axiom_prefixes: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub file: Utf8PathBuf,
    pub line: usize,
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct NoSorryAudit {
    policy: AuditPolicy,
}

impl NoSorryAudit {
    pub fn new(policy: AuditPolicy) -> Self {
        Self { policy }
    }

    pub fn audit_root(&self, root: &Utf8Path) -> Result<Vec<AuditFinding>> {
        let mut findings = Vec::new();
        for entry in WalkDir::new(root) {
            let entry = entry.map_err(|e| LeanRefusal::Io {
                path: root.to_path_buf(),
                source: std::io::Error::other(e.to_string()),
            })?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = Utf8PathBuf::from_path_buf(entry.path().to_path_buf())
                .map_err(LeanRefusal::NonUtf8Path)?;
            if path.extension() != Some("lean") {
                continue;
            }
            findings.extend(self.audit_file(&path)?);
        }
        Ok(findings)
    }

    pub fn audit_file(&self, path: &Utf8Path) -> Result<Vec<AuditFinding>> {
        let text = fs::read_to_string(path).map_err(|source| LeanRefusal::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let scannable = strip_lean_comments(&text);
        let axiom_re = Regex::new(r"^\s*axiom\s+([A-Za-z0-9_'.]+)").unwrap();
        let mut out = Vec::new();

        // Iterate the ORIGINAL text for display purposes (so a finding's
        // `text` field shows the real source line), but decide whether to
        // flag a line using the comment-stripped version at the same line
        // index -- so a line comment or block comment mentioning "sorry" in
        // prose never produces a finding, only real code does.
        for (idx, (orig_line, scan_line)) in text.lines().zip(scannable.lines()).enumerate() {
            let line_no = idx + 1;
            let trimmed = scan_line.trim();

            if self.policy.forbid_sorry && token_contains(trimmed, "sorry") {
                out.push(finding(path, line_no, "sorry", orig_line));
            }
            if self.policy.forbid_admit && token_contains(trimmed, "admit") {
                out.push(finding(path, line_no, "admit", orig_line));
            }
            if self.policy.forbid_axiom {
                if let Some(caps) = axiom_re.captures(scan_line) {
                    let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let allowed = self
                        .policy
                        .allowed_axiom_prefixes
                        .iter()
                        .any(|p| name.starts_with(p));
                    if !allowed {
                        out.push(finding(path, line_no, "axiom", orig_line));
                    }
                }
            }
        }

        Ok(out)
    }
}

fn finding(path: &Utf8Path, line: usize, kind: &str, text: &str) -> AuditFinding {
    AuditFinding {
        file: path.to_path_buf(),
        line,
        kind: kind.to_string(),
        text: text.trim().to_string(),
    }
}

fn token_contains(line: &str, token: &str) -> bool {
    // Intentionally simple first-pass audit. Later upgrade can use Lean parser
    // output instead of text scan.
    line.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|part| part == token)
}

/// Replace the contents of Lean `--` line comments and `/- ... -/` block
/// comments (which may nest, per Lean's own comment syntax) with spaces,
/// preserving every newline and the overall character-column layout, so
/// line numbers computed against the result line up exactly with the
/// original source. This runs before the sorry/admit/axiom scan so a
/// comment merely mentioning one of these words in prose is never mistaken
/// for real code.
fn strip_lean_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut block_depth: u32 = 0;

    while let Some(c) = chars.next() {
        if block_depth > 0 {
            if c == '/' && chars.peek() == Some(&'-') {
                chars.next();
                block_depth += 1;
                out.push(' ');
                out.push(' ');
                continue;
            }
            if c == '-' && chars.peek() == Some(&'/') {
                chars.next();
                block_depth -= 1;
                out.push(' ');
                out.push(' ');
                continue;
            }
            out.push(if c == '\n' { '\n' } else { ' ' });
            continue;
        }

        if c == '/' && chars.peek() == Some(&'-') {
            chars.next();
            block_depth += 1;
            out.push(' ');
            out.push(' ');
            continue;
        }

        if c == '-' && chars.peek() == Some(&'-') {
            // Line comment: consume through end of line, preserving the
            // newline itself (if any) so line counts stay aligned.
            out.push(' ');
            out.push(' ');
            for c2 in chars.by_ref() {
                if c2 == '\n' {
                    out.push('\n');
                    break;
                }
                out.push(' ');
            }
            continue;
        }

        out.push(c);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_line_and_block_comments_preserving_lines() {
        let src = "def x := 1 -- sorry mentioned in a comment\n/- block sorry comment\nspanning lines -/\ntheorem t : True := by\n  sorry\n";
        let stripped = strip_lean_comments(src);
        assert_eq!(src.lines().count(), stripped.lines().count());
        assert!(!stripped.contains("sorry mentioned"));
        assert!(!stripped.contains("block sorry comment"));
        // The real, uncommented `sorry` on the last-but-one line must survive.
        assert!(stripped.lines().nth(4).unwrap().contains("sorry"));
    }

    #[test]
    fn comment_only_sorry_produces_no_finding_but_real_sorry_does() {
        let dir = tempfile::tempdir().unwrap();
        let file = Utf8PathBuf::from_path_buf(dir.path().join("Mixed.lean")).unwrap();
        std::fs::write(
            &file,
            "-- no sorry here, just prose\ntheorem t : True := by\n  sorry\n",
        )
        .unwrap();

        let audit = NoSorryAudit::new(AuditPolicy::default());
        let findings = audit.audit_file(&file).unwrap();
        assert_eq!(
            findings.len(),
            1,
            "expected exactly one real sorry finding, got {findings:?}"
        );
        assert_eq!(findings[0].line, 3);
    }

    #[test]
    fn admit_as_a_real_identifier_is_not_flagged_by_default() {
        // Mirrors the real corpus's `LifeHom` constructor pattern
        // (`tools/paper-factory/lean-pilot/def_lifecat.lean` and others):
        // `admit` used as a legitimate identifier, not a Coq-style proof
        // hole. Lean 4 has no `admit` tactic, so this must produce zero
        // findings under the default policy.
        let dir = tempfile::tempdir().unwrap();
        let file = Utf8PathBuf::from_path_buf(dir.path().join("LifeHom.lean")).unwrap();
        std::fs::write(
            &file,
            "inductive LifeHom (Val Admd : Type) where\n  | admit : LifeHom Val Admd\n",
        )
        .unwrap();

        let audit = NoSorryAudit::new(AuditPolicy::default());
        let findings = audit.audit_file(&file).unwrap();
        assert!(
            findings.is_empty(),
            "expected no findings for a real `admit` identifier, got {findings:?}"
        );
    }
}
