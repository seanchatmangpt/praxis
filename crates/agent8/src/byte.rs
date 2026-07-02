//! [`AgentByte`] — an 8-bit projection of one agent's governance state.
//!
//! # Prior art
//!
//! This is a hand-authored sibling of the *generated* `Status8Field` in
//! `semantic_bit` (`/Users/sac/semantic_bit/src/status8.rs`), which is
//! manufactured from admitted graph law (`FIELD-STATUS8`). That field carries
//! observational status bits (OK / WARN / BLOCKED / …) and selects
//! Grant/Deny/Review from a low-mask compare. `AgentByte` reuses the same
//! shape — a `#[repr(transparent)]` `u8` newtype, named position constants, and
//! const `with`/`carries`/`select` — but projects an *agent's* admission
//! posture rather than a system's status, so its bit vocabulary and its grant
//! rule are its own (see [`AgentByte::GRANT_REQUIRED`]).

use serde::{Deserialize, Serialize};

/// Selected continuation for an agent, mirroring `Status8Condition` in the
/// `semantic_bit` prior art (Grant / Deny — no Review lane here: an agent
/// either satisfies the required mask or it does not).
#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum AgentSelect {
    /// Every bit in [`AgentByte::GRANT_REQUIRED`] is set.
    Grant = 0,
    /// At least one required bit is missing.
    Deny = 1,
}

/// An 8-bit projection of a single agent's governance state.
///
/// Each named constant is a single-bit position. The byte is the wire slot
/// carried in [`crate::abi::Env64::pb`] (the "pattern byte"), so the whole
/// admission posture of an agent travels in one byte of the ingress envelope.
///
/// Serialises transparently as its inner `u8`.
///
/// # Examples
///
/// ```
/// use agent8::AgentByte;
/// let a = AgentByte::empty()
///     .with(AgentByte::ADMITTED)
///     .with(AgentByte::EVIDENCE_OK);
/// assert!(a.carries(AgentByte::ADMITTED));
/// assert!(!a.carries(AgentByte::HEALTHY));
/// assert_eq!(a.raw(), 0b0000_0011);
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentByte(u8);

impl AgentByte {
    /// Position 0: the agent passed prerequisite admission.
    pub const ADMITTED: u8 = 0x01;
    /// Position 1: the agent's supporting evidence verified.
    pub const EVIDENCE_OK: u8 = 0x02;
    /// Position 2: the agent is operating within its credit/rate budget.
    pub const WITHIN_BUDGET: u8 = 0x04;
    /// Position 3: the agent's actions are bound to a granted authority.
    pub const AUTHORITY_BOUND: u8 = 0x08;
    /// Position 4: the agent reports a healthy operational state (advisory).
    pub const HEALTHY: u8 = 0x10;
    /// Position 5: the agent conforms to its declared law/schema.
    pub const CONFORMANT: u8 = 0x20;
    /// Position 6: the agent's step was immutably receipted.
    pub const RECEIPTED: u8 = 0x40;
    /// Position 7: the agent's transition can be deterministically replayed
    /// (advisory: a post-hoc property, not a precondition of grant).
    pub const REPLAYABLE: u8 = 0x80;

    /// The documented required mask for [`AgentSelect::Grant`].
    ///
    /// Grant requires the six *load-bearing governance* bits:
    /// `ADMITTED | EVIDENCE_OK | WITHIN_BUDGET | AUTHORITY_BOUND | CONFORMANT |
    /// RECEIPTED` (`0x6F`). [`HEALTHY`](Self::HEALTHY) is an *operational*
    /// signal and [`REPLAYABLE`](Self::REPLAYABLE) is a *post-hoc* property;
    /// both are advisory and intentionally excluded — an agent can be granted
    /// while momentarily unhealthy, and replayability is asserted after the
    /// fact, not demanded before acting. This mirrors the `semantic_bit`
    /// prior art selecting on a documented sub-mask (`0x1F`) rather than the
    /// whole byte.
    pub const GRANT_REQUIRED: u8 = Self::ADMITTED
        | Self::EVIDENCE_OK
        | Self::WITHIN_BUDGET
        | Self::AUTHORITY_BOUND
        | Self::CONFORMANT
        | Self::RECEIPTED;

    /// The empty projection (no bits set).
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Construct directly from a raw byte.
    #[must_use]
    pub const fn from_raw(byte: u8) -> Self {
        Self(byte)
    }

    /// The underlying byte.
    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// Set the bits in `position` (const builder; OR-in).
    #[must_use]
    pub const fn with(mut self, position: u8) -> Self {
        self.0 |= position;
        self
    }

    /// Clear the bits in `position` (const builder; AND-NOT).
    #[must_use]
    pub const fn without(mut self, position: u8) -> Self {
        self.0 &= !position;
        self
    }

    /// True iff *every* bit in `position` is set. Note this takes a mask, so
    /// `carries(A | B)` means "carries both A and B".
    #[must_use]
    pub const fn carries(self, position: u8) -> bool {
        self.0 & position == position
    }

    /// The denial word for this byte against `required_mask`, in the ported
    /// `unibit` denial polarity: **zero means all required bits present
    /// (admitted); nonzero identifies exactly the missing required bits.**
    #[must_use]
    pub const fn denial(self, required_mask: u8) -> u8 {
        // Prior art: unibit-kernel `admit3` prereq gate — `prereq & !state`.
        required_mask & !self.0
    }

    /// Grant iff every bit in `required_mask` is set. Pass
    /// [`Self::GRANT_REQUIRED`] for the documented default policy.
    #[must_use]
    pub const fn select(self, required_mask: u8) -> AgentSelect {
        if self.denial(required_mask) == 0 {
            AgentSelect::Grant
        } else {
            AgentSelect::Deny
        }
    }
}

impl core::fmt::Display for AgentByte {
    /// 8-char flag string, high bit first, one unique letter per set bit and
    /// `-` for each clear bit, for an at-a-glance read:
    ///
    /// `P R C H U B E A` → `Replayable Receipted Conformant Healthy aUthority
    /// Budget Evidence Admitted`.
    ///
    /// ```
    /// use agent8::AgentByte;
    /// let a = AgentByte::from_raw(0xFF);
    /// assert_eq!(a.to_string(), "PRCHUBEA");
    /// let b = AgentByte::empty().with(AgentByte::ADMITTED).with(AgentByte::RECEIPTED);
    /// assert_eq!(b.to_string(), "-R-----A");
    /// ```
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // (bit, glyph) from MSB (bit 7) down to LSB (bit 0).
        const LANES: [(u8, char); 8] = [
            (AgentByte::REPLAYABLE, 'P'),
            (AgentByte::RECEIPTED, 'R'),
            (AgentByte::CONFORMANT, 'C'),
            (AgentByte::HEALTHY, 'H'),
            (AgentByte::AUTHORITY_BOUND, 'U'),
            (AgentByte::WITHIN_BUDGET, 'B'),
            (AgentByte::EVIDENCE_OK, 'E'),
            (AgentByte::ADMITTED, 'A'),
        ];
        for (bit, glyph) in LANES {
            let c = if self.0 & bit != 0 { glyph } else { '-' };
            f.write_str(&c.to_string())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_via_raw() {
        for byte in 0u8..=255 {
            let a = AgentByte::from_raw(byte);
            assert_eq!(a.raw(), byte);
            // with(without(x)) idempotence over the whole mask
            assert_eq!(a.without(0xFF).raw(), 0);
            assert_eq!(a.with(0xFF).raw(), 0xFF);
        }
    }

    #[test]
    fn serde_is_a_bare_u8() {
        let a = AgentByte::from_raw(0x6F);
        let j = serde_json::to_string(&a).expect("ser");
        assert_eq!(j, "111"); // 0x6F == 111, serialised as a plain number
        let back: AgentByte = serde_json::from_str(&j).expect("de");
        assert_eq!(back, a);
    }

    #[test]
    fn grant_requires_the_six_governance_bits() {
        let full = AgentByte::from_raw(AgentByte::GRANT_REQUIRED);
        assert_eq!(full.select(AgentByte::GRANT_REQUIRED), AgentSelect::Grant);
        // Drop any single required bit -> Deny, and the denial word names it.
        for bit in [
            AgentByte::ADMITTED,
            AgentByte::EVIDENCE_OK,
            AgentByte::WITHIN_BUDGET,
            AgentByte::AUTHORITY_BOUND,
            AgentByte::CONFORMANT,
            AgentByte::RECEIPTED,
        ] {
            let missing = full.without(bit);
            assert_eq!(missing.select(AgentByte::GRANT_REQUIRED), AgentSelect::Deny);
            assert_eq!(missing.denial(AgentByte::GRANT_REQUIRED), bit);
        }
    }

    #[test]
    fn advisory_bits_do_not_block_grant() {
        // GRANT_REQUIRED set, HEALTHY + REPLAYABLE clear -> still Grant.
        let a = AgentByte::from_raw(AgentByte::GRANT_REQUIRED);
        assert!(!a.carries(AgentByte::HEALTHY));
        assert!(!a.carries(AgentByte::REPLAYABLE));
        assert_eq!(a.select(AgentByte::GRANT_REQUIRED), AgentSelect::Grant);
    }

    #[test]
    fn display_is_eight_chars() {
        assert_eq!(AgentByte::empty().to_string(), "--------");
        assert_eq!(AgentByte::from_raw(0xFF).to_string().len(), 8);
    }
}
