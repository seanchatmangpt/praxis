//! The minimum phase-change cell, tested: durable replay (kill -9), the
//! 10,000-agent cell with roll-up receipts, selective challenge, and the
//! foreign verifier (a second implementation in another language agreeing
//! on the hashes).

mod common;

use std::path::PathBuf;

use common::lawobject_domain;
use praxis_synthesis::cell::{challenge_member, run_cell, verify_cell, verify_group};
use praxis_synthesis::dag::MemoCache;
use praxis_synthesis::fleet::lane;
use praxis_synthesis::wal::Wal;
use praxis_synthesis::dag::DagNode;
use praxis_synthesis::{
    BoundedCsp, Dag, HashRunner, NodeRunner, SequenceProblem, Solver,
};

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "praxis-synth-cell-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

fn lawobject_dag() -> (Dag, praxis_synthesis::SequencePlan, SequenceProblem) {
    let (mut p, caps, goal) = lawobject_domain();
    p.saturate().expect("saturation");
    let problem = SequenceProblem::new(&p, caps, goal, 6, Vec::new()).expect("problem");
    let plan = BoundedCsp.solve(&problem).expect("solvable");
    let dag = Dag::from_plan(&plan, &problem);
    (dag, plan, problem)
}

// ── Step 2: durability ──────────────────────────────────────────────────────

/// A runner that sleeps per node — gives the parent a window to kill the
/// child mid-DAG.
struct SlowRunner;
impl NodeRunner for SlowRunner {
    fn run(&mut self, node: &DagNode, inputs: &[Vec<u8>]) -> Vec<u8> {
        std::thread::sleep(std::time::Duration::from_millis(400));
        HashRunner.run(node, inputs)
    }
}

/// Child-process worker for the kill -9 test. Runs only when the env var is
/// set (the parent spawns this same test binary with it).
#[test]
fn wal_child_worker() {
    let Ok(wal_path) = std::env::var("SYNTH_WAL_CHILD_PATH") else {
        return; // normal test runs: no-op
    };
    let (dag, _, _) = lawobject_dag();
    let mut wal = Wal::open(std::path::Path::new(&wal_path)).expect("wal");
    let mut cache = MemoCache::new();
    // Slow execution: the parent will SIGKILL us partway through.
    let _ = dag.execute_journaled(&mut SlowRunner, &mut cache, &mut wal);
}

#[test]
fn kill_dash_nine_mid_dag_then_replay_yields_the_identical_receipt() {
    // Clean reference run (fast runner, no WAL).
    let (dag, _, _) = lawobject_dag();
    let clean = dag
        .execute(&mut HashRunner, &mut MemoCache::new())
        .expect("clean run");

    // Spawn THIS test binary as a child running the worker, and kill -9 it
    // mid-DAG (5 nodes x 400ms = 2s of work; kill at 1.2s → ~2-3 nodes
    // journaled even after child-process startup overhead).
    let wal_path = temp_path("kill9.wal");
    let exe = std::env::current_exe().expect("test binary path");
    let mut child = std::process::Command::new(exe)
        .args(["wal_child_worker", "--exact", "--nocapture"])
        .env("SYNTH_WAL_CHILD_PATH", &wal_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn child");
    std::thread::sleep(std::time::Duration::from_millis(1200));
    child.kill().expect("SIGKILL"); // Child::kill is SIGKILL on unix
    let _ = child.wait();

    // Recover: rehydrate the memo cache from the surviving WAL frames.
    let (mut cache, frames, _torn) = Wal::recover(&wal_path).expect("recover");
    assert!(frames > 0, "the child journaled at least one node before dying");
    assert!(
        frames < dag.nodes.len(),
        "the kill landed mid-DAG ({frames}/{} nodes) — else the test proved nothing",
        dag.nodes.len()
    );

    // Resume: same deterministic execution, journaled nodes replay, the
    // rest recompute. The receipt must be byte-identical to the clean run.
    let mut wal = Wal::open(&wal_path).expect("reopen wal");
    let resumed = dag
        .execute_journaled(&mut HashRunner, &mut cache, &mut wal)
        .expect("resumed run");
    assert_eq!(resumed.root_hash, clean.root_hash, "the chain survived machine death");
    assert_eq!(
        resumed.node_receipts.last().map(|n| &n.chain),
        clean.node_receipts.last().map(|n| &n.chain)
    );
    assert_eq!(resumed.replayed_count, frames, "exactly the journaled nodes replayed");
    let _ = std::fs::remove_file(&wal_path);
}

#[test]
fn torn_tail_is_detected_and_dropped() {
    let wal_path = temp_path("torn.wal");
    {
        let mut wal = Wal::open(&wal_path).expect("wal");
        wal.append("key-1", b"output-1").expect("frame 1");
        wal.append("key-2", b"output-2").expect("frame 2");
    }
    // Simulate a crash mid-write: truncate the file into frame 2.
    let len = std::fs::metadata(&wal_path).expect("meta").len();
    let f = std::fs::OpenOptions::new().write(true).open(&wal_path).expect("open");
    f.set_len(len - 5).expect("truncate");
    drop(f);
    let (cache, frames, torn) = Wal::recover(&wal_path).expect("recover");
    assert_eq!(frames, 1, "only the intact frame survives");
    assert!(torn, "the torn tail is reported, not trusted");
    assert_eq!(cache.len(), 1);
    let _ = std::fs::remove_file(&wal_path);
}

// ── Step 4: the cell ────────────────────────────────────────────────────────

#[test]
fn the_cell_composes_and_verifies_from_rollups_alone() {
    // The full minimum cell: 10,000 agents, 100 groups, 100 per group.
    let (cell, groups) = run_cell(10_000, 100, 8);
    assert_eq!(cell.n, 10_000);
    assert_eq!(cell.g, 100);
    assert_eq!(cell.admitted + cell.refused, 10_000);
    assert!(cell.refused > 0, "the template mix includes certified-unsat members");
    // The composition law: the cell verifies from 100 roll-ups, zero
    // interiors read.
    assert!(verify_cell(&cell, &groups));
    // Each group verifies from its members.
    assert!(groups.iter().all(verify_group));
    // The refusal register is populated with real reasons.
    assert!(
        cell.refusal_register.keys().any(|k| k.contains("unsat")),
        "register: {:?}",
        cell.refusal_register
    );
}

#[test]
fn tampered_rollup_fails_cell_verification() {
    let (cell, mut groups) = run_cell(400, 4, 8);
    assert!(verify_cell(&cell, &groups));
    // An adversarial group forges one member's terminal hash.
    groups[2].members[0].terminal_hash = "0".repeat(64);
    assert!(!verify_group(&groups[2]), "the group's replay root catches it");
    // And forging the replay root itself breaks the cell hash.
    groups[2].replay_root = "0".repeat(64);
    assert!(!verify_cell(&cell, &groups), "the cell hash catches the forged root");
}

#[test]
fn selective_replay_challenges_one_member_without_reading_the_rest() {
    let (_, groups) = run_cell(400, 4, 8);
    // Challenge one admitted member and one refused member: full fresh
    // recomputation of just that agent reproduces its recorded projection.
    let admitted = groups
        .iter()
        .flat_map(|g| &g.members)
        .find(|m| m.byte & lane::A_ADMITTED != 0)
        .expect("an admitted member");
    let refused = groups
        .iter()
        .flat_map(|g| &g.members)
        .find(|m| m.byte & lane::H_HALTED != 0)
        .expect("a refused member");
    assert!(challenge_member(admitted, 8));
    assert!(challenge_member(refused, 8));
    // A forged record fails the challenge.
    let mut forged = admitted.clone();
    forged.terminal_hash = "0".repeat(64);
    assert!(!challenge_member(&forged, 8));
}

// ── Step 3: the foreign verifier ────────────────────────────────────────────

fn b3sum_available() -> bool {
    std::process::Command::new("b3sum")
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[test]
fn foreign_verifier_agrees_on_the_dag_receipt() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    let (dag, _, _) = lawobject_dag();
    let receipt = dag
        .execute(&mut HashRunner, &mut MemoCache::new())
        .expect("run");
    let path = temp_path("dag-receipt.json");
    std::fs::write(&path, serde_json::to_string(&receipt).expect("json")).expect("write");
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/foreign_verify.py");
    let out = std::process::Command::new("python3")
        .args([script, "dag", path.to_str().expect("utf8 path")])
        .output()
        .expect("run foreign verifier");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "foreign verifier disagreed: {stdout} {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("VERIFIED dag"), "{stdout}");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn foreign_verifier_agrees_on_the_cell_and_catches_tampering() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH");
        return;
    }
    let (cell, groups) = run_cell(400, 4, 8);
    let cell_path = temp_path("cell.json");
    let groups_path = temp_path("groups.json");
    std::fs::write(&cell_path, serde_json::to_string(&cell).expect("json")).expect("write");
    std::fs::write(&groups_path, serde_json::to_string(&groups).expect("json"))
        .expect("write");
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/foreign_verify.py");
    let run = |c: &std::path::Path, g: &std::path::Path| {
        std::process::Command::new("python3")
            .args([
                script,
                "cell",
                c.to_str().expect("utf8"),
                g.to_str().expect("utf8"),
            ])
            .output()
            .expect("run foreign verifier")
    };
    let ok = run(&cell_path, &groups_path);
    assert!(ok.status.success(), "{}", String::from_utf8_lossy(&ok.stdout));
    // Tamper with one roll-up; the foreign verifier must catch it too.
    let mut tampered = groups.clone();
    tampered[1].replay_root = "0".repeat(64);
    std::fs::write(&groups_path, serde_json::to_string(&tampered).expect("json"))
        .expect("write");
    let bad = run(&cell_path, &groups_path);
    assert!(!bad.status.success(), "tampering must fail foreign verification");
    assert!(String::from_utf8_lossy(&bad.stdout).contains("MISMATCH"));
    let _ = std::fs::remove_file(&cell_path);
    let _ = std::fs::remove_file(&groups_path);
}
