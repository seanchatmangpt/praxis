//! Example: Complete retrofit validation workflow
//!
//! This example demonstrates:
//! 1. Auditing a repository for pre-retrofit compliance baseline
//! 2. Applying retrofit changes
//! 3. Running comprehensive post-retrofit validation
//! 4. Handling validation results with rollback capability

// Recorded lint debt (v26.7.6 verification gate) -- see src/lib.rs and
// docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(missing_docs, dead_code)]
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

use std::path::Path;

use praxis_retrofit::{
    apply::apply_retrofit, audit::scan_repository, generate::generate_retrofit_plan, CiGateName,
    PraxisSpec, Result, RetrofitPhase, RetrofitValidationStatus, RetrofitValidator,
    ValidationReport,
};

/// Demonstrates the complete retrofit and validation workflow
#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Praxis Retrofit Validation Example ===\n");

    // For demonstration, we'll create a mock scenario
    // In real usage, replace with actual repository path
    let repo_path = Path::new(".");

    validate_workflow(repo_path).await?;

    Ok(())
}

/// Complete workflow: audit → apply → validate → report
async fn validate_workflow(repo_path: &Path) -> Result<()> {
    let spec = PraxisSpec::default();

    // Step 1: Audit pre-retrofit baseline
    println!("Step 1: Auditing baseline compliance...");
    let pre_report = scan_repository(repo_path, &spec).await?;
    let pre_score = pre_report.score();
    println!("  Pre-retrofit compliance score: {:.1}%", pre_score);
    print_compliance_breakdown(&pre_report);

    // Step 2: Generate retrofit plan
    println!("\nStep 2: Generating retrofit plan...");
    let plan = generate_retrofit_plan(repo_path, RetrofitPhase::Phase1Lints, &spec)?;
    println!("  Phase: {:?}", plan.phase);
    println!("  Actions: {}", plan.actions.len());
    println!("  Risk level: {:?}", plan.estimated_risk);

    // Step 3: Apply retrofit (normally done by apply_retrofit)
    println!("\nStep 3: Applying retrofit changes...");
    let _results = apply_retrofit(repo_path, &plan).await?;
    println!("  Retrofit applied ({})", _results.len());

    // Step 4: Validate retrofit with comprehensive checks
    println!("\nStep 4: Running validation gates...");
    let validator = RetrofitValidator::new();
    let validation = validator.validate_retrofit(repo_path, &pre_report).await?;

    // Step 5: Report validation results
    println!("\nValidation Results:");
    print_validation_report(&validation);

    // Step 6: Make decision based on results
    println!("\nDecision:");
    match validation.status {
        RetrofitValidationStatus::Pass => {
            println!("✓ Retrofit validation PASSED - safe to merge");
        }
        RetrofitValidationStatus::Warn => {
            println!("⚠ Retrofit validation WARN - review warnings before merge");
            println!("  Messages: {:?}", validation.messages);
        }
        RetrofitValidationStatus::Fail => {
            println!("✗ Retrofit validation FAILED");
            if validation.rolled_back {
                println!("  Repository rolled back to pre-retrofit state");
            }
            println!("  Messages: {:?}", validation.messages);
        }
    }

    Ok(())
}

/// Print compliance breakdown from a report
fn print_compliance_breakdown(report: &praxis_retrofit::ComplianceReport) {
    println!("  Compliance checks:");
    for check in &report.checks {
        let status_icon = match check.status {
            praxis_retrofit::ComplianceStatus::Pass => "✓",
            praxis_retrofit::ComplianceStatus::Warn => "⚠",
            praxis_retrofit::ComplianceStatus::Fail => "✗",
        };
        println!("    {} {} ({})", status_icon, check.name, check.evidence);
    }
}

/// Print validation report details
fn print_validation_report(report: &ValidationReport) {
    println!("  {}", report.summary());

    println!("\n  Pre-retrofit checks:");
    for check in &report.pre_checks {
        print_check_status(check);
    }

    println!("\n  Post-retrofit checks:");
    for check in &report.post_checks {
        print_check_status(check);
    }

    println!("\n  CI Gate Results:");
    for gate_result in &report.ci_results {
        let status_icon = if gate_result.passed { "✓" } else { "✗" };
        println!("    {} {} ({} ms)", status_icon, gate_result.gate, gate_result.duration_ms);

        if !gate_result.passed {
            if let Some(error) = &gate_result.error {
                let error_msg =
                    if error.len() > 100 { format!("{}...", &error[..100]) } else { error.clone() };
                println!("       Error: {}", error_msg);
            }
        }
    }

    println!("\n  Compliance Improvement:");
    println!("    {} → {} ({:+.1}%)", report.pre_score, report.post_score, report.delta);
}

/// Print individual compliance check status
fn print_check_status(check: &praxis_retrofit::ComplianceItem) {
    let status_icon = match check.status {
        praxis_retrofit::ComplianceStatus::Pass => "✓",
        praxis_retrofit::ComplianceStatus::Warn => "⚠",
        praxis_retrofit::ComplianceStatus::Fail => "✗",
    };
    println!("    {} {}", status_icon, check.name);
}

/// Example: Custom validation with specific gate configuration
#[allow(dead_code)]
async fn custom_validation_example(repo_path: &Path) -> Result<()> {
    use praxis_retrofit::ValidationConfig;

    let spec = PraxisSpec::default();

    // Get baseline
    let pre_report = scan_repository(repo_path, &spec).await?;

    // Create validator with custom config
    let config = ValidationConfig {
        run_tests: false, // Skip tests to be faster
        run_clippy: true,
        check_fmt: true,
        check_deny: true,
        check_typos: false,   // Skip typos
        auto_rollback: false, // Require manual review
        keep_report: true,
        max_output_size: 32768, // Larger output size
    };

    let validator = RetrofitValidator::with_config(config);
    let validation = validator.validate_retrofit(repo_path, &pre_report).await?;

    // Inspect specific gates
    if let Some(fmt_result) = validation.ci_result(CiGateName::Fmt) {
        println!("Format check: {}", if fmt_result.passed { "PASS" } else { "FAIL" });
    }

    if let Some(clippy_result) = validation.ci_result(CiGateName::Clippy) {
        println!("Clippy check: {}", if clippy_result.passed { "PASS" } else { "FAIL" });
        if !clippy_result.passed {
            println!("Output: {}", clippy_result.output);
        }
    }

    Ok(())
}

/// Example: Integration with external systems
#[allow(dead_code)]
async fn export_validation_results(report: &ValidationReport) -> Result<()> {
    // Export as JSON
    let json = serde_json::to_string_pretty(report)?;
    println!("JSON Report:\n{}", json);

    // Export metrics for monitoring systems
    println!("\nMetrics for time-series DB:");
    println!("  compliance_score_pre: {}", report.pre_score);
    println!("  compliance_score_post: {}", report.post_score);
    println!("  compliance_delta: {}", report.delta);
    println!("  ci_gates_passed: {}", report.ci_results.iter().filter(|r| r.passed).count());
    println!("  ci_gates_total: {}", report.ci_results.len());
    println!("  rolled_back: {}", report.rolled_back);

    Ok(())
}
