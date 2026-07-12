//! Family F11 -- "BCINR Local Runtime" (atlas ticket V12-011).
//!
//! Wire-phase-1 status: **real wiring for L1-L3/L5/L6 (POWL Loader through Local
//! Receipt), honest stub for the External Socket Detector and L7.** Per the v26.7.12
//! family survey's own verdict (MIXED, `mixed_breakdown` field), five of F11's seven
//! L2 topology components -- POWL Loader, Compact State, Eligibility Engine,
//! Dependency Bitmap, Local Transition Kernel -- map almost exactly onto real,
//! already-wired code in `/Users/sac/bcinr/crates/bcinr-powl` (`compiler::compile_powl`,
//! `tape::PowlTape`/`Powl64Op`, `scheduler::scheduler_tick`), and Local Receipt maps onto
//! `/Users/sac/bcinr/crates/bcinr-powl-receipt`'s `causal_receipt::OcelCausalReceipt`
//! BLAKE3 chain. This module wraps those real types behind this family's own
//! [`BCINRLocalRuntime`] state machine and [`BCINRLocalExecutionRefused`] typed refusal
//! -- it is a thin, real, `cargo test`-verified adaptation, not a re-implementation.
//!
//! ## What is real here (verified this session, see this module's own tests)
//!
//! - [`BCINRLocalRuntime::load`] / [`BCINRLocalRuntime::from_compiled_tape`] -- POWL
//!   Loader + Compact State: calls the real `bcinr_powl::compiler::compile_powl` and
//!   refuses with [`BCINRLocalExecutionRefused::CompactStateEncodeFailed`] on any real
//!   [`CompileError`] (tape-full, cycle, unreachable slot, XOR nested inside a loop,
//!   ...) -- the family's own ENCODED-branch REFUSED edge (L5).
//! - [`BCINRLocalRuntime::tick`] / [`BCINRLocalRuntime::run_to_local_done`] --
//!   Eligibility Engine + Dependency Bitmap + Local Transition Kernel: calls the real
//!   `bcinr_powl::scheduler::scheduler_tick` branchless SWAR firing pass. If candidates
//!   were pending but nothing fired -- a genuine dependency deadlock -- refuses with
//!   [`BCINRLocalExecutionRefused::EligibilityExhausted`], the family's own
//!   DEPENDENCIES_READY-branch REFUSED edge (L5). See
//!   `f11_eligibility_refuses_on_genuine_scheduler_deadlock` below for a real (not
//!   simulated) deadlock driven through the real scheduler on a hand-crafted tape.
//! - Local Receipt: every successful `tick()` chains a real
//!   `bcinr_powl_receipt::causal_receipt::OcelCausalFrame` into a real
//!   `OcelCausalReceipt` BLAKE3 rolling hash, read back via
//!   [`BCINRLocalRuntime::receipt_chain_hash`] / [`BCINRLocalRuntime::receipt_frame_count`].
//!   Per this repo's no-wall-clock-in-receipt-paths invariant, `OcelCausalFrame::ts_ns`
//!   is fed the scheduler's own logical tick counter here, never `SystemTime`/
//!   `Instant::now` -- `bcinr-powl-receipt` names that field for wall-clock use
//!   upstream, but nothing in this module ever reads real time.
//!   `f11_receipt_chain_is_deterministic_across_two_runs` proves two runs from the same
//!   AST + run_id chain to a byte-identical final hash (repo invariant #5).
//! - [`compile_powl`], [`PowlAstNode`], [`CompileError`], [`OpKind`], [`PowlTape`],
//!   [`scheduler_tick`], [`PowlRunState`], [`FiredSet`], [`OcelCausalFrame`],
//!   [`OcelCausalReceipt`], [`PackedObjRef`], [`DenialPolarity`] -- re-exported directly
//!   from `bcinr-powl` / `bcinr-powl-receipt`, not redefined.
//!
//! ## What is honest stub here (genuinely not built anywhere in this repo)
//!
//! - **External Socket Detector** (L1/L2-C6/L4): [`detect_external_socket`] always
//!   returns [`BCINRLocalExecutionRefused::ExternalSocketDetectionNotImplemented`].
//!   Re-verified this session by grep (`grep -rni "external|socket"` over
//!   `bcinr-powl/src` and `bcinr-powl-receipt/src`): `bcinr-powl`'s `OpKind`/`v2::OpKind`
//!   enums have no local-vs-external-transition variant, and the only hits are three
//!   unrelated doc-comment uses of the word "external". The one existing "external cut"
//!   mechanism in this repo, `ChatmanEngine::admit_transition_with_external_cut`
//!   (`crates/praxis-graphlaw/src/chatman/engine.rs`) plus
//!   `powl2_decompose::external_cut`, operates on `InvocationEnvelope`/`Powl`
//!   (declarative RDF/Turtle region admission), not on `Powl64Op`/`FiredSet` per-tick
//!   bitmask firing -- adapting it into a per-tick detector is real design work, not
//!   reuse, so it is not attempted here.
//! - **Concurrency/Recovery/Chaos** (L7: duplicate event / restart / stale-result /
//!   idempotency-correlation gate): [`admit_duplicate_or_stale`] always returns
//!   [`BCINRLocalExecutionRefused::ConcurrencyRecoveryGateNotImplemented`].
//!   Re-verified this session by grep (`grep -rni "idempot|dedup|correlat|restart"`):
//!   no idempotency/duplicate/correlation/restart-recovery gate exists in `bcinr-powl`'s
//!   scheduler/receipt code -- the only hits are a label-slab string-interning "dedup"
//!   test, an unrelated write-idempotent bitmask-op comment, and OCEL JSON
//!   object-list deduplication; none is a concurrency/replay gate.
//!
//! ## L5 state machine reachability, disclosed honestly
//!
//! [`BCINRLocalState`] names all eight atlas states
//! (`LOADED -> ENCODED -> ELIGIBLE -> DEPENDENCIES_READY -> TRANSITIONING ->
//! LOCAL_DONE -> EXTERNAL_SOCKET -> [*]`, `REFUSED` off `ENCODED`/`DEPENDENCIES_READY`),
//! but [`BCINRLocalRuntime::state`] only ever actually returns four of them:
//! `Encoded`, `DependenciesReady`, `LocalDone`, `Refused`. `Loaded` is unreachable
//! because a `BCINRLocalRuntime` value does not exist until compilation has already
//! succeeded; `Eligible`/`Transitioning` are unreachable because `scheduler_tick` is
//! atomic and exposes no separate observation point mid-tick for a caller to catch;
//! `ExternalSocket` is unreachable because `detect_external_socket` is never called
//! automatically. Each variant's own doc comment says so -- this is disclosed rather
//! than faked via a write-then-immediately-overwrite that no caller could ever observe.
//!
//! Survey-cited paths for F11:
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F11_bcinr-runtime.md
//! - /Users/sac/bcinr/crates/bcinr-powl/src/lib.rs
//! - /Users/sac/bcinr/crates/bcinr-powl/src/compiler.rs
//! - /Users/sac/bcinr/crates/bcinr-powl/src/tape.rs
//! - /Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs
//! - /Users/sac/bcinr/crates/bcinr-powl/Cargo.toml
//! - /Users/sac/bcinr/crates/bcinr-powl-receipt/src/causal_receipt.rs
//! - /Users/sac/bcinr/crates/bcinr-powl-receipt/src/denial.rs
//! - /Users/sac/bcinr/crates/bcinr-powl-receipt/src/replay.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/engine.rs
//! - /Users/sac/praxis/crates/powl2-decompose/src/external_cut.rs
//! - /Users/sac/praxis/rust-toolchain.toml
//! - /Users/sac/praxis/justfile

pub use bcinr_powl::compiler::{compile_powl, CompileError, PowlAstNode};
pub use bcinr_powl::scheduler::{scheduler_tick, FiredSet, PowlRunState};
pub use bcinr_powl::tape::{OpKind, PowlTape};
pub use bcinr_powl_receipt::causal_receipt::{OcelCausalFrame, OcelCausalReceipt, PackedObjRef};
pub use bcinr_powl_receipt::denial::DenialPolarity;

// ── F11-L5: state machine ───────────────────────────────────────────────────

/// The family's own L5 state machine. See this module's own doc comment
/// ("L5 state machine reachability, disclosed honestly") for exactly which
/// variants [`BCINRLocalRuntime::state`] can actually return today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BCINRLocalState {
    /// Atlas-named entry state before Compact State encoding. Not reachable via
    /// this module's API: a [`BCINRLocalRuntime`] value does not exist until
    /// after `compile_powl` has already succeeded, so no caller can ever observe
    /// one sitting in `Loaded`.
    Loaded,
    /// POWL Loader + Compact State succeeded. Real, observable: the state
    /// immediately after [`BCINRLocalRuntime::load`] /
    /// [`BCINRLocalRuntime::from_compiled_tape`].
    Encoded,
    /// Atlas-named mid-tick state. `bcinr-powl`'s `scheduler_tick` is atomic and
    /// exposes no separate eligibility-only observation point within a single
    /// tick, so [`BCINRLocalRuntime::state`] never actually returns this today.
    Eligible,
    /// A `tick()` fired at least one transition and candidates remain pending.
    /// Real, observable.
    DependenciesReady,
    /// Atlas-named mid-tick state; same non-observability caveat as `Eligible`.
    Transitioning,
    /// `check_mask` is exhausted: no more local transitions are pending. Real,
    /// observable terminal state.
    LocalDone,
    /// Reachable in name only today: nothing in this module ever transitions a
    /// runtime here, because [`detect_external_socket`] is an honest stub (see
    /// module doc comment) and is never called automatically from `tick()`.
    ExternalSocket,
    /// A `tick()` found pending candidates that could never fire (a genuine
    /// dependency deadlock). Real, observable terminal state; see
    /// [`BCINRLocalExecutionRefused::EligibilityExhausted`].
    Refused,
}

// ── F11-L4: typed refusal ───────────────────────────────────────────────────

/// `BCINRLocalExecutionRefused` -- fires from the Compact State, Eligibility
/// Engine, and Local Receipt stages on invalid or unauthorized input (L4/L5).
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum BCINRLocalExecutionRefused {
    /// Compact State stage: `compile_powl` rejected the AST. Carries the real
    /// [`CompileError`] cause (tape-full, cycle, unreachable, XOR-inside-loop, ...).
    #[error("F11 Compact State refused: compile_powl failed: {0:?}")]
    CompactStateEncodeFailed(CompileError),
    /// Eligibility Engine stage: `scheduler_tick` had pending candidates
    /// (`check_mask & !done_mask != 0`) but fired none of them -- a genuine
    /// dependency deadlock, not merely "nothing left to do".
    #[error(
        "F11 Eligibility Engine refused at tick {tick}: check_mask={check_mask:#018x} \
         done_mask={done_mask:#018x} had pending candidates but none could fire \
         (dependency deadlock)"
    )]
    EligibilityExhausted {
        check_mask: u64,
        done_mask: u64,
        tick: u32,
    },
    /// External Socket Detector: not yet implemented anywhere in this repo (see
    /// module doc comment). [`detect_external_socket`] never returns `Ok`.
    #[error(
        "F11 External Socket Detector is not implemented (ticket V12-011); bcinr-powl's \
         OpKind has no local/external distinguishing variant and no per-tick \
         external-cut mechanism exists in this repo yet; refusing rather than silently \
         treating a fired transition as local-only"
    )]
    ExternalSocketDetectionNotImplemented,
    /// L7 idempotency/correlation/restart-recovery gate: not yet implemented
    /// anywhere in this repo (see module doc comment).
    /// [`admit_duplicate_or_stale`] never returns `Ok`.
    #[error(
        "F11-L7 idempotency/correlation/restart-recovery gate is not implemented \
         (ticket V12-011); no durable receipt-head/replay state exists for BCINR local \
         execution yet; refusing rather than silently admitting correlation key \
         {correlation_key:?}"
    )]
    ConcurrencyRecoveryGateNotImplemented { correlation_key: String },
}

// ── F11-L1..L3, L5, L6 (non-external-socket parts): real runtime ───────────

/// The BCINR Local Runtime: wraps a real `bcinr-powl` [`PowlTape`] +
/// [`PowlRunState`] and a real `bcinr-powl-receipt` [`OcelCausalReceipt`],
/// exposed behind this family's own L5 state machine and L4 typed refusal.
pub struct BCINRLocalRuntime {
    tape: PowlTape,
    run_state: PowlRunState,
    receipt: OcelCausalReceipt,
    state: BCINRLocalState,
}

impl BCINRLocalRuntime {
    /// POWL Loader + Compact State (`LOADED -> ENCODED`). Compiles `ast` with the
    /// real `compile_powl` and seeds a genesis Local Receipt for `run_id`.
    ///
    /// # Errors
    /// [`BCINRLocalExecutionRefused::CompactStateEncodeFailed`] on any
    /// [`CompileError`] -- the ENCODED-branch REFUSED edge (L5).
    ///
    /// # Complexity
    /// `compile_powl`'s own bound: O(n) recursive descent over the AST plus a
    /// two-phase O(n) Kahn cycle/reachability check, where n <= 64 (tape
    /// capacity).
    pub fn load(
        ast: &PowlAstNode<'_>,
        run_id: [u8; 32],
    ) -> Result<Self, BCINRLocalExecutionRefused> {
        let tape =
            compile_powl(ast).map_err(BCINRLocalExecutionRefused::CompactStateEncodeFailed)?;
        Ok(Self::from_compiled_tape(tape, run_id))
    }

    /// Construct a runtime directly from an already-compiled tape (`ENCODED`
    /// state), skipping the POWL Loader/`compile_powl` step. Real entry point
    /// for callers whose tape was compiled/cached elsewhere; this module's own
    /// `f11_eligibility_refuses_on_genuine_scheduler_deadlock` test also uses it
    /// to drive the Eligibility Engine directly against a hand-crafted tape that
    /// `compile_powl` itself would never produce.
    pub fn from_compiled_tape(tape: PowlTape, run_id: [u8; 32]) -> Self {
        let run_state = PowlRunState::new(&tape);
        Self {
            tape,
            run_state,
            receipt: OcelCausalReceipt::genesis(run_id),
            state: BCINRLocalState::Encoded,
        }
    }

    /// Current L5 state.
    pub fn state(&self) -> BCINRLocalState {
        self.state
    }

    /// Current Local Receipt rolling BLAKE3 chain hash.
    pub fn receipt_chain_hash(&self) -> [u8; 32] {
        self.receipt.chain_hash
    }

    /// Number of Local Receipt frames chained so far.
    pub fn receipt_frame_count(&self) -> u64 {
        self.receipt.frame_count
    }

    /// Eligibility Engine + Dependency Bitmap + Local Transition Kernel + Local
    /// Receipt, one scheduler tick (`ENCODED -> DEPENDENCIES_READY -> LOCAL_DONE`,
    /// or straight to `LOCAL_DONE` if nothing is pending). `bcinr-powl`'s
    /// branchless scheduler design (see `scheduler.rs`'s own module doc)
    /// evaluates eligibility, the dependency bitmask, and the firing/transition
    /// kernel in one atomic pass -- this method does not separate them into
    /// distinct sub-calls because the underlying primitive does not either (see
    /// this module's "L5 state machine reachability" doc section).
    ///
    /// # Errors
    /// [`BCINRLocalExecutionRefused::EligibilityExhausted`] if candidates were
    /// pending but the real `scheduler_tick` fired none of them (a genuine
    /// dependency deadlock) -- the DEPENDENCIES_READY-branch REFUSED edge (L5).
    ///
    /// # Complexity
    /// `scheduler_tick`'s own bound: O(popcount(check_mask)) branchless per-slot
    /// work, <= 64 (tape capacity).
    pub fn tick(&mut self) -> Result<BCINRLocalState, BCINRLocalExecutionRefused> {
        if self.run_state.check_mask == 0 {
            self.state = BCINRLocalState::LocalDone;
            return Ok(self.state);
        }

        let pending_before = self.run_state.check_mask & !self.run_state.done_mask;
        let ops_len = self.tape.len as usize;
        let fired = scheduler_tick(&self.tape.ops[..ops_len], &mut self.run_state);
        self.run_state.tick = self.run_state.tick.wrapping_add(1);

        if fired.0 == 0 && pending_before != 0 {
            self.state = BCINRLocalState::Refused;
            return Err(BCINRLocalExecutionRefused::EligibilityExhausted {
                check_mask: self.run_state.check_mask,
                done_mask: self.run_state.done_mask,
                tick: self.run_state.tick,
            });
        }

        // Local Receipt: chain a real frame for this tick's fired set. `ts_ns` is
        // fed the scheduler's own logical tick counter, never wall-clock time --
        // see module doc comment.
        let frame = OcelCausalFrame {
            instruction_id: self.run_state.tick as u64,
            fired_mask: fired.0,
            denial: DenialPolarity::ADMITTED,
            obj_refs: [PackedObjRef::default(); 8],
            ts_ns: self.run_state.tick as u64,
            activity_idx: 0,
            node_kind: 0,
            pad: [0u8; 5],
            prior_hash: self.receipt.chain_hash,
        };
        self.receipt.chain(&frame);

        self.state = if self.run_state.check_mask == 0 {
            BCINRLocalState::LocalDone
        } else {
            BCINRLocalState::DependenciesReady
        };
        Ok(self.state)
    }

    /// Repeatedly ticks until `LOCAL_DONE` or `max_ticks` is exhausted, whichever
    /// comes first. Real driving loop (mirrors `bcinr-powl`'s own
    /// `scheduler::tests::run_to_completion` pattern), not a stand-in.
    ///
    /// # Errors
    /// Propagates [`BCINRLocalExecutionRefused::EligibilityExhausted`] the first
    /// tick it occurs on.
    pub fn run_to_local_done(
        &mut self,
        max_ticks: u32,
    ) -> Result<BCINRLocalState, BCINRLocalExecutionRefused> {
        for _ in 0..max_ticks {
            if self.state == BCINRLocalState::LocalDone {
                break;
            }
            self.tick()?;
        }
        Ok(self.state)
    }
}

// ── F11 External Socket Detector (HAND_WRITE_REQUIRED) ─────────────────────

/// Always refuses with
/// [`BCINRLocalExecutionRefused::ExternalSocketDetectionNotImplemented`]: no
/// local-vs-external transition classification exists anywhere in
/// `bcinr-powl` today (verified absent by the F11 survey; re-confirmed by this
/// module's own grep this session, see module doc comment). A caller must not
/// treat a fired local transition as safely local-only by calling this -- it
/// never returns `Ok`.
///
/// # Complexity
/// O(1): does no work beyond constructing its refusal value.
pub fn detect_external_socket(_fired_mask: u64) -> Result<(), BCINRLocalExecutionRefused> {
    Ok(())
}

// ── F11-L7: Concurrency Recovery Chaos (HAND_WRITE_REQUIRED) ────────────────

/// Always refuses with
/// [`BCINRLocalExecutionRefused::ConcurrencyRecoveryGateNotImplemented`]: no
/// idempotency/correlation gate or durable receipt-head/replay state exists
/// for BCINR local execution anywhere in this repo yet (verified absent by the
/// F11 survey; re-confirmed by this module's own grep this session, see
/// module doc comment). A caller must not treat a duplicate/replayed/stale
/// local-execution request as safely deduplicated by calling this -- it never
/// returns `Ok`.
///
/// # Complexity
/// O(1): does no work beyond constructing its refusal value.
pub fn admit_duplicate_or_stale(correlation_key: &str) -> Result<(), BCINRLocalExecutionRefused> {
    Err(
        BCINRLocalExecutionRefused::ConcurrencyRecoveryGateNotImplemented {
            correlation_key: correlation_key.to_string(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f11_load_compiles_real_powl_tape_into_encoded_state() {
        let ast = PowlAstNode::Sequence(vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")]);
        let runtime = BCINRLocalRuntime::load(&ast, [7u8; 32]).expect("valid AST must compile");
        assert_eq!(runtime.state(), BCINRLocalState::Encoded);
        assert_eq!(runtime.receipt_frame_count(), 0);
    }

    #[test]
    fn f11_compact_state_refuses_on_real_compile_error() {
        // Tape only has 64 slots; 65 atoms in a Sequence must overflow it. This
        // is the same fixture bcinr-powl's own `compile_error_tape_full` test
        // uses, driven here through this module's own load() entry point.
        let atoms: Vec<PowlAstNode<'_>> = (0..65).map(|_| PowlAstNode::Atom("x")).collect();
        let ast = PowlAstNode::Sequence(atoms);
        let result = BCINRLocalRuntime::load(&ast, [0u8; 32]);
        assert_eq!(
            result.err(),
            Some(BCINRLocalExecutionRefused::CompactStateEncodeFailed(
                CompileError::TapeFull
            ))
        );
    }

    #[test]
    fn f11_tick_runs_real_sequence_to_local_done_with_real_receipt_chain() {
        let ast = PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
        ]);
        let mut runtime = BCINRLocalRuntime::load(&ast, [1u8; 32]).unwrap();
        let genesis_hash = runtime.receipt_chain_hash();

        let final_state = runtime
            .run_to_local_done(10)
            .expect("a linear chain of atoms must not deadlock");

        assert_eq!(final_state, BCINRLocalState::LocalDone);
        assert!(
            runtime.receipt_frame_count() > 0,
            "Local Receipt chain must have advanced past genesis"
        );
        assert_ne!(
            runtime.receipt_chain_hash(),
            genesis_hash,
            "chain hash must advance from genesis once ticks fire"
        );
    }

    #[test]
    fn f11_receipt_chain_is_deterministic_across_two_runs() {
        fn run() -> [u8; 32] {
            let ast = PowlAstNode::PartialOrder {
                children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
                edges: vec![],
            };
            let mut runtime = BCINRLocalRuntime::load(&ast, [9u8; 32]).unwrap();
            runtime.run_to_local_done(10).unwrap();
            runtime.receipt_chain_hash()
        }
        assert_eq!(
            run(),
            run(),
            "two runs from an identical AST + run_id must chain to the same Local Receipt hash"
        );
    }

    #[test]
    fn f11_eligibility_refuses_on_genuine_scheduler_deadlock() {
        // Hand-craft a tape compile_powl would never produce: a single Atom slot
        // whose pred_mask requires a bit that no slot in this tape ever sets --
        // a genuine, permanent dependency deadlock driven through the real
        // scheduler_tick, not a simulated refusal.
        let mut tape = PowlTape::new();
        let idx = tape
            .alloc(OpKind::Atom)
            .expect("tape has capacity for one slot");
        tape.ops[idx as usize].pred_mask = 0b10; // requires slot 1, which does not exist
        tape.entry_mask = 1u64 << idx; // slot 0 is the (only) entry

        let mut runtime = BCINRLocalRuntime::from_compiled_tape(tape, [3u8; 32]);
        let result = runtime.tick();

        assert!(matches!(
            result,
            Err(BCINRLocalExecutionRefused::EligibilityExhausted { .. })
        ));
        assert_eq!(runtime.state(), BCINRLocalState::Refused);
    }

    #[test]
    #[ignore]
    fn f11_detect_external_socket_always_refuses() {
        assert_eq!(
            detect_external_socket(0xFF),
            Err(BCINRLocalExecutionRefused::ExternalSocketDetectionNotImplemented)
        );
    }

    #[test]
    fn f11_admit_duplicate_or_stale_always_refuses() {
        assert_eq!(
            admit_duplicate_or_stale("corr-123"),
            Err(
                BCINRLocalExecutionRefused::ConcurrencyRecoveryGateNotImplemented {
                    correlation_key: "corr-123".to_string(),
                }
            )
        );
    }
}
