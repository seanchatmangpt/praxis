//! CLI configuration for `{{project-name}}`.
//!
//! Defines the top-level [`Cli`] struct parsed by [`clap::Parser`], plus the
//! [`OutputFormat`] and [`ColorMode`] enums. Output rendering lives in
//! [`print_output`] so every verb can share a consistent serialization path.

use std::fmt;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Top-level CLI struct
// ---------------------------------------------------------------------------

/// Command-line interface for `{{project-name}}`.
#[derive(Debug, Parser)]
#[command(
    name = "{{project-name}}",
    version,
    about = "{{description}}",
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
