//! Example: Integrating Compliance Gates in a GitHub Actions Workflow
//!
//! This example demonstrates how to use the ci_gate module to programmatically
//! validate compliance and generate remediation suggestions for a CI/CD workflow.
//!
//! Run with:
//! ```bash
//! cargo run --example compliance-gate-integration -- /path/to/repo
//! ```

// Recorded lint debt (v26.7.6 verification gate) -- see src/lib.rs and
// docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

use std::path::PathBuf;

use praxis_retrofit::{
    format_remediation_markdown, validate_compliance, BadgeGenerator, ComplianceGate,
    GateCheckOutput, GateResult, RemediationPriority,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let repo_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    println!("=== Praxis Compliance Gate Integration Example ===\n");
    println!("Repository: {}\n", repo_path.display());

    // Step 1: Audit the repository
    println!("📊 Step 1: Running compliance audit...\n");
    let report = validate_compliance(&repo_path).await?;

    println!("Repository: {}", report.repository.name);
    println!("Timestamp: {}", report.timestamp);
    println!("Compliance Score: {:.1}%\n", report.score());

    // Step 2: Create a compliance gate with default config
    println!("🚪 Step 2: Creating compliance gate...\n");
    let gate = ComplianceGate::new();

    // Step 3: Run the gate check
    println!("✅ Step 3: Running gate check...\n");
    let output = gate.check(&report).await?;

    print_gate_output(&output);

    // Step 4: Generate remediation markdown (for PR comment)
    println!("\n🔧 Step 4: Generating remediation suggestions...\n");
    let markdown = format_remediation_markdown(&output.remediation_steps);
    println!("{}", markdown);

    // Step 5: Generate compliance badge
    println!("\n📊 Step 5: Generating compliance badge...\n");
    let svg = BadgeGenerator::generate_svg(output.score, &output.badge_label, &output.badge_color);
    println!("SVG Badge:\n{}\n", svg);

    // Step 6: Print summary for CI output
    println!("\n=== Gate Summary ===\n");
    println!("Gate Result: {:?}", output.gate_result);
    println!(
        "Score: {:.1}% (threshold: {:.1}%)",
        output.score, output.threshold
    );
    println!("Message: {}\n", output.message);

    if !output.blocking_issues.is_empty() {
        println!("❌ Blocking Issues ({}):", output.blocking_issues.len());
        for issue in &output.blocking_issues {
            println!("  - {}", issue);
        }
    }

    if !output.warnings.is_empty() {
        println!("\n⚠️  Warnings ({}):", output.warnings.len());
        for warning in &output.warnings {
            println!("  - {}", warning);
        }
    }

    if !output.remediation_steps.is_empty() {
        println!(
            "\n🔧 Remediation Steps ({}):",
            output.remediation_steps.len()
        );
        for (i, step) in output.remediation_steps.iter().enumerate() {
            println!(
                "\n  {}. [{}] {}",
                i + 1,
                format!("{:?}", step.priority),
                step.issue
            );
            println!("     Suggestion: {}", step.suggestion);
            if let Some(cmd) = &step.command {
                println!("     Command: {}", cmd);
            }
            if let Some(ref_link) = &step.reference {
                println!("     Reference: {}", ref_link);
            }
        }
    }

    // Step 7: Determine exit code for CI
    println!("\n=== Exit Code ===\n");
    match output.gate_result {
        GateResult::Pass => {
            println!("✅ Gate PASSED - PR can merge");
            Ok(())
        }
        GateResult::Fail => {
            println!("❌ Gate FAILED - PR is blocked");
            println!("\nTo fix: {}", output.message);
            anyhow::bail!("Compliance gate failed")
        }
        GateResult::Warning => {
            println!("⚠️  Gate returned WARNING");
            println!("Message: {}", output.message);
            Ok(())
        }
    }
}

fn print_gate_output(output: &GateCheckOutput) {
    println!("Gate Result: {:?}", output.gate_result);
    println!("Score: {:.1}%", output.score);
    println!("Threshold: {:.1}%", output.threshold);
    println!("Badge: {} ({})", output.badge_label, output.badge_color);
    println!("Message: {}\n", output.message);

    if !output.blocking_issues.is_empty() {
        println!("Blocking Issues:");
        for issue in &output.blocking_issues {
            println!("  ❌ {}", issue);
        }
    }

    if !output.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &output.warnings {
            println!("  ⚠️  {}", warning);
        }
    }

    if !output.remediation_steps.is_empty() {
        println!("\nRemediation Steps Available:");
        for step in &output.remediation_steps {
            let icon = match step.priority {
                RemediationPriority::Critical => "🚨",
                RemediationPriority::High => "⚠️ ",
                RemediationPriority::Medium => "🔧",
                RemediationPriority::Low => "💡",
            };
            println!(
                "  {} {} ({})",
                icon,
                step.issue,
                format!("{:?}", step.priority)
            );
        }
    }
}
