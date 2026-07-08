//! End-to-end + differential tests for `agent8`.

use agent8::{pulse64_from_receipt_record, AgentByte, AgentSelect, Env64, Fleet, Pulse64};
use praxis_core::{law::Andon, receipt_record::RECEIPT_RECORD_VERSION, ReceiptRecord};

#[test]
fn abi_structs_are_cache_line_sized() {
    assert_eq!(std::mem::size_of::<Env64>(), 64);
    assert_eq!(std::mem::align_of::<Env64>(), 64);
    assert_eq!(std::mem::size_of::<Pulse64>(), 64);
    assert_eq!(std::mem::align_of::<Pulse64>(), 64);
}

#[test]
fn agent_rides_the_envelope_pattern_byte() {
    let agent = AgentByte::empty()
        .with(AgentByte::ADMITTED)
        .with(AgentByte::EVIDENCE_OK)
        .with(AgentByte::WITHIN_BUDGET)
        .with(AgentByte::AUTHORITY_BOUND)
        .with(AgentByte::CONFORMANT)
        .with(AgentByte::RECEIPTED);
    assert_eq!(agent.select(AgentByte::GRANT_REQUIRED), AgentSelect::Grant);

    let env = Env64::new().with_agent(agent);
    assert!(env.validate());
    assert_eq!(env.agent(), agent);
    assert_eq!(env.agent().to_string(), "-RC-UBEA");
}

#[test]
fn receipt_record_bridges_to_a_valid_pulse() {
    let rec = ReceiptRecord {
        version: RECEIPT_RECORD_VERSION,
        instruction_id: 2,
        activity_idx: 0,
        activity: None,
        node_kind: 0,
        ts_ns: 777,
        duration_ms: None,
        payload_hash_hex: "11".repeat(32),
        prev_chain_hash_hex: "00".repeat(32),
        chain_hash_hex: "22".repeat(32),
        andon: Andon::Green,
        obligation_count: 2,
        object_ids: vec![],
    };
    let pulse = pulse64_from_receipt_record(&rec);
    assert!(pulse.validate());
    assert_eq!(pulse.receipt, [0x22; 16]); // chain-hash fragment
    assert_eq!(pulse.ticks, 2);
    assert_eq!(pulse.hop, 2);
    assert!(!pulse.has_error());

    // A pulse can be folded back into a fleet agent.
    let mut fleet = Fleet::with_fill(
        8,
        AgentByte::from_raw(AgentByte::GRANT_REQUIRED & !AgentByte::RECEIPTED),
    );
    assert_eq!(
        fleet.get(0).select(AgentByte::GRANT_REQUIRED),
        AgentSelect::Deny // missing RECEIPTED
    );
    fleet.update_from_pulse(0, &pulse);
    assert_eq!(
        fleet.get(0).select(AgentByte::GRANT_REQUIRED),
        AgentSelect::Grant // pulse produced RECEIPTED
    );
}

/// Differential: SWAR fleet stats must equal an independent naive loop over a
/// large pseudo-random fleet (Day 3 doctrine — two implementations, one truth).
#[test]
fn fleet_sweep_matches_naive_loop_differential() {
    let words = 100_000; // 800_000 agents
    let mut fleet = Fleet {
        bytes: vec![0u64; words],
    };
    let mut state: u64 = 0xdead_beef_cafe_f00d;
    for w in fleet.bytes.iter_mut() {
        state = state
            .wrapping_mul(2862933555777941757)
            .wrapping_add(3037000493);
        *w = state;
    }

    for &required in &[
        AgentByte::GRANT_REQUIRED,
        0x00,
        0xFF,
        0x6F,
        AgentByte::HEALTHY,
    ] {
        let fast = fleet.sweep_stats(required);

        // Naive oracle.
        let total = fleet.len() as u64;
        let mut admitted = 0u64;
        let mut receipted = 0u64;
        let mut replayable = 0u64;
        for i in 0..fleet.len() {
            let b = fleet.get(i);
            if b.denial(required) == 0 {
                admitted += 1;
            }
            if b.carries(AgentByte::RECEIPTED) {
                receipted += 1;
            }
            if b.carries(AgentByte::REPLAYABLE) {
                replayable += 1;
            }
        }
        assert_eq!(fast.total, total);
        assert_eq!(
            fast.admitted, admitted,
            "admitted mismatch @ {required:#04x}"
        );
        assert_eq!(fast.blocked, total - admitted);
        assert_eq!(
            fast.receipted, receipted,
            "receipted mismatch @ {required:#04x}"
        );
        assert_eq!(
            fast.replayable, replayable,
            "replayable mismatch @ {required:#04x}"
        );
    }
}
