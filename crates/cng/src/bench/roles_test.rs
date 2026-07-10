#![cfg(test)]

use std::fs;
use std::path::PathBuf;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use super::{metric_count, run_construct, select_rows};
use crate::bench::templates::QuerySet;

/// The Phase-0 fixture (one observation per kind) must satisfy every
/// CONSTRUCT + metric SELECT contract end to end.
#[test]
fn fixture_obs_materialize_and_count() {
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/bench-obs/sample-observations.ttl");
    let turtle = fs::read_to_string(&fixture).expect("fixture readable");
    let obs = Store::new().expect("store");
    obs.load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .expect("fixture parses");
    let evidence = Store::new().expect("store");
    for construct in [
        "ocel-events.construct",
        "ocel-objects.construct",
        "ocel-e2o.construct",
        "ocel-o2o-sockets.construct",
        "ocel-receipts.construct",
        "ocel-log.construct",
    ] {
        run_construct(&obs, queries.get(construct).expect("query"), &evidence)
            .expect("construct runs");
    }
    // Fixture: 1 worker, 3 workflow ids (wf-A, wf-B via socket, wf-C).
    let count = |name: &str| {
        metric_count(&evidence, queries.get(name).expect("query"), name).expect("count")
    };
    assert_eq!(count("metric-workers"), 1);
    assert_eq!(count("metric-recursive-attachments"), 1);
    assert_eq!(count("metric-receipts"), 1);
    assert_eq!(count("metric-refusals"), 1);
    assert_eq!(count("metric-conformance"), 1);
    assert_eq!(count("metric-replay"), 0);
    // attachments-with-parent runs over the OBS graph and keeps the
    // parentActivity binding.
    let rows =
        select_rows(&obs, queries.get("attachments-with-parent").expect("query")).expect("rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].get("parentActivity").map(String::as_str),
        Some("http://example.org/rwai#activity-step-1")
    );
}
