//! Lane D — 10^7 members under audit discipline.
//!
//! The supra-cell is a roll-up of roll-ups: cells → supra mirrors
//! groups → cell. The claim under test: the supra receipt is verifiable
//! reading O(cells + groups), never O(members), with any single member
//! still challengeable by full deterministic replay.
//!
//! Default-run tests prove the supra-hash law at small n. The `#[ignore]`d
//! measurement climbs the ladder 10^5 → 10^6 → 10^7 under a pre-stated
//! 1800s wall budget and writes `receipts/scale_1e7.json`.

mod common;

use std::time::Instant;

use praxis_synthesis::cell::{
    challenge_member, run_cell, summarize_cell, supra_hash, verify_supra,
    CellSummary, MemberRecord,
};

/// splitmix64 — the crate's fault-injection mixer, reproduced locally
/// (it is `pub(crate)` in `cell_supervise`); deterministic, no `rand` dep.
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Run `cells` cells of `per_cell` members each, retaining only summaries
/// (and the one challenged member's record). Peak retention = one cell's
/// interiors + the summaries — no cells×per_cell structure ever exists.
fn run_rung(
    cells: usize,
    per_cell: usize,
    groups_per_cell: usize,
    templates: usize,
    challenged: usize,
) -> (Vec<CellSummary>, Option<MemberRecord>, u128) {
    let mut summaries = Vec::with_capacity(cells);
    let mut challenged_record = None;
    let rung_start = Instant::now();
    for ci in 0..cells {
        let cell_start = Instant::now();
        let (cell, groups) = run_cell(per_cell, groups_per_cell, templates);
        let elapsed = cell_start.elapsed().as_nanos();
        if challenged / per_cell == ci {
            let local = challenged % per_cell;
            let per_group = per_cell / groups_per_cell;
            challenged_record = Some(
                groups[local / per_group].members[local % per_group].clone(),
            );
        }
        summaries.push(summarize_cell(ci, elapsed, &cell, &groups));
        // groups (member interiors) dropped here — streaming at cell grain.
    }
    (summaries, challenged_record, rung_start.elapsed().as_nanos())
}

// ── default-run correctness tests ──────────────────────────────────────────

#[test]
fn supra_hash_binds_counts_and_cell_hashes() {
    let mut summaries = Vec::new();
    for ci in 0..4 {
        let (cell, groups) = run_cell(64, 4, 8);
        summaries.push(summarize_cell(ci, 1, &cell, &groups));
    }
    let supra = supra_hash(&summaries);
    assert!(verify_supra(&supra, &summaries));

    // Tamper a cell hash — the supra law catches it.
    let mut t1 = summaries.clone();
    t1[2].cell_hash = format!("{}0", &t1[2].cell_hash[..t1[2].cell_hash.len() - 1]);
    assert!(!verify_supra(&supra, &t1));

    // Tamper one group root — the level-up law catches it too.
    let mut t2 = summaries.clone();
    t2[1].group_roots[3] =
        format!("{}0", &t2[1].group_roots[3][..t2[1].group_roots[3].len() - 1]);
    assert!(!verify_supra(&supra, &t2));

    // Tamper the COUNTS (keep the invariant admitted+refused == n so the
    // internal consistency check alone would pass) — the count-bound
    // summary line breaks the supra. This was the adversarial-review
    // finding: counts must be hash-bound, not attested.
    let mut t3 = summaries.clone();
    t3[0].admitted += 1;
    t3[0].refused -= 1;
    assert!(!verify_supra(&supra, &t3));
}

#[test]
fn summaries_carry_no_members() {
    let (cell, groups) = run_cell(64, 4, 8);
    let s = summarize_cell(0, 1, &cell, &groups);
    let json = serde_json::to_string(&s).expect("summary renders");
    assert!(!json.contains("terminal_hash"));
    assert!(!json.contains("\"members\""));
    assert_eq!(s.admitted + s.refused, s.n);
    assert_eq!(s.group_roots.len(), 4);
}

#[test]
fn spot_challenge_replays_one_member() {
    let (_cell, groups) = run_cell(64, 4, 8);
    let record = groups[1].members[3].clone();
    assert!(challenge_member(&record, 8));
    let mut corrupt = record;
    corrupt.terminal_hash =
        format!("{}0", &corrupt.terminal_hash[..corrupt.terminal_hash.len() - 1]);
    assert!(!challenge_member(&corrupt, 8));
}

// ── the measurement ─────────────────────────────────────────────────────────

#[test]
#[ignore = "measurement run; execute with --ignored --release to regenerate receipts/scale_1e7.json"]
fn supra_scale_1e7_receipt() {
    // Criterion block built FIRST — before any run produces a number.
    const WALL_BUDGET_SECS: u128 = 1800;
    const CRITERION: &str = "verification cost is O(cells + groups), never \
        O(members): supra verified by refolding C cell hashes from C*G group \
        roots; per-cell latency verdict fields are p50/p99/worst \
        (nearest-rank, never min); throughput recomputed from (members, \
        elapsed) via stats::per_second; one member spot-challenged by full \
        deterministic replay; wall budget 1800s pre-stated — a rung that \
        would exceed it is refused, not run.";
    let per_cell = 10_000usize;
    let groups_per_cell = 100usize;
    let templates = 8usize;
    let ladder: [usize; 3] = [10, 100, 1000]; // cells → 10^5, 10^6, 10^7 members
    let total_target = ladder[ladder.len() - 1] * per_cell; // 10^7
    #[allow(clippy::cast_possible_truncation)]
    let challenged = (splitmix64(0xA11_D17) % total_target as u64) as usize;

    let mut rungs = Vec::new();
    let mut cumulative_elapsed_ns: u128 = 0;
    let mut refusal: Option<serde_json::Value> = None;
    let mut measured_ceiling: usize = 0;
    let mut spot: Option<(usize, MemberRecord)> = None; // (rung members, record)
    let mut throughputs: Vec<f64> = Vec::new();
    let mut anomaly = false;

    for (ri, &cells) in ladder.iter().enumerate() {
        let members = cells * per_cell;
        // Capture the challenged member within THIS rung's member space so a
        // budget ceiling never leaves the challenge without a record.
        let rung_challenged = challenged % members;
        let (summaries, record, rung_ns) =
            run_rung(cells, per_cell, groups_per_cell, templates, rung_challenged);
        cumulative_elapsed_ns += rung_ns;

        let supra = supra_hash(&summaries);
        let verified = verify_supra(&supra, &summaries);
        assert!(verified, "supra verification must recompute true");

        let mut lat: Vec<u128> = summaries.iter().map(|s| s.elapsed_ns).collect();
        let p50 = common::stats::percentile(&mut lat, 50.0);
        let p99 = common::stats::percentile(&mut lat, 99.0);
        let worst = common::stats::percentile(&mut lat, 100.0);
        let mps = common::stats::per_second(members, rung_ns);

        // Flattering-anomaly guard: superlinear speedup at larger scale is an
        // instrument bug, not a result.
        if let Some(prev) = throughputs.last() {
            if mps > prev * 2.0 {
                anomaly = true;
            }
        }
        throughputs.push(mps);

        let admitted: usize = summaries.iter().map(|s| s.admitted).sum();
        let refused: usize = summaries.iter().map(|s| s.refused).sum();
        rungs.push(serde_json::json!({
            "members": members,
            "cells": cells,
            "p50_ns": p50,
            "p99_ns": p99,
            "worst_ns": worst,
            "members_per_sec_recomputed": mps,
            "admitted": admitted,
            "refused": refused,
            "supra_hash": supra,
            "verify_supra_recomputed": verified,
            "verification_reads": {
                "cells": cells,
                "group_roots": cells * groups_per_cell,
                "member_records_read": 1,
            },
        }));
        spot = Some((members, record.expect("challenged member captured")));
        measured_ceiling = members;

        // Pre-stated wall budget: refuse the next rung if the projection
        // (each rung is 10x the last) would blow it. The honest ceiling,
        // not a silent downgrade.
        if ri + 1 < ladder.len() {
            let projected = cumulative_elapsed_ns + rung_ns * 10;
            if projected > WALL_BUDGET_SECS * 1_000_000_000 {
                refusal = Some(serde_json::json!({
                    "kind": "BudgetExceeded",
                    "what": format!(
                        "rung {} members refused: projected wall cost exceeds \
                         the pre-stated budget", ladder[ri + 1] * per_cell),
                    "budget_secs": WALL_BUDGET_SECS,
                    "spent_ns": cumulative_elapsed_ns,
                    "salvage": format!(
                        "measured ceiling {measured_ceiling} members with a \
                         verified supra; projected next-rung cost ~{} ns",
                        rung_ns * 10),
                }));
                break;
            }
        }
    }

    // Spot challenge: full deterministic replay of one member from the last
    // completed rung.
    let (spot_members, record) = spot.expect("at least one rung completed");
    let replayed_ok = challenge_member(&record, templates);
    assert!(replayed_ok, "spot challenge must replay true — else REFUTED");

    let verdict = if anomaly {
        "WITHHELD: self-refuting — a rung's members_per_sec exceeded the \
         previous rung's by >2x (superlinear speedup at larger scale is an \
         instrument bug); no verdict is offered"
            .to_string()
    } else if !replayed_ok {
        "REFUTED: spot-challenge replay diverged from the recorded member \
         projection"
            .to_string()
    } else if measured_ceiling >= total_target {
        format!(
            "SURVIVES: 10^7 members verified reading O(cells+groups) — \
             {} cell hashes refolded from {} group roots, 1 member record \
             read for the spot challenge; every rung's supra recomputed true",
            ladder[ladder.len() - 1],
            ladder[ladder.len() - 1] * groups_per_cell
        )
    } else {
        format!(
            "CEILING: measured ceiling {measured_ceiling} members — 10^7 \
             refused under the pre-stated {WALL_BUDGET_SECS}s wall budget \
             (refusal object records the projection)"
        )
    };

    let receipt = serde_json::json!({
        "what": "supra-cell scale: cells -> supra roll-up (mirroring groups \
                 -> cell), ladder 10^5 -> 10^6 -> 10^7 members, verification \
                 by hierarchical refold from summaries alone",
        "criterion": CRITERION,
        "wall_budget_secs": WALL_BUDGET_SECS,
        "cell_shape": { "per_cell": per_cell, "groups_per_cell": groups_per_cell,
                        "templates": templates },
        "rungs": rungs,
        "measured_ceiling_members": measured_ceiling,
        "refusal": refusal,
        "spot_challenge": {
            "challenged_global_index": challenged,
            "rung_members": spot_members,
            "agent": record.agent,
            "byte": record.byte,
            "terminal_hash_prefix": &record.terminal_hash[..16.min(record.terminal_hash.len())],
            "replayed_ok": replayed_ok,
        },
        "retention": "peak = one cell's 10_000 member records + C summaries; \
                      no 10^7-member structure ever exists",
        "streaming_note": "no streaming API change needed: run_cell retains \
                           one cell's interiors at a time and the harness \
                           drops groups after summarize_cell; a \
                           run_cell_streaming(sink) variant is deliberately \
                           not built — the current API already streams at \
                           cell granularity",
        "cumulative_elapsed_ns": cumulative_elapsed_ns,
        "self_refuting": anomaly,
        "verdict": verdict,
    });

    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../receipts");
    std::fs::create_dir_all(dir).expect("receipts dir");
    let path = format!("{dir}/scale_1e7.json");
    let pretty = serde_json::to_string_pretty(&receipt).expect("receipt renders");
    std::fs::write(&path, pretty).expect("receipt written");
    eprintln!("wrote {path}: {verdict}");
}
