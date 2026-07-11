//! Human-readable pretty-printers for the dense JSON reports emitted by
//! `cng benchmark workday` and `cng engine serve`/`resume` (PROJ-dx-pretty-
//! report). The JSON reports (`WorkdayReport`, `EngineServeReport`, and
//! `DecomposeReport` in `main.rs`) are the machine surface — great for
//! `jq`, painful for a human staring at a failed run. `--pretty` (wired in
//! `main.rs`) renders one of these strings INSTEAD of the JSON dump, never
//! alongside it.
//!
//! `DecomposeReport` itself is declared in the `cng` binary crate
//! (`main.rs`), not this library crate, so its renderer lives next to it
//! there and reuses the [`use_color`]/[`paint`]/[`short_digest`] helpers
//! exported from this module — this module cannot depend on the binary
//! crate's types, only the reverse.
//!
//! No new terminal-UI dependency: plain ANSI SGR codes, gated by
//! [`use_color`] (TTY check + `NO_COLOR`, https://no-color.org/).

use std::io::IsTerminal;

use super::engine::EngineServeReport;
use super::workday::WorkdayReport;

/// ANSI SGR codes used by the pretty renderers. [`paint`] always closes
/// with a reset code, so a color never bleeds into later output.
#[derive(Debug, Clone, Copy)]
pub enum Ansi {
    Bold,
    Dim,
    Green,
    Red,
    BoldRed,
    Yellow,
    Cyan,
}

impl Ansi {
    fn code(self) -> &'static str {
        match self {
            Ansi::Bold => "\x1b[1m",
            Ansi::Dim => "\x1b[2m",
            Ansi::Green => "\x1b[32m",
            Ansi::Red => "\x1b[31m",
            Ansi::BoldRed => "\x1b[1;31m",
            Ansi::Yellow => "\x1b[33m",
            Ansi::Cyan => "\x1b[36m",
        }
    }
}

/// Whether ANSI color should be emitted: stdout is a TTY and `NO_COLOR`
/// (https://no-color.org/) is unset. Re-checked on every render call (one
/// env lookup + one `isatty`) rather than cached, so output piped into a
/// file or `jq` — even mid-session — degrades to plain text.
pub fn use_color() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
}

/// Wraps `text` in `code`'s SGR escape + reset when `color_enabled`;
/// returns `text` unchanged otherwise.
pub fn paint(text: &str, code: Ansi, color_enabled: bool) -> String {
    if color_enabled {
        format!("{}{text}\x1b[0m", code.code())
    } else {
        text.to_string()
    }
}

/// Shortens a long digest (`blake3:<64-hex>` or any other long token) to
/// its first 8 characters plus a note of the omitted length, so a human
/// scanning a report is not stopped cold by a 64-hex-char BLAKE3 string.
/// The full value is always still available via the JSON report
/// (`--format json`, i.e. without `--pretty`).
pub fn short_digest(full: &str) -> String {
    const HEAD: usize = 8;
    const MIN_TRUNCATE_LEN: usize = HEAD + 8;
    match full.split_once(':') {
        Some((scheme, rest)) if rest.len() > MIN_TRUNCATE_LEN => {
            format!(
                "{scheme}:{}\u{2026} ({}-char digest, truncated — see --format json)",
                &rest[..HEAD],
                rest.len()
            )
        }
        _ if full.len() > MIN_TRUNCATE_LEN => {
            format!(
                "{}\u{2026} ({}-char digest, truncated — see --format json)",
                &full[..HEAD],
                full.len()
            )
        }
        _ => full.to_string(),
    }
}

/// Renders one SPARQL-derived success marker as a green check (true) or a
/// bold-red cross with the marker name itself highlighted (false). A false
/// marker cannot actually reach this renderer today — `CNG_R20
/// MarkerFalse` refuses before a `WorkdayReport` exists — but the renderer
/// still handles it honestly rather than assuming.
fn marker_line(name: &str, value: bool, color: bool) -> String {
    if value {
        format!("  {} {name}", paint("\u{2714}", Ansi::Green, color))
    } else {
        format!(
            "  {} {}",
            paint("\u{2718}", Ansi::BoldRed, color),
            paint(name, Ansi::BoldRed, color)
        )
    }
}

/// One row of the graph-derived-vs-telemetry count comparison table.
/// `telemetry` is `None` when the field has no in-process twin (the graph
/// SELECT is the only source for that count).
struct CountRow {
    label: &'static str,
    graph: u64,
    telemetry: Option<u64>,
}

/// Renders an aligned `label  graph  telemetry` table. Reconciliation
/// between the two columns is already enforced by a typed refusal before
/// any report reaches this renderer (the graph is the authority; see
/// `workday.rs`/`engine.rs` module docs) — a visible `MISMATCH` tag is
/// defensive, not expected to ever fire on a report that got this far.
///
/// # Complexity
/// O(rows) — one pass to compute column widths, one pass to render.
fn render_count_table(rows: &[CountRow], color: bool) -> String {
    let label_w = rows.iter().map(|r| r.label.len()).max().unwrap_or(6).max(6);
    let graph_w = rows
        .iter()
        .map(|r| r.graph.to_string().len())
        .max()
        .unwrap_or(5)
        .max(5);
    let mut out = String::new();
    out.push_str(&format!(
        "  {:<label_w$}  {:>graph_w$}  telemetry\n",
        "metric",
        "graph",
        label_w = label_w,
        graph_w = graph_w
    ));
    for row in rows {
        let telemetry_str = match row.telemetry {
            Some(t) => t.to_string(),
            None => "-".to_string(),
        };
        let line = format!(
            "  {:<label_w$}  {:>graph_w$}  {telemetry_str}",
            row.label,
            row.graph,
            label_w = label_w,
            graph_w = graph_w
        );
        let mismatch = matches!(row.telemetry, Some(t) if t != row.graph);
        if mismatch {
            out.push_str(&paint(&format!("{line}  MISMATCH"), Ansi::BoldRed, color));
        } else {
            out.push_str(&line);
        }
        out.push('\n');
    }
    out
}

/// Renders a [`WorkdayReport`] as a human-readable summary: a marker
/// pass/fail block, an aligned graph-vs-telemetry count table, the
/// dispatch-closure facet counts, and short-hashed digests. Called only
/// when `--pretty` is passed to `cng benchmark workday` (see `main.rs`);
/// otherwise the report is JSON, as always.
///
/// # Complexity
/// O(markers + count rows + dispatch_closure entries) — all bounded,
/// small collections (17 markers, ~20 count rows).
pub fn render_workday_report_human(report: &WorkdayReport) -> String {
    let color = use_color();
    let mut out = String::new();

    out.push_str(&paint(
        &format!(
            "== cng benchmark workday :: {} ==\n",
            report.measurement_class
        ),
        Ansi::Bold,
        color,
    ));
    out.push_str(&format!("  out_dir : {}\n", report.out_dir));
    out.push_str(&format!(
        "  seed={}  ticks={}\n\n",
        report.seed, report.ticks
    ));

    out.push_str(&paint("MARKERS\n", Ansi::Bold, color));
    let true_count = report.markers.values().filter(|v| **v).count();
    let total = report.markers.len();
    for (name, value) in &report.markers {
        out.push_str(&marker_line(name, *value, color));
        out.push('\n');
    }
    let verdict = if true_count == total && total > 0 {
        paint("PASS", Ansi::Green, color)
    } else {
        paint("FAIL", Ansi::BoldRed, color)
    };
    out.push_str(&format!(
        "  {verdict} ({true_count}/{total} markers true)\n\n"
    ));

    out.push_str(&paint(
        "COUNTS (graph-derived / telemetry)\n",
        Ansi::Bold,
        color,
    ));
    let rows = [
        CountRow {
            label: "workers_represented",
            graph: report.workers_represented,
            telemetry: None,
        },
        CountRow {
            label: "workflow_instances",
            graph: report.workflow_instances,
            telemetry: None,
        },
        CountRow {
            label: "executed_transitions",
            graph: report.executed_transitions,
            telemetry: Some(report.telemetry_transitions as u64),
        },
        CountRow {
            label: "refusals",
            graph: report.refusals,
            telemetry: Some(report.telemetry_refusals as u64),
        },
        CountRow {
            label: "receipts",
            graph: report.receipts,
            telemetry: None,
        },
        CountRow {
            label: "admission_requests",
            graph: report.admission_requests,
            telemetry: None,
        },
        CountRow {
            label: "admissions_granted",
            graph: report.admissions_granted,
            telemetry: None,
        },
        CountRow {
            label: "resumes",
            graph: report.resumes,
            telemetry: None,
        },
        CountRow {
            label: "replay_verified",
            graph: report.replay_verified,
            telemetry: None,
        },
        CountRow {
            label: "hook_receipts",
            graph: report.hook_receipts,
            telemetry: Some(report.telemetry_hook_actuations as u64),
        },
        CountRow {
            label: "dispatches_sent",
            graph: report.dispatches_sent,
            telemetry: Some(report.telemetry_dispatches_sent as u64),
        },
        CountRow {
            label: "consequences_admitted",
            graph: report.consequences_admitted,
            telemetry: Some(report.telemetry_consequences_admitted as u64),
        },
        CountRow {
            label: "consequences_refused",
            graph: report.consequences_refused,
            telemetry: None,
        },
        CountRow {
            label: "dispatch_timeouts",
            graph: report.dispatch_timeouts,
            telemetry: None,
        },
        CountRow {
            label: "remediations",
            graph: report.remediations,
            telemetry: None,
        },
        CountRow {
            label: "engine_instances",
            graph: report.engine_instances,
            telemetry: None,
        },
        CountRow {
            label: "remote_dispatches",
            graph: report.remote_dispatches,
            telemetry: Some(report.telemetry_remote_dispatches as u64),
        },
        CountRow {
            label: "remote_consequences_received",
            graph: report.remote_consequences_received,
            telemetry: Some(report.telemetry_remote_consequences_received as u64),
        },
        CountRow {
            label: "arazzo_workflows_generated",
            graph: report.arazzo_workflows_generated,
            telemetry: Some(report.telemetry_arazzo_generated as u64),
        },
        CountRow {
            label: "arazzo_workflows_dispatched",
            graph: report.arazzo_workflows_dispatched,
            telemetry: Some(report.telemetry_arazzo_dispatched as u64),
        },
    ];
    out.push_str(&render_count_table(&rows, color));
    out.push('\n');

    if !report.dispatch_closure.is_empty() {
        out.push_str(&paint("DISPATCH CLOSURE\n", Ansi::Bold, color));
        let width = report
            .dispatch_closure
            .keys()
            .map(|k| k.len())
            .max()
            .unwrap_or(0);
        for (facet, count) in &report.dispatch_closure {
            out.push_str(&format!("  {facet:<width$} : {count}\n"));
        }
        out.push('\n');
    }

    out.push_str(&paint("DIGESTS\n", Ansi::Bold, color));
    out.push_str(&format!(
        "  evidence_chain_digest : {}\n",
        short_digest(&report.evidence_chain_digest)
    ));
    out.push_str(&format!(
        "  ocel_graph_digest     : {}\n",
        short_digest(&report.ocel_graph_digest)
    ));
    out.push_str(&format!(
        "  obs_digest            : {}\n",
        short_digest(&report.obs_digest)
    ));
    out.push_str(&format!(
        "  run_hook_hash         : {}\n",
        short_digest(&report.run_hook_hash)
    ));

    out
}

/// Renders an [`EngineServeReport`] as a human-readable summary of one
/// `cng engine serve`/`resume` pass. `resumed`/`quiesced` are status
/// badges, not pass/fail markers — a poll-budget-exhausted engine
/// (`quiesced: false`) is an honest partial, not a failure (see
/// `engine.rs`'s `EngineServeReport` doc comment).
///
/// # Complexity
/// O(1) — fixed field count.
pub fn render_engine_serve_report_human(report: &EngineServeReport) -> String {
    let color = use_color();
    let mut out = String::new();

    out.push_str(&paint(
        &format!("== cng engine :: {} ==\n", report.measurement_class),
        Ansi::Bold,
        color,
    ));
    out.push_str(&format!("  engine_id      : {}\n", report.engine_id));
    out.push_str(&format!("  engine_version : {}\n", report.engine_version));
    out.push_str(&format!("  instance_nonce : {}\n\n", report.instance_nonce));

    out.push_str(&paint("STATUS\n", Ansi::Bold, color));
    out.push_str(&format!(
        "  resumed  : {}\n",
        if report.resumed {
            paint(
                &format!(
                    "yes ({} ledger entries verified)",
                    report.ledger_entries_verified
                ),
                Ansi::Cyan,
                color,
            )
        } else {
            "no (fresh serve)".to_string()
        }
    ));
    out.push_str(&format!(
        "  quiesced : {}\n\n",
        if report.quiesced {
            paint(
                "\u{2714} clean quiesce (control/quiesce.ttl honored)",
                Ansi::Green,
                color,
            )
        } else {
            paint(
                "\u{2049} poll budget exhausted (not an error — honest partial)",
                Ansi::Yellow,
                color,
            )
        }
    ));

    out.push_str(&paint("COUNTS\n", Ansi::Bold, color));
    let rows = [
        CountRow {
            label: "polls",
            graph: report.polls,
            telemetry: None,
        },
        CountRow {
            label: "contracts_executed",
            graph: report.contracts_executed as u64,
            telemetry: None,
        },
        CountRow {
            label: "ledger_entries_verified",
            graph: report.ledger_entries_verified,
            telemetry: None,
        },
    ];
    out.push_str(&render_count_table(&rows, color));
    out.push('\n');

    out.push_str(&paint("DIGESTS\n", Ansi::Bold, color));
    out.push_str(&format!(
        "  receipt_chain_digest : {}\n",
        short_digest(&report.receipt_chain_digest)
    ));

    out
}

#[cfg(test)]
mod tests {
    #![cfg(test)]

    use chicago_tdd_tools::prelude::*;

    use std::fs;
    use std::path::PathBuf;

    use super::{render_workday_report_human, short_digest};
    use crate::bench::{workday, WorkdayConfig};

    fn scratch_dir(test_name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/chatman/cng-tests/report_pretty")
            .join(test_name)
    }

    // PROJ-dx-pretty-report: renders a REAL `workday()` report (not a
    // hand-built fixture) and asserts the human-readable form carries the
    // marker names, a PASS/FAIL-style verdict token, an aligned count
    // table, and a truncated (not full 64-hex) digest.
    test!(render_workday_report_human_is_sensible_on_a_real_run, {
        // Arrange: a small, real, healthy workday run (no injected
        // refusals so every marker is expected true).
        let cfg = WorkdayConfig {
            seed: 42,
            ticks: 4,
            refusal_per_mille: 0,
        };
        let dir = scratch_dir("pretty_smoke");
        let _ = fs::remove_dir_all(&dir);

        // Act
        let report = workday(&dir, &cfg, None).expect("real workday run");
        let rendered = render_workday_report_human(&report);

        // Assert: non-empty, and carries the substance a human would look
        // for — not just a re-serialization of the JSON.
        assert!(!rendered.is_empty());
        assert!(rendered.contains("cng benchmark workday"));
        assert!(rendered.contains("MARKERS"));
        // At least one real marker name from the map appears verbatim —
        // `evaluate_marker_map` keys `WorkdayReport.markers` by the
        // ALL_CAPS proof names (e.g. `HOOK_ACTUATION_PROVEN`), not the
        // dash-separated query stems (`marker-hook-actuation`); the
        // conjunction key is always present when the run is Ok().
        assert!(rendered.contains("V26_7_10_PRODUCTION_READY"));
        assert!(rendered.contains("HOOK_ACTUATION_PROVEN"));
        assert!(rendered.contains("PASS") || rendered.contains("FAIL"));
        assert!(rendered.contains("COUNTS"));
        assert!(rendered.contains("executed_transitions"));
        assert!(rendered.contains("DIGESTS"));
        // The digest is truncated, not the raw 64-hex BLAKE3 string.
        assert!(rendered.contains("blake3:"));
        assert!(!rendered.contains(&report.evidence_chain_digest));
        // A healthy zero-refusal seeded run passes every marker.
        assert!(report.markers.values().all(|v| *v));
        assert!(rendered.contains("PASS"));
    });

    test!(
        short_digest_truncates_long_hex_and_passes_through_short_strings,
        {
            let long = format!("blake3:{}", "a".repeat(64));
            let shortened = short_digest(&long);
            assert!(shortened.len() < long.len());
            assert!(shortened.starts_with("blake3:aaaaaaaa"));
            assert!(shortened.contains("truncated"));

            let short = "blake3:deadbeef";
            assert_eq!(short_digest(short), short);
        }
    );
}
