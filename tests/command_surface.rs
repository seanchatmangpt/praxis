//! Command-surface conformance: every documented noun/verb on the root
//! binary is typed-refusal-or-behavior complete (exit criterion 5,
//! `docs/releases/v26.7.6/RELEASE_CONTROL.md` Sec. 5).
//!
//! The noun and verb lists are enumerated from the compiled binary's own
//! `--help` output (the same source `docs/releases/v26.7.6/CLI.md` documents
//! from), so newly registered verbs are covered automatically. Each
//! `<noun> <verb>` is invoked with no payload, stdin closed, and cwd set to a
//! fresh temp sandbox; the contract asserted is:
//!
//! - the process terminates (a hang is a failure),
//! - it never panics (no `panicked at` on either stream, exit != 101),
//! - it either behaves (exit 0) or refuses with a typed, non-empty error.
//!
//! The `ggen` binary's equivalent proof lives in
//! `crates/ggen/tests/cli_boundary.rs`; `praxis-l4`'s surface is exercised by
//! `crates/praxis-lean/tests/no_sorry.rs` plus its unit tests.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_my-conforming-project");
const PER_COMMAND_TIMEOUT: Duration = Duration::from_secs(90);

/// Verbs that launch full-gate work (cargo builds, whole-workspace checks,
/// matrix evaluation) when invoked bare. Probed via `--help` instead: the
/// no-panic/typed-surface contract is still asserted against the real
/// binary, and their full behavior is exercised by `just verify-all`
/// (`doctor check`, `dod matrix`, `frontier matrix` all run inside the
/// `just verify-all` gate itself).
const HEAVY: &[(&str, &str)] = &[
    ("dod", "matrix"),
    ("doctor", "check"),
    ("frontier", "matrix"),
    ("frontier", "summary"),
    ("frontier", "counts"),
];

/// Run the binary with `args`, stdin closed, cwd in `sandbox`. Returns
/// `(exit_code, stdout, stderr)`. Fails the test if the process outlives
/// [`PER_COMMAND_TIMEOUT`] (a hang is neither behavior nor a typed refusal).
fn run(args: &[&str], sandbox: &std::path::Path) -> (i32, String, String) {
    let mut child = Command::new(BIN)
        .args(args)
        .current_dir(sandbox)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn {args:?}: {e}"));

    let start = Instant::now();
    let status = loop {
        match child.try_wait().expect("try_wait") {
            Some(status) => break status,
            None if start.elapsed() > PER_COMMAND_TIMEOUT => {
                let _ = child.kill();
                panic!("command {args:?} hung past {PER_COMMAND_TIMEOUT:?}");
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };
    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(mut s) = child.stdout.take() {
        let _ = s.read_to_string(&mut stdout);
    }
    if let Some(mut s) = child.stderr.take() {
        let _ = s.read_to_string(&mut stderr);
    }
    // A killed process has no code; the timeout branch already panicked, so
    // any signal death here is a real crash — surface it as code 101.
    (status.code().unwrap_or(101), stdout, stderr)
}

/// Parse the `Commands:` section of a clap help text into command names.
fn parse_commands(help: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_commands = false;
    for line in help.lines() {
        if line.starts_with("Commands:") {
            in_commands = true;
            continue;
        }
        if in_commands {
            if line.starts_with("Options:") {
                break;
            }
            // Command rows are exactly two-space indented; continuation/doc
            // lines are indented deeper.
            if let Some(rest) = line.strip_prefix("  ") {
                if !rest.starts_with(' ') {
                    if let Some(name) = rest.split_whitespace().next() {
                        if name != "help" {
                            out.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    out
}

/// The refusal contract, shared by every probe below.
fn assert_typed_refusal_or_behavior(label: &str, code: i32, stdout: &str, stderr: &str) {
    assert_ne!(
        code, 101,
        "{label}: exit 101 (Rust panic / signal death)\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    for (stream, text) in [("stdout", stdout), ("stderr", stderr)] {
        assert!(
            !text.contains("panicked at"),
            "{label}: panic message on {stream}:\n{text}"
        );
    }
    if code != 0 {
        assert!(
            !stderr.trim().is_empty() || !stdout.trim().is_empty(),
            "{label}: refused (exit {code}) with no diagnostic on either stream — silent default"
        );
    }
}

#[test]
fn every_documented_verb_is_typed_refusal_or_behavior() {
    let sandbox = tempfile::tempdir().expect("tempdir");
    let sandbox = sandbox.path();

    let (code, help, err) = run(&["--help"], sandbox);
    assert_eq!(code, 0, "--help must exit 0; stderr:\n{err}");
    let nouns = parse_commands(&help);
    assert!(
        nouns.len() >= 10,
        "expected the documented noun surface (CLI.md lists 10+), got {nouns:?}"
    );

    let mut probed = 0usize;
    for noun in &nouns {
        let (code, noun_help, err) = run(&[noun, "--help"], sandbox);
        assert_typed_refusal_or_behavior(&format!("{noun} --help"), code, &noun_help, &err);
        // Nouns with a default verb (receipt/config/doctor) print the verb
        // help; nouns that are pure dispatchers list their verbs.
        let verbs = parse_commands(&noun_help);
        if verbs.is_empty() {
            // Leaf or defaulted noun: probe the bare noun itself.
            let (code, out, err) = run(&[noun.as_str()], sandbox);
            assert_typed_refusal_or_behavior(noun, code, &out, &err);
            probed += 1;
            continue;
        }
        for verb in &verbs {
            let heavy = HEAVY.contains(&(noun.as_str(), verb.as_str()));
            let args: &[&str] = if heavy {
                &[noun, verb, "--help"]
            } else {
                &[noun, verb]
            };
            let (code, out, err) = run(args, sandbox);
            assert_typed_refusal_or_behavior(&format!("{noun} {verb}"), code, &out, &err);
            probed += 1;
        }
    }
    // The surface documented in CLI.md is far larger than this floor; the
    // floor only guards against the help parser silently matching nothing.
    // The verb count grows with enabled features (default ~20, --all-features
    // 30+), so the floor sits below the smallest real surface rather than
    // tracking any one feature combination.
    let floor = 15;
    assert!(
        probed >= floor,
        "probed only {probed} verbs (floor {floor}) — help parsing regressed"
    );
}

/// Unknown nouns and verbs are refused by name (closed command vocabulary),
/// not panicked on and not silently defaulted.
#[test]
fn unknown_noun_and_verb_are_refused_by_name() {
    let sandbox = tempfile::tempdir().expect("tempdir");
    let sandbox = sandbox.path();

    let (code, out, err) = run(&["frobnicate"], sandbox);
    assert_ne!(code, 0, "unknown noun must not succeed");
    assert_typed_refusal_or_behavior("frobnicate", code, &out, &err);
    let combined = format!("{out}{err}");
    assert!(
        combined.contains("frobnicate"),
        "refusal must name the unknown noun:\n{combined}"
    );

    let (code, out, err) = run(&["law", "frobnicate"], sandbox);
    assert_ne!(code, 0, "unknown verb must not succeed");
    assert_typed_refusal_or_behavior("law frobnicate", code, &out, &err);
    let combined = format!("{out}{err}");
    assert!(
        combined.contains("frobnicate"),
        "refusal must name the unknown verb:\n{combined}"
    );
}
