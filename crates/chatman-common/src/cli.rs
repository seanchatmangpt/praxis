//! CLI helpers: output format, color mode, print utilities, and global args.

use clap::{ArgAction, Parser, ValueEnum};

use crate::Result;

/// Output format selection.
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum OutputFormat {
    /// JSON output.
    Json,
    /// YAML output.
    Yaml,
    /// Human-readable text output.
    Text,
}

/// Color output control.
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum ColorMode {
    /// Use color when the output is a terminal (default).
    Auto,
    /// Always use color.
    Always,
    /// Never use color.
    Never,
}

impl ColorMode {
    /// Return `true` if color should be written to `stream`.
    ///
    /// `is_tty` — caller passes `std::io::IsTerminal::is_terminal(&std::io::stdout())`.
    pub fn enabled(&self, is_tty: bool) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => is_tty,
        }
    }
}

/// ANSI color/style helpers.  Only emitted when color is enabled.
pub mod color {
    /// Wrap `text` in bold ANSI escape sequences.
    pub fn bold(text: &str) -> String {
        format!("\x1b[1m{text}\x1b[0m")
    }

    /// Wrap `text` in green ANSI escape sequences.
    pub fn green(text: &str) -> String {
        format!("\x1b[32m{text}\x1b[0m")
    }

    /// Wrap `text` in red ANSI escape sequences.
    pub fn red(text: &str) -> String {
        format!("\x1b[31m{text}\x1b[0m")
    }

    /// Wrap `text` in yellow ANSI escape sequences.
    pub fn yellow(text: &str) -> String {
        format!("\x1b[33m{text}\x1b[0m")
    }

    /// Wrap `text` in dim ANSI escape sequences.
    pub fn dim(text: &str) -> String {
        format!("\x1b[2m{text}\x1b[0m")
    }
}

/// Print `value` to stdout in the requested `format`.
///
/// `T` must implement [`serde::Serialize`] so it can be rendered as JSON or
/// YAML.  For [`OutputFormat::Text`] the caller's `text_fn` closure is invoked
/// to produce a human-readable string.
///
/// # Errors
/// Returns an error when JSON serialization fails.  YAML output falls back to
/// JSON serialization when the `serde_json` pretty-printer is unavailable
/// (no extra dep required for basic YAML-like output).
pub fn print_output<T, F>(value: &T, format: &OutputFormat, text_fn: F) -> Result<()>
where
    T: serde::Serialize,
    F: FnOnce(&T) -> String,
{
    let out = match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(value)
                .map_err(|e| crate::Error::msg(format!("JSON serialization failed: {e}")))?
        }
        OutputFormat::Yaml => {
            // Lightweight YAML-ish: pretty JSON is valid YAML for our types.
            // Replace with `serde_yaml` when that dep is added.
            serde_json::to_string_pretty(value)
                .map_err(|e| crate::Error::msg(format!("YAML serialization failed: {e}")))?
        }
        OutputFormat::Text => text_fn(value),
    };
    println!("{out}");
    Ok(())
}

/// Global arguments shared across all subcommands.
#[derive(Debug, Clone, Parser)]
pub struct GlobalArgs {
    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Color output.
    #[arg(long, value_enum, default_value = "auto")]
    pub color: ColorMode,

    /// Verbosity level (pass multiple times to increase).
    #[arg(short = 'v', action = ArgAction::Count)]
    pub verbose: u8,
}

impl GlobalArgs {
    /// Return `true` if color should be emitted to stdout.
    pub fn color_enabled(&self) -> bool {
        use std::io::IsTerminal as _;
        self.color.enabled(std::io::stdout().is_terminal())
    }
}

/// Initialize global settings from parsed args.
///
/// Sets up tracing with the provided `service_name` (defaults to the binary
/// name when empty).
pub fn init(args: &GlobalArgs, service_name: &str) -> Result<()> {
    let name = if service_name.is_empty() {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "app".to_string())
    } else {
        service_name.to_string()
    };

    crate::telemetry::init_tracing(&name)?;

    tracing::debug!(
        format = ?args.format,
        color = ?args.color,
        verbose = args.verbose,
        "CLI initialized"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_mode_auto_respects_tty() {
        assert!(ColorMode::Always.enabled(false));
        assert!(!ColorMode::Never.enabled(true));
        assert!(ColorMode::Auto.enabled(true));
        assert!(!ColorMode::Auto.enabled(false));
    }

    #[test]
    fn color_helpers_wrap_text() {
        assert!(color::bold("hi").contains("hi"));
        assert!(color::green("ok").contains("ok"));
        assert!(color::red("err").contains("err"));
    }

    #[test]
    fn print_output_json() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct Dummy {
            val: u32,
        }

        // Should not panic / error.
        print_output(&Dummy { val: 42 }, &OutputFormat::Json, |d| {
            format!("val={}", d.val)
        })
        .unwrap();
    }
}
