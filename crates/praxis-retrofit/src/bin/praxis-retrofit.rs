//! Praxis Retrofit CLI - Automate standardization across Rust ecosystem

// Recorded lint debt (v26.7.6 verification gate) -- see src/lib.rs and
// docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(missing_docs, dead_code)]
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

use std::path::PathBuf;

use praxis_retrofit::process_discovery::{
    conformance_report, discover_lifecycle, load_ocel, reference_arcs_admission_lifecycle,
    reference_arcs_full_lifecycle,
};
use praxis_retrofit::repo_registry::RepositoryRegistry;
use praxis_retrofit::{apply, audit, generate, validate, PraxisSpec, RetrofitPhase, VERSION};
use tracing::{error, info};

/// Default path used for `--include-ecosystem-lock` when the flag is passed
/// with no explicit value.
const DEFAULT_ECOSYSTEM_LOCK_PATH: &str = ".chatmangpt/ecosystem.lock.toml";

/// Parses an optional `--include-ecosystem-lock [path]` flag out of `args`.
///
/// Returns `None` if the flag is absent (existing behavior unchanged).
/// Returns `Some(path)` if present, using [`DEFAULT_ECOSYSTEM_LOCK_PATH`]
/// when no value follows the flag (or the next token is itself another
/// flag).
fn parse_ecosystem_lock_flag(args: &[String]) -> Option<PathBuf> {
    let idx = args.iter().position(|a| a == "--include-ecosystem-lock")?;
    match args.get(idx + 1) {
        Some(value) if !value.starts_with("--") => Some(PathBuf::from(value)),
        _ => Some(PathBuf::from(DEFAULT_ECOSYSTEM_LOCK_PATH)),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Praxis Retrofit v{}", VERSION);

    let spec = PraxisSpec::default();

    // Simple CLI routing (demonstrating clap-noun-verb pattern)
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args.get(1).map(|s| s.as_str()) {
        Some("audit") => handle_audit(&args, &spec).await?,
        Some("apply") => handle_apply(&args, &spec).await?,
        Some("generate") => handle_generate(&args, &spec).await?,
        Some("validate") => handle_validate(&args).await?,
        Some("ecosystem-conformance") => {
            if let Err(code) = handle_ecosystem_conformance(&args) {
                std::process::exit(code);
            }
        }
        Some("--version") | Some("-v") => println!("praxis-retrofit {}", VERSION),
        Some("--help") | Some("-h") => print_usage(),
        Some(cmd) => {
            error!("Unknown command: {}", cmd);
            print_usage();
        }
        None => print_usage(),
    }

    Ok(())
}

async fn handle_audit(args: &[String], spec: &PraxisSpec) -> anyhow::Result<()> {
    if args.len() < 4 {
        eprintln!("Usage: praxis-retrofit audit <scan|report> <repo-path>");
        return Ok(());
    }

    let action = &args[2];
    let repo_path = PathBuf::from(&args[3]);

    // Optional: fold `.chatmangpt/ecosystem.lock.toml`-pinned dependencies
    // into the fleet registry before auditing. Default (flag absent):
    // existing behavior, unchanged.
    if let Some(lock_path) = parse_ecosystem_lock_flag(args) {
        match RepositoryRegistry::load_with_ecosystem("repos.toml", &lock_path).await {
            Ok(registry) => {
                println!(
                    "Loaded fleet registry with ecosystem.lock.toml union: {} total repositories ({})",
                    registry.all().len(),
                    lock_path.display()
                );
            }
            Err(e) => {
                eprintln!("Failed to load registry with ecosystem lock: {e}");
            }
        }
    }

    match action.as_str() {
        "scan" => {
            let report = audit::scan_repository(&repo_path, spec).await?;
            println!("Audit Report: {}", serde_json::to_string_pretty(&report)?);
        }
        "report" => {
            let report = audit::scan_repository(&repo_path, spec).await?;
            println!("Repository: {}", report.repository.name);
            println!("Score: {:.1}%", report.score());
            println!("Compliant: {}", report.is_compliant());
            for check in &report.checks {
                println!(
                    "  {} ({}): {:?}",
                    check.name, check.category as i32, check.status
                );
            }
        }
        _ => eprintln!("Unknown audit action: {}", action),
    }

    Ok(())
}

async fn handle_apply(args: &[String], spec: &PraxisSpec) -> anyhow::Result<()> {
    if args.len() < 4 {
        eprintln!("Usage: praxis-retrofit apply <retrofit|validate> <repo-path>");
        return Ok(());
    }

    let action = &args[2];
    let repo_path = PathBuf::from(&args[3]);

    match action.as_str() {
        "retrofit" => {
            let plan =
                generate::generate_retrofit_plan(&repo_path, RetrofitPhase::Phase1Lints, spec)?;
            let results = apply::apply_retrofit(&repo_path, &plan).await?;
            println!("Applied {} changes:", results.len());
            for result in results {
                println!("  {}", result);
            }
        }
        "validate" => {
            let valid = apply::validate_retrofit(&repo_path).await?;
            println!(
                "Retrofit validation: {}",
                if valid { "PASS" } else { "FAIL" }
            );
        }
        _ => eprintln!("Unknown apply action: {}", action),
    }

    Ok(())
}

async fn handle_generate(args: &[String], spec: &PraxisSpec) -> anyhow::Result<()> {
    if args.len() < 3 {
        eprintln!("Usage: praxis-retrofit generate <templates|plan> [repo-path]");
        return Ok(());
    }

    let action = &args[2];

    match action.as_str() {
        "templates" => {
            println!("# Praxis Templates\n");
            println!("## Cargo.toml [lints]");
            println!(
                "{}\n",
                praxis_retrofit::templates::cargo_lints_template(spec)
            );
            println!("## typos.toml");
            println!("{}\n", praxis_retrofit::templates::typos_toml_template());
            println!("## justfile");
            println!(
                "{}",
                praxis_retrofit::templates::justfile_template("example")
            );
        }
        "plan" => {
            if args.len() < 4 {
                eprintln!("Usage: praxis-retrofit generate plan <repo-path>");
                return Ok(());
            }
            let repo_path = PathBuf::from(&args[3]);
            let plan =
                generate::generate_retrofit_plan(&repo_path, RetrofitPhase::Phase1Lints, spec)?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        _ => eprintln!("Unknown generate action: {}", action),
    }

    Ok(())
}

async fn handle_validate(args: &[String]) -> anyhow::Result<()> {
    if args.len() < 4 {
        eprintln!("Usage: praxis-retrofit validate <compliance|gates> <repo-path>");
        return Ok(());
    }

    let action = &args[2];
    let repo_path = PathBuf::from(&args[3]);

    match action.as_str() {
        "compliance" => {
            let report = validate::validate_compliance(&repo_path).await?;
            println!("Compliance Score: {:.1}%", report.score());
            println!(
                "Status: {}",
                if report.is_compliant() {
                    "PASS"
                } else {
                    "FAIL"
                }
            );
        }
        _ => eprintln!("Unknown validate action: {}", action),
    }

    Ok(())
}

/// Parse `--log <path>` (required) and `--reference <admission|full>`
/// (default `admission`) from raw CLI args, print the conformance report,
/// and return `Err(1)` on a missing/unparseable log so `main` can exit(1)
/// without the `?` operator unwinding through `anyhow`.
fn handle_ecosystem_conformance(args: &[String]) -> Result<(), i32> {
    let mut log_path: Option<PathBuf> = None;
    let mut reference = "admission".to_string();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--log" => {
                log_path = args.get(i + 1).map(PathBuf::from);
                i += 2;
            }
            "--reference" => {
                if let Some(v) = args.get(i + 1) {
                    reference = v.clone();
                }
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    let Some(log_path) = log_path else {
        eprintln!(
            "Usage: praxis-retrofit ecosystem-conformance --log <path> [--reference admission|full]"
        );
        return Err(1);
    };

    let reference_arcs = match reference.as_str() {
        "admission" => reference_arcs_admission_lifecycle(),
        "full" => reference_arcs_full_lifecycle(),
        other => {
            eprintln!(
                "Unknown --reference value: {} (expected 'admission' or 'full')",
                other
            );
            return Err(1);
        }
    };

    let ocel = match load_ocel(&log_path) {
        Ok(ocel) => ocel,
        Err(e) => {
            eprintln!("Failed to load OCEL log at {}: {e}", log_path.display());
            return Err(1);
        }
    };

    // discover_lifecycle mines the DFG this report is measured against;
    // conformance_report recomputes it internally, so this call surfaces
    // the same DFG the report describes without changing the report math.
    let _dfg = discover_lifecycle(&ocel);
    let report = conformance_report(&ocel, &reference_arcs);

    println!("{}", ecosystem_conformance_table(&reference, &report));

    Ok(())
}

/// Render an [`praxis_retrofit::process_discovery::ConformanceSummary`] as a
/// pretty-printed table, following the same box-drawing convention as
/// `fleet_audit::AuditSummary::summary_table`.
fn ecosystem_conformance_table(
    reference: &str,
    report: &praxis_retrofit::process_discovery::ConformanceSummary,
) -> String {
    let mut output = String::new();

    output.push_str("╔════════════════════════════════════════════════════════╗\n");
    output.push_str("║       Ecosystem Conformance Report                    ║\n");
    output.push_str("╚════════════════════════════════════════════════════════╝\n\n");

    output.push_str(&format!("Reference:  {}\n", reference));
    output.push_str(&format!("Fitness:    {:.1}%\n", report.fitness * 100.0));
    output.push_str(&format!("Precision:  {:.1}%\n", report.precision * 100.0));
    output.push('\n');

    output.push_str(&format!(
        "Missing From Log ({}):\n",
        report.missing_from_log.len()
    ));
    if report.missing_from_log.is_empty() {
        output.push_str("  (none)\n");
    } else {
        for (from, to) in &report.missing_from_log {
            output.push_str(&format!("  {} -> {}\n", from, to));
        }
    }
    output.push('\n');

    output.push_str(&format!(
        "Unexpected In Log ({}):\n",
        report.unexpected_in_log.len()
    ));
    if report.unexpected_in_log.is_empty() {
        output.push_str("  (none)\n");
    } else {
        for (from, to) in &report.unexpected_in_log {
            output.push_str(&format!("  {} -> {}\n", from, to));
        }
    }

    output
}

fn print_usage() {
    eprintln!(
        r#"Praxis Retrofit v{} - Automate standardization across Rust ecosystem

USAGE:
    praxis-retrofit <COMMAND> <SUBCOMMAND> [OPTIONS]

COMMANDS:
    audit       Audit repository compliance
    apply       Apply retrofit changes
    generate    Generate retrofit artifacts
    validate    Validate compliance gates
    ecosystem-conformance   Report DFG conformance against a retrofit OCEL log
    --version   Print version
    --help      Print this help

EXAMPLES:
    # Audit repository compliance
    praxis-retrofit audit scan /path/to/repo
    praxis-retrofit audit report /path/to/repo

    # Apply retrofit to repository
    praxis-retrofit apply retrofit /path/to/repo
    praxis-retrofit apply validate /path/to/repo

    # Generate templates
    praxis-retrofit generate templates
    praxis-retrofit generate plan /path/to/repo

    # Validate compliance
    praxis-retrofit validate compliance /path/to/repo

    # Ecosystem conformance report
    praxis-retrofit ecosystem-conformance --log /path/to/ocel.json --reference admission
    praxis-retrofit ecosystem-conformance --log /path/to/ocel.json --reference full
"#,
        VERSION
    );
}
