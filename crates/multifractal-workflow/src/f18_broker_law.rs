//! Family F18 -- "Broker and Zero Unreceipted Actuation" (atlas ticket V12-018).
//!
//! Survey verdict: **MIXED**. This pass wires the REUSE_ADAPT primary slice: a
//! real, in-process, real-tested broker choke point implementing the family's
//! 8-stage L2/L3 pipeline (Broker Entry Gate -> Standing Verifier -> Authority
//! Verifier -> Atomic Idempotency Gate -> Correlation Binder -> Hook/Dispatch
//! Router -> Consequence Capture -> Broker Receipt) and its L5 lifecycle
//! ([`BrokerState`]), refusing via typed [`UnreceiptedActuationRefused`] on
//! invalid standing, a forged/invalid authority token, a duplicate idempotency
//! claim, a correlation mismatch, an unlawful stage transition, or a tampered
//! receipt on replay.
//!
//! # Adaptation sources (REUSE_ADAPT, both re-read and re-verified this pass)
//!
//! - `crates/cng/src/bench/dispatch.rs` (Rust, in-repo): the 16-state
//!   `DispatchState::lawful_to` pattern (an explicit `matches!` transition
//!   table refusing any edge not named in it) is the direct model for
//!   [`BrokerState::lawful_to`] below, and `ledger_chain_hash`'s
//!   `blake3(prev | material)` per-workflow chain-fold is the direct model
//!   for [`Broker::fold_consequence_hash`]. Not copy-pasted: that file's
//!   ledger is an oxigraph/Turtle-backed `LedgerSink` trait wired to a
//!   filesystem loopback dispatch adapter (its own doc comment already
//!   discloses "live third-party endpoints are out of scope (UNVERIFIED)");
//!   this module's ledger is a plain in-process `Mutex<HashMap<..>>` and its
//!   dispatch step is a caller-supplied closure, not an RDF/filesystem
//!   surface -- extracting the state-machine/chain-fold *shape* while
//!   dropping the RDF/filesystem machinery this family does not need.
//! - `apps/arazzo_runner/src/arazzo_runner_broker.erl` (Erlang, cross-runtime
//!   reference; ported, not linked): `check_required_prior_receipts/5`'s
//!   `ets:insert_new/2` atomic-claim pattern is the direct model for
//!   [`Broker::claim_idempotency`] (here: `HashMap::entry` under a `Mutex`,
//!   the same single-compare-and-swap-style atomicity `ets:insert_new/2`
//!   gives, just single-node instead of BEAM-distributed). `broker_secret/0`
//!   + `make_token/1` (`sha256(secret ++ tagged_parts)`, secret generated
//!   once via `crypto:strong_rand_bytes/32` and never returned by any
//!   exported function) is the direct model for [`BrokerSecret`] and
//!   [`AuthorityToken::issue`] -- ported to `blake3::keyed_hash` instead of
//!   a manual secret-prefix-concat SHA-256: BLAKE3 is this repo's canonical
//!   receipt digest primitive (invariant #2), and its native keyed mode does
//!   not carry the length-extension characteristics of naive
//!   `hash(secret || message)` construction, so this is a disclosed
//!   *improvement* over the ported design, not a silent substitution.
//!   `test_actuation_token_requires_server_secret` and
//!   `test_concurrent_duplicate_dispatch_claims_exactly_once` (both directly
//!   re-read this pass) are the properties [`forged_token_from_public_ids_is_refused`]
//!   and [`concurrent_duplicate_claims_yield_exactly_one_winner`] below port
//!   into this crate's own test suite, in Rust, against this module's own
//!   code (not a claim about the Erlang code, which was independently
//!   verified passing via `just erlang-test` in the prior survey pass, not
//!   re-run in this Rust-only wiring pass).
//!
//! # Interpretation call, disclosed (not silently resolved)
//!
//! The family's own L5 lens text says REFUSED "branches off STANDING_VERIFIED
//! and CORRELATED" (naming two stages), while the family's L2/L3 requirements
//! text separately requires refusal "on invalid standing, invalid/duplicate
//! idempotency claim, or invalid receipt" (naming three *different* trigger
//! points -- Standing Verifier, Atomic Idempotency Gate, Broker Receipt).
//! [`BrokerState::lawful_to`] resolves this by making REFUSED reachable from
//! every non-terminal stage, matching the broader, more specific L2/L3 text
//! rather than the narrower L5 diagram gloss; this is named here so a later
//! pass can correct it against the atlas source file directly if the L5
//! diagram was in fact meant to be exhaustive.
//!
//! # Disclosed gaps (HAND_WRITE_REQUIRED-style, not dressed up as done)
//!
//! - **No durable persistence.** [`Broker`]'s ledger is
//!   `Mutex<HashMap<ActionId, LedgerEntry>>`, in-process memory only. A
//!   process restart loses every in-flight ledger entry; there is no
//!   on-disk/receipt-head replay-recovery path in this module (unlike
//!   `crates/cng/src/bench/dispatch.rs`'s `FileLedgerSink`, which this
//!   module deliberately did not port -- see adaptation note above). The
//!   family's L7 "process/engine restart ... must resolve through ... durable
//!   receipt/replay recovery" requirement is UNVERIFIED for this module.
//! - **No concrete external dispatch adapter.** [`Broker::actuate`] takes a
//!   caller-supplied `FnOnce() -> Vec<u8>` closure for the actual effect;
//!   this module is the gate/ledger/receipt mechanism, not a network,
//!   filesystem, or process adapter (mirrors `dispatch.rs`'s own disclosed
//!   "live third-party endpoints are out of scope" boundary).
//! - **No production caller in this repo.** Nothing in `crates/
//!   multifractal-workflow` yet routes a real external actuation through
//!   this [`Broker`] -- there is no production reachability trace for any of
//!   the three L8 claim-ceiling booleans. This module's own unit/negative/
//!   concurrency tests (below) are real and passing (ALIVE at the library
//!   level, verified this session via `just multifractal-workflow-test-isolated`),
//!   but that is a narrower claim than the L8 ceiling's "production
//!   reachability trace" bar, and this doc comment does not claim to meet
//!   that bar.
//!
//! # L6 provenance chain (GGEN_GENERATABLE slice, wired this pass)
//!
//! [`PROVENANCE_CHAIN`] and [`REFUSAL_CATALOG`] below (`include!`d from
//! `f18_broker_law_generated.rs`) are generated by `crates/ggen` from
//! `packs/f18-broker-law-pack/ontology.ttl`'s `mfwbrk:ProvenanceStage` and
//! `mfwbrk:RefusalKind` individuals -- a pure data projection of the
//! family's L6 8-node chain (ActionArtifact -> StandingReceipt ->
//! AuthorityToken -> IdempotencyRecord -> CorrelationBinding ->
//! ActuationResult -> ConsequenceObservation -> BrokerReceipt) and of this
//! module's own [`UnreceiptedActuationRefused`] variant set, respectively.
//! Regenerated twice from an isolated scratch ggen project (does not touch
//! the shared root `ggen.toml`; see the generated file's own header for the
//! exact commands) and confirmed byte-identical both times (`diff`) before
//! being copied in. [`tests::refusal_catalog_matches_the_real_enum`] cross-
//! references [`REFUSAL_CATALOG`]'s label set against the real enum's 7
//! variants so this generated catalog cannot silently drift from the code
//! it describes. This is real, generated Rust content wired into the crate
//! -- it is still only a **data projection**, not a materialized RDF/PROV-O
//! graph and not itself enforcement; [`BrokerReceipt`] (the enforcement
//! path's own output) carries the actual per-action chain content.
//!
//! Survey-cited paths (from the family survey handed to this Wire session):
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F18_broker-law.md
//! - /Users/sac/praxis/apps/arazzo_runner/src/arazzo_runner_broker.erl
//! - /Users/sac/praxis/apps/arazzo_runner/include/arazzo_broker.hrl
//! - /Users/sac/praxis/apps/arazzo_runner/test/arazzo_runner_broker_test.erl
//! - /Users/sac/praxis/apps/arazzo_runner/test/arazzo_runner_broker_event_receipt_test.erl
//! - /Users/sac/praxis/crates/cng/src/bench/dispatch.rs
//! - /Users/sac/praxis/justfile
//! - /Users/sac/praxis/packs/f18-broker-law-pack (GGEN_GENERATABLE L6 slice, wired this pass)
//! - /Users/sac/praxis/crates/multifractal-workflow

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use thiserror::Error;

include!("f18_broker_law_generated.rs");

// ---------------------------------------------------------------------------
// L5: 8 lawful stages plus the off-ladder REFUSED terminal.
// ---------------------------------------------------------------------------

/// F18-L5 lifecycle state. One state per L2/L3 stage the action has passed
/// through, plus the off-ladder `Refused` terminal. See the module's
/// "Interpretation call" doc section above for how `Refused` reachability
/// was resolved against the family's own (mutually narrower/broader) L5 vs.
/// L2/L3 text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrokerState {
    /// Broker Entry Gate: the action has arrived at the broker.
    ActionReceived,
    /// Standing Verifier passed.
    StandingVerified,
    /// Authority Verifier minted a token for this action.
    Authorized,
    /// Atomic Idempotency Gate atomically claimed this action's dedup key.
    IdempotencyClaimed,
    /// Correlation Binder confirmed the observed correlation id.
    Correlated,
    /// Hook/Dispatch Router is running the actuation closure.
    Actuating,
    /// Consequence Capture hashed and chained the raw consequence.
    ConsequenceCaptured,
    /// Broker Receipt was issued; terminal success state.
    Receipted,
    /// Terminal refusal state; reachable from any non-terminal stage (see
    /// module doc "Interpretation call").
    Refused,
}

impl BrokerState {
    /// Every state, in pipeline order. Drift check surface for tests.
    pub const ALL: [BrokerState; 9] = [
        BrokerState::ActionReceived,
        BrokerState::StandingVerified,
        BrokerState::Authorized,
        BrokerState::IdempotencyClaimed,
        BrokerState::Correlated,
        BrokerState::Actuating,
        BrokerState::ConsequenceCaptured,
        BrokerState::Receipted,
        BrokerState::Refused,
    ];

    /// Vocabulary name (for logs/receipts; stable spelling, no wall clock).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            BrokerState::ActionReceived => "ACTION_RECEIVED",
            BrokerState::StandingVerified => "STANDING_VERIFIED",
            BrokerState::Authorized => "AUTHORIZED",
            BrokerState::IdempotencyClaimed => "IDEMPOTENCY_CLAIMED",
            BrokerState::Correlated => "CORRELATED",
            BrokerState::Actuating => "ACTUATING",
            BrokerState::ConsequenceCaptured => "CONSEQUENCE_CAPTURED",
            BrokerState::Receipted => "RECEIPTED",
            BrokerState::Refused => "REFUSED",
        }
    }

    /// Whether `self -> to` is a lawful edge. Adapted from
    /// `crates/cng/src/bench/dispatch.rs`'s `DispatchState::lawful_to`
    /// pattern: an explicit `matches!` table, so any edge not named here is
    /// mechanically unlawful, not merely undocumented.
    ///
    /// # Complexity
    /// O(1): fixed-arity pattern match, no traversal.
    #[must_use]
    pub fn lawful_to(self, to: BrokerState) -> bool {
        use BrokerState as S;
        matches!(
            (self, to),
            (S::ActionReceived, S::StandingVerified | S::Refused)
                | (S::StandingVerified, S::Authorized | S::Refused)
                | (S::Authorized, S::IdempotencyClaimed | S::Refused)
                | (S::IdempotencyClaimed, S::Correlated | S::Refused)
                | (S::Correlated, S::Actuating | S::Refused)
                | (S::Actuating, S::ConsequenceCaptured | S::Refused)
                | (S::ConsequenceCaptured, S::Receipted | S::Refused)
        )
    }
}

// ---------------------------------------------------------------------------
// Public identifiers vs. server-side secret (PUBLIC_ID_TOKEN_BYPASS_REFUSED).
// ---------------------------------------------------------------------------

/// The public identifiers of one broker action: known to any caller (and, by
/// design, to any attacker who observed a workflow run). MUST NOT be
/// sufficient by themselves to derive a valid [`AuthorityToken`] -- that is
/// exactly the property [`forged_token_from_public_ids_is_refused`] tests.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionId {
    pub workflow_id: String,
    pub step_id: String,
    pub idempotency_key: String,
}

impl ActionId {
    #[must_use]
    pub fn new(
        workflow_id: impl Into<String>,
        step_id: impl Into<String>,
        idempotency_key: impl Into<String>,
    ) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            step_id: step_id.into(),
            idempotency_key: idempotency_key.into(),
        }
    }
}

/// Server-side authority secret. Deliberately carries no `Debug`, `Display`,
/// `Clone`, or accessor -- adapted from `arazzo_runner_broker.erl`'s
/// `broker_secret/0` (32 random bytes, generated once, held only in
/// `persistent_term`, never returned by any exported function). This module
/// does not itself source randomness (no RNG dependency is added to this
/// crate): a CSPRNG seed is the caller's responsibility, matching this
/// repo's determinism discipline (no randomness inside receipt/hash logic
/// itself -- the secret is an *input*, not something this module generates
/// nondeterministically at hash time).
pub struct BrokerSecret([u8; 32]);

impl BrokerSecret {
    #[must_use]
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

/// A forgery-resistant authority token bound to one [`ActionId`] and one
/// [`BrokerSecret`]. Equality is the only exposed operation; the underlying
/// digest is never displayed except via [`AuthorityToken::to_hex`], matching
/// this repo's "auditable: raw material is human-readable" receipt
/// discipline for material that legitimately needs to be logged/compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorityToken(blake3::Hash);

impl AuthorityToken {
    /// `blake3::keyed_hash(secret, "f18-authority-token|" | workflow_id |
    /// "|" | step_id | "|" | idempotency_key)`. See the module's adaptation
    /// note above for why this substitutes BLAKE3's native keyed mode for
    /// the ported `sha256(secret ++ parts)` construction.
    ///
    /// # Complexity
    /// O(|workflow_id| + |step_id| + |idempotency_key|): one keyed hash over
    /// a small, fixed-shape byte buffer.
    fn issue(secret: &BrokerSecret, action: &ActionId) -> Self {
        let mut data = Vec::with_capacity(
            24 + action.workflow_id.len() + action.step_id.len() + action.idempotency_key.len(),
        );
        data.extend_from_slice(b"f18-authority-token|");
        data.extend_from_slice(action.workflow_id.as_bytes());
        data.push(b'|');
        data.extend_from_slice(action.step_id.as_bytes());
        data.push(b'|');
        data.extend_from_slice(action.idempotency_key.as_bytes());
        Self(blake3::keyed_hash(&secret.0, &data))
    }

    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.to_hex().to_string()
    }
}

// ---------------------------------------------------------------------------
// Typed refusal taxonomy.
// ---------------------------------------------------------------------------

/// F18's typed refusal taxonomy. Every variant below has >= 1 dedicated test
/// in this module's `tests` submodule (see the per-variant doc reference).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum UnreceiptedActuationRefused {
    /// Standing Verifier refused. See [`tests::standing_invalid_refuses`].
    #[error("standing invalid for actor {actor:?} on action {action:?}: {reason}")]
    StandingInvalid {
        action: ActionId,
        actor: String,
        reason: String,
    },
    /// Authority Verifier / Atomic Idempotency Gate rejected a token that
    /// does not match the one this broker would issue for these public
    /// identifiers -- the direct mechanism behind
    /// PUBLIC_ID_TOKEN_BYPASS_REFUSED. See
    /// [`tests::forged_token_from_public_ids_is_refused`].
    #[error("authority token invalid for action {action:?} (forged, stale, or issued under a different secret)")]
    AuthorityInvalid { action: ActionId },
    /// Atomic Idempotency Gate refused a second claim for an
    /// already-claimed action -- the direct mechanism behind
    /// DUPLICATE_DISPATCH_SINGLE_ACTUATION. See
    /// [`tests::duplicate_idempotency_claim_is_refused`] and
    /// [`tests::concurrent_duplicate_claims_yield_exactly_one_winner`].
    #[error("action {action:?} already claimed at state {existing_state:?}; duplicate dispatch refused, single actuation preserved")]
    DuplicateIdempotencyClaim {
        action: ActionId,
        existing_state: BrokerState,
    },
    /// Correlation Binder refused a mismatched correlation id. See
    /// [`tests::correlation_mismatch_refuses_and_marks_refused`].
    #[error("correlation mismatch on action {action:?}: expected {expected}, observed {observed}")]
    CorrelationMismatch {
        action: ActionId,
        expected: String,
        observed: String,
    },
    /// Broker Receipt replay found a receipt whose recorded hash does not
    /// recompute -- a tampered or malformed receipt. See
    /// [`tests::replay_of_tampered_receipt_is_refused`].
    #[error("receipt invalid for action {action:?}: {reason}")]
    InvalidReceipt { action: ActionId, reason: String },
    /// A stage was invoked out of order (mechanical bypass prevention: no
    /// stage method advances the ledger to a state
    /// [`BrokerState::lawful_to`] refuses). See
    /// [`tests::unlawful_transition_refused`] and
    /// [`tests::stale_second_capture_after_receipt_is_refused`].
    #[error("unlawful broker-state transition for action {action:?}: {from:?} -> {to:?}")]
    UnlawfulTransition {
        action: ActionId,
        from: BrokerState,
        to: BrokerState,
    },
    /// A stage past the Atomic Idempotency Gate was invoked for an action
    /// that was never claimed. See
    /// [`tests::malformed_result_without_prior_claim_is_refused`].
    #[error(
        "no ledger entry for action {action:?} (never claimed, or already terminal and evicted)"
    )]
    UnknownAction { action: ActionId },
}

// ---------------------------------------------------------------------------
// Broker Receipt (L6 terminal content; L8 replay).
// ---------------------------------------------------------------------------

/// The Broker Receipt: terminal, human-readable (hex, not binary) evidence
/// for one action's full lawful traversal of the pipeline. All hashing is
/// BLAKE3, canonical (fixed field order, no `HashMap`-iteration-derived
/// order anywhere in the input), and content-derived only -- no wall clock
/// enters `receipt_hash` or `consequence_hash`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokerReceipt {
    pub workflow_id: String,
    pub step_id: String,
    pub idempotency_key: String,
    pub correlation_id: String,
    pub authority_token_hex: String,
    pub consequence_hash_hex: String,
    pub receipt_hash_hex: String,
}

/// Canonical, fixed-order receipt-hash input. Shared by [`Broker::issue_receipt`]
/// (compute) and [`Broker::replay_receipt`] (recompute + compare), so the two
/// can never silently drift apart into two different digest constructions.
///
/// # Complexity
/// O(sum of field byte lengths): one BLAKE3 hash over a small, fixed-shape
/// buffer.
fn receipt_hash_input(
    action: &ActionId,
    correlation_id: &str,
    authority_token_hex: &str,
    consequence_hash_hex: &str,
) -> blake3::Hash {
    let mut h = blake3::Hasher::new();
    h.update(b"f18-broker-receipt|");
    h.update(action.workflow_id.as_bytes());
    h.update(b"|");
    h.update(action.step_id.as_bytes());
    h.update(b"|");
    h.update(action.idempotency_key.as_bytes());
    h.update(b"|");
    h.update(correlation_id.as_bytes());
    h.update(b"|");
    h.update(authority_token_hex.as_bytes());
    h.update(b"|");
    h.update(consequence_hash_hex.as_bytes());
    h.finalize()
}

// ---------------------------------------------------------------------------
// Ledger entry + Broker.
// ---------------------------------------------------------------------------

/// One action's ledger record. Only ever mutated through [`Broker`]'s stage
/// methods, in lawful order (checked against [`BrokerState::lawful_to`] at
/// every mutation).
#[derive(Debug, Clone)]
struct LedgerEntry {
    state: BrokerState,
    token: AuthorityToken,
    correlation_id: Option<String>,
    consequence_hash_hex: Option<String>,
    receipt: Option<BrokerReceipt>,
}

/// F18's lawful actuation choke point. Owns the atomic idempotency ledger and
/// the per-workflow consequence-hash chain head; every stage method checks
/// the action's current ledger state against [`BrokerState::lawful_to`]
/// before mutating it, so calling stages out of order is a typed refusal,
/// never a silent state jump.
///
/// See the module's disclosed-gaps doc section for what this type does
/// *not* do (durable persistence, a concrete external dispatch adapter, a
/// production caller in this repo).
pub struct Broker {
    secret: BrokerSecret,
    ledger: Mutex<HashMap<ActionId, LedgerEntry>>,
    /// workflow_id -> latest consequence-chain hash (hex; empty string is
    /// the legitimate initial head for a workflow with no prior consequence
    /// -- mirrors `arazzo_runner_broker.erl`'s `chain_head/1`, `[] -> <<>>`).
    chain_heads: Mutex<HashMap<String, String>>,
}

impl Broker {
    #[must_use]
    pub fn new(secret: BrokerSecret) -> Self {
        Self {
            secret,
            ledger: Mutex::new(HashMap::new()),
            chain_heads: Mutex::new(HashMap::new()),
        }
    }

    /// Recovers from mutex poisoning rather than propagating a second panic:
    /// this crate's discipline is zero panics in fallible code (invariant
    /// #1), so a poisoned lock (meaning some other caller already panicked,
    /// which should never happen under that same discipline) is treated as
    /// still-valid data rather than compounding the failure into a panic
    /// here. The guarded data itself is never left structurally invalid by
    /// any method below (every mutation is a single, complete map
    /// insert/update), so recovering it is safe.
    fn lock_ledger(&self) -> std::sync::MutexGuard<'_, HashMap<ActionId, LedgerEntry>> {
        self.ledger.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn lock_chain_heads(&self) -> std::sync::MutexGuard<'_, HashMap<String, String>> {
        self.chain_heads
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    // -- Stage 1+2: Broker Entry Gate + Standing Verifier -------------------

    /// Stateless (no ledger write): standing is re-checkable and does not
    /// itself need dedup -- only actuation does. `has_standing` is the
    /// caller's own standing-lookup result (this module does not itself
    /// judge standing; see F01 "Standing Algebra" for that family).
    ///
    /// # Complexity
    /// O(1).
    pub fn verify_standing(
        &self,
        action: &ActionId,
        actor: &str,
        has_standing: bool,
        reason_if_invalid: &str,
    ) -> Result<BrokerState, UnreceiptedActuationRefused> {
        if has_standing {
            Ok(BrokerState::StandingVerified)
        } else {
            Err(UnreceiptedActuationRefused::StandingInvalid {
                action: action.clone(),
                actor: actor.to_string(),
                reason: reason_if_invalid.to_string(),
            })
        }
    }

    // -- Stage 3: Authority Verifier -----------------------------------------

    /// Mints the deterministic authority token for `action`. Pure function
    /// of `(secret, action)` -- two calls for the same action under the same
    /// secret always agree (see
    /// [`tests::authorize_is_deterministic_for_same_action`]), mirroring
    /// `arazzo_runner_broker.erl`'s `dispatch_token/3` determinism note (any
    /// number of concurrent racers compute the identical value
    /// independently, before any ledger write).
    ///
    /// # Complexity
    /// O(|action|): one keyed BLAKE3 hash.
    #[must_use]
    pub fn authorize(&self, action: &ActionId) -> (BrokerState, AuthorityToken) {
        (
            BrokerState::Authorized,
            AuthorityToken::issue(&self.secret, action),
        )
    }

    // -- Stage 4: Atomic Idempotency Gate ------------------------------------

    /// Atomically claims `action`'s dedup key. `HashMap::entry` under a
    /// single `Mutex` gives the same single-compare-and-swap atomicity
    /// `ets:insert_new/2` gives in the Erlang reference: two threads racing
    /// this call for the same `action` can never both observe `Vacant` (see
    /// [`tests::concurrent_duplicate_claims_yield_exactly_one_winner`]).
    /// `token` is verified against the one this broker would itself issue
    /// for `action` BEFORE the claim is attempted, so a forged token can
    /// never occupy a ledger slot (PUBLIC_ID_TOKEN_BYPASS_REFUSED).
    ///
    /// # Complexity
    /// O(|action|) for the token re-derivation + O(1) amortized `HashMap`
    /// entry lookup/insert.
    pub fn claim_idempotency(
        &self,
        action: ActionId,
        token: AuthorityToken,
    ) -> Result<BrokerState, UnreceiptedActuationRefused> {
        let expected = AuthorityToken::issue(&self.secret, &action);
        if token != expected {
            return Err(UnreceiptedActuationRefused::AuthorityInvalid { action });
        }
        let mut ledger = self.lock_ledger();
        match ledger.entry(action) {
            Entry::Occupied(existing) => {
                let existing_state = existing.get().state;
                let action = existing.key().clone();
                Err(UnreceiptedActuationRefused::DuplicateIdempotencyClaim {
                    action,
                    existing_state,
                })
            }
            Entry::Vacant(slot) => {
                slot.insert(LedgerEntry {
                    state: BrokerState::IdempotencyClaimed,
                    token,
                    correlation_id: None,
                    consequence_hash_hex: None,
                    receipt: None,
                });
                Ok(BrokerState::IdempotencyClaimed)
            }
        }
    }

    /// Advances `action`'s ledger entry from `from` to `to` if lawful, else
    /// (on the `Refused` branch of a failed check) marks it `Refused` and
    /// returns the caller-supplied refusal unchanged. Shared by every
    /// ledger-mutating stage below so the lawful-transition check is never
    /// duplicated (and therefore never silently skipped in one call site).
    ///
    /// # Complexity
    /// O(1) amortized `HashMap` lookup + O(1) state-machine check.
    fn advance(
        &self,
        action: &ActionId,
        from: BrokerState,
        to: BrokerState,
    ) -> Result<(), UnreceiptedActuationRefused> {
        let mut ledger = self.lock_ledger();
        let entry =
            ledger
                .get_mut(action)
                .ok_or_else(|| UnreceiptedActuationRefused::UnknownAction {
                    action: action.clone(),
                })?;
        if entry.state != from || !from.lawful_to(to) {
            let unlawful_from = entry.state;
            return Err(UnreceiptedActuationRefused::UnlawfulTransition {
                action: action.clone(),
                from: unlawful_from,
                to,
            });
        }
        entry.state = to;
        Ok(())
    }

    // -- Stage 5: Correlation Binder -----------------------------------------

    /// Confirms `observed_correlation_id` against `expected_correlation_id`
    /// (the value bound to this action at dispatch time by the caller, e.g.
    /// from a workflow identity). A mismatch arriving at the legitimate
    /// Correlation Binder window (ledger state `IdempotencyClaimed`) marks
    /// the ledger entry `Refused` (chaos: "stale/malformed results ...
    /// resolve through ... a typed refusal, never silent duplicate
    /// actuation") rather than leaving it stuck mid-pipeline. A mismatch
    /// arriving AFTER the action has already lawfully advanced past
    /// `Correlated` is a stale/duplicate/forged delivery for an action a
    /// prior lawful call already owns: it is still refused
    /// (`CorrelationMismatch`), but does NOT mutate that action's ledger
    /// state -- see [`tests::stale_correlation_mismatch_after_actuation_does_not_orphan_the_action`]
    /// for the adversarial case this guards (a stale mismatch landing after
    /// `actuate()` already ran the real-world dispatch closure must not
    /// orphan that already-actuated action by flipping it to `Refused`
    /// before it can reach a receipt).
    ///
    /// # Complexity
    /// O(1) amortized `HashMap` lookup/update + O(|correlation_id|) string
    /// compare.
    pub fn bind_correlation(
        &self,
        action: &ActionId,
        expected_correlation_id: &str,
        observed_correlation_id: &str,
    ) -> Result<BrokerState, UnreceiptedActuationRefused> {
        if expected_correlation_id != observed_correlation_id {
            // Mark Refused ONLY if this mismatch arrives at the legitimate
            // Correlation Binder window (state == IdempotencyClaimed) --
            // i.e. this call is the actual first correlation attempt for
            // this action, not a stale/duplicate/forged delivery arriving
            // after the action already advanced lawfully. Checking only
            // `entry.state.lawful_to(Refused)` (any prior revision of this
            // branch) is unsound: `Actuating.lawful_to(Refused)` is true, so
            // a mismatched delivery landing AFTER a legitimate
            // bind_correlation + actuate() -- i.e. after the real-world
            // dispatch closure already ran -- could flip
            // `Actuating -> Refused` and permanently orphan an
            // already-actuated action (capture_consequence would then
            // refuse with UnlawfulTransition, and no BrokerReceipt could
            // ever be issued): an actuation with no receipt, which is
            // exactly the failure this module (Zero Unreceipted Actuation)
            // exists to prevent. See
            // [`tests::stale_correlation_mismatch_after_actuation_does_not_orphan_the_action`],
            // which reproduces the unfixed bug sequentially (no concurrency
            // required) and pins the fixed behavior.
            let mut ledger = self.lock_ledger();
            if let Some(entry) = ledger.get_mut(action) {
                if entry.state == BrokerState::IdempotencyClaimed {
                    entry.state = BrokerState::Refused;
                }
            }
            return Err(UnreceiptedActuationRefused::CorrelationMismatch {
                action: action.clone(),
                expected: expected_correlation_id.to_string(),
                observed: observed_correlation_id.to_string(),
            });
        }
        self.advance(
            action,
            BrokerState::IdempotencyClaimed,
            BrokerState::Correlated,
        )?;
        let mut ledger = self.lock_ledger();
        if let Some(entry) = ledger.get_mut(action) {
            entry.correlation_id = Some(observed_correlation_id.to_string());
        }
        Ok(BrokerState::Correlated)
    }

    // -- Stage 6: Hook/Dispatch Router ---------------------------------------

    /// Runs `dispatch` -- the caller-supplied actuation effect -- ONLY after
    /// advancing `Correlated -> Actuating` succeeds; the closure is
    /// mechanically unreachable for an action that has not passed standing,
    /// authority, idempotency, and correlation (no public method on
    /// [`Broker`] can invoke a dispatch closure from any other ledger
    /// state). This is the module's zero-unreceipted-actuation mechanism at
    /// the library level; see the module's disclosed-gaps section for what
    /// this claim does NOT yet cover (a production caller / reachability
    /// trace).
    ///
    /// # Complexity
    /// O(1) state check + whatever `dispatch` itself costs (opaque to this
    /// module by design).
    pub fn actuate<F>(
        &self,
        action: &ActionId,
        dispatch: F,
    ) -> Result<Vec<u8>, UnreceiptedActuationRefused>
    where
        F: FnOnce() -> Vec<u8>,
    {
        self.advance(action, BrokerState::Correlated, BrokerState::Actuating)?;
        Ok(dispatch())
    }

    // -- Stage 7: Consequence Capture -----------------------------------------

    /// Folds `raw_consequence` into this action's workflow-scoped BLAKE3
    /// chain (`blake3(prev_head | raw_consequence)`; adapted from
    /// `crates/cng/src/bench/dispatch.rs`'s `ledger_chain_hash` /
    /// `arazzo_runner_broker.erl`'s `consequence_hash/2`) and advances
    /// `Actuating -> ConsequenceCaptured`. A second call for the same action
    /// (replayed/duplicate delivery of a result) is refused by `advance`'s
    /// state check, not silently re-hashed -- see
    /// [`tests::stale_second_capture_after_receipt_is_refused`].
    ///
    /// # Complexity
    /// O(|raw_consequence|): one BLAKE3 hash over the previous head plus the
    /// new bytes.
    pub fn capture_consequence(
        &self,
        action: &ActionId,
        raw_consequence: &[u8],
    ) -> Result<BrokerState, UnreceiptedActuationRefused> {
        self.advance(
            action,
            BrokerState::Actuating,
            BrokerState::ConsequenceCaptured,
        )?;
        let hash_hex = self.fold_consequence_hash(&action.workflow_id, raw_consequence);
        let mut ledger = self.lock_ledger();
        if let Some(entry) = ledger.get_mut(action) {
            entry.consequence_hash_hex = Some(hash_hex);
        }
        Ok(BrokerState::ConsequenceCaptured)
    }

    /// # Complexity
    /// O(|raw_consequence|): one BLAKE3 hash.
    fn fold_consequence_hash(&self, workflow_id: &str, raw_consequence: &[u8]) -> String {
        let mut heads = self.lock_chain_heads();
        // Option, not Result: an absent entry means "no prior consequence
        // for this workflow yet" -- the correct empty chain head, mirroring
        // arazzo_runner_broker.erl's chain_head/1 (`[] -> <<>>`), not a
        // swallowed error.
        let prev = heads.get(workflow_id).cloned().unwrap_or_default();
        let mut h = blake3::Hasher::new();
        h.update(prev.as_bytes());
        h.update(raw_consequence);
        let next = h.finalize().to_hex().to_string();
        heads.insert(workflow_id.to_string(), next.clone());
        next
    }

    // -- Stage 8: Broker Receipt ----------------------------------------------

    /// Issues the terminal [`BrokerReceipt`] and advances
    /// `ConsequenceCaptured -> Receipted`. `receipt_hash` is computed via
    /// [`receipt_hash_input`] -- the same function [`Broker::replay_receipt`]
    /// uses to recompute it, so replay can never silently diverge from
    /// issuance.
    ///
    /// # Complexity
    /// O(sum of field byte lengths): one BLAKE3 hash.
    pub fn issue_receipt(
        &self,
        action: &ActionId,
    ) -> Result<BrokerReceipt, UnreceiptedActuationRefused> {
        self.advance(
            action,
            BrokerState::ConsequenceCaptured,
            BrokerState::Receipted,
        )?;
        let mut ledger = self.lock_ledger();
        let entry =
            ledger
                .get_mut(action)
                .ok_or_else(|| UnreceiptedActuationRefused::UnknownAction {
                    action: action.clone(),
                })?;
        let correlation_id = entry.correlation_id.clone().ok_or_else(|| {
            UnreceiptedActuationRefused::InvalidReceipt {
                action: action.clone(),
                reason: "no correlation_id bound before receipt issuance".to_string(),
            }
        })?;
        let consequence_hash_hex = entry.consequence_hash_hex.clone().ok_or_else(|| {
            UnreceiptedActuationRefused::InvalidReceipt {
                action: action.clone(),
                reason: "no consequence_hash bound before receipt issuance".to_string(),
            }
        })?;
        let authority_token_hex = entry.token.to_hex();
        let receipt_hash_hex = receipt_hash_input(
            action,
            &correlation_id,
            &authority_token_hex,
            &consequence_hash_hex,
        )
        .to_hex()
        .to_string();
        let receipt = BrokerReceipt {
            workflow_id: action.workflow_id.clone(),
            step_id: action.step_id.clone(),
            idempotency_key: action.idempotency_key.clone(),
            correlation_id,
            authority_token_hex,
            consequence_hash_hex,
            receipt_hash_hex,
        };
        entry.receipt = Some(receipt.clone());
        Ok(receipt)
    }

    /// L6 replay: recomputes `receipt.receipt_hash_hex` from the receipt's
    /// own recorded fields via the SAME [`receipt_hash_input`] function
    /// `issue_receipt` used, and refuses (typed, not a silent `false`) if it
    /// does not match -- "replay must reconstruct an equivalent
    /// consequence" for the non-tampered case, and a typed refusal for the
    /// tampered case, never a silent divergence.
    ///
    /// # Complexity
    /// O(sum of field byte lengths): one BLAKE3 hash.
    pub fn replay_receipt(
        &self,
        action: &ActionId,
        receipt: &BrokerReceipt,
    ) -> Result<(), UnreceiptedActuationRefused> {
        let recomputed = receipt_hash_input(
            action,
            &receipt.correlation_id,
            &receipt.authority_token_hex,
            &receipt.consequence_hash_hex,
        )
        .to_hex()
        .to_string();
        if recomputed == receipt.receipt_hash_hex {
            Ok(())
        } else {
            Err(UnreceiptedActuationRefused::InvalidReceipt {
                action: action.clone(),
                reason: format!(
                    "receipt_hash does not recompute: recorded {}, recomputed {recomputed}",
                    receipt.receipt_hash_hex
                ),
            })
        }
    }

    /// Introspection only (tests + a future admin surface, mirrors
    /// `arazzo_runner_broker.erl`'s `get_ledger_entry/1`): current
    /// [`BrokerState`] for `action`, or `None` if never claimed.
    ///
    /// # Complexity
    /// O(1) amortized `HashMap` lookup.
    #[must_use]
    pub fn state_of(&self, action: &ActionId) -> Option<BrokerState> {
        self.lock_ledger().get(action).map(|e| e.state)
    }
}

// ---------------------------------------------------------------------------
// Tests: unit + negative fixtures for every UnreceiptedActuationRefused
// variant, plus the three L8 claim-ceiling properties named in the family
// survey (ZERO_UNRECEIPTED_ACTUATION, DUPLICATE_DISPATCH_SINGLE_ACTUATION,
// PUBLIC_ID_TOKEN_BYPASS_REFUSED). All real, run this session via
// `just multifractal-workflow-test-isolated <name> -- f18_broker_law`.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn secret() -> BrokerSecret {
        BrokerSecret::new([0x42; 32])
    }

    fn action(n: u8) -> ActionId {
        ActionId::new(format!("wf-{n}"), format!("step-{n}"), format!("idem-{n}"))
    }

    /// Drives one action through every stage to a receipt; the common-path
    /// helper every other test's setup reuses.
    fn full_lifecycle(broker: &Broker, action: &ActionId, correlation_id: &str) -> BrokerReceipt {
        broker
            .verify_standing(action, "actor-1", true, "")
            .expect("standing verified");
        let (_, token) = broker.authorize(action);
        broker
            .claim_idempotency(action.clone(), token)
            .expect("idempotency claimed");
        broker
            .bind_correlation(action, correlation_id, correlation_id)
            .expect("correlation bound");
        let consequence = broker
            .actuate(action, || b"real-consequence-bytes".to_vec())
            .expect("actuated");
        broker
            .capture_consequence(action, &consequence)
            .expect("consequence captured");
        broker.issue_receipt(action).expect("receipt issued")
    }

    // -- StandingInvalid ------------------------------------------------------

    #[test]
    fn standing_invalid_refuses() {
        let broker = Broker::new(secret());
        let a = action(1);
        let err = broker
            .verify_standing(&a, "actor-1", false, "no admitted standing for actor-1")
            .unwrap_err();
        assert!(matches!(
            err,
            UnreceiptedActuationRefused::StandingInvalid { .. }
        ));
    }

    #[test]
    fn standing_valid_advances_to_standing_verified() {
        let broker = Broker::new(secret());
        let a = action(2);
        let state = broker.verify_standing(&a, "actor-1", true, "").unwrap();
        assert_eq!(state, BrokerState::StandingVerified);
    }

    // -- Authority determinism + PUBLIC_ID_TOKEN_BYPASS_REFUSED --------------

    #[test]
    fn authorize_is_deterministic_for_same_action() {
        let broker = Broker::new(secret());
        let a = action(3);
        let (_, t1) = broker.authorize(&a);
        let (_, t2) = broker.authorize(&a);
        assert_eq!(
            t1, t2,
            "same action + same secret must yield the same token"
        );
    }

    #[test]
    fn authorize_differs_across_actions() {
        let broker = Broker::new(secret());
        let (_, t1) = broker.authorize(&action(4));
        let (_, t2) = broker.authorize(&action(5));
        assert_ne!(t1, t2);
    }

    /// PUBLIC_ID_TOKEN_BYPASS_REFUSED: an attacker who knows only the public
    /// identifiers (workflow_id/step_id/idempotency_key) -- and NOT the
    /// broker's secret -- cannot forge a token that
    /// [`Broker::claim_idempotency`] will accept. Two forgery attempts are
    /// exercised: (a) hashing the public ids with no secret at all
    /// (plain, unkeyed BLAKE3), and (b) hashing them with a guessed/wrong
    /// secret. Both are refused; only the token minted by THIS broker's own
    /// `authorize` (i.e. requiring its actual secret) is accepted.
    #[test]
    fn forged_token_from_public_ids_is_refused() {
        let broker = Broker::new(secret());
        let a = action(6);

        // (a) Unkeyed hash of the public identifiers alone.
        let unkeyed = blake3::hash(
            format!(
                "f18-authority-token|{}|{}|{}",
                a.workflow_id, a.step_id, a.idempotency_key
            )
            .as_bytes(),
        );
        let forged_unkeyed = AuthorityToken(unkeyed);
        let err = broker
            .claim_idempotency(a.clone(), forged_unkeyed)
            .unwrap_err();
        assert!(matches!(
            err,
            UnreceiptedActuationRefused::AuthorityInvalid { .. }
        ));

        // (b) Keyed hash under a WRONG secret (attacker guessed a value).
        let wrong_secret = BrokerSecret::new([0x99; 32]);
        let forged_keyed = AuthorityToken::issue(&wrong_secret, &a);
        let err = broker
            .claim_idempotency(a.clone(), forged_keyed)
            .unwrap_err();
        assert!(matches!(
            err,
            UnreceiptedActuationRefused::AuthorityInvalid { .. }
        ));

        // The action was never actually claimed by either forgery attempt.
        assert_eq!(broker.state_of(&a), None);

        // The REAL token (issued by this broker, under its real secret) is
        // accepted.
        let (_, real_token) = broker.authorize(&a);
        broker
            .claim_idempotency(a.clone(), real_token)
            .expect("real token is accepted");
        assert_eq!(broker.state_of(&a), Some(BrokerState::IdempotencyClaimed));
    }

    // -- DuplicateIdempotencyClaim / DUPLICATE_DISPATCH_SINGLE_ACTUATION ------

    #[test]
    fn duplicate_idempotency_claim_is_refused() {
        let broker = Broker::new(secret());
        let a = action(7);
        let (_, token) = broker.authorize(&a);
        broker
            .claim_idempotency(a.clone(), token)
            .expect("first claim succeeds");
        let err = broker.claim_idempotency(a.clone(), token).unwrap_err();
        match err {
            UnreceiptedActuationRefused::DuplicateIdempotencyClaim { existing_state, .. } => {
                assert_eq!(existing_state, BrokerState::IdempotencyClaimed);
            }
            other => panic!("expected DuplicateIdempotencyClaim, got {other:?}"),
        }
    }

    /// DUPLICATE_DISPATCH_SINGLE_ACTUATION, concurrency form: N threads race
    /// `claim_idempotency` for the SAME action. Exactly one must win
    /// (`Ok`); every other racer must see `DuplicateIdempotencyClaim` --
    /// never a second `Ok`, and never a lost/overwritten claim. Adapted from
    /// `arazzo_runner_broker.erl`'s
    /// `test_concurrent_duplicate_dispatch_claims_exactly_once`.
    #[test]
    fn concurrent_duplicate_claims_yield_exactly_one_winner() {
        use std::sync::Arc;
        let broker = Arc::new(Broker::new(secret()));
        let a = action(8);
        let (_, token) = broker.authorize(&a);

        let handles: Vec<_> = (0..16)
            .map(|_| {
                let broker = Arc::clone(&broker);
                let a = a.clone();
                std::thread::spawn(move || broker.claim_idempotency(a, token))
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let wins = results.iter().filter(|r| r.is_ok()).count();
        let dup_refusals = results
            .iter()
            .filter(|r| {
                matches!(
                    r,
                    Err(UnreceiptedActuationRefused::DuplicateIdempotencyClaim { .. })
                )
            })
            .count();
        assert_eq!(wins, 1, "exactly one racer must win the claim");
        assert_eq!(
            dup_refusals, 15,
            "every other racer must see DuplicateIdempotencyClaim"
        );
    }

    // -- CorrelationMismatch ----------------------------------------------------

    #[test]
    fn correlation_mismatch_refuses_and_marks_refused() {
        let broker = Broker::new(secret());
        let a = action(9);
        let (_, token) = broker.authorize(&a);
        broker.claim_idempotency(a.clone(), token).unwrap();
        let err = broker
            .bind_correlation(&a, "corr-expected", "corr-forged")
            .unwrap_err();
        assert!(matches!(
            err,
            UnreceiptedActuationRefused::CorrelationMismatch { .. }
        ));
        assert_eq!(broker.state_of(&a), Some(BrokerState::Refused));
    }

    // -- UnlawfulTransition / UnknownAction -------------------------------------

    #[test]
    fn unlawful_transition_refused() {
        let broker = Broker::new(secret());
        let a = action(10);
        // Never claimed at all -> capture_consequence must refuse, not panic
        // or silently create a phantom entry.
        let err = broker.capture_consequence(&a, b"x").unwrap_err();
        assert!(matches!(
            err,
            UnreceiptedActuationRefused::UnknownAction { .. }
        ));
    }

    #[test]
    fn malformed_result_without_prior_claim_is_refused() {
        let broker = Broker::new(secret());
        let a = action(11);
        // Claimed and correlated, but actuate() was never called: capturing
        // a "result" now is out of order.
        let (_, token) = broker.authorize(&a);
        broker.claim_idempotency(a.clone(), token).unwrap();
        broker.bind_correlation(&a, "c", "c").unwrap();
        let err = broker
            .capture_consequence(&a, b"stale-or-malformed")
            .unwrap_err();
        assert!(matches!(
            err,
            UnreceiptedActuationRefused::UnlawfulTransition { .. }
        ));
    }

    /// Chaos: a stale/duplicate consequence delivered again AFTER the
    /// action already reached `Receipted` must be refused, never silently
    /// re-actuated or re-captured.
    #[test]
    fn stale_second_capture_after_receipt_is_refused() {
        let broker = Broker::new(secret());
        let a = action(12);
        let _receipt = full_lifecycle(&broker, &a, "corr-12");
        assert_eq!(broker.state_of(&a), Some(BrokerState::Receipted));
        let err = broker
            .capture_consequence(&a, b"replayed-duplicate")
            .unwrap_err();
        assert!(matches!(
            err,
            UnreceiptedActuationRefused::UnlawfulTransition { .. }
        ));
        // Still Receipted -- the stale delivery did not perturb the terminal
        // state.
        assert_eq!(broker.state_of(&a), Some(BrokerState::Receipted));
    }

    // -- InvalidReceipt / replay -------------------------------------------------

    #[test]
    fn replay_of_tampered_receipt_is_refused() {
        let broker = Broker::new(secret());
        let a = action(13);
        let mut receipt = full_lifecycle(&broker, &a, "corr-13");
        receipt.consequence_hash_hex = "0".repeat(64); // tamper
        let err = broker.replay_receipt(&a, &receipt).unwrap_err();
        assert!(matches!(
            err,
            UnreceiptedActuationRefused::InvalidReceipt { .. }
        ));
    }

    #[test]
    fn replay_of_untampered_receipt_reconstructs_equivalent_consequence() {
        let broker = Broker::new(secret());
        let a = action(14);
        let receipt = full_lifecycle(&broker, &a, "corr-14");
        broker
            .replay_receipt(&a, &receipt)
            .expect("untampered receipt replays clean");
    }

    // -- ZERO_UNRECEIPTED_ACTUATION: the full happy path ------------------------

    /// ZERO_UNRECEIPTED_ACTUATION at the library level: the ONLY way to
    /// reach a [`BrokerReceipt`] is by successfully passing every stage in
    /// order (standing -> authority -> idempotency -> correlation ->
    /// actuation -> consequence capture); this test drives that full path
    /// and confirms every field the receipt is supposed to bind is actually
    /// bound, plus confirms `state_of` shows `Receipted` only at the end.
    #[test]
    fn full_lifecycle_produces_a_bound_receipt() {
        let broker = Broker::new(secret());
        let a = action(15);
        assert_eq!(broker.state_of(&a), None);
        let receipt = full_lifecycle(&broker, &a, "corr-15");
        assert_eq!(receipt.workflow_id, a.workflow_id);
        assert_eq!(receipt.step_id, a.step_id);
        assert_eq!(receipt.idempotency_key, a.idempotency_key);
        assert_eq!(receipt.correlation_id, "corr-15");
        assert!(!receipt.consequence_hash_hex.is_empty());
        assert!(!receipt.authority_token_hex.is_empty());
        assert!(!receipt.receipt_hash_hex.is_empty());
        assert_eq!(broker.state_of(&a), Some(BrokerState::Receipted));
    }

    #[test]
    fn issue_receipt_before_consequence_captured_is_unlawful() {
        let broker = Broker::new(secret());
        let a = action(16);
        let (_, token) = broker.authorize(&a);
        broker.claim_idempotency(a.clone(), token).unwrap();
        broker.bind_correlation(&a, "c", "c").unwrap();
        broker.actuate(&a, || b"consequence".to_vec()).unwrap();
        // capture_consequence skipped: issue_receipt must refuse.
        let err = broker.issue_receipt(&a).unwrap_err();
        assert!(matches!(
            err,
            UnreceiptedActuationRefused::UnlawfulTransition { .. }
        ));
    }

    // -- State machine drift/coverage -----------------------------------------

    #[test]
    fn lawful_to_table_has_no_edges_out_of_terminal_states() {
        for to in BrokerState::ALL {
            assert!(
                !BrokerState::Receipted.lawful_to(to),
                "Receipted must be terminal"
            );
            assert!(
                !BrokerState::Refused.lawful_to(to),
                "Refused must be terminal"
            );
        }
    }

    // -- Generated L6 catalog cross-reference (drift check) --------------------

    /// The ggen-generated [`REFUSAL_CATALOG`] must name exactly the real
    /// enum's 7 variants -- neither more (a stale/invented entry) nor fewer
    /// (a variant the ontology forgot). This is what keeps the generated
    /// pack from silently drifting away from the hand-written code it
    /// describes.
    #[test]
    fn refusal_catalog_matches_the_real_enum() {
        let mut generated: Vec<&str> = REFUSAL_CATALOG.iter().map(|e| e.label).collect();
        generated.sort_unstable();
        let mut expected = vec![
            "AuthorityInvalid",
            "CorrelationMismatch",
            "DuplicateIdempotencyClaim",
            "InvalidReceipt",
            "StandingInvalid",
            "UnknownAction",
            "UnlawfulTransition",
        ];
        expected.sort_unstable();
        assert_eq!(generated, expected);
    }

    #[test]
    fn provenance_chain_has_all_8_l6_nodes_in_order() {
        assert_eq!(PROVENANCE_CHAIN.len(), 8);
        for (i, stage) in PROVENANCE_CHAIN.iter().enumerate() {
            assert_eq!(stage.chain_order as usize, i);
        }
        assert_eq!(PROVENANCE_CHAIN[0].label, "ActionArtifact");
        assert_eq!(PROVENANCE_CHAIN[7].label, "BrokerReceipt");
    }

    // -- Adversarial audit additions (this pass): direct concurrency proof for
    // the Actuating stage (item 2 of the audit -- prior coverage only raced
    // claim_idempotency, not actuate), and a same-action-stale-delivery probe
    // for the family's own "loser cannot overwrite winner" property (item 3).
    // ---------------------------------------------------------------------

    /// Direct concurrency proof that `actuate` (not just `claim_idempotency`)
    /// is exclusive: N threads race `actuate` for the SAME action, already
    /// `Correlated`. Exactly one thread's `advance(Correlated -> Actuating)`
    /// may succeed, so the caller-supplied dispatch closure -- which stands
    /// in for a real external side effect -- must run exactly once, never
    /// zero or more than once.
    #[test]
    fn concurrent_actuate_calls_run_the_dispatch_closure_exactly_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let broker = Arc::new(Broker::new(secret()));
        let a = action(20);
        broker.verify_standing(&a, "actor-1", true, "").unwrap();
        let (_, token) = broker.authorize(&a);
        broker.claim_idempotency(a.clone(), token).unwrap();
        broker.bind_correlation(&a, "corr-20", "corr-20").unwrap();
        assert_eq!(broker.state_of(&a), Some(BrokerState::Correlated));

        let dispatch_count = Arc::new(AtomicUsize::new(0));
        let handles: Vec<_> = (0..16)
            .map(|_| {
                let broker = Arc::clone(&broker);
                let a = a.clone();
                let dispatch_count = Arc::clone(&dispatch_count);
                std::thread::spawn(move || {
                    broker.actuate(&a, || {
                        dispatch_count.fetch_add(1, Ordering::SeqCst);
                        b"side-effect".to_vec()
                    })
                })
            })
            .collect();
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let wins = results.iter().filter(|r| r.is_ok()).count();
        assert_eq!(wins, 1, "exactly one racer may pass the Actuating gate");
        assert_eq!(
            dispatch_count.load(Ordering::SeqCst),
            1,
            "the dispatch closure (the real-world side effect) must run exactly once, \
             even under 16-way concurrent actuate() calls for the same action"
        );
    }

    /// Adversarial finding (this pass), FIXED by this pass: before the fix
    /// below, `bind_correlation`'s mismatch branch marked an action `Refused`
    /// based only on whether the CURRENT ledger state had a lawful edge to
    /// `Refused` -- not on whether the mismatch arrived at the legitimate
    /// Correlation Binder window (`IdempotencyClaimed`). Since
    /// `Actuating.lawful_to(Refused)` is true, a stale/duplicate mismatched
    /// correlation delivery landing AFTER a legitimate `bind_correlation` +
    /// `actuate` (i.e. after the real-world dispatch closure had ALREADY
    /// run) could flip `Actuating -> Refused`, permanently orphaning an
    /// already-actuated action: `capture_consequence` then refuses
    /// (`UnlawfulTransition`, wrong `from` state) and no `BrokerReceipt` can
    /// ever be issued for it. That is an actuation with no receipt --
    /// exactly the failure this module's name (Zero Unreceipted Actuation)
    /// says cannot happen. No concurrency is even required to trigger it:
    /// plain sequential calls reproduce it, which is what this test drives.
    ///
    /// The fix scopes the mismatch-branch's `Refused` write to the one
    /// legitimate window (`entry.state == IdempotencyClaimed`); a mismatch
    /// arriving after the action has already lawfully advanced past
    /// `Correlated` is still refused (`CorrelationMismatch`, unchanged) but
    /// no longer mutates a ledger entry that a concurrent/prior lawful chain
    /// already owns.
    #[test]
    fn stale_correlation_mismatch_after_actuation_does_not_orphan_the_action() {
        let broker = Broker::new(secret());
        let a = action(21);
        broker.verify_standing(&a, "actor-1", true, "").unwrap();
        let (_, token) = broker.authorize(&a);
        broker.claim_idempotency(a.clone(), token).unwrap();
        broker.bind_correlation(&a, "corr-21", "corr-21").unwrap();
        assert_eq!(broker.state_of(&a), Some(BrokerState::Correlated));

        // The real-world side effect happens here.
        let consequence = broker
            .actuate(&a, || b"already-happened-side-effect".to_vec())
            .expect("actuation runs");
        assert_eq!(broker.state_of(&a), Some(BrokerState::Actuating));

        // A stale/duplicate correlation delivery for the SAME action, with a
        // mismatched id, arrives late. It must still be refused...
        let err = broker
            .bind_correlation(&a, "corr-21", "corr-STALE-OR-FORGED")
            .unwrap_err();
        assert!(matches!(
            err,
            UnreceiptedActuationRefused::CorrelationMismatch { .. }
        ));

        // ...but (post-fix) it must NOT perturb an action that already
        // lawfully advanced past the Correlation Binder window.
        assert_eq!(
            broker.state_of(&a),
            Some(BrokerState::Actuating),
            "a stale mismatch arriving after Correlated must not clobber the \
             ledger state of an action already advanced by a lawful caller"
        );

        // The already-actuated side effect can still reach a receipt: this
        // is the crux of the finding -- an orphaned actuation (dispatch ran,
        // no receipt reachable) would show up here as an unexpected
        // UnlawfulTransition.
        broker
            .capture_consequence(&a, &consequence)
            .expect("consequence capture must still succeed after a stale mismatch");
        let receipt = broker
            .issue_receipt(&a)
            .expect("receipt must still be issuable");
        assert_eq!(receipt.correlation_id, "corr-21");
        assert_eq!(broker.state_of(&a), Some(BrokerState::Receipted));
    }

    #[test]
    fn consequence_chain_folds_across_actions_in_same_workflow() {
        let broker = Broker::new(secret());
        let a1 = ActionId::new("wf-chain", "step-a", "idem-a");
        let a2 = ActionId::new("wf-chain", "step-b", "idem-b");
        let r1 = full_lifecycle(&broker, &a1, "corr-a");
        let r2 = full_lifecycle(&broker, &a2, "corr-b");
        // Same raw consequence bytes in full_lifecycle, but chained after a
        // different prior head -> different consequence_hash_hex.
        assert_ne!(r1.consequence_hash_hex, r2.consequence_hash_hex);
    }
}
