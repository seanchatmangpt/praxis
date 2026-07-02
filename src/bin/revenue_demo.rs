//! `revenue_demo` — the Genesis Day 2 Revenue Physics pipe, as one command.
//!
//! Runs the whole observation→proposal→plan→admission→receipt chain in-process
//! over the shared ops functions ([`my_conforming_project::revenue::run_demo`])
//! and prints the transcript as pretty JSON. Deterministic: a fixed `ts_ns`
//! makes the closing `chain_hash` stable, so this doubles as a live proof of
//! the same invariant `tests/revenue_pipe.rs` pins.
//!
//! Invoked by `just revenue-demo`.

#![allow(clippy::print_stdout)]

use std::process::exit;

/// Fixed receipt timestamp: keeps the demo's `chain_hash` stable run to run.
const DEMO_TS_NS: u64 = 1_751_328_000_000_000_000;

fn main() {
    match my_conforming_project::revenue::run_demo(DEMO_TS_NS) {
        Ok(transcript) => match serde_json::to_string_pretty(&transcript) {
            Ok(text) => {
                println!("{text}");
                exit(0);
            }
            Err(e) => {
                eprintln!("revenue-demo: failed to render transcript: {e}");
                exit(2);
            }
        },
        Err(e) => {
            eprintln!("revenue-demo: pipe refused (broken seam): {e}");
            exit(1);
        }
    }
}
