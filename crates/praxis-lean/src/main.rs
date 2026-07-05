//! Binary entrypoint for `praxis-l4`.
//!
//! Default entry uses this workspace's real `clap-noun-verb` registry
//! (matching `crates/ggen/src/main.rs`'s exact pattern). Build with
//! `--features standalone-cli --no-default-features` for a lighter-weight
//! plain-`clap` entry instead.

// Link the library crate so its `#[verb]`-registered verbs (via the
// `verbs` module) are included in the binary.
use praxis_lean as _;

#[cfg(feature = "standalone-cli")]
fn main() -> anyhow::Result<()> {
    use clap::Parser;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let cli = praxis_lean::cli::Cli::parse();
    praxis_lean::cli::run_cli(cli)
}

#[cfg(not(feature = "standalone-cli"))]
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    ::clap_noun_verb::cli::CommandRegistry::set_app_metadata(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );

    let registry_mutex = ::clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry_mutex
        .lock()
        .map_err(|e| anyhow::anyhow!("failed to lock registry: {e}"))?;
    registry.run(args).map_err(|e| anyhow::anyhow!("{e}"))
}
