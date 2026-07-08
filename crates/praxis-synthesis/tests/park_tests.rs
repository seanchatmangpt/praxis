//! Park tests: typed-cause quarantine, all three re-admission policies, and
//! kill-9 durability (the gap knhk's in-memory park never closed).

use std::path::PathBuf;

use praxis_synthesis::park::{ParkCause, ParkManager, ParkedEntry, ReAdmission};
use praxis_synthesis::wal::Wal;

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "praxis-synth-park-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

fn entry(id: &str, cause: ParkCause, readmission: ReAdmission, run: u64) -> ParkedEntry {
    ParkedEntry {
        node_id: id.into(),
        cause,
        readmission,
        parked_at_run: run,
        input_fingerprint: format!("fp-{id}-v1"),
    }
}

#[test]
fn park_is_idempotent_and_typed() {
    let mut mgr = ParkManager::new();
    let e = entry("n1", ParkCause::CrashLoop, ReAdmission::Manual, 1);
    assert!(mgr.park(e.clone(), None).expect("park"));
    assert!(
        !mgr.park(e, None).expect("re-park"),
        "second park is a no-op"
    );
    assert_eq!(mgr.parked_count(), 1);
    assert_eq!(
        mgr.get("n1").expect("entry").cause.description(),
        "restart intensity exhausted (crash loop)"
    );
}

#[test]
fn after_runs_readmits_at_the_boundary_not_before() {
    let mut mgr = ParkManager::new();
    mgr.park(
        entry(
            "n1",
            ParkCause::TickBudgetExceeded,
            ReAdmission::AfterRuns(3),
            5,
        ),
        None,
    )
    .expect("park");
    assert!(
        mgr.readmit(7, |_| None).is_empty(),
        "run 7 < 5+3: still parked"
    );
    let back = mgr.readmit(8, |_| None);
    assert_eq!(back.len(), 1, "run 8 == 5+3: re-admitted");
    assert_eq!(mgr.parked_count(), 0);
}

#[test]
fn on_input_change_readmits_only_when_the_fingerprint_moves() {
    let mut mgr = ParkManager::new();
    mgr.park(
        entry(
            "n1",
            ParkCause::UpstreamParked,
            ReAdmission::OnInputChange,
            1,
        ),
        None,
    )
    .expect("park");
    // Same fingerprint: stays parked.
    assert!(mgr.readmit(99, |id| Some(format!("fp-{id}-v1"))).is_empty());
    // Upstream fixed → fingerprint changed → re-admitted.
    let back = mgr.readmit(100, |id| Some(format!("fp-{id}-v2")));
    assert_eq!(back.len(), 1);
}

#[test]
fn manual_never_auto_readmits() {
    let mut mgr = ParkManager::new();
    mgr.park(
        entry("n1", ParkCause::CrashLoop, ReAdmission::Manual, 1),
        None,
    )
    .expect("park");
    assert!(mgr
        .readmit(u64::MAX, |id| Some(format!("changed-{id}")))
        .is_empty());
    assert!(
        mgr.readmit_manual("n1").is_some(),
        "the authority path works"
    );
    assert_eq!(mgr.parked_count(), 0);
}

#[test]
fn quarantine_survives_kill_minus_nine() {
    let wal_path = temp_path("park.wal");
    {
        let mut wal = Wal::open(&wal_path).expect("wal");
        let mut mgr = ParkManager::new();
        mgr.park(
            entry("poison-node", ParkCause::CrashLoop, ReAdmission::Manual, 7),
            Some(&mut wal),
        )
        .expect("park");
        mgr.park(
            entry(
                "slow-node",
                ParkCause::TickBudgetExceeded,
                ReAdmission::AfterRuns(2),
                7,
            ),
            Some(&mut wal),
        )
        .expect("park");
        // Process "dies" here: mgr dropped, only the WAL survives.
    }
    let recovered = ParkManager::recover(&wal_path).expect("recover");
    assert_eq!(
        recovered.parked_count(),
        2,
        "quarantine outlived the process"
    );
    assert_eq!(
        recovered.get("poison-node").expect("entry").cause,
        ParkCause::CrashLoop
    );
    assert_eq!(
        recovered.get("slow-node").expect("entry").readmission,
        ReAdmission::AfterRuns(2)
    );
    let _ = std::fs::remove_file(&wal_path);
}

#[test]
fn park_records_coexist_with_memo_records_in_one_wal() {
    // The same WAL carries memo frames (64-hex keys) and park frames
    // (park/v1/ prefix); each consumer recovers only its own population.
    let wal_path = temp_path("mixed.wal");
    {
        let mut wal = Wal::open(&wal_path).expect("wal");
        wal.append(&"ab".repeat(32), b"memo-output")
            .expect("memo frame");
        let mut mgr = ParkManager::new();
        mgr.park(
            entry("n1", ParkCause::RunLengthExceeded, ReAdmission::Manual, 1),
            Some(&mut wal),
        )
        .expect("park");
    }
    let (memo, frames, torn) = Wal::recover(&wal_path).expect("recover");
    assert_eq!(frames, 2);
    assert!(!torn);
    assert_eq!(memo.len(), 2, "raw recovery sees both");
    let parks = ParkManager::recover(&wal_path).expect("park recover");
    assert_eq!(
        parks.parked_count(),
        1,
        "park recovery filters to its prefix"
    );
    let _ = std::fs::remove_file(&wal_path);
}
