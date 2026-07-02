//! Verification pipeline: `cargo build` / `cargo test` / `cargo clippy` / safety audit,
//! wrapped in [`praxis_core::verify::VerifyGuard`] for per-stage pass/fail timing.
//!
//! Reuses `VerifyGuard`/`VerifyMetrics` from `praxis_core` (moved there from
//! `src/verbs/verifier.rs` in the root crate per the first-principles review — see
//! `~/.claude/plans/crispy-squishing-garden.md`, finding F2) rather than depending on
//! the root crate directly, since this crate is itself a dependency of the root.

use std::path::{Path, PathBuf};
use std::process::Command;

use praxis_core::verify::{VerifyGuard, VerifyMetrics};

use crate::spec::TaskType;

/// One undocumented risky pattern found by the [`safety_audit`] stub.
#[derive(Debug, Clone)]
pub struct RiskyFinding {
    /// File the pattern was found in, relative to the scanned root.
    pub file: PathBuf,
    /// 1-based line number.
    pub line: usize,
    /// Which pattern matched (`"unsafe"`, `"ecb"`, `"md5"`, `"sha1"`, `"hardcoded_iv"`).
    pub pattern: &'static str,
    /// The offending line, for diagnostics.
    pub snippet: String,
}

/// Run the full verification pipeline against a staged sandbox directory (see
/// [`crate::sandbox::stage_fixture`]), with no task-type-specific gating on the
/// `safety_audit` stage (it always passes — see [`run_pipeline_for_task`] to gate it).
#[must_use]
pub fn run_pipeline(dir: &Path) -> VerifyMetrics {
    run_pipeline_for_task(dir, None)
}

/// Run the full verification pipeline, gating the `safety_audit` stage on `task_type`:
/// for [`TaskType::CryptoCodegen`] and [`TaskType::UnsafeAudit`] tasks, the stage fails
/// if undocumented risky patterns are found; for all other task types (or `None`), the
/// stage always passes (findings are still collected and can be inspected via
/// [`find_risky_patterns`] directly, e.g. for reporting).
#[must_use]
pub fn run_pipeline_for_task(dir: &Path, task_type: Option<TaskType>) -> VerifyMetrics {
    let mut guard = VerifyGuard::new();

    guard.begin_stage("cargo_build");
    let build_ok = run_cargo(dir, &["build"]);
    guard.end_stage(build_ok);

    guard.begin_stage("cargo_test");
    let test_ok = run_cargo(dir, &["test"]);
    guard.end_stage(test_ok);

    guard.begin_stage("cargo_clippy");
    let clippy_ok = run_cargo(dir, &["clippy", "--", "-D", "warnings"]);
    guard.end_stage(clippy_ok);

    guard.begin_stage("safety_audit");
    let audit_ok = safety_audit(dir, task_type);
    guard.end_stage(audit_ok);

    guard.finish()
}

/// Run `cargo <subcommand> --manifest-path <dir>/Cargo.toml [rest...]`, returning
/// whether it exited successfully. Any failure to even launch `cargo` (missing
/// toolchain, bad path, ...) is treated as a failed stage rather than propagated as
/// an error, since `VerifyGuard` stages are boolean pass/fail by design.
///
/// `--manifest-path` is placed *after* the subcommand (`args[0]`), not before it:
/// current `cargo` rejects `cargo --manifest-path <path> build` with "unexpected
/// argument '--manifest-path' found" — it must be `cargo build --manifest-path
/// <path>`. Any remaining args (e.g. clippy's `["--", "-D", "warnings"]`) are
/// appended after `--manifest-path`, which keeps `--` working as the flag
/// separator cargo expects.
fn run_cargo(dir: &Path, args: &[&str]) -> bool {
    let manifest_path = dir.join("Cargo.toml");
    let mut command = Command::new("cargo");
    let Some((subcommand, rest)) = args.split_first() else {
        return false;
    };
    command.arg(subcommand);
    command.arg("--manifest-path").arg(&manifest_path);
    command.args(rest);

    match command.output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Safety-audit stub: grep the directory's `.rs` files for `unsafe` blocks and a short
/// list of risky crypto patterns, gated by `task_type` per [`run_pipeline_for_task`]'s
/// docs.
///
/// // TODO: integrate a real unsafe/crypto-misuse scanner (see
/// // `RUST_CLAUDE_COMPREHENSIVE_RESEARCH.md` — deepSURF class tool). This grep-based
/// // stub exists only to keep the 4-stage pipeline shape correct; it makes no claim
/// // to the coverage of a real static analyzer.
fn safety_audit(dir: &Path, task_type: Option<TaskType>) -> bool {
    let findings = find_risky_patterns(dir);
    match task_type {
        Some(TaskType::CryptoCodegen | TaskType::UnsafeAudit) => findings.is_empty(),
        _ => true,
    }
}

/// Walk `dir` for `.rs` files and collect undocumented risky patterns.
///
/// A finding is "documented" (and therefore excluded) if the matched line or either of
/// the two preceding lines contains `SAFETY:` or `AUDITED:` (case-insensitive) — the
/// same convention this workspace already uses for `unsafe` justifications (see
/// `ggen_core::prompt_mfg::PromptCompiler::default`'s `// SAFETY:` comment).
#[must_use]
pub fn find_risky_patterns(dir: &Path) -> Vec<RiskyFinding> {
    let mut findings = Vec::new();
    walk_rs_files(dir, dir, &mut findings);
    findings
}

fn walk_rs_files(root: &Path, current: &Path, findings: &mut Vec<RiskyFinding>) {
    let Ok(entries) = std::fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            // Skip target/ build output — it can contain large generated .rs-like
            // artifacts that aren't source and would slow/skew the audit.
            if entry.file_name() == "target" {
                continue;
            }
            walk_rs_files(root, &path, findings);
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            scan_file(root, &path, findings);
        }
    }
}

const RISKY_PATTERNS: &[(&str, &str)] = &[
    ("unsafe", "unsafe"),
    ("ecb", "ecb"),
    ("md5", "md5"),
    ("sha1", "sha1"),
];

fn scan_file(root: &Path, path: &Path, findings: &mut Vec<RiskyFinding>) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<&str> = content.lines().collect();
    let rel_path = path.strip_prefix(root).unwrap_or(path).to_path_buf();

    for (idx, line) in lines.iter().enumerate() {
        let lower = line.to_ascii_lowercase();
        for (needle, pattern_name) in RISKY_PATTERNS {
            if lower.contains(needle) && !is_documented(&lines, idx) {
                findings.push(RiskyFinding {
                    file: rel_path.clone(),
                    line: idx + 1,
                    pattern: pattern_name,
                    snippet: (*line).to_string(),
                });
            }
        }
        if is_hardcoded_iv(&lower) && !is_documented(&lines, idx) {
            findings.push(RiskyFinding {
                file: rel_path.clone(),
                line: idx + 1,
                pattern: "hardcoded_iv",
                snippet: (*line).to_string(),
            });
        }
    }
}

/// Heuristic for a hardcoded IV literal: a line naming an `iv` variable and containing
/// an array/byte-string literal. Intentionally loose — see the `safety_audit` TODO.
fn is_hardcoded_iv(lower_line: &str) -> bool {
    (lower_line.contains("iv") || lower_line.contains("nonce"))
        && (lower_line.contains('[') || lower_line.contains("b\""))
        && lower_line.contains('=')
}

/// Whether the matched line or either of the two preceding lines carries a
/// `SAFETY:`/`AUDITED:` justification comment.
fn is_documented(lines: &[&str], idx: usize) -> bool {
    let start = idx.saturating_sub(2);
    lines[start..=idx].iter().any(|l| {
        let upper = l.to_ascii_uppercase();
        upper.contains("SAFETY:") || upper.contains("AUDITED:")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safety_audit_passes_for_non_gated_task_types_regardless_of_findings() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("lib.rs"), "unsafe { do_thing(); }\n").expect("write");

        assert!(safety_audit(dir.path(), None));
        assert!(safety_audit(dir.path(), Some(TaskType::FunctionLevelBugfix)));
    }

    #[test]
    fn safety_audit_fails_for_gated_task_types_with_undocumented_unsafe() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("lib.rs"), "unsafe { do_thing(); }\n").expect("write");

        assert!(!safety_audit(dir.path(), Some(TaskType::UnsafeAudit)));
        assert!(!safety_audit(dir.path(), Some(TaskType::CryptoCodegen)));
    }

    #[test]
    fn safety_audit_passes_when_unsafe_is_documented() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("lib.rs"),
            "// SAFETY: bounds checked above\nunsafe { do_thing(); }\n",
        )
        .expect("write");

        assert!(safety_audit(dir.path(), Some(TaskType::UnsafeAudit)));
    }

    #[test]
    fn find_risky_patterns_detects_crypto_smells() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("lib.rs"),
            "let cipher = Aes128Ecb::new(key);\nlet iv = [0u8; 16];\n",
        )
        .expect("write");

        let findings = find_risky_patterns(dir.path());
        assert!(findings.iter().any(|f| f.pattern == "ecb"));
        assert!(findings.iter().any(|f| f.pattern == "hardcoded_iv"));
    }
}
