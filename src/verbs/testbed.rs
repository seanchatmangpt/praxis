//! `testbed` verb dispatcher — run, list, report.
//!
//! Thin wrappers over `rust_fable_testbed`'s library functions: loading a task spec,
//! compiling its prompt, calling the model, applying the model's output to a sandboxed
//! fixture, running the verification pipeline, and appending a BLAKE3-chained receipt.
//! All the actual logic lives in the `rust-fable-testbed` crate; this module only
//! wires it into the praxis CLI's `#[verb]`/linkme registration.

use std::path::{Path, PathBuf};

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::{arg, verb};

use rust_fable_testbed::model_client::{AnthropicClient, Message, MessageRequest, ModelClient};
use rust_fable_testbed::receipt::{append_receipt, chain_receipt, last_chain_hash, TestbedReceipt};
use rust_fable_testbed::sandbox::{apply_model_output, stage_fixture};
use rust_fable_testbed::spec::load_task;
use rust_fable_testbed::{pipeline, prompt};

/// Default directory containing `<task_id>.ttl` task spec files.
const DEFAULT_TASKS_DIR: &str = "crates/rust-fable-testbed/tasks";
/// Default path to the receipt ledger (JSONL, appended to).
const DEFAULT_LEDGER: &str = "testbed_receipts.jsonl";

/// Map any error implementing `Display` into a `NounVerbError::argument_error`.
fn arg_err<E: std::fmt::Display>(e: E) -> NounVerbError {
    NounVerbError::argument_error(e.to_string())
}

// ── Domain logic ──────────────────────────────────────────────────────────

/// Load `task_id`'s spec, compile its prompt, call the model, apply the output to a
/// sandboxed fixture, run the verification pipeline, and append a chained receipt.
fn run_task(task_id: &str, model_override: Option<&str>) -> std::result::Result<String, String> {
    let tasks_dir = PathBuf::from(DEFAULT_TASKS_DIR);
    let ledger_path = PathBuf::from(DEFAULT_LEDGER);

    let ttl_path = tasks_dir.join(format!("{task_id}.ttl"));
    let task = load_task(&ttl_path).map_err(|e| e.to_string())?;

    let compiled = prompt::compile_task_prompt(&task).map_err(|e| e.to_string())?;

    let client = AnthropicClient::from_env().map_err(|e| e.to_string())?;
    let model = model_override.unwrap_or(&task.model);
    let request = MessageRequest {
        model,
        max_tokens: 16_000,
        system: None,
        messages: vec![Message::user(compiled.content())],
        effort: None,
    };
    let response = client.send(&request).map_err(|e| e.to_string())?;
    let model_output = response.text().map_err(|e| e.to_string())?;

    // Fixture path in the task spec is relative to the .ttl file's directory.
    let base_dir = ttl_path.parent().unwrap_or_else(|| Path::new("."));
    let fixture_dir = base_dir.join(&task.fixture);
    let staged = stage_fixture(&fixture_dir).map_err(|e| e.to_string())?;

    // v1 convention: the target file to overwrite is the file name of the first
    // Code-block source path referenced in the task's prompt sections, placed under
    // the staged fixture's `src/` directory.
    let target_rel_path = task
        .prompt_sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .find_map(|b| b.source_path.as_deref())
        .and_then(|p| Path::new(p).file_name())
        .map_or_else(|| PathBuf::from("src/lib.rs"), |name| Path::new("src").join(name));

    apply_model_output(staged.path(), &target_rel_path, &model_output).map_err(|e| e.to_string())?;

    let metrics = pipeline::run_pipeline_for_task(staged.path(), Some(task.task_type));
    let summary = metrics.summary_line();

    let prev = last_chain_hash(&ledger_path).map_err(|e| e.to_string())?;
    let receipt = chain_receipt(&prev, task_id, compiled.hash(), model, &metrics).map_err(|e| e.to_string())?;
    append_receipt(&ledger_path, &receipt).map_err(|e| e.to_string())?;

    Ok(summary)
}

/// Glob `<tasks_dir>/*.ttl` and collect their file stems (task IDs), sorted.
fn list_task_ids() -> std::result::Result<Vec<String>, String> {
    let tasks_dir = PathBuf::from(DEFAULT_TASKS_DIR);
    if !tasks_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut ids: Vec<String> = std::fs::read_dir(&tasks_dir)
        .map_err(|e| format!("failed to read {}: {e}", tasks_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "ttl"))
        .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .collect();
    ids.sort();
    Ok(ids)
}

/// Read `testbed_receipts.jsonl` (if present) and render a `task_id | model | metrics`
/// summary line per receipt.
fn report_lines() -> std::result::Result<Vec<String>, String> {
    let ledger_path = PathBuf::from(DEFAULT_LEDGER);
    if !ledger_path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&ledger_path)
        .map_err(|e| format!("failed to read {}: {e}", ledger_path.display()))?;

    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let receipt: TestbedReceipt =
                serde_json::from_str(line).map_err(|e| format!("invalid receipt line: {e}"))?;
            Ok(format!(
                "{} | {} | {}",
                receipt.task_id, receipt.model, receipt.metrics_summary
            ))
        })
        .collect()
}

// ── Verb registration ──────────────────────────────────────────────────────

/// Run a testbed task: load its spec, compile the prompt, call the model, verify the
/// result, and append a chained receipt.
#[verb]
pub fn run(
    #[arg(help = "Task ID (matches tasks_dir/<task_id>.ttl)")] task_id: String,
    #[arg(help = "Override the task's default model")] model: Option<String>,
) -> Result<()> {
    let summary = run_task(&task_id, model.as_deref()).map_err(arg_err)?;
    println!("{summary}");
    Ok(())
}

/// List available testbed task IDs (from `crates/rust-fable-testbed/tasks/*.ttl`).
#[verb]
pub fn list() -> Result<()> {
    let ids = list_task_ids().map_err(arg_err)?;
    if ids.is_empty() {
        println!("(no tasks found)");
    } else {
        for id in ids {
            println!("{id}");
        }
    }
    Ok(())
}

/// Print a summary table (task_id, model, pass/fail metrics) from the receipt ledger.
#[verb]
pub fn report() -> Result<()> {
    let lines = report_lines().map_err(arg_err)?;
    if lines.is_empty() {
        println!("(no receipts found)");
    } else {
        for line in lines {
            println!("{line}");
        }
    }
    Ok(())
}
