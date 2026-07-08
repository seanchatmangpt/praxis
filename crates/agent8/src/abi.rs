//! [`Env64`] + [`Pulse64`] — Rust ports of the bytestar 64-byte wire ABI.
//!
//! Ported from `/Users/sac/bytestar/bytecore/abi/envelope.h` (`env64_t`) and
//! `pulse.h` (`pulse64_t`). Both are cache-line-sized (`#[repr(C, align(64))]`)
//! with compile-time `size_of == 64` assertions, using the same
//! `const _: () = { assert!(...) }` idiom as `OcelCausalFrame` in
//! `bcinr-powl-receipt` (`causal_receipt.rs`).
//!
//! Unlike the C originals we do **not** mark the structs `packed`: the field
//! order is chosen so every field is naturally aligned and the struct is
//! exactly 64 bytes with zero padding, so plain `repr(C, align(64))` reproduces
//! the byte layout without the aliasing hazards of packed field references
//! (this crate `forbid`s `unsafe_code`).

use crate::byte::AgentByte;
use praxis_core::{law::Andon, ReceiptRecord};

/// Ingress envelope magic (`ENVELOPE_MAGIC`, `0xBE64`).
pub const ENV_MAGIC: u16 = 0xBE64;
/// Observer pulse magic (`PULSE_MAGIC`, `0x5064`).
pub const PULSE_MAGIC: u16 = 0x5064;
/// ByteCore ABI version this port targets.
pub const ABI_VERSION: u8 = 3;

/// Maximum priority / ticks / hops the ABI permits (all `<= 8`, and priority
/// `<= 7`).
pub const MAX_PRIORITY: u8 = 7;
/// Maximum ticks / hops per the pulse ABI (`<= 8`).
pub const MAX_STEP: u8 = 8;

// ── env64_t ──────────────────────────────────────────────────────────────────

/// 64-byte, 64-byte-aligned ingress envelope. Direct port of `env64_t`.
///
/// Byte layout matches the C header exactly:
/// ```text
/// [ 0.. 1] magic   [ 2] ver    [ 3] pb (AgentByte)  [ 4.. 5] budget
/// [ 6] flags       [ 7] priority                    [ 8..23] in_cid (128-bit)
/// [24..31] timestamp                                [32..35] seq_num
/// [36..39] source  [40..63] aux
/// ```
#[repr(C, align(64))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Env64 {
    /// `ENVELOPE_MAGIC` (`0xBE64`).
    pub magic: u16,
    /// ABI version.
    pub ver: u8,
    /// Pattern byte — **the [`AgentByte`] wire slot** (execution mode / agent
    /// admission posture travels here).
    pub pb: u8,
    /// Credits / rate-limit budget.
    pub budget: u16,
    /// Processing flags.
    pub flags: u8,
    /// Execution priority (`0..=7`).
    pub priority: u8,
    /// Payload content id (first 128 bits).
    pub in_cid: [u8; 16],
    /// Ingress timestamp (nanoseconds).
    pub timestamp: u64,
    /// Sequence number.
    pub seq_num: u32,
    /// Source identifier.
    pub source: [u8; 4],
    /// Pattern-specific auxiliary data.
    pub aux: [u8; 24],
}

// Prior art: OcelCausalFrame's `const _: () = { assert!(size_of == N) }`.
const _: () = {
    assert!(
        core::mem::size_of::<Env64>() == 64,
        "Env64 must be exactly 64 bytes"
    );
    assert!(
        core::mem::align_of::<Env64>() == 64,
        "Env64 must be 64-byte aligned"
    );
};

impl Env64 {
    /// A zeroed envelope stamped with the correct magic and version.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            magic: ENV_MAGIC,
            ver: ABI_VERSION,
            pb: 0,
            budget: 0,
            flags: 0,
            priority: 0,
            in_cid: [0; 16],
            timestamp: 0,
            seq_num: 0,
            source: [0; 4],
            aux: [0; 24],
        }
    }

    /// Set the [`AgentByte`] carried in the pattern-byte slot.
    #[must_use]
    pub const fn with_agent(mut self, agent: AgentByte) -> Self {
        self.pb = agent.raw();
        self
    }

    /// The [`AgentByte`] projection carried in the pattern-byte slot.
    #[must_use]
    pub const fn agent(self) -> AgentByte {
        AgentByte::from_raw(self.pb)
    }

    /// Validate as branchless mask compares (no data-dependent branches):
    /// magic, version, and `priority <= 7` must all hold.
    #[must_use]
    pub const fn validate(&self) -> bool {
        let magic_ok = self.magic == ENV_MAGIC;
        let ver_ok = self.ver == ABI_VERSION;
        let prio_ok = self.priority <= MAX_PRIORITY;
        // Bitwise `&` on bools: fold the three checks without short-circuit
        // branching on any single field.
        magic_ok & ver_ok & prio_ok
    }
}

impl Default for Env64 {
    fn default() -> Self {
        Self::new()
    }
}

// ── pulse64_t ─────────────────────────────────────────────────────────────────

/// Pulse observer flag: pulse contains a valid observation.
pub const PULSE_FLAG_VALID: u8 = 0x01;
/// Pulse observer flag: final pulse in a sequence.
pub const PULSE_FLAG_FINAL: u8 = 0x02;
/// Pulse observer flag: an error condition was observed.
pub const PULSE_FLAG_ERROR: u8 = 0x04;

/// 64-byte, 64-byte-aligned observer pulse. Direct port of `pulse64_t`.
///
/// Byte layout matches the C header exactly:
/// ```text
/// [ 0.. 1] magic   [ 2] ver   [ 3] flags   [ 4..19] in_cid  [20..35] out_cid
/// [36..51] receipt (ρ fragment)            [52] ticks   [53] hop
/// [54] cube_pos    [55] observer_id        [56..63] timestamp
/// ```
#[repr(C, align(64))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pulse64 {
    /// `PULSE_MAGIC` (`0x5064`).
    pub magic: u16,
    /// ABI version.
    pub ver: u8,
    /// Observer flags.
    pub flags: u8,
    /// Input content id (first 128 bits).
    pub in_cid: [u8; 16],
    /// Output content id (first 128 bits).
    pub out_cid: [u8; 16],
    /// ρ_{t+1} receipt fragment (first 128 bits of the chain hash).
    pub receipt: [u8; 16],
    /// Execution ticks (`<= 8`).
    pub ticks: u8,
    /// Hop count (`<= 8`).
    pub hop: u8,
    /// Core/lane position metadata.
    pub cube_pos: u8,
    /// Observer instance identifier.
    pub observer_id: u8,
    /// Observation timestamp (nanoseconds).
    pub timestamp: u64,
}

const _: () = {
    assert!(
        core::mem::size_of::<Pulse64>() == 64,
        "Pulse64 must be exactly 64 bytes"
    );
    assert!(
        core::mem::align_of::<Pulse64>() == 64,
        "Pulse64 must be 64-byte aligned"
    );
};

impl Pulse64 {
    /// A zeroed pulse stamped with the correct magic and version.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            magic: PULSE_MAGIC,
            ver: ABI_VERSION,
            flags: 0,
            in_cid: [0; 16],
            out_cid: [0; 16],
            receipt: [0; 16],
            ticks: 0,
            hop: 0,
            cube_pos: 0,
            observer_id: 0,
            timestamp: 0,
        }
    }

    /// Validate as branchless mask compares: magic, version, `ticks <= 8`,
    /// `hop <= 8`.
    #[must_use]
    pub const fn validate(&self) -> bool {
        let magic_ok = self.magic == PULSE_MAGIC;
        let ver_ok = self.ver == ABI_VERSION;
        let ticks_ok = self.ticks <= MAX_STEP;
        let hop_ok = self.hop <= MAX_STEP;
        magic_ok & ver_ok & ticks_ok & hop_ok
    }

    /// True iff the error flag is set.
    #[must_use]
    pub const fn has_error(&self) -> bool {
        self.flags & PULSE_FLAG_ERROR != 0
    }
}

impl Default for Pulse64 {
    fn default() -> Self {
        Self::new()
    }
}

/// Decode the first 16 bytes of a 64-hex-char hash string into a fixed buffer,
/// zero-filling on any short/malformed input (the bridge is total: a bad hex
/// field yields a zeroed fragment rather than an error, matching the pulse
/// ABI's "zeros = genesis/absent" convention).
fn first16_of_hex(s: &str) -> [u8; 16] {
    let mut out = [0u8; 16];
    // 16 bytes == 32 hex chars.
    if s.len() >= 32 {
        if let Ok(bytes) = hex::decode(&s[..32]) {
            out.copy_from_slice(&bytes[..16]);
        }
    }
    out
}

/// Bridge a praxis [`ReceiptRecord`] into a wire [`Pulse64`].
///
/// Mapping (documented, illustrative where the ABI is wider than the record):
/// - `in_cid`  ← first 16 bytes of `prev_chain_hash` (the input chain state)
/// - `out_cid` ← first 16 bytes of `payload_hash`     (what was processed)
/// - `receipt` ← first 16 bytes of `chain_hash`       (ρ receipt fragment)
/// - `ticks`   ← `obligation_count`  capped at 8 (`MAX_STEP`)
/// - `hop`     ← `instruction_id`     capped at 8 (`MAX_STEP`)
/// - `flags`   ← `VALID`, plus `ERROR` when the Andon outcome is not `Green`
/// - `timestamp` ← `ts_ns`
///
/// The result always satisfies [`Pulse64::validate`] (magic/version stamped,
/// ticks/hops capped `<= 8`).
#[must_use]
pub fn pulse64_from_receipt_record(record: &ReceiptRecord) -> Pulse64 {
    let cap8 = |v: u64| -> u8 {
        if v > MAX_STEP as u64 {
            MAX_STEP
        } else {
            v as u8
        }
    };
    let mut flags = PULSE_FLAG_VALID;
    if !matches!(record.andon, Andon::Green) {
        flags |= PULSE_FLAG_ERROR;
    }
    Pulse64 {
        magic: PULSE_MAGIC,
        ver: ABI_VERSION,
        flags,
        in_cid: first16_of_hex(&record.prev_chain_hash_hex),
        out_cid: first16_of_hex(&record.payload_hash_hex),
        receipt: first16_of_hex(&record.chain_hash_hex),
        ticks: cap8(u64::from(record.obligation_count)),
        hop: cap8(record.instruction_id),
        cube_pos: 0,
        observer_id: 0,
        timestamp: record.ts_ns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env64_layout_is_locked() {
        assert_eq!(core::mem::size_of::<Env64>(), 64);
        assert_eq!(core::mem::align_of::<Env64>(), 64);
        let e = Env64::new();
        assert!(e.validate());
    }

    #[test]
    fn pulse64_layout_is_locked() {
        assert_eq!(core::mem::size_of::<Pulse64>(), 64);
        assert_eq!(core::mem::align_of::<Pulse64>(), 64);
        let p = Pulse64::new();
        assert!(p.validate());
    }

    #[test]
    fn validate_rejects_bad_magic_and_version() {
        let mut e = Env64::new();
        e.magic = 0xDEAD;
        assert!(!e.validate());
        let mut e = Env64::new();
        e.ver = 99;
        assert!(!e.validate());
        let mut e = Env64::new();
        e.priority = 8; // > 7
        assert!(!e.validate());

        let mut p = Pulse64::new();
        p.magic = 0x0000;
        assert!(!p.validate());
        let mut p = Pulse64::new();
        p.ticks = 9; // > 8
        assert!(!p.validate());
        let mut p = Pulse64::new();
        p.hop = 9;
        assert!(!p.validate());
    }

    #[test]
    fn agent_slot_round_trips_through_pb() {
        let a = AgentByte::from_raw(AgentByte::GRANT_REQUIRED);
        let e = Env64::new().with_agent(a);
        assert_eq!(e.pb, a.raw());
        assert_eq!(e.agent(), a);
    }

    fn sample_record(andon: Andon, oblig: u32, iid: u64) -> ReceiptRecord {
        ReceiptRecord {
            version: praxis_core::receipt_record::RECEIPT_RECORD_VERSION,
            instruction_id: iid,
            activity_idx: 0,
            activity: None,
            node_kind: 0,
            ts_ns: 12345,
            duration_ms: None,
            payload_hash_hex: "aa".repeat(32),
            prev_chain_hash_hex: "bb".repeat(32),
            chain_hash_hex: "cc".repeat(32),
            andon,
            obligation_count: oblig,
            object_ids: vec![],
        }
    }

    #[test]
    fn bridge_maps_fields_and_caps_at_eight() {
        let rec = sample_record(Andon::Green, 3, 5);
        let p = pulse64_from_receipt_record(&rec);
        assert!(p.validate());
        assert_eq!(p.in_cid, [0xbb; 16]); // prev_chain_hash
        assert_eq!(p.out_cid, [0xaa; 16]); // payload_hash
        assert_eq!(p.receipt, [0xcc; 16]); // chain_hash fragment
        assert_eq!(p.ticks, 3);
        assert_eq!(p.hop, 5);
        assert_eq!(p.flags, PULSE_FLAG_VALID);
        assert!(!p.has_error());
        assert_eq!(p.timestamp, 12345);

        // Over-cap values clamp to 8, and non-Green raises the error flag.
        let rec = sample_record(
            Andon::Halted {
                unmet: vec![],
                refusals: vec![],
                at: 0,
            },
            100,
            999,
        );
        let p = pulse64_from_receipt_record(&rec);
        assert_eq!(p.ticks, MAX_STEP);
        assert_eq!(p.hop, MAX_STEP);
        assert!(p.has_error());
        assert!(p.validate());
    }

    #[test]
    fn bridge_zero_fills_malformed_hex() {
        let mut rec = sample_record(Andon::Green, 0, 0);
        rec.chain_hash_hex = "not-hex".to_string();
        let p = pulse64_from_receipt_record(&rec);
        assert_eq!(p.receipt, [0u8; 16]);
    }
}
