//! src/bin/dod.rs — Definition of Done verification
//! Exit 0: all pass  |  Exit 1: soft (tests ok, artifacts stale)  |  Exit 2: hard (broken)

#![allow(clippy::print_stdout)]

use std::process::{exit, Command};

fn run(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd).args(args).status().map(|s| s.success()).unwrap_or(false)
}

/// The crate name/version pair `cicd-evidence-gen` is invoked with. Keep in
/// sync with this crate's own `[package]` in `Cargo.toml`.
const CRATE_NAME: &str = "my-conforming-project";
const CRATE_VERSION: &str = "26.7.2";

/// Soft check: `cicd-evidence-gen --check` against `receipt.json`.
///
/// Never a hard gate: if `receipt.json` doesn't exist yet, or the
/// `cicd-evidence-gen` binary isn't installed on this machine, that's
/// reported and treated as a pass (nothing to check yet) rather than a
/// failure — only an actual failed validation (binary ran, exited nonzero)
/// counts against the soft check.
fn evidence_check_ok() -> bool {
    if !std::path::Path::new("receipt.json").exists() {
        eprintln!("[DOD] evidence-check skipped: receipt.json not found");
        return true;
    }
    match Command::new("cicd-evidence-gen")
        .args([CRATE_NAME, CRATE_VERSION, "--receipt", "receipt.json", "--check"])
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => {
            eprintln!("[DOD] evidence-check skipped: cicd-evidence-gen binary not found on PATH");
            true
        }
    }
}

fn main() {
    let fmt_ok = run("cargo", &["fmt", "--all", "--check"]);
    let lint_ok =
        run("cargo", &["clippy", "--workspace", "--all-features", "--", "-D", "warnings"]);
    let test_ok = run("cargo", &["test", "--workspace", "--all-features"]);
    let hard_ok = fmt_ok && lint_ok && test_ok;

    // Soft check: receipts/ directory must exist and be non-empty
    let receipts_exist =
        std::fs::read_dir("receipts").map(|mut d| d.next().is_some()).unwrap_or(false);

    // Soft check: the [evidence] TOML block validates against receipt.json
    // (see `just evidence-check`). Skipped (not failed) when either the
    // binary or the receipt file isn't present on this machine yet.
    let evidence_ok = evidence_check_ok();

    if !hard_ok {
        eprintln!("[DOD] HARD FAILURE: fmt={fmt_ok} lint={lint_ok} test={test_ok}");
        exit(2);
    }
    let mut soft_ok = true;
    if !receipts_exist {
        eprintln!("[DOD] SOFT FAILURE: receipts/ missing or empty");
        soft_ok = false;
    }
    if !evidence_ok {
        eprintln!("[DOD] SOFT FAILURE: cicd-evidence-gen --check reported issues");
        soft_ok = false;
    }
    if !soft_ok {
        exit(1);
    }
    println!("[DOD] ALL PASS");
    exit(0);
}
