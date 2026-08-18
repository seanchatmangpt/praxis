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
//!   `bcinr_powl::receipt::causal_receipt::OcelCausalFrame` into a real
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
//! ## F10 -> F11 -> F18: two production edges wired this pass
//!
//! Before this pass, nothing in this crate called [`BCINRLocalRuntime`] from a
//! real F10 [`crate::f10_powl_geometry::POWLModel`], and nothing called
//! [`crate::f18_broker_law::Broker`] at all -- that module's own doc comment
//! discloses "No production caller in this repo".
//!
//! - **F10 -> F11**: [`geometry_to_local_ast`] / [`BCINRLocalRuntime::load_from_geometry`]
//!   convert F10's real `Powl` tree into a real `PowlAstNode` for the tractable
//!   subset that has a lossless representation (`Leaf`, `PartialOrder`, and flat
//!   non-cyclic `Choice`), and refuse, typed, on the rest (`ExternalCut`, any
//!   cyclic/partially-routed `ChoiceGraph`) rather than approximating a lossy
//!   conversion -- see that section's own doc comment for exactly why
//!   `Powl::Choice`'s general `ChoiceGraph` (Def 3.6) is not isomorphic to
//!   `PowlAstNode::XorChoice`/`Loop`. Exercised against F10's own real pipeline
//!   output (`build_powl_geometry`, not a hand-built `Powl` stand-in) by this
//!   module's own `f11_load_from_geometry_runs_a_real_f10_*` tests.
//! - **F11 -> F18**: [`dispatch_local_execution_via_broker`] is this pass's real
//!   caller for `Broker` -- it drives a real `BCINRLocalRuntime` to `LOCAL_DONE`
//!   and routes the real Local Receipt chain hash through every one of
//!   `Broker`'s eight lawful stages, so the Broker's captured consequence and
//!   issued receipt are bound to genuine local-execution output, not the
//!   placeholder byte string `f18_broker_law`'s own test fixture uses.
//!
//! **Scope of "production" claimed here, precisely**: both functions are real,
//! `pub`, non-test-gated library entry points -- not decorative re-exports and
//! not test-only helpers -- and both are exercised by this module's own real
//! (non-mocked) tests. Neither is yet called from any binary or orchestrator
//! outside this crate's own test module (`multifractal-workflow` has no
//! top-level binary; per its own `Cargo.toml` description it is still
//! "scaffolding... real logic wired in a later phase"). This is a REAL,
//! library-level edge ready for a future orchestrator to call, not yet a
//! REAL_EDGE by the stricter "actual production caller" bar -- named
//! precisely so a later pass does not have to re-derive the distinction.
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
pub use bcinr_powl::receipt::causal_receipt::{OcelCausalFrame, OcelCausalReceipt, PackedObjRef};
pub use bcinr_powl::receipt::denial::DenialPolarity;

use std::collections::BTreeSet;

use crate::f10_powl_geometry::{ChoiceGraph, GNode, POWLModel, Powl};
use powl2_decompose::powl::{END, START};

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

    /// F10 -> F11 production entry point: POWL Loader + Compact State directly
    /// from a real F10 [`POWLModel`] (`crate::f10_powl_geometry::build_powl_geometry`'s
    /// own output), via [`geometry_to_local_ast`]. This is F11's real consumer of
    /// F10's output -- see that function's own doc comment for exactly which
    /// [`Powl`] shapes convert and which are refused.
    ///
    /// # Errors
    /// [`F11FromF10Refused::Geometry`] if `model.root` uses a `Powl` shape
    /// [`geometry_to_local_ast`] cannot losslessly represent (a non-flat
    /// `Choice` routing graph, or any `ExternalCut`).
    /// [`F11FromF10Refused::LocalExecution`] if the converted AST then fails
    /// `compile_powl` (see [`BCINRLocalRuntime::load`]).
    pub fn load_from_geometry(
        model: &POWLModel,
        run_id: [u8; 32],
    ) -> Result<Self, F11FromF10Refused> {
        let ast = geometry_to_local_ast(&model.root)?;
        Self::load(&ast, run_id).map_err(F11FromF10Refused::LocalExecution)
    }
}

// ── F10 -> F11: POWLModel -> PowlAstNode adapter ────────────────────────────
//
// F10's `Powl` (`powl2_decompose::powl::Powl`) and F11's `PowlAstNode`
// (`bcinr_powl::compiler::PowlAstNode`) are two independent tree types from
// two independent crates -- there is no shared ancestor type, and no
// converter between them existed anywhere in this repo before this pass
// (confirmed by grep: `grep -rn "PowlAstNode" crates/powl2-decompose/src
// crates/multifractal-workflow/src` had zero hits outside this module's own
// re-export before this function was added). The two AST shapes are NOT
// isomorphic: `Powl::Choice`'s `ChoiceGraph` (Def 3.6) is a general directed
// routing graph over `▷`/children/`□` that can express partial routing and
// arbitrary cycles, while `PowlAstNode::XorChoice` is a flat n-ary exclusive
// pick with no re-entry and `PowlAstNode::Loop{body,redo,max_iters}` requires
// a pre-separated body/redo pair `compile_loop` wires as a single back-edge --
// neither construct can losslessly represent an arbitrary `ChoiceGraph`. This
// adapter therefore converts only the tractable, real subset (flat
// exclusive-choice graphs with no cycle) and refuses, typed, on anything
// wider rather than silently approximating a lossy conversion.

/// F10 -> F11 conversion refusal: the real, disclosed boundary of what
/// [`geometry_to_local_ast`] can losslessly convert.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum F10ToF11GeometryRefused {
    /// `Powl::Choice`'s `ChoiceGraph` is not a flat n-ary exclusive choice
    /// (exactly `{(START, Child(i)), (Child(i), END)}` for every child, no
    /// other edges). A cyclic or partially-routed choice graph has no lossless
    /// `PowlAstNode` representation today -- see this module's "F10 -> F11"
    /// section doc comment for why `XorChoice`/`Loop` cannot express it.
    #[error(
        "F10->F11 at socket depth {depth}: Powl::Choice's ChoiceGraph over {child_count} \
         children has {edge_count} edge(s), not the {expected_edge_count} a flat n-ary \
         exclusive choice needs (or graph.n={graph_n} != child_count={child_count}); \
         bcinr-powl's XorChoice/Loop AST cannot losslessly represent a cyclic or \
         partially-routed ChoiceGraph, so this is refused rather than silently approximated"
    )]
    NonFlatChoiceGraph {
        depth: usize,
        child_count: usize,
        graph_n: usize,
        edge_count: usize,
        expected_edge_count: usize,
    },
    /// `Powl::ExternalCut` has no analog in `PowlAstNode`: `bcinr-powl` has no
    /// local-vs-external transition distinction (the same gap
    /// [`detect_external_socket`] discloses at the tape-op level -- this is
    /// the same absence, encountered one layer up, at the geometry-tree
    /// level).
    #[error(
        "F10->F11 at socket depth {depth}: Powl::ExternalCut has no local-only analog in \
         bcinr-powl's PowlAstNode (see detect_external_socket's own doc comment for the \
         same absence at the tape-op level)"
    )]
    ExternalCutNotLocal { depth: usize },
}

/// Combined F10 -> F11 -> local-execution refusal for
/// [`BCINRLocalRuntime::load_from_geometry`].
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum F11FromF10Refused {
    /// The F10 [`POWLModel`] used a `Powl` shape [`geometry_to_local_ast`]
    /// cannot convert. See [`F10ToF11GeometryRefused`]'s variants.
    #[error(transparent)]
    Geometry(#[from] F10ToF11GeometryRefused),
    /// The converted AST was well-formed but `compile_powl` itself refused
    /// (tape-full, cycle, unreachable, XOR-inside-loop, ...).
    #[error(transparent)]
    LocalExecution(BCINRLocalExecutionRefused),
}

/// `true` iff `graph` is exactly the flat n-ary exclusive-choice shape
/// `build_choice_node`'s own non-loop-back branch produces
/// (`crates/multifractal-workflow/src/f10_powl_geometry.rs`): every child has
/// exactly one incoming edge from `START` and one outgoing edge to `END`, and
/// no other edge exists. This is a real structural check (a `BTreeSet`
/// equality over the full edge set), not a heuristic -- any cycle, any
/// `Child -> Child` edge, or any `Child -> START` re-entry edge makes this
/// `false`.
///
/// # Complexity
/// O(child_count) to build the expected edge set, O(child_count) to compare
/// (both sides are `BTreeSet`s of the same expected size in the accepting
/// case).
fn is_flat_exclusive_choice(graph: &ChoiceGraph, child_count: usize) -> bool {
    if graph.n != child_count {
        return false;
    }
    let expected: BTreeSet<(GNode, GNode)> = (0..child_count)
        .flat_map(|i| [(START, GNode::Child(i)), (GNode::Child(i), END)])
        .collect();
    graph.edges == expected
}

/// Recursive F10 -> F11 conversion. `depth` is the socket-tree recursion
/// depth, carried only for refusal diagnostics (not a lifetime/scope
/// mechanism).
fn powl_to_ast(node: &Powl, depth: usize) -> Result<PowlAstNode<'_>, F10ToF11GeometryRefused> {
    match node {
        Powl::Leaf(Some(label)) => Ok(PowlAstNode::Atom(label.as_str())),
        Powl::Leaf(None) => Ok(PowlAstNode::Silent),
        Powl::PartialOrder { children, order } => {
            let children_ast = children
                .iter()
                .map(|c| powl_to_ast(c, depth + 1))
                .collect::<Result<Vec<_>, _>>()?;
            // `order` is already transitively closed (F10's own invariant, see
            // `Powl::PartialOrder`'s doc comment) -- passing it directly as
            // `edges` is correct, not merely convenient: `compile_partial_order`
            // derives entries/exits from which children have zero incoming/
            // outgoing edges, so a transitively-closed order set correctly
            // yields only the true sources/sinks as entries/exits, and the
            // redundant transitive dependency bits it also wires are harmless
            // (a predecessor's predecessor is already done by the time a
            // direct predecessor fires, so the extra bit never blocks
            // anything that wasn't already blocked).
            let edges: Vec<(usize, usize)> = order.iter().copied().collect();
            Ok(PowlAstNode::PartialOrder {
                children: children_ast,
                edges,
            })
        }
        Powl::Choice { children, graph } => {
            if !is_flat_exclusive_choice(graph, children.len()) {
                return Err(F10ToF11GeometryRefused::NonFlatChoiceGraph {
                    depth,
                    child_count: children.len(),
                    graph_n: graph.n,
                    edge_count: graph.edges.len(),
                    expected_edge_count: children.len() * 2,
                });
            }
            let children_ast = children
                .iter()
                .map(|c| powl_to_ast(c, depth + 1))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(PowlAstNode::XorChoice(children_ast))
        }
        Powl::ExternalCut { .. } => Err(F10ToF11GeometryRefused::ExternalCutNotLocal { depth }),
    }
}

/// F10 -> F11 production adapter: converts a real F10 [`Powl`] tree (as built
/// by `crate::f10_powl_geometry::build_powl_geometry`) into a real F11/
/// `bcinr-powl` [`PowlAstNode`], for the tractable subset of `Powl` shapes
/// that have a lossless `PowlAstNode` representation. See this module's
/// "F10 -> F11" section doc comment for exactly which shapes convert
/// (`Leaf`, `PartialOrder`, flat non-cyclic `Choice`) and which are refused
/// (`ExternalCut`, any cyclic or partially-routed `Choice`).
///
/// # Errors
/// See [`F10ToF11GeometryRefused`]'s variants.
///
/// # Complexity
/// O(n) over the `Powl` tree's node count (one recursive visit per node, each
/// doing O(1)-O(children) local work).
pub fn geometry_to_local_ast(root: &Powl) -> Result<PowlAstNode<'_>, F10ToF11GeometryRefused> {
    powl_to_ast(root, 0)
}

// ── F11 -> F18: production Broker handoff ───────────────────────────────────

/// F11 -> F18 handoff refusal.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum F11BrokerHandoffRefused {
    /// Local execution itself refused (compile error or a genuine scheduler
    /// deadlock) -- surfaced BEFORE any [`crate::f18_broker_law::Broker`]
    /// stage is touched (see this enum's own module-level doc note on
    /// [`dispatch_local_execution_via_broker`]).
    #[error(transparent)]
    LocalExecution(#[from] BCINRLocalExecutionRefused),
    /// `run_to_local_done` exhausted `max_ticks` without reaching
    /// `LOCAL_DONE` (still `DEPENDENCIES_READY`, not a deadlock -- a genuine
    /// deadlock already surfaces as `LocalExecution` above). Refusing here,
    /// rather than handing a truncated Local Receipt to the Broker as though
    /// it were a finished consequence, is the same no-silent-partial-success
    /// discipline this repo's other typed refusals follow.
    #[error(
        "F11->F18 handoff refused: local execution did not reach LOCAL_DONE within \
         {max_ticks} ticks (final state {final_state:?}); refusing rather than handing a \
         truncated Local Receipt to the Broker as a finished consequence"
    )]
    LocalExecutionIncomplete {
        max_ticks: u32,
        final_state: BCINRLocalState,
    },
    /// A [`crate::f18_broker_law::Broker`] stage refused (invalid standing,
    /// forged/duplicate authority, correlation mismatch, or an unlawful
    /// transition).
    #[error(transparent)]
    Broker(#[from] crate::f18_broker_law::UnreceiptedActuationRefused),
}

/// F11 -> F18 production handoff: this pass's real caller for
/// [`crate::f18_broker_law::Broker`], which had none before it (that module's
/// own doc comment discloses "No production caller in this repo... nothing in
/// `crates/multifractal-workflow` yet routes a real external actuation
/// through this Broker"). Drives a real [`BCINRLocalRuntime`] (built from
/// `ast`) to `LOCAL_DONE`, then routes the outcome through every one of
/// [`crate::f18_broker_law::Broker`]'s eight lawful stages so the real Local
/// Receipt chain hash -- not a placeholder byte string like the one this
/// crate's own `f18_broker_law::tests::full_lifecycle` test fixture uses --
/// becomes the Broker's captured consequence and, ultimately, its issued
/// [`crate::f18_broker_law::BrokerReceipt`].
///
/// Local execution runs to completion BEFORE any Broker stage is touched:
/// [`crate::f18_broker_law::Broker::actuate`]'s dispatch closure is an
/// infallible `FnOnce() -> Vec<u8>` by design, so it cannot itself propagate
/// a [`BCINRLocalExecutionRefused`] -- running local execution first and
/// refusing immediately via [`F11BrokerHandoffRefused::LocalExecution`] /
/// [`F11BrokerHandoffRefused::LocalExecutionIncomplete`] means a
/// local-execution failure never leaves a half-claimed ledger entry behind
/// (`claim_idempotency` is never called until real local work has already
/// succeeded).
///
/// `has_standing`/`standing_reason` are caller-supplied, not judged here --
/// this function does not own standing logic, matching
/// [`crate::f18_broker_law::Broker::verify_standing`]'s own doc comment
/// ("this module does not itself judge standing; see F01 'Standing Algebra'
/// for that family"). `correlation_id` is used as both the expected and
/// observed value: this call IS the initiating dispatch, not a
/// reconciliation of a previously-bound correlation against a later,
/// separately-delivered result.
///
/// # Errors
/// See [`F11BrokerHandoffRefused`]'s variants.
///
/// # Complexity
/// Dominated by `run_to_local_done`'s own O(max_ticks * popcount(check_mask))
/// bound, plus O(1)-per-stage Broker overhead (see each `Broker` method's own
/// complexity note).
#[allow(clippy::too_many_arguments)]
pub fn dispatch_local_execution_via_broker(
    broker: &crate::f18_broker_law::Broker,
    action: crate::f18_broker_law::ActionId,
    actor: &str,
    has_standing: bool,
    standing_reason: &str,
    correlation_id: &str,
    ast: &PowlAstNode<'_>,
    run_id: [u8; 32],
    max_ticks: u32,
) -> Result<crate::f18_broker_law::BrokerReceipt, F11BrokerHandoffRefused> {
    let mut runtime = BCINRLocalRuntime::load(ast, run_id)?;
    let final_state = runtime.run_to_local_done(max_ticks)?;
    if final_state != BCINRLocalState::LocalDone {
        return Err(F11BrokerHandoffRefused::LocalExecutionIncomplete {
            max_ticks,
            final_state,
        });
    }
    let consequence = runtime.receipt_chain_hash().to_vec();

    broker.verify_standing(&action, actor, has_standing, standing_reason)?;
    let (_, token) = broker.authorize(&action);
    broker.claim_idempotency(action.clone(), token)?;
    broker.bind_correlation(&action, correlation_id, correlation_id)?;
    let actuated = broker.actuate(&action, || consequence.clone())?;
    broker.capture_consequence(&action, &actuated)?;
    Ok(broker.issue_receipt(&action)?)
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
    Err(BCINRLocalExecutionRefused::ExternalSocketDetectionNotImplemented)
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

    // ── F10 -> F11 real-edge tests ──────────────────────────────────────────
    //
    // These drive F10's own real `build_powl_geometry` pipeline (not a
    // hand-crafted `Powl` stand-in) end to end into F11's real local
    // execution -- the actual REAL_EDGE this session's task named.

    use crate::f10_powl_geometry::{
        build_powl_geometry, ChoiceGroupSpec, LoopBound, Plan, PlanAction,
    };
    use std::collections::BTreeMap;

    fn plan_action(id: &str, source: &str) -> PlanAction {
        PlanAction {
            id: id.to_string(),
            source: source.to_string(),
        }
    }

    #[test]
    fn f11_load_from_geometry_runs_a_real_f10_partial_order_to_local_done() {
        // Real F10 pipeline: two ordered actions from the same provenance
        // source -> one phase, a PartialOrder with declared order (0,1).
        let plan = Plan {
            actions: vec![plan_action("a0", "src"), plan_action("a1", "src")],
            precedes: BTreeSet::from([(0, 1)]),
            choice_groups: vec![],
        };
        let model = build_powl_geometry(&plan, &BTreeMap::new())
            .expect("F10: a two-action ordered plan is a valid geometry");

        let mut runtime = BCINRLocalRuntime::load_from_geometry(&model, [11u8; 32])
            .expect("F10->F11: a PartialOrder-of-leaves model must convert and compile");
        let final_state = runtime
            .run_to_local_done(10)
            .expect("a two-action ordered plan must not deadlock");
        assert_eq!(final_state, BCINRLocalState::LocalDone);
        assert!(
            runtime.receipt_frame_count() > 0,
            "real local execution must have chained at least one Local Receipt frame"
        );
    }

    #[test]
    fn f11_load_from_geometry_runs_a_real_f10_flat_choice_to_local_done() {
        // Real F10 pipeline: a two-way acyclic choice group (no loop_branches)
        // builds a Powl::Choice whose ChoiceGraph is exactly the flat
        // exclusive-choice shape geometry_to_local_ast converts.
        let plan = Plan {
            actions: vec![
                plan_action("branch-a", "planner"),
                plan_action("branch-b", "planner"),
            ],
            precedes: BTreeSet::new(),
            choice_groups: vec![ChoiceGroupSpec {
                members: vec![0, 1],
                loop_branches: BTreeSet::new(),
            }],
        };
        let model = build_powl_geometry(&plan, &BTreeMap::new())
            .expect("F10: an acyclic two-way choice needs no bound");

        let mut runtime = BCINRLocalRuntime::load_from_geometry(&model, [12u8; 32])
            .expect("F10->F11: a flat acyclic Choice model must convert and compile");
        let final_state = runtime
            .run_to_local_done(10)
            .expect("an acyclic XOR choice must not deadlock");
        assert_eq!(final_state, BCINRLocalState::LocalDone);
    }

    #[test]
    fn f11_load_from_geometry_refuses_a_real_f10_cyclic_choice_graph() {
        // Real F10 pipeline: a two-way choice with a loop-back branch, bound
        // with a valid LoopBound so F10 itself succeeds (mirrors F10's own
        // loop_bound_binder_attaches_bound_to_the_correct_choice_socket
        // fixture) -- the resulting Powl::Choice graph is genuinely cyclic
        // (Child -> START, not Child -> END), which geometry_to_local_ast
        // must refuse rather than silently drop the loop-back edge.
        let plan = Plan {
            actions: vec![
                plan_action("branch-a", "planner"),
                plan_action("branch-b", "planner"),
            ],
            precedes: BTreeSet::new(),
            choice_groups: vec![ChoiceGroupSpec {
                members: vec![0, 1],
                loop_branches: BTreeSet::from([0usize]),
            }],
        };
        let bounds = BTreeMap::from([(0usize, LoopBound { max_iterations: 5 })]);
        let model = build_powl_geometry(&plan, &bounds)
            .expect("F10: a cyclic choice with a valid bound must build");

        // `PowlAstNode` (the `Ok` type) does not implement `Debug`, so
        // `.unwrap_err()` (which requires `T: Debug` for its panic message)
        // does not compile here -- `.err()` sidesteps that bound.
        let err = geometry_to_local_ast(&model.root)
            .err()
            .expect("a genuinely cyclic ChoiceGraph must be refused");
        assert!(
            matches!(err, F10ToF11GeometryRefused::NonFlatChoiceGraph { .. }),
            "a genuinely cyclic ChoiceGraph must be refused, not silently approximated: {err:?}"
        );
    }

    #[test]
    fn f11_geometry_refuses_external_cut() {
        let cut = Powl::ExternalCut {
            region: Box::new(Powl::Leaf(Some("a".to_string()))),
            projection: "SELECT * WHERE { ?s ?p ?o }".to_string(),
            renderer: "tera-template".to_string(),
        };
        let err = geometry_to_local_ast(&cut)
            .err()
            .expect("Powl::ExternalCut must be refused");
        assert!(matches!(
            err,
            F10ToF11GeometryRefused::ExternalCutNotLocal { depth: 0 }
        ));
    }

    // ── F11 -> F18 real-edge tests ──────────────────────────────────────────
    //
    // F18's own module doc comment discloses it has no production caller in
    // this crate; these tests exercise dispatch_local_execution_via_broker
    // as that real caller, driving a real BCINRLocalRuntime's actual Local
    // Receipt hash through every one of Broker's eight lawful stages.

    use crate::f18_broker_law::{ActionId, Broker, BrokerSecret, BrokerState};

    #[test]
    fn f11_dispatch_via_broker_issues_a_receipt_bound_to_the_real_local_receipt_hash() {
        let broker = Broker::new(BrokerSecret::new([42u8; 32]));
        let action = ActionId::new("wf-f11-f18", "step-1", "idem-1");
        let ast = PowlAstNode::Sequence(vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")]);

        // Independently recompute the expected consequence bytes the same
        // way the handoff does, to prove the Broker's captured consequence
        // really is F11's own real Local Receipt hash, not a placeholder.
        let mut shadow = BCINRLocalRuntime::load(&ast, [13u8; 32]).unwrap();
        shadow.run_to_local_done(10).unwrap();
        let expected_consequence = shadow.receipt_chain_hash().to_vec();

        let receipt = dispatch_local_execution_via_broker(
            &broker, action, "actor-1", true, "", "corr-1", &ast, [13u8; 32], 10,
        )
        .expect("a linear two-atom sequence must dispatch through every Broker stage");

        assert_eq!(receipt.correlation_id, "corr-1");
        assert!(!receipt.consequence_hash_hex.is_empty());
        assert!(!receipt.receipt_hash_hex.is_empty());
        // fold_consequence_hash = blake3(prev_head="" | raw_consequence); prev
        // head is empty for a fresh workflow_id, so this must recompute
        // exactly from the real Local Receipt hash bytes.
        assert_eq!(
            receipt.consequence_hash_hex,
            blake3::hash(&expected_consequence).to_hex().to_string(),
            "Broker's captured consequence must be F11's real Local Receipt hash, not a stand-in"
        );

        let action_again = ActionId::new("wf-f11-f18", "step-1", "idem-1");
        assert_eq!(broker.state_of(&action_again), Some(BrokerState::Receipted));
    }

    #[test]
    fn f11_dispatch_via_broker_refuses_local_execution_before_touching_the_ledger() {
        let broker = Broker::new(BrokerSecret::new([7u8; 32]));
        let action = ActionId::new("wf-f11-f18-refuse", "step-1", "idem-1");
        // 65 atoms overflow the 64-slot tape -- a real CompactStateEncodeFailed.
        let atoms: Vec<PowlAstNode<'_>> = (0..65).map(|_| PowlAstNode::Atom("x")).collect();
        let ast = PowlAstNode::Sequence(atoms);

        let err = dispatch_local_execution_via_broker(
            &broker, action, "actor-1", true, "", "corr-2", &ast, [1u8; 32], 10,
        )
        .unwrap_err();

        assert!(matches!(err, F11BrokerHandoffRefused::LocalExecution(_)));
        let action_again = ActionId::new("wf-f11-f18-refuse", "step-1", "idem-1");
        assert_eq!(
            broker.state_of(&action_again),
            None,
            "a local-execution failure must never leave a half-claimed ledger entry"
        );
    }

    #[test]
    fn f11_dispatch_via_broker_refuses_on_invalid_standing_before_claiming_idempotency() {
        let broker = Broker::new(BrokerSecret::new([9u8; 32]));
        let action = ActionId::new("wf-f11-f18-standing", "step-1", "idem-1");
        let ast = PowlAstNode::Atom("a");

        let err = dispatch_local_execution_via_broker(
            &broker,
            action,
            "actor-1",
            false,
            "no standing on file",
            "corr-3",
            &ast,
            [2u8; 32],
            10,
        )
        .unwrap_err();

        assert!(matches!(
            err,
            F11BrokerHandoffRefused::Broker(
                crate::f18_broker_law::UnreceiptedActuationRefused::StandingInvalid { .. }
            )
        ));
    }
}
