//! Example demonstrating PR generation for mass retrofit
//!
//! This example shows how to:
//! 1. Configure the PR generator
//! 2. Generate PR templates for each phase
//! 3. Track PR status across a fleet
//! 4. Create a comprehensive status report
//!
//! Run with: cargo run --example pr_generation_demo

use std::path::PathBuf;

use praxis_retrofit::{
    PRStatus, PullRequestGenerator, PullRequestGeneratorConfig, PullRequestInfo,
    RepositoryMetadata, RetrofitPhase, RiskLevel,
};

fn main() {
    println!("=== Praxis Retrofit: PR Generation Demo ===\n");

    // Step 1: Configure the PR generator
    let config = PullRequestGeneratorConfig {
        github_owner: "seanchatmangpt".to_string(),
        create_as_draft: true,
        auto_assign_reviewers: vec!["@seanchatmangpt".to_string()],
        labels: vec!["retrofit".to_string(), "praxis".to_string(), "automated".to_string()],
        base_branch: "main".to_string(),
        branch_prefix: "praxis/retrofit".to_string(),
    };

    let generator = PullRequestGenerator::new(config.clone());

    println!("Configuration:\n{:#?}\n", config);

    // Step 2: Generate PR templates for a sample repository
    let sample_repo = RepositoryMetadata {
        path: PathBuf::from("/home/seanchatmangpt/wasm4pm"),
        name: "wasm4pm".to_string(),
        workspace_root: PathBuf::from("/home/seanchatmangpt/wasm4pm"),
        crate_count: 3,
        has_workspace: true,
    };

    println!("Sample Repository: {}", sample_repo.name);
    println!("Path: {}\n", sample_repo.path.display());

    // Generate templates for each phase
    let phases = vec![
        ("Phase 1: Lints", RetrofitPhase::Phase1Lints),
        ("Phase 2: Dependencies", RetrofitPhase::Phase2Deps),
        ("Phase 3: Justfile", RetrofitPhase::Phase3Justfile),
        ("Phase 4: Typos", RetrofitPhase::Phase4Typos),
        ("Phase 5: Documentation", RetrofitPhase::Phase5Docs),
    ];

    for (phase_name, phase) in &phases {
        println!("--- {} ---", phase_name);
        let template = PullRequestGenerator::template_for_phase(*phase, &sample_repo, 5);

        println!("Title:\n  {}\n", template.title);
        println!("Labels: {}\n", template.labels.join(", "));
        println!(
            "Body preview (first 400 chars):\n  {}\n",
            template.body.chars().take(400).collect::<String>()
        );
    }

    // Step 3: Generate branch names for each phase
    println!("\n=== Generated Branch Names ===\n");
    for (phase_name, phase) in &phases {
        let branch = generator.branch_name(&sample_repo.name, *phase);
        println!("{}: {}", phase_name, branch);
    }

    // Step 4: Create mock PR tracking data
    println!("\n=== Fleet-wide PR Status Simulation ===\n");

    let mock_prs = vec![
        PullRequestInfo {
            repository: RepositoryMetadata {
                path: PathBuf::from("/repos/wasm4pm"),
                name: "wasm4pm".to_string(),
                workspace_root: PathBuf::from("/repos/wasm4pm"),
                crate_count: 3,
                has_workspace: true,
            },
            url: Some("https://github.com/seanchatmangpt/wasm4pm/pull/42".to_string()),
            number: Some(42),
            status: PRStatus::Merged,
            branch_name: "praxis/retrofit/phase-1-lints/wasm4pm".to_string(),
            phase: RetrofitPhase::Phase1Lints,
            created_at: Some("2026-06-23T10:00:00+00:00".to_string()),
            estimated_risk: RiskLevel::Low,
            files_changed: 3,
            commits: 1,
            review_comments: vec![],
        },
        PullRequestInfo {
            repository: RepositoryMetadata {
                path: PathBuf::from("/repos/pm4py-rs"),
                name: "pm4py-rs".to_string(),
                workspace_root: PathBuf::from("/repos/pm4py-rs"),
                crate_count: 2,
                has_workspace: false,
            },
            url: Some("https://github.com/seanchatmangpt/pm4py-rs/pull/15".to_string()),
            number: Some(15),
            status: PRStatus::Open,
            branch_name: "praxis/retrofit/phase-1-lints/pm4py-rs".to_string(),
            phase: RetrofitPhase::Phase1Lints,
            created_at: Some("2026-06-23T11:00:00+00:00".to_string()),
            estimated_risk: RiskLevel::Low,
            files_changed: 4,
            commits: 1,
            review_comments: vec!["Ready for review after passing CI".to_string()],
        },
        PullRequestInfo {
            repository: RepositoryMetadata {
                path: PathBuf::from("/repos/dteam"),
                name: "dteam".to_string(),
                workspace_root: PathBuf::from("/repos/dteam"),
                crate_count: 1,
                has_workspace: false,
            },
            url: Some("https://github.com/seanchatmangpt/dteam/pull/8".to_string()),
            number: Some(8),
            status: PRStatus::ReviewRequested,
            branch_name: "praxis/retrofit/phase-1-lints/dteam".to_string(),
            phase: RetrofitPhase::Phase1Lints,
            created_at: Some("2026-06-23T09:30:00+00:00".to_string()),
            estimated_risk: RiskLevel::Low,
            files_changed: 2,
            commits: 1,
            review_comments: vec!["Awaiting maintainer review".to_string()],
        },
        PullRequestInfo {
            repository: RepositoryMetadata {
                path: PathBuf::from("/repos/miniml"),
                name: "miniml".to_string(),
                workspace_root: PathBuf::from("/repos/miniml"),
                crate_count: 4,
                has_workspace: true,
            },
            url: Some("https://github.com/seanchatmangpt/miniml/pull/23".to_string()),
            number: Some(23),
            status: PRStatus::Draft,
            branch_name: "praxis/retrofit/phase-2-deps/miniml".to_string(),
            phase: RetrofitPhase::Phase2Deps,
            created_at: Some("2026-06-23T14:00:00+00:00".to_string()),
            estimated_risk: RiskLevel::Low,
            files_changed: 8,
            commits: 2,
            review_comments: vec![],
        },
    ];

    // Print individual PR status
    for pr in &mock_prs {
        println!("Repository: {}", pr.repository.name);
        println!("  URL: {:?}", pr.url);
        println!("  Status: {:?}", pr.status);
        println!("  Phase: {:?}", pr.phase);
        println!("  Files Changed: {}", pr.files_changed);
        println!("  Branch: {}", pr.branch_name);
        println!();
    }

    // Step 5: Generate fleet summary
    println!("\n=== Fleet Summary ===\n");
    let summary = PullRequestGenerator::summarize_fleet_prs(&mock_prs);
    println!("Total PRs: {}", summary.total);
    println!("Status Breakdown:");
    println!("  Open: {}", summary.by_status.open);
    println!("  Draft: {}", summary.by_status.draft);
    println!("  Review Requested: {}", summary.by_status.review_requested);
    println!("  Approved: {}", summary.by_status.approved);
    println!("  Merged: {}", summary.by_status.merged);
    println!("  Closed: {}", summary.by_status.closed);
    println!("\nGenerated At: {}", summary.generated_at);

    // Step 6: Print example Phase 1 PR body
    println!("\n=== Example: Phase 1 Lints PR Body (full) ===\n");
    let phase1_template = PullRequestGenerator::template_phase1_lints(&sample_repo, 5);
    println!("{}", phase1_template.body);
}
