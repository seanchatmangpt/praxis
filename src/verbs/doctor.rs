//! `doctor` verb — a holistic health check of the whole praxis workspace in
//! one command, the CLI-diagnostic analogue of `cargo doctor` / `brew doctor`.
//!
//! Aggregates, in one place, what each lane otherwise exposes separately:
//! whether the workspace type-checks (`cargo check --workspace`), the
//! admitted [`my_conforming_project::config::PraxisConfig`]'s witness hash
//! (config lane), the capability-frontier `DfCM` matrix's
//! `pass_rate`/`coverage` (frontier lane, [`my_conforming_project::frontier`]),
//! whether required external tools (`git`, `cicd-evidence-gen`) are on
//! `PATH`, the receipts ledger's existence/record count (receipt lane), and
//! which Cargo feature flags this binary was actually compiled with.
//!
//! `doctor check` (bare) prints a colored, human-readable summary to
//! stdout; `doctor check --format json` prints the same data as a single
//! JSON object instead (no colors, no prose) for machine/CI consumption.
//!
//! Every check here is read-only: `doctor` never runs `cargo build`, never
//! writes to the receipts directory, and never mutates configuration. The
//! one check that shells out (`cargo check --workspace`, unless
//! `--skip-build` is given) is bounded by a hard timeout so a slow or wedged
//! toolchain can't hang the diagnostic forever.

use std::{
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::verb;
use my_conforming_project::{config as cfg, frontier};
use serde_json::{json, Map, Value};

/// Feature flags this diagnostic reports on, in the same order as their
/// `[features]` declaration in the root `Cargo.toml` (excluding the
/// `all-features` alias, which is not itself a compiled-in flag).
const FEATURE_FLAGS: &[&str] = &[
    "typestate",
    "repl",
    "otel",
    "discovery",
    "lsp",
    "andon",
    "mcp",
    "ggen",
    "law-signed",
    "law-ocel",
    "testbed",
    "proposer",
];

/// External binaries this diagnostic checks for on `PATH`. `git` backs the
/// release/versioning workflow; `cicd-evidence-gen` powers `just
/// evidence`/`evidence-check` and `dod`'s soft evidence-check.
const REQUIRED_TOOLS: &[&str] = &["git", "cicd-evidence-gen"];

/// Hard timeout for the `cargo check --workspace` build probe. Generous
/// enough for a cold incremental cache, but bounded so `doctor` can never
/// hang indefinitely on a wedged toolchain.
const BUILD_CHECK_TIMEOUT: Duration = Duration::from_secs(90);

/// Outcome of the (optional) `cargo check --workspace` build probe.
struct BuildCheck {
    /// Whether the probe was actually run (`false` when `--skip-build` was passed).
    ran: bool,
    /// Whether `cargo check --workspace` exited successfully. Meaningless when `ran` is `false`.
    ok: bool,
    /// Wall-clock time the probe took, in milliseconds.
    duration_ms: u128,
    /// Human-readable detail: success/failure/timeout/skip reason.
    detail: String,
}

/// Run `cargo check --workspace` as a child process, polling for completion
/// rather than blocking indefinitely, so a wedged toolchain can be killed at
/// `timeout` instead of hanging `doctor` forever.
fn run_build_check(timeout: Duration) -> BuildCheck {
    let start = Instant::now();
    let mut child = match Command::new("cargo")
        .args(["check", "--workspace", "--message-format=short"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            return BuildCheck {
                ran: false,
                ok: false,
                duration_ms: 0,
                detail: format!("could not spawn `cargo check`: {e}"),
            };
        }
    };

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let duration_ms = start.elapsed().as_millis();
                return if status.success() {
                    BuildCheck {
                        ran: true,
                        ok: true,
                        duration_ms,
                        detail: "cargo check --workspace succeeded".to_string(),
                    }
                } else {
                    BuildCheck {
                        ran: true,
                        ok: false,
                        duration_ms,
                        detail: format!("cargo check --workspace exited with {status}"),
                    }
                };
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return BuildCheck {
                        ran: true,
                        ok: false,
                        duration_ms: start.elapsed().as_millis(),
                        detail: format!(
                            "cargo check --workspace timed out after {}s",
                            timeout.as_secs()
                        ),
                    };
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return BuildCheck {
                    ran: false,
                    ok: false,
                    duration_ms: start.elapsed().as_millis(),
                    detail: format!("error waiting on `cargo check`: {e}"),
                };
            }
        }
    }
}

/// Whether `tool` is runnable from `PATH` (spawns `tool --version` and
/// checks the process could be *started* at all — exit code is irrelevant,
/// some tools use nonzero exit for `--version`).
fn tool_on_path(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Load the admitted `PraxisConfig` and report its witness hash, or the
/// admission error if the config lane fails to load.
fn check_config() -> Value {
    match cfg::load_config() {
        Ok(admitted) => json!({
            "admitted": true,
            "witness": admitted.witness().hash(),
            "receipts_dir": admitted.value().receipts.dir,
        }),
        Err(e) => json!({ "admitted": false, "error": e.to_string() }),
    }
}

/// The receipts directory to inspect: the admitted config's `receipts.dir`
/// if it loads cleanly, otherwise the hardcoded default `"receipts"` (must
/// never block the rest of `doctor` from running).
fn receipts_dir_from_config() -> String {
    cfg::load_config().map_or_else(
        |_| "receipts".to_string(),
        |a| a.value().receipts.dir.clone(),
    )
}

/// Report the receipts ledger's existence and record count. Read-only:
/// unlike `praxis_core::receipt_store::ReceiptStore::open`, this never
/// creates the directory as a side effect of merely checking it.
fn check_receipts(dir: &str) -> Value {
    let dir_path = Path::new(dir);
    if !dir_path.exists() {
        return json!({ "dir": dir, "dir_exists": false, "record_count": Value::Null });
    }
    let ledger = dir_path.join(praxis_core::receipt_store::LEDGER_FILE_NAME);
    if !ledger.exists() {
        return json!({ "dir": dir, "dir_exists": true, "ledger_exists": false, "record_count": 0 });
    }
    match std::fs::read_to_string(&ledger) {
        Ok(content) => {
            let count = content.lines().filter(|l| !l.trim().is_empty()).count();
            json!({ "dir": dir, "dir_exists": true, "ledger_exists": true, "record_count": count })
        }
        Err(e) => json!({
            "dir": dir, "dir_exists": true, "ledger_exists": true, "error": e.to_string(),
        }),
    }
}

/// Build the capability-frontier report and reduce it to the fields useful
/// for a health summary (full per-cell detail belongs to `frontier`/`dod`).
fn check_frontier() -> Value {
    match frontier::frontier_report() {
        Ok(report) => json!({
            "total": report.total,
            "evaluated": report.evaluated,
            "passing": report.passing,
            "coverage": report.coverage,
            "pass_rate": report.pass_rate,
            "failure_count": report.failures.len(),
        }),
        Err(e) => json!({
            "error": e.to_string()
        })
    }
}

/// Which of [`FEATURE_FLAGS`] this binary was actually compiled with.
fn feature_flags() -> Value {
    let mut m = Map::new();
    for flag in FEATURE_FLAGS {
        // Not a `matches!` candidate despite the shape: each arm's `cfg!`
        // call checks a *different* feature name, so collapsing this into
        // `matches!(*flag, "typestate" | "repl" | ...)` (clippy's literal
        // suggestion here) would make every known flag report `true`
        // unconditionally — losing the actual per-feature compiled-in check.
        #[allow(clippy::match_like_matches_macro)]
        let on = match *flag {
            "typestate" => cfg!(feature = "typestate"),
            "repl" => cfg!(feature = "repl"),
            "otel" => cfg!(feature = "otel"),
            "discovery" => cfg!(feature = "discovery"),
            "lsp" => cfg!(feature = "lsp"),
            "andon" => cfg!(feature = "andon"),
            "mcp" => cfg!(feature = "mcp"),
            "ggen" => cfg!(feature = "ggen"),
            "law-signed" => cfg!(feature = "law-signed"),
            "law-ocel" => cfg!(feature = "law-ocel"),
            "testbed" => cfg!(feature = "testbed"),
            "proposer" => cfg!(feature = "proposer"),
            _ => false,
        };
        m.insert((*flag).to_string(), json!(on));
    }
    Value::Object(m)
}

/// Which of [`REQUIRED_TOOLS`] are runnable from `PATH`.
fn tool_report() -> Value {
    let mut m = Map::new();
    for tool in REQUIRED_TOOLS {
        m.insert((*tool).to_string(), json!(tool_on_path(tool)));
    }
    Value::Object(m)
}

/// ANSI color codes for the human-readable summary. No color-support
/// detection: this is a developer diagnostic run in an interactive
/// terminal, and raw ANSI is harmless (if ugly) when piped.
mod ansi {
    pub(super) const GREEN: &str = "\x1b[32m";
    pub(super) const RED: &str = "\x1b[31m";
    pub(super) const YELLOW: &str = "\x1b[33m";
    pub(super) const BOLD: &str = "\x1b[1m";
    pub(super) const RESET: &str = "\x1b[0m";
}

/// Print the human-readable, colored doctor summary to stdout.
#[allow(clippy::too_many_lines)]
fn print_human_summary(
    build: Option<&BuildCheck>,
    config: &Value,
    frontier: &Value,
    receipts: &Value,
    tools: &Value,
    features: &Value,
) {
    use ansi::{BOLD, GREEN, RED, RESET, YELLOW};

    println!("{BOLD}praxis doctor{RESET}");
    println!("{BOLD}============={RESET}");

    println!();
    println!("{BOLD}Build{RESET}");
    match build {
        None => println!("  [{YELLOW}SKIP{RESET}] cargo check --workspace (--skip-build passed)"),
        Some(b) if !b.ran => println!("  [{YELLOW}WARN{RESET}] {}", b.detail),
        Some(b) if b.ok => {
            println!("  [{GREEN} OK {RESET}] {} ({} ms)", b.detail, b.duration_ms);
        }
        Some(b) => println!("  [{RED}FAIL{RESET}] {} ({} ms)", b.detail, b.duration_ms),
    }

    println!();
    println!("{BOLD}Config{RESET}");
    if config
        .get("admitted")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let witness = config.get("witness").and_then(Value::as_str).unwrap_or("?");
        let dir = config
            .get("receipts_dir")
            .and_then(Value::as_str)
            .unwrap_or("?");
        println!("  [{GREEN} OK {RESET}] admitted; witness={witness}; receipts.dir={dir}");
    } else {
        let err = config
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("unknown error");
        println!("  [{RED}FAIL{RESET}] config not admitted: {err}");
    }

    println!();
    println!("{BOLD}Frontier{RESET}");
    let pass_rate = frontier
        .get("pass_rate")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let coverage = frontier
        .get("coverage")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let evaluated = frontier
        .get("evaluated")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total = frontier.get("total").and_then(Value::as_u64).unwrap_or(0);
    let failure_count = frontier
        .get("failure_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let frontier_color = if failure_count == 0 { GREEN } else { RED };
    let frontier_tag = if failure_count == 0 { " OK " } else { "FAIL" };
    println!(
        "  [{frontier_color}{frontier_tag}{RESET}] pass_rate={pass_rate:.2} coverage={coverage:.2} \
         evaluated={evaluated}/{total} failures={failure_count}"
    );

    println!();
    println!("{BOLD}Receipts{RESET}");
    let dir = receipts.get("dir").and_then(Value::as_str).unwrap_or("?");
    if receipts
        .get("dir_exists")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        match receipts.get("record_count").and_then(Value::as_u64) {
            Some(count) => println!("  [{GREEN} OK {RESET}] {dir}/ ({count} records)"),
            None => println!("  [{YELLOW}WARN{RESET}] {dir}/ exists but ledger is unreadable"),
        }
    } else {
        println!("  [{YELLOW}WARN{RESET}] {dir}/ does not exist yet (no receipts issued)");
    }

    println!();
    println!("{BOLD}Tools on PATH{RESET}");
    if let Value::Object(map) = tools {
        for (tool, on_path) in map {
            let is_optional = tool == "cicd-evidence-gen";
            if on_path.as_bool().unwrap_or(false) {
                println!("  [{GREEN} OK {RESET}] {tool}");
            } else if is_optional {
                println!(
                    "  [{YELLOW}WARN{RESET}] {tool} not found (optional; used by `just evidence`)"
                );
            } else {
                println!("  [{RED}FAIL{RESET}] {tool} not found");
            }
        }
    }

    println!();
    println!("{BOLD}Feature flags compiled in{RESET}");
    if let Value::Object(map) = features {
        use std::fmt::Write as _;
        let mut line = String::from("  ");
        for (flag, on) in map {
            let on = on.as_bool().unwrap_or(false);
            let color = if on { GREEN } else { "\x1b[90m" };
            let _ = write!(line, "{color}{flag}={on}{RESET} ");
        }
        println!("{}", line.trim_end());
    }

    let hard_ok = build.is_none_or(|b| !b.ran || b.ok)
        && config
            .get("admitted")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    let soft_ok = failure_count == 0
        && receipts
            .get("dir_exists")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        && tools
            .get("cicd-evidence-gen")
            .and_then(Value::as_bool)
            .unwrap_or(false);

    println!();
    if !hard_ok {
        println!("{BOLD}Overall: {RED}UNHEALTHY{RESET}");
    } else if !soft_ok {
        println!("{BOLD}Overall: {YELLOW}DEGRADED{RESET}");
    } else {
        println!("{BOLD}Overall: {GREEN}HEALTHY{RESET}");
    }
}

/// Domain logic for `check`: run every health check and either print the
/// human-readable summary (returning `Value::Null`) or assemble the single
/// machine-readable JSON report, depending on `format`. Kept out of the
/// `#[verb]` function itself so `check` stays a thin CLI wrapper (the
/// `#[verb]` macro caps verb-function cyclomatic complexity at 5).
fn run_doctor(format: &str, skip_build: bool) -> Value {
    let build = if skip_build {
        None
    } else {
        Some(run_build_check(BUILD_CHECK_TIMEOUT))
    };
    let config = check_config();
    let frontier = check_frontier();
    let receipts_dir = receipts_dir_from_config();
    let receipts = check_receipts(&receipts_dir);
    let tools = tool_report();
    let features = feature_flags();

    if format == "text" {
        print_human_summary(
            build.as_ref(),
            &config,
            &frontier,
            &receipts,
            &tools,
            &features,
        );
        return Value::Null;
    }

    let build_json = match &build {
        None => json!({ "ran": false, "ok": Value::Null, "detail": "skipped (--skip-build)" }),
        Some(b) => {
            json!({ "ran": b.ran, "ok": b.ok, "duration_ms": b.duration_ms, "detail": b.detail })
        }
    };

    json!({
        "build": build_json,
        "config": config,
        "frontier": frontier,
        "receipts": receipts,
        "tools": tools,
        "features": features,
    })
}

/// Run every health check and print a holistic summary of the workspace:
/// build status, config witness, frontier `pass_rate`/coverage, external
/// tools on `PATH`, the receipts ledger, and compiled-in feature flags.
///
/// Human-readable by default; pass `--format json` for a single machine
/// -readable JSON object instead (matching the rest of the CLI's
/// `--format text|json` convention, e.g. `law show`).
#[verb]
pub fn check(
    #[arg(
        default_value = "text",
        help = "Output format: text (human-readable, colored) or json (machine-readable)"
    )]
    format: String,
    #[arg(
        help = "Skip the `cargo check --workspace` build probe (faster, but omits build status)"
    )]
    skip_build: bool,
) -> Result<Value> {
    Ok(run_doctor(&format, skip_build))
}
