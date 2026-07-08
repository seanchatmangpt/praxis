//! Thin CLI driver over `rust_fable_testbed`'s library functions.
//!
//! `testbed run <task_id>` loads a task spec, compiles its prompt, sends it to
//! Claude, applies the model's output to a sandboxed copy of the task's fixture, runs
//! the verification pipeline, prints a summary, and appends a chained receipt. All the
//! actual logic lives in the library modules — this file only wires them together.

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use rust_fable_testbed::model_client::{AnthropicClient, Message, MessageRequest, ModelClient};
use rust_fable_testbed::receipt::{append_receipt, chain_receipt, last_chain_hash};
use rust_fable_testbed::sandbox::{apply_model_output, stage_fixture};
use rust_fable_testbed::spec::{load_task, TaskSpec};
use rust_fable_testbed::{prompt, Error, Result};

/// Resolve the fixture-relative path the model's response should overwrite.
///
/// Prefers the task's explicit `tb:targetPath` (required for any task whose prompt
/// shows more than one `ggen:Code` block, e.g. `RepoLevelTranslation` tasks that
/// include read-only sibling-module context alongside the file to fix — see
/// `spec::TaskSpec::target_path`'s docs). Falls back to the file name of the *last*
/// Code-block source path in the task's prompt sections for tasks authored before
/// `tb:targetPath` existed (all such tasks show exactly one Code block, so "last" and
/// "first" coincide there; "last" is used rather than "first" so a future
/// single-block-but-not-first-in-document-order task degrades safely too).
fn resolve_target_path(task: &TaskSpec) -> PathBuf {
    if let Some(target) = &task.target_path {
        return target.clone();
    }
    task.prompt_sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter_map(|b| b.source_path.as_deref())
        .next_back()
        .and_then(|p| Path::new(p).file_name())
        .map_or_else(
            || PathBuf::from("src/lib.rs"),
            |name| Path::new("src").join(name),
        )
}

#[derive(Parser)]
#[command(
    name = "testbed",
    about = "Rust/Claude testbed: eval-harness + spec-driven-dev CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Load a task spec, run it against a model, verify the result, and append a
    /// receipt.
    Run {
        /// Task ID (matches `<tasks_dir>/<task_id>.ttl`).
        task_id: String,

        /// Directory containing `<task_id>.ttl` task spec files.
        #[arg(long, default_value = "tasks")]
        tasks_dir: PathBuf,

        /// Override the task's `tb:model` value.
        #[arg(long)]
        model: Option<String>,

        /// Path to the receipt ledger (JSONL, appended to).
        #[arg(long, default_value = "testbed_receipts.jsonl")]
        ledger: PathBuf,

        /// `max_tokens` for the model request.
        #[arg(long, default_value_t = 16_000)]
        max_tokens: u32,
    },
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Run {
            task_id,
            tasks_dir,
            model,
            ledger,
            max_tokens,
        } => match run(task_id, tasks_dir, model.as_deref(), ledger, *max_tokens) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("testbed run {task_id}: {err}");
                std::process::ExitCode::FAILURE
            }
        },
    }
}

fn run(
    task_id: &str,
    tasks_dir: &Path,
    model_override: Option<&str>,
    ledger_path: &Path,
    max_tokens: u32,
) -> Result<()> {
    let ttl_path = tasks_dir.join(format!("{task_id}.ttl"));
    let task = load_task(&ttl_path).map_err(|e| Error::Spec(e.to_string()))?;

    let compiled = prompt::compile_task_prompt(&task)?;

    let client = AnthropicClient::from_env().map_err(|e| Error::Model(e.to_string()))?;
    let model = model_override.unwrap_or(&task.model);
    let request = MessageRequest {
        model,
        max_tokens,
        system: None,
        messages: vec![Message::user(compiled.content())],
        effort: None,
    };
    let response = client
        .send(&request)
        .map_err(|e| Error::Model(e.to_string()))?;
    let model_output = response.text().map_err(|e| Error::Model(e.to_string()))?;

    // Fixture path in the task spec is relative to the .ttl file's directory.
    let base_dir = ttl_path.parent().unwrap_or_else(|| Path::new("."));
    let fixture_dir = base_dir.join(&task.fixture);
    let staged = stage_fixture(&fixture_dir).map_err(|e| Error::Sandbox(e.to_string()))?;

    let target_rel_path = resolve_target_path(&task);

    apply_model_output(staged.path(), &target_rel_path, &model_output)
        .map_err(|e| Error::Sandbox(e.to_string()))?;

    let metrics =
        rust_fable_testbed::pipeline::run_pipeline_for_task(staged.path(), Some(task.task_type));
    println!("{}", metrics.summary_line());

    let prev = last_chain_hash(ledger_path)?;
    let receipt = chain_receipt(&prev, task_id, compiled.hash(), model, &metrics)?;
    append_receipt(ledger_path, &receipt)?;

    Ok(())
}
