//! Example demonstrating the repository registry module.
//!
//! Run with:
//!   cargo run --example registry_demo

use std::path::PathBuf;

use praxis_retrofit::repo_registry::RepositoryRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Construct path to repos.toml (two levels up from crate)
    let registry_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("Cannot find repos.toml"))?
        .join("repos.toml");

    println!("Loading registry from: {:?}", registry_path);
    println!();

    let registry = match RepositoryRegistry::load(&registry_path).await {
        Ok(r) => r,
        Err(e) => {
            println!("Error loading registry: {}", e);
            println!("(This is expected if repos.toml is not in the expected location)");
            return Ok(());
        }
    };

    // Display ecosystem overview
    println!("=== ECOSYSTEM OVERVIEW ===");
    println!("Ecosystem: {}", registry.metadata.ecosystem_name);
    println!("Total repos: {}", registry.metadata.total_repos);
    println!("Total crates: {}", registry.metadata.total_crates);
    println!("Survey date: {}", registry.metadata.survey_date);
    println!("House MSRV: {}", registry.metadata.house_msrv);
    println!("House Edition: {}", registry.metadata.house_edition);
    println!();

    // Show readiness summary
    println!("=== RETROFIT READINESS ===");
    println!("{}", registry.readiness_summary());
    println!();

    // List top 5 by priority
    println!("=== TOP 5 PRIORITIES ===");
    for (i, repo) in registry.sorted_by_priority().iter().take(5).enumerate() {
        let effort = repo.estimated_effort_weeks();
        println!(
            "{:2}. {} (priority: {}, effort: {:.1}w, risk: {})",
            i + 1,
            repo.name,
            repo.priority_score,
            effort,
            repo.risk_level
        );
    }
    println!();

    // Show repos with no CI
    println!("=== REPOS NEEDING CI ===");
    let no_ci = registry.no_ci_repos();
    if no_ci.is_empty() {
        println!("  (None — all repos have CI!)");
    } else {
        for repo in no_ci {
            println!("  - {} (add ci.yml in phase 2)", repo.name);
        }
    }
    println!();

    // Show repos with legal considerations
    println!("=== LEGAL CONSIDERATIONS ===");
    let legal = registry.non_standard_licenses();
    if legal.is_empty() {
        println!("  (None — all repos use standard licenses)");
    } else {
        for repo in legal {
            println!(
                "  - {} (risk: {}, notes: {})",
                repo.name,
                repo.risk_level,
                &repo.notes[..60.min(repo.notes.len())]
            );
        }
    }
    println!();

    // Trace downstream dependents
    println!("=== DEPENDENCY EXAMPLES ===");
    for upstream in &["clap-noun-verb", "ggen"] {
        let consumers = registry.downstream_consumers(upstream);
        if !consumers.is_empty() {
            println!("{} is consumed by:", upstream);
            for consumer in consumers {
                println!("  - {} (phase {}/5)", consumer.name, consumer.retrofit_phase_complete);
            }
        }
    }
    println!();

    // Show sorted by effort (easiest first)
    println!("=== RECOMMENDED BATCH (BY EFFORT) ===");
    for (i, (repo, effort)) in registry.sorted_by_effort().iter().take(5).enumerate() {
        println!("{:2}. {} (~{:.2} weeks)", i + 1, repo.name, effort);
    }
    println!();

    // Show recommended retrofit order
    println!("=== RECOMMENDED RETROFIT ORDER ===");
    for (i, repo) in registry.recommended_retrofit_order().iter().take(10).enumerate() {
        println!(
            "{:2}. {} (phase {}/5, readiness: {})",
            i + 1,
            repo.name,
            repo.retrofit_phase_complete,
            repo.retrofit_readiness
        );
    }
    println!();

    Ok(())
}
