//! [`Fleet`] — a branchless SWAR kernel packing 8 [`AgentByte`]s per 64-bit
//! word.
//!
//! # Prior art
//!
//! The word-level admission primitive is a port of `admit4`/`commit_masked`
//! from `unibit-kernel` (`/Users/sac/unibit/crates/unibit-kernel/src/lib.rs`,
//! read-only reference — **not** a dependency; the three needed const fns are
//! reproduced here, ~40 lines, per that crate's documented denial-polarity
//! doctrine). The single most important invariant carried over verbatim:
//!
//! > **Zero means admitted; nonzero means denied.**
//!
//! Where `unibit` gates one 64-bit truth word, we broadcast an 8-bit required
//! mask across all eight byte-lanes of a word and gate 8 agents at once (SWAR),
//! so a [`Fleet`] of `N` agents sweeps in `N/8` word operations.

use crate::abi::Pulse64;
use crate::byte::AgentByte;

/// Number of agents packed into one 64-bit word.
pub const LANES_PER_WORD: usize = 8;

/// Broadcast an 8-bit mask into every byte-lane of a word:
/// `0xAB -> 0xABAB_ABAB_ABAB_ABAB`.
const fn broadcast(mask: u8) -> u64 {
    (mask as u64) * 0x0101_0101_0101_0101
}

/// High bit of every lane (`0x8080…`).
const LANE_HIGH: u64 = 0x8080_8080_8080_8080;
/// Low 7 bits of every lane (`0x7f7f…`).
const LANE_LOW7: u64 = 0x7f7f_7f7f_7f7f_7f7f;

/// Mark, in `0x80`-per-lane form, which byte-lanes of `word` are **zero**.
///
/// Borrow-safe per-lane detector (the naive `(v - 0x0101…) & !v & 0x8080…`
/// "has-zero" trick is unusable here — its full-width subtraction propagates
/// borrows *between* lanes, corrupting a per-lane count). Instead:
/// `((v & 0x7f7f…) + 0x7f7f…) | v` sets each lane's high bit iff that byte is
/// nonzero (the masked add stays within its lane, max `0x7f+0x7f=0xfe`, no
/// carry-out), so inverting and masking `0x8080…` yields the zero lanes.
/// `count_ones` then counts zero lanes directly (one set bit per lane).
const fn zero_lane_mask(word: u64) -> u64 {
    let nonzero_high = ((word & LANE_LOW7).wrapping_add(LANE_LOW7)) | word;
    !nonzero_high & LANE_HIGH
}

/// **Ported `admit`/sweep primitive.** Gate all 8 agents in `word` against a
/// broadcast `required_mask`, returning a *denial word*: each byte-lane holds
/// exactly the required bits missing from that agent (`required & !agent`), so
/// a **zero lane means that agent is admitted** and a nonzero lane names its
/// missing bits.
///
/// This is the `unibit` `admit3` prereq gate (`prereq & !state`) lifted to 8
/// SWAR lanes; `commit_masked`'s `allow = !deny` shape is reused by
/// [`Fleet::update_from_pulse`].
///
/// # Examples
///
/// ```
/// use agent8::{sweep_admit, AgentByte};
/// // lane 0 fully granted, lane 1 missing RECEIPTED
/// let granted = AgentByte::GRANT_REQUIRED as u64;
/// let missing = (AgentByte::GRANT_REQUIRED & !AgentByte::RECEIPTED) as u64;
/// let word = granted | (missing << 8);
/// let denial = sweep_admit(word, AgentByte::GRANT_REQUIRED);
/// assert_eq!(denial & 0xFF, 0);                         // lane 0 admitted
/// assert_eq!((denial >> 8) & 0xFF, AgentByte::RECEIPTED as u64); // lane 1 denied
/// ```
#[must_use]
pub const fn sweep_admit(word: u64, required_mask: u8) -> u64 {
    broadcast(required_mask) & !word
}

/// Statistics over a fleet sweep.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct FleetStats {
    /// Total agents considered.
    pub total: u64,
    /// Agents admitted against the required mask (denial lane == 0).
    pub admitted: u64,
    /// Agents blocked (`total - admitted`).
    pub blocked: u64,
    /// Agents carrying the [`AgentByte::RECEIPTED`] bit.
    pub receipted: u64,
    /// Agents carrying the [`AgentByte::REPLAYABLE`] bit.
    pub replayable: u64,
}

/// A packed fleet of agents, 8 per 64-bit word.
#[derive(Clone, Debug, Default)]
pub struct Fleet {
    /// Backing words; lane `i` of word `w` is agent `w * 8 + i`.
    pub bytes: Vec<u64>,
}

impl Fleet {
    /// An empty fleet.
    #[must_use]
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    /// A fleet of `count` agents (rounded up to a whole word), every agent
    /// initialised to `fill`.
    #[must_use]
    pub fn with_fill(count: usize, fill: AgentByte) -> Self {
        let words = count.div_ceil(LANES_PER_WORD);
        Self {
            bytes: vec![broadcast(fill.raw()); words],
        }
    }

    /// Number of agents (word count × 8).
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len() * LANES_PER_WORD
    }

    /// True iff the fleet holds no words.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Read one agent's projection.
    #[must_use]
    pub fn get(&self, agent: usize) -> AgentByte {
        let (w, lane) = (agent / LANES_PER_WORD, agent % LANES_PER_WORD);
        let byte = (self.bytes[w] >> (lane * 8)) as u8;
        AgentByte::from_raw(byte)
    }

    /// Overwrite one agent's projection.
    pub fn set(&mut self, agent: usize, value: AgentByte) {
        let (w, lane) = (agent / LANES_PER_WORD, agent % LANES_PER_WORD);
        let shift = lane * 8;
        let cleared = self.bytes[w] & !(0xFFu64 << shift);
        self.bytes[w] = cleared | ((value.raw() as u64) << shift);
    }

    /// Sweep the whole fleet against `required_mask` and return popcount
    /// statistics. Pass [`AgentByte::GRANT_REQUIRED`] for the default policy.
    #[must_use]
    pub fn sweep_stats(&self, required_mask: u8) -> FleetStats {
        let receipted_bcast = broadcast(AgentByte::RECEIPTED);
        let replayable_bcast = broadcast(AgentByte::REPLAYABLE);
        let mut admitted = 0u64;
        let mut receipted = 0u64;
        let mut replayable = 0u64;
        for &word in &self.bytes {
            let denial = sweep_admit(word, required_mask);
            // Each admitted lane contributes exactly one set bit here.
            admitted += u64::from(zero_lane_mask(denial).count_ones());
            // RECEIPTED/REPLAYABLE are single bits, so masking then counting
            // ones yields the number of lanes carrying that bit.
            receipted += u64::from((word & receipted_bcast).count_ones());
            replayable += u64::from((word & replayable_bcast).count_ones());
        }
        let total = self.len() as u64;
        FleetStats {
            total,
            admitted,
            blocked: total - admitted,
            receipted,
            replayable,
        }
    }

    /// Fold an observed [`Pulse64`] back into one agent's projection, using the
    /// `unibit` `commit_masked` transition shape (`allow = !deny`; consume then
    /// produce): a valid, error-free pulse *produces* `RECEIPTED` (a receipt
    /// fragment was observed) and, being replayed evidence, `REPLAYABLE`; an
    /// error pulse instead *consumes* `HEALTHY`. The transition is suppressed
    /// (agent unchanged) when the pulse fails [`Pulse64::validate`].
    pub fn update_from_pulse(&mut self, agent: usize, pulse: &Pulse64) {
        // Denial polarity: 0 = admit the update, u8::MAX = suppress it.
        let deny: u8 = if pulse.validate() { 0 } else { u8::MAX };
        let (produce, consume) = if pulse.has_error() {
            (0u8, AgentByte::HEALTHY)
        } else {
            (AgentByte::RECEIPTED | AgentByte::REPLAYABLE, 0u8)
        };
        let allow = !deny;
        let old = self.get(agent).raw();
        // commit_masked: (old & !(consume & allow)) | (produce & allow)
        let new = (old & !(consume & allow)) | (produce & allow);
        self.set(agent, AgentByte::from_raw(new));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Naive per-agent oracle for the differential test (Day 3 doctrine).
    fn naive_stats(fleet: &Fleet, required: u8) -> FleetStats {
        let total = fleet.len() as u64;
        let mut s = FleetStats {
            total,
            ..Default::default()
        };
        for i in 0..fleet.len() {
            let b = fleet.get(i);
            if b.denial(required) == 0 {
                s.admitted += 1;
            }
            if b.carries(AgentByte::RECEIPTED) {
                s.receipted += 1;
            }
            if b.carries(AgentByte::REPLAYABLE) {
                s.replayable += 1;
            }
        }
        s.blocked = total - s.admitted;
        s
    }

    #[test]
    fn get_set_round_trip() {
        let mut f = Fleet::with_fill(16, AgentByte::empty());
        f.set(0, AgentByte::from_raw(0x6F));
        f.set(7, AgentByte::from_raw(0x80));
        f.set(8, AgentByte::from_raw(0x01));
        assert_eq!(f.get(0).raw(), 0x6F);
        assert_eq!(f.get(7).raw(), 0x80);
        assert_eq!(f.get(8).raw(), 0x01);
        assert_eq!(f.get(1).raw(), 0x00);
    }

    #[test]
    fn sweep_matches_naive_oracle_across_a_pseudo_random_fleet() {
        // Deterministic LCG fill so the fleet spans the full byte space.
        let mut state: u64 = 0x1234_5678_9abc_def0;
        let mut f = Fleet {
            bytes: vec![0u64; 4096],
        }; // 32_768 agents
        for w in f.bytes.iter_mut() {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *w = state;
        }
        for &required in &[AgentByte::GRANT_REQUIRED, 0x00, 0xFF, AgentByte::ADMITTED] {
            let fast = f.sweep_stats(required);
            let slow = naive_stats(&f, required);
            assert_eq!(fast, slow, "mismatch for required mask {required:#04x}");
        }
    }

    #[test]
    fn all_granted_fleet_admits_everyone() {
        let f = Fleet::with_fill(1000, AgentByte::from_raw(AgentByte::GRANT_REQUIRED));
        let s = f.sweep_stats(AgentByte::GRANT_REQUIRED);
        assert_eq!(s.total, 1000);
        assert_eq!(s.admitted, 1000);
        assert_eq!(s.blocked, 0);
        assert_eq!(s.receipted, 1000); // RECEIPTED is in GRANT_REQUIRED
        assert_eq!(s.replayable, 0); // REPLAYABLE is advisory, not set here
    }

    #[test]
    fn update_from_pulse_sets_receipted_and_replayable() {
        use crate::abi::{Pulse64, PULSE_FLAG_ERROR, PULSE_FLAG_VALID};
        let mut f = Fleet::with_fill(8, AgentByte::from_raw(AgentByte::HEALTHY));
        let mut ok = Pulse64::new();
        ok.flags = PULSE_FLAG_VALID;
        f.update_from_pulse(3, &ok);
        assert!(f.get(3).carries(AgentByte::RECEIPTED));
        assert!(f.get(3).carries(AgentByte::REPLAYABLE));
        assert!(f.get(3).carries(AgentByte::HEALTHY)); // untouched

        // Error pulse clears HEALTHY, does not receipt.
        let mut err = Pulse64::new();
        err.flags = PULSE_FLAG_VALID | PULSE_FLAG_ERROR;
        f.update_from_pulse(4, &err);
        assert!(!f.get(4).carries(AgentByte::HEALTHY));
        assert!(!f.get(4).carries(AgentByte::RECEIPTED));

        // Invalid pulse suppresses the transition entirely.
        let mut bad = Pulse64::new();
        bad.magic = 0;
        let before = f.get(5);
        f.update_from_pulse(5, &bad);
        assert_eq!(f.get(5), before);
    }
}
