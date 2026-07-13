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

/// v26.7.12/13 Stage 2: the 5 SOC2 audit-engagement standing roles (control-
/// owner, internal-audit-lead, compliance-program-manager, remediation-
/// engineer, evidence-custodian) agree between the two independent
/// inference layers this crate already runs side by side for every other
/// role — Mycin forward-chaining (`infer_soc2_standing_role`, certainty-
/// factor rules) and the real praxis-graphlaw Datalog engine
/// (`derive_roles_datalog` over `rules/bench-roles.dl`). This EXTENDS the
/// existing agreement mechanism `derive_roles_datalog` already performs for
/// the 5 bench-roster roles (a Datalog-derived role that contradicts the
/// declared role is `CngRefusal::HardcodingSuspicion`) to the 5 new SOC2
/// roles, and additionally asserts full TEXT parity between Mycin's
/// `next=<action>` conclusion and Datalog's `:obligation` atom for each —
/// not merely that a role identity round-trips.
#[test]
fn soc2_standing_roles_mycin_and_datalog_agree() {
    use super::{derive_roles_datalog, infer_soc2_standing_role, RosterWorker};

    let rules_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("rules")
        .join("bench-roles.dl");
    let rules_text = fs::read_to_string(&rules_path).expect("bench-roles.dl reads");

    // (responsibility, expected standing role, expected lawful next action)
    let cases = [
        (
            "control-design-and-evidence",
            "control-owner",
            "document-control-design-and-attach-evidence",
        ),
        (
            "readiness-and-oe-testing",
            "internal-audit-lead",
            "execute-readiness-assessment-and-oe-testing",
        ),
        (
            "scoping-and-bundle-coordination",
            "compliance-program-manager",
            "coordinate-scope-and-assemble-evidence-bundle",
        ),
        (
            "exception-remediation",
            "remediation-engineer",
            "implement-remediation-for-identified-exception",
        ),
        (
            "evidence-chain-of-custody",
            "evidence-custodian",
            "maintain-evidence-chain-of-custody",
        ),
    ];

    // Mycin side: each responsibility infers the expected terminal
    // `next=<action>` conclusion.
    for (responsibility, _role, action) in cases {
        let mycin = infer_soc2_standing_role(responsibility);
        assert_eq!(
            mycin.as_deref(),
            Some(format!("next={action}").as_str()),
            "Mycin must infer next={action} for responsibility={responsibility}"
        );
    }

    // Datalog side: a small roster of 5 "workers" declared with the SOC2
    // standing-role names Mycin above just derived. The SAME real Datalog
    // engine and the SAME identity+obligation rules the 5-role bench roster
    // already exercises — no new parity mechanism.
    let workers: Vec<RosterWorker> = cases
        .iter()
        .enumerate()
        .map(|(i, (_, role, _))| RosterWorker {
            worker_id: format!("soc2-w{i}"),
            role: (*role).to_string(),
            department: "solace-cloud-audit".to_string(),
        })
        .collect();
    let datalog = derive_roles_datalog(&workers, &rules_text)
        .expect("Datalog derives the 5 SOC2 roles without contradicting the declared roster");
    assert_eq!(datalog.derived.len(), 5, "one derivedRole fact per worker");

    for (i, (_responsibility, role, action)) in cases.iter().enumerate() {
        let worker_id = format!("soc2-w{i}");
        assert_eq!(
            datalog.derived.get(&worker_id).map(String::as_str),
            Some(*role),
            "Datalog-derived role must equal the declared SOC2 standing role"
        );
        assert_eq!(
            datalog.obligations.get(&worker_id).map(String::as_str),
            Some(*action),
            "Datalog :obligation for {role} must equal Mycin's next=<action> text exactly \
             (parity: bench/roles.rs::soc2_role_rules vs rules/bench-roles.dl)"
        );
    }
}
