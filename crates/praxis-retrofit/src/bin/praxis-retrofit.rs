//! Praxis Retrofit CLI - Automate standardization across Rust ecosystem

// Recorded lint debt (v26.7.6 verification gate) -- see src/lib.rs and
// docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(missing_docs, dead_code)]
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

use std::path::PathBuf;

use praxis_retrofit::{apply, audit, generate, validate, PraxisSpec, RetrofitPhase, VERSION};
use tracing::{error, info};

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
"#,
        VERSION
    );
}
