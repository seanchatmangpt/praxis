//! Example: Applying retrofits to a fleet of repositories
//!
//! This example demonstrates how to use the RetrofitApplier to apply
//! retrofit changes across multiple repositories with automatic worktree
//! management, validation, and commit creation.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example fleet_apply_example --release
//! ```

// Recorded lint debt (v26.7.6 verification gate) -- see src/lib.rs and
// docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(missing_docs, dead_code)]
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

use praxis_retrofit::{PraxisSpec, RetrofitApplier, RetrofitPhase};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?),
        )
        .init();

    // Create praxis spec (use default standards)
    let spec = PraxisSpec::default();

    // Create the retrofit applier
    let applier = RetrofitApplier::new(spec)?;

    // Configure concurrent limit (default: 4)
    let mut applier = applier.with_concurrent_limit(2);

    // Example: Add repositories to retrofit
    // You would replace these with actual repository paths
    let example_repos = vec![
        ("./test-repo-1", RetrofitPhase::Phase1Lints),
        ("./test-repo-2", RetrofitPhase::Phase1Lints),
        ("./test-repo-3", RetrofitPhase::Phase2Deps),
    ];

    // Register repositories (skip if they don't exist)
    for (repo_path, phase) in example_repos {
        match applier.add_repository(repo_path, phase) {
            Ok(()) => println!("Registered: {}", repo_path),
            Err(e) => println!("Skipping {}: {}", repo_path, e),
        }
    }

    if applier.repositories().is_empty() {
        println!("No repositories to retrofit. Create test repos first:");
        println!("  git init test-repo-1");
        println!("  git init test-repo-2");
        println!("  git init test-repo-3");
        return Ok(());
    }

    // Apply retrofits to all registered repositories
    println!("\nApplying retrofits...");
    let results = applier.apply_all().await?;

    // Generate and display summary report
    let report = RetrofitApplier::summary(&results);
    report.print_summary();

    // Export detailed results as JSON
    let json = serde_json::to_string_pretty(&report)?;
    println!("\nDetailed results (JSON):");
    println!("{}", json);

    Ok(())
}
