//! Spec-kit rendering — the same task ontology used by the eval harness ([`crate::spec`],
//! [`crate::pipeline`]) doubles as spec-driven-dev input: `spec.md` from the task's
//! description/criteria, `tasks.md` checked off as pipeline stages pass.
//!
//! Deliberately simple `format!`-based rendering; full Tera templating (as ggen uses
//! for prompts) isn't warranted for these two small, fixed-shape documents.

use std::fmt::Write as _;

use praxis_core::verify::VerifyMetrics;

use crate::spec::TaskSpec;

/// Render a task spec as a `spec.md` document: description, task type/difficulty,
/// pass criteria, and the fixture it operates on.
///
/// Uses `write!` into a `String` buffer (per `clippy::format_push_string`); the
/// `Result` is intentionally discarded since writing to a `String` cannot fail.
#[must_use]
pub fn render_spec_md(task: &TaskSpec) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {}\n", task.id);
    let _ = writeln!(out, "**Type:** {:?}  ", task.task_type);
    let _ = writeln!(out, "**Difficulty:** {}  ", task.difficulty);
    let _ = writeln!(out, "**Model:** {}  ", task.model);
    let _ = writeln!(out, "**Fixture:** `{}`\n", task.fixture.display());

    out.push_str("## Description\n\n");
    out.push_str(&task.description);
    out.push_str("\n\n");

    out.push_str("## Pass Criteria\n\n");
    if let Some(cargo_test) = &task.pass_criteria.cargo_test {
        let _ = writeln!(out, "- `cargo test` command: `{cargo_test}`");
    }
    let _ = writeln!(
        out,
        "- Clippy deny-warnings: {}",
        if task.pass_criteria.clippy_deny_warnings {
            "yes"
        } else {
            "no"
        }
    );
    out.push('\n');

    if !task.expected_steps.is_empty() {
        out.push_str("## Expected Steps\n\n");
        for step in &task.expected_steps {
            let _ = writeln!(out, "- {step}");
        }
        out.push('\n');
    }

    out
}

/// Render a `tasks.md` checklist: one row per pipeline stage, checked if it passed.
#[must_use]
pub fn render_tasks_md(metrics: &VerifyMetrics) -> String {
    let mut out = String::new();
    out.push_str("# Tasks\n\n");
    for stage in &metrics.stages {
        let checkbox = if stage.passed { "[x]" } else { "[ ]" };
        let ms = stage.duration.as_secs_f64() * 1_000.0;
        let _ = writeln!(out, "- {checkbox} `{}` ({ms:.2}ms)", stage.name);
    }
    out.push('\n');
    let _ = writeln!(out, "**Summary:** {}", metrics.summary_line());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{PassCriteria, PromptSectionSpec, TaskType};
    use praxis_core::verify::VerifyGuard;
    use std::path::PathBuf;

    fn sample_task() -> TaskSpec {
        TaskSpec {
            id: "function_bugfix_001".to_string(),
            task_type: TaskType::FunctionLevelBugfix,
            difficulty: "medium".to_string(),
            model: "claude-opus-4-8".to_string(),
            description: "Fix an off-by-one bug.".to_string(),
            fixture: PathBuf::from("fixtures/function_bugfix_001/"),
            target_path: Some(PathBuf::from("src/lib.rs")),
            expected_steps: vec!["Build".to_string(), "Test".to_string()],
            pass_criteria: PassCriteria {
                cargo_test: Some("cargo test".to_string()),
                clippy_deny_warnings: true,
            },
            prompt_sections: Vec::<PromptSectionSpec>::new(),
        }
    }

    #[test]
    fn render_spec_md_includes_key_fields() {
        let task = sample_task();
        let md = render_spec_md(&task);
        assert!(md.contains("function_bugfix_001"));
        assert!(md.contains("Fix an off-by-one bug."));
        assert!(md.contains("cargo test"));
        assert!(md.contains("Build"));
    }

    #[test]
    fn render_tasks_md_checks_passed_stages() {
        let mut guard = VerifyGuard::new();
        guard.begin_stage("cargo_build");
        guard.end_stage(true);
        guard.begin_stage("cargo_test");
        guard.end_stage(false);
        let metrics = guard.finish();

        let md = render_tasks_md(&metrics);
        assert!(md.contains("[x] `cargo_build`"));
        assert!(md.contains("[ ] `cargo_test`"));
    }
}
