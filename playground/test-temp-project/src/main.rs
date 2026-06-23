//! Binary entrypoint for my-conforming-project.

use anyhow::Result;

fn main() -> Result<()> {
    let raw: Vec<String> = std::env::args().collect();

    // Cargo external subcommand protocol: `cargo foo bar` → argv = ["cargo-foo", "foo", "bar"]
    // Re-exec self with the injected noun stripped so the rest of main sees clean argv.
    if raw.get(1).map(String::as_str) == Some(env!("CARGO_BIN_NAME").trim_start_matches("cargo-")) {
        let status = std::process::Command::new(&raw[0]).args(&raw[2..]).status()?;
        std::process::exit(status.code().unwrap_or(1));
    }

    let args = inject_default_verbs(raw);

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let registry_mutex = ::clap_noun_verb::cli::CommandRegistry::get();
    let registry =
        registry_mutex.lock().map_err(|e| anyhow::anyhow!("Failed to lock registry: {}", e))?;
    registry.run(args).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Map bare nouns (no verb given) to sensible default verbs.
///
/// Examples:
/// - `tool status`   → `tool status show`
/// - `tool receipt`  → `tool receipt verify`
/// - `tool evidence` → `tool evidence doctor`
///
/// Nouns that need to bypass CliBuilder entirely should use the `run_direct()`
/// escape hatch before this function is called (or before `registry.route()`).
fn inject_default_verbs(mut args: Vec<String>) -> Vec<String> {
    let noun = args.get(1).cloned().unwrap_or_default();
    let has_verb = args.get(2).map(|a| !a.starts_with('-')).unwrap_or(false);
    if !has_verb {
        let default_verb = match noun.as_str() {
            "status" => Some("show"),
            "receipt" => Some("verify"),
            "evidence" => Some("doctor"),
            _ => None,
        };
        if let Some(verb) = default_verb {
            args.insert(2, verb.to_string());
        }
    }
    args
}
