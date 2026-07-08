//! Standalone benchmark (`harness = false`): sweep 10_000_000 simulated agents
//! (1.25M packed words) and record the wall-clock time.
//!
//! Runs with `cargo bench -p agent8`. No criterion dependency — a plain timing
//! `main` keeps the crate light and sidesteps the workspace `panic = "abort"`
//! release profile. Target: single-digit milliseconds.

use agent8::{AgentByte, Fleet};
use std::hint::black_box;
use std::time::Instant;

const AGENTS: usize = 10_000_000;

fn main() {
    // Deterministic LCG fill so lanes span the full byte space.
    let words = AGENTS / agent8::LANES_PER_WORD;
    let mut fleet = Fleet {
        bytes: vec![0u64; words],
    };
    let mut state: u64 = 0x0f0f_0f0f_0f0f_0f0f;
    for w in fleet.bytes.iter_mut() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *w = state;
    }

    // Warm, then time the sweep.
    let _ = black_box(fleet.sweep_stats(AgentByte::GRANT_REQUIRED));
    let start = Instant::now();
    let stats = black_box(fleet.sweep_stats(AgentByte::GRANT_REQUIRED));
    let elapsed = start.elapsed();

    let ms = elapsed.as_secs_f64() * 1e3;
    println!(
        "agent8 fleet sweep: {AGENTS} agents ({words} words) in {ms:.3} ms  \
         (admitted={}, blocked={}, receipted={}, replayable={})",
        stats.admitted, stats.blocked, stats.receipted, stats.replayable
    );
    println!(
        "throughput: {:.2} G agents/s",
        AGENTS as f64 / elapsed.as_secs_f64() / 1e9
    );
    assert_eq!(stats.total, AGENTS as u64);
    assert_eq!(stats.admitted + stats.blocked, AGENTS as u64);
}
