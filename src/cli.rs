//! CLI configuration for `my-conforming-project`.
//!
//! Defines the top-level [`Cli`] struct parsed by [`clap::Parser`], plus the
//! [`OutputFormat`] and [`ColorMode`] enums. Output rendering lives in
//! [`print_output`] so every verb can share a consistent serialization path.

#![allow(clippy::print_stdout)]

use std::fmt;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Top-level CLI struct
// ---------------------------------------------------------------------------

/// Command-line interface for `my-conforming-project`.
#[derive(Debug, Parser)]
#[command(
    name = "my-conforming-project",
    version,
    about = "CLI tool for my-conforming-project",
    long_about = None,
)]
pub struct Cli {
    /// Output format.
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = OutputFormat::Text,
        env = "OUTPUT_FORMAT"
    )]
    pub format: OutputFormat,

    /// Color mode for terminal output.
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = ColorMode::Auto,
        env = "COLOR_MODE"
    )]
    pub color: ColorMode,

    /// Enable verbose output.
    #[arg(long, short = 'v', global = true, action = clap::ArgAction::SetTrue)]
    pub verbose: bool,

    /// Emit JSON Schema tool definitions for LLM tool-calling (mirrors clap-noun-verb --introspect).
    ///
    /// Prints a JSON array of [`ToolDefinition`] objects to stdout, one per subcommand,
    /// then exits. Each definition is immediately consumable as an LLM tool-call target.
    #[arg(long, global = true, action = clap::ArgAction::SetTrue)]
    pub introspect: bool,
}

// ---------------------------------------------------------------------------
// Tool introspection
// ---------------------------------------------------------------------------

/// A single LLM tool-call target derived from a CLI subcommand.
///
/// Mirrors the `ToolDefinition` surface emitted by `clap-noun-verb --introspect`.
/// The `parameters` field is a JSON Schema `object` describing the subcommand's
/// arguments so any LLM tool-calling API can consume it without additional
/// transformation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolDefinition {
    /// Stable tool name (typically `<noun>_<verb>` or `<verb>`).
    pub name: String,
    /// Human-readable description taken from the subcommand's `about` string.
    pub description: String,
    /// JSON Schema `{"type":"object","properties":{...},"required":[...]}` for
    /// the subcommand's arguments.
    pub parameters: serde_json::Value,
}

/// Walk `cmd`'s subcommand tree and collect one [`ToolDefinition`] per leaf.
///
/// Call this when `--introspect` is set; print the result as JSON and exit.
///
/// ```no_run
/// use my_conforming_project::cli::{collect_tools_from_cmd, Cli};
/// use clap::CommandFactory;
///
/// let tools = collect_tools_from_cmd(&Cli::command());
/// println!("{}", serde_json::to_string_pretty(&tools).unwrap());
/// ```
pub fn collect_tools_from_cmd(cmd: &clap::Command) -> Vec<ToolDefinition> {
    let mut out = Vec::new();
    collect_recursive(cmd, &[], &mut out);
    out
}

fn collect_recursive(cmd: &clap::Command, prefix: &[&str], out: &mut Vec<ToolDefinition>) {
    let subs: Vec<_> = cmd.get_subcommands().collect();
    if subs.is_empty() {
        // Leaf — emit a ToolDefinition.
        let parts: Vec<&str> =
            prefix.iter().copied().chain(std::iter::once(cmd.get_name())).collect();
        let name = parts.join("_");
        let description = cmd
            .get_about()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Run the {} command.", name));

        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for arg in cmd.get_arguments() {
            // Skip global / help / version flags that are not domain args.
            if matches!(
                arg.get_id().as_str(),
                "help" | "version" | "format" | "color" | "verbose" | "introspect"
            ) {
                continue;
            }
            let prop_name = arg.get_id().to_string();
            let schema_type = if arg.get_action().takes_values() { "string" } else { "boolean" };
            let mut prop = serde_json::json!({ "type": schema_type });
            if let Some(help) = arg.get_help() {
                prop["description"] = serde_json::Value::String(help.to_string());
            }
            properties.insert(prop_name.clone(), prop);
            if arg.is_required_set() {
                required.push(prop_name);
            }
        }

        let parameters = serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required,
        });

        out.push(ToolDefinition { name, description, parameters });
    } else {
        let new_prefix: Vec<&str> =
            prefix.iter().copied().chain(std::iter::once(cmd.get_name())).collect();
        for sub in subs {
            collect_recursive(sub, &new_prefix, out);
        }
    }
}

// ---------------------------------------------------------------------------
// OutputFormat
// ---------------------------------------------------------------------------

/// Serialization format for command output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Compact JSON (machine-readable).
    Json,
    /// YAML (human-friendly structured).
    Yaml,
    /// Plain text (default, human-readable).
    Text,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
            OutputFormat::Text => write!(f, "text"),
        }
    }
}

// ---------------------------------------------------------------------------
// ColorMode
// ---------------------------------------------------------------------------

/// Terminal color behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorMode {
    /// Enable color if the terminal supports it and `NO_COLOR` is unset.
    Auto,
    /// Always enable color.
    On,
    /// Never emit color codes.
    Off,
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorMode::Auto => write!(f, "auto"),
            ColorMode::On => write!(f, "on"),
            ColorMode::Off => write!(f, "off"),
        }
    }
}

impl ColorMode {
    /// Return `true` when ANSI color codes should be emitted.
    ///
    /// `Auto` defers to the `NO_COLOR` environment variable: if the variable is
    /// set (any value), color is disabled per <https://no-color.org/>.
    #[must_use]
    pub fn enabled(self) -> bool {
        match self {
            ColorMode::On => true,
            ColorMode::Off => false,
            ColorMode::Auto => std::env::var("NO_COLOR").is_err(),
        }
    }
}

// ---------------------------------------------------------------------------
// Output rendering
// ---------------------------------------------------------------------------

/// Render `value` to stdout using the requested `format`.
///
/// - [`OutputFormat::Json`] — compact single-line JSON via `serde_json`.
/// - [`OutputFormat::Yaml`] — YAML via `serde_json` round-trip (no extra dep).
/// - [`OutputFormat::Text`] — pretty-printed JSON (human-readable default).
///
/// All three paths go through `serde::Serialize` so callers need only one
/// implementation.
pub fn print_output<T: Serialize>(value: &T, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let s = serde_json::to_string(value)?;
            println!("{s}");
        }
        OutputFormat::Yaml => {
            // Avoid pulling in serde_yaml; round-trip through serde_json::Value
            // and emit a hand-rolled YAML-like representation for common types.
            let v = serde_json::to_value(value)?;
            print_yaml_value(&v, 0);
        }
        OutputFormat::Text => {
            let s = serde_json::to_string_pretty(value)?;
            println!("{s}");
        }
    }
    Ok(())
}

/// Minimal YAML emitter for `serde_json::Value` (covers objects, arrays, scalars).
fn print_yaml_value(v: &serde_json::Value, indent: usize) {
    let pad = "  ".repeat(indent);
    match v {
        serde_json::Value::Object(map) => {
            for (k, val) in map {
                match val {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        println!("{pad}{k}:");
                        print_yaml_value(val, indent + 1);
                    }
                    _ => {
                        print!("{pad}{k}: ");
                        print_yaml_scalar(val);
                    }
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                match item {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        println!("{pad}-");
                        print_yaml_value(item, indent + 1);
                    }
                    _ => {
                        print!("{pad}- ");
                        print_yaml_scalar(item);
                    }
                }
            }
        }
        scalar => print_yaml_scalar(scalar),
    }
}

fn print_yaml_scalar(v: &serde_json::Value) {
    match v {
        serde_json::Value::String(s) => println!("{s}"),
        serde_json::Value::Null => println!("null"),
        other => println!("{other}"),
    }
}

// ---------------------------------------------------------------------------
// Tool introspection implementation
// ---------------------------------------------------------------------------

/// Walk `cmd`'s subcommand tree and collect one [`ToolDefinition`] per leaf subcommand.
pub fn collect_tool_definitions(cmd: &clap::Command) -> Vec<ToolDefinition> {
    collect_tools_from_cmd(cmd)
}

/// If `--introspect` was passed, emit tool definitions as JSON to stdout and
/// return `true`. The caller should exit immediately after.
pub fn handle_introspect(cli: &Cli, cmd: &clap::Command) -> bool {
    if !cli.introspect {
        return false;
    }
    let tools = collect_tool_definitions(cmd);
    #[allow(clippy::print_stdout)]
    {
        println!(
            "{}",
            serde_json::to_string_pretty(&tools)
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        );
    }
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_format_display() {
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Yaml.to_string(), "yaml");
        assert_eq!(OutputFormat::Text.to_string(), "text");
    }

    #[test]
    fn color_mode_off_never_enabled() {
        assert!(!ColorMode::Off.enabled());
    }

    #[test]
    fn color_mode_on_always_enabled() {
        assert!(ColorMode::On.enabled());
    }

    #[test]
    fn color_mode_auto_respects_no_color() {
        // Force NO_COLOR; auto must be disabled.
        std::env::set_var("NO_COLOR", "1");
        assert!(!ColorMode::Auto.enabled());
        std::env::remove_var("NO_COLOR");
    }

    #[test]
    fn print_output_json_roundtrip() {
        use serde::Serialize;
        #[derive(Serialize)]
        struct Dummy {
            x: u32,
        }
        // Should not panic.
        print_output(&Dummy { x: 42 }, OutputFormat::Json).unwrap();
    }
}
