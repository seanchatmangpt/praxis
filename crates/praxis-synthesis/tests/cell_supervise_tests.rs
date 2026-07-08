//! Supervised-cell tests: combo-lane legality, recovery/park accounting,
//! MAPE-K quarantine by epoch 2, seed-deterministic replay, and the
//! foreign verifier passing UNTOUCHED on supervised receipts.

use std::collections::BTreeSet;
use std::path::PathBuf;

use praxis_synthesis::cell::{verify_cell, verify_group};
use praxis_synthesis::cell_supervise::{
    run_cell_supervised, run_member_supervised, FaultScript, PlanAction, SupervisorPolicy,
};
use praxis_synthesis::dag::MemoCache;
use praxis_synthesis::fleet::lane;
use praxis_synthesis::solver8::CoreCache;

fn script(seed: u64, transient: u16, crashloop: Option<usize>) -> FaultScript {
    FaultScript {
        seed,
        transient_per_mille: transient,
        crashloop_template: crashloop,
    }
}

#[test]
fn combo_lanes_are_unreachable_by_the_unsupervised_run() {
    // Exhaustive legality: enumerate every byte the BASE run_member can emit
    // over all template classes; assert none matches a supervision combo.
    let mut memo = MemoCache::new();
    let mut cores = CoreCache::new();
    for agent in 0..16 {
        let m = praxis_synthesis::cell::run_member(agent, 8, &mut memo, &mut cores);
        assert_ne!(
            m.byte & lane::S_RECOVERED,
            lane::S_RECOVERED,
            "H|A must be unreachable unsupervised: agent {agent} byte {:#010b}",
            m.byte
        );
        assert_ne!(
            m.byte & lane::S_GEOMETRY_GAP,
            lane::S_GEOMETRY_GAP,
            "H|U|E must be unreachable unsupervised: {:#010b}",
            m.byte
        );
    }
}

#[test]
fn recovered_members_carry_the_combo_and_count_inside_admitted() {
    let (cell, groups, _) = run_cell_supervised(
        400,
        4,
        8,
        1,
        SupervisorPolicy::default(),
        script(42, 200, None), // ~20% transient faults
    );
    assert_eq!(cell.admitted + cell.refused, 400, "the invariant holds");
    assert!(cell.recovered > 0, "the lottery injected transients");
    // Recovered members are admitted members with the H flag retained.
    let recovered_member = groups
        .iter()
        .flat_map(|g| &g.members)
        .find(|m| m.byte & lane::S_RECOVERED == lane::S_RECOVERED)
        .expect("a recovered member exists");
    assert!(recovered_member.byte & lane::A_ADMITTED != 0);
    assert_eq!(recovered_member.restarts, 1);
    // agent8 compatibility: a recovered member still sweeps as admitted.
    assert!(recovered_member.byte & lane::A_ADMITTED == lane::A_ADMITTED);
}

#[test]
fn crashloop_template_is_quarantined_by_epoch_two() {
    // Template 1 crash-loops everywhere. Epoch 0 witnesses it in >= 3
    // groups; the receipted plan quarantines it; epoch 1 members of t1
    // refuse cheaply with the register head "template quarantined".
    let (cell, groups, plans) = run_cell_supervised(
        400,
        4,
        8,
        2,
        SupervisorPolicy::default(),
        script(7, 0, Some(1)),
    );
    // Epoch 0's plan carries the quarantine action with quorum witnesses.
    let PlanAction::QuarantineTemplate {
        template,
        witness_groups,
    } = &plans[0].actions[0];
    assert_eq!(*template, 1);
    assert!(*witness_groups >= 3, "cross-group quorum: {witness_groups}");
    // The cell receipt records it.
    assert_eq!(cell.quarantined_templates, vec![1]);
    assert_eq!(
        cell.epoch_plan_hashes.len(),
        2,
        "plan chain has one link per epoch"
    );
    // Final epoch: quarantine visible in the EXISTING refusal register via
    // the existing ':'-head bucketing — zero register changes.
    assert!(
        cell.refusal_register
            .keys()
            .any(|k| k == "template quarantined"),
        "register: {:?}",
        cell.refusal_register
    );
    // And no member of t1 parked in the final epoch (they refused instead).
    let final_t1_parks = groups
        .iter()
        .flat_map(|g| &g.members)
        .filter(|m| m.refusal.starts_with("crash loop: template t1"))
        .count();
    assert_eq!(final_t1_parks, 0, "quarantine replaced the crash-looping");
}

#[test]
fn parked_members_count_inside_refused_with_first_class_receipts() {
    let (cell, groups, _) = run_cell_supervised(
        400,
        4,
        8,
        1,
        SupervisorPolicy::default(),
        script(7, 0, Some(1)),
    );
    assert!(cell.parked > 0);
    assert_eq!(
        cell.admitted + cell.refused,
        400,
        "parks live inside refused"
    );
    let parked = groups
        .iter()
        .flat_map(|g| &g.members)
        .find(|m| m.byte == lane::S_PARKED)
        .expect("a parked member");
    assert!(parked.refusal.starts_with("crash loop"));
    assert_eq!(parked.restarts, SupervisorPolicy::default().max_restarts);
}

#[test]
fn supervised_runs_replay_byte_exactly_with_the_seed() {
    let quarantine = BTreeSet::new();
    let policy = SupervisorPolicy::default();
    let s = script(1234, 300, None);
    let run = |agent: usize| {
        let mut memo = MemoCache::new();
        let mut cores = CoreCache::new();
        run_member_supervised(agent, 8, &quarantine, policy, s, &mut memo, &mut cores)
    };
    for agent in 0..24 {
        let a = run(agent);
        let b = run(agent);
        assert_eq!(a.byte, b.byte, "agent {agent}");
        assert_eq!(a.terminal_hash, b.terminal_hash);
        assert_eq!(a.restarts, b.restarts);
    }
}

#[test]
fn cell_verification_and_the_untouched_foreign_verifier_still_pass() {
    let (cell, groups, _) = run_cell_supervised(
        400,
        4,
        8,
        2,
        SupervisorPolicy::default(),
        script(42, 150, Some(1)),
    );
    // Rust-side verification: byte-identical fold functions.
    assert!(verify_cell(&cell, &groups));
    assert!(groups.iter().all(verify_group));

    // Foreign verifier — the script is NOT modified for supervision; the
    // additive fields must be invisible to it.
    let b3 = std::process::Command::new("b3sum")
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !b3 {
        eprintln!("SKIPPED (deferred): b3sum not on PATH");
        return;
    }
    let dir: PathBuf = std::env::temp_dir();
    let cell_path = dir.join(format!("sup-cell-{}.json", std::process::id()));
    let groups_path = dir.join(format!("sup-groups-{}.json", std::process::id()));
    std::fs::write(&cell_path, serde_json::to_string(&cell).expect("json")).expect("write");
    std::fs::write(&groups_path, serde_json::to_string(&groups).expect("json")).expect("write");
    let script_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/foreign_verify.py"
    );
    let out = std::process::Command::new("python3")
        .args([
            script_path,
            "cell",
            cell_path.to_str().expect("utf8"),
            groups_path.to_str().expect("utf8"),
        ])
        .output()
        .expect("run foreign verifier");
    assert!(
        out.status.success(),
        "foreign verifier must pass UNTOUCHED on supervised receipts: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let _ = std::fs::remove_file(&cell_path);
    let _ = std::fs::remove_file(&groups_path);
}
