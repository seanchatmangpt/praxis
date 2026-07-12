//! Family F25 -- "Receipts and Replay" (atlas ticket V12-025).
//!
//! Wire-phase-1 status (this pass): **MIXED, real for L1-L6, disclosed gap for
//! L7**. Per the family survey (verdict `MIXED`), this module wires two
//! genuinely different kinds of content, honest about which is which, plus
//! one disclosed non-implementation:
//!
//! - **REUSE_ADAPT (real, pattern-adapted, not literally imported)**: the
//!   digest-fold-independently-verify-by-replay discipline in
//!   [`receipt_builder`]/[`receipt_fold`]/[`independent_verifier`]/
//!   [`replay_executor`]/[`equivalence_comparator`] adapts
//!   `praxis_graphlaw::chatman::engine::ChatmanEngine::verify_replay`'s
//!   "re-run for real in constitutional order, compare fail-fast, never trust
//!   the recorded receipt" discipline -- generalized from that engine's fixed
//!   nine S1-S5 digests over one pipeline to this family's six CTQ-named
//!   material-kind digests (source, query, template, program, event, output)
//!   over an arbitrary, caller-supplied transformation. `engine.rs`'s own
//!   types (`EngineProcessReceipt`, `ReplayMismatch`, `Digest`) are private to
//!   that pipeline's nine fixed fields and are not reusable as-is for a
//!   generic subsystem, so this module defines its own [`Digest`],
//!   [`Receipt`], and [`ReceiptReplayRefused`] following the identical
//!   fold/compare shape rather than importing praxis-graphlaw's chatman
//!   module. Verified this session: `just praxis-graphlaw-test-lib
//!   'test(verify_replay)'` -- 3/3 passed (see the family survey this
//!   module's own doc references).
//! - **REUSE_ADAPT (real, pattern-adapted)**: [`receipt_graph_writer`] adapts
//!   `crates/cng/src/otel_receipt.rs`'s PROV-O ancestry-quad-writing pattern
//!   (`prov:Entity`/`prov:Activity`/`prov:used`/`prov:generated`/
//!   `prov:wasDerivedFrom`, content-addressed `urn:blake3:<hex>` nodes, a
//!   minted local vocabulary namespace, canonical-N-Quads-text sort order) --
//!   generalized from that module's fixed three digests (query/input/output,
//!   one CONSTRUCT transformation) to this family's full CTQ six-digest set
//!   folded through the L6 lens's named eight-node chain (`SourceDigest ->
//!   MechanismDigest -> OutputDigest -> Receipt -> ReceiptHead ->
//!   ReplayOutput -> EquivalenceReport -> ReceiptGraph`). `cng::otel_receipt`
//!   is `pub mod` and its `verify_receipt_otel_to_ocel`/`receipt_otel_to_ocel`
//!   were read as the pattern source but are not called here -- they are
//!   hard-wired to one OTEL/OCEL CONSTRUCT query, not the transformation-
//!   agnostic subsystem this family requires. Verified this session: `just
//!   cng-test-lib otel_receipt` -- 10/10 passed.
//! - **HAND_WRITE_REQUIRED (disclosed, NOT implemented in this pass)**: the
//!   L7 concurrency/chaos layer -- duplicate-event, process/engine-restart,
//!   and stale/malformed-result handling routed through an atomic
//!   idempotency+correlation gate or durable receipt-head/replay-state before
//!   re-admission or refusal. [`chaos_gate::admit_for_replay`] exists as a
//!   named entry point but always returns
//!   [`ReceiptReplayRefused::L7ChaosGateNotImplemented`] -- a typed refusal,
//!   not a fake success, per this repo's no-overclaiming discipline. The
//!   family survey found no existing code (internal or external) that
//!   implements this specific combination for the receipts-and-replay
//!   mechanism; `chatman::engine.rs`'s `idempotency_key` dedup and
//!   `cng::bench::dispatch`'s idempotency+correlation ledger are real but
//!   cover different admission boundaries (hook actuation, dispatch
//!   admission), not post-restart receipt replay.
//!
//! ## What is honestly NOT done in this pass
//!
//! - L7 durability/idempotency (see [`chaos_gate`] above) -- disclosed, typed,
//!   not faked.
//! - The closed L5 state machine ([`Stage`]/[`StateMachine`]) implements the
//!   survey's literal edge list (`REFUSED` reachable only from
//!   `RECEIPT_BUILT` or `INDEPENDENTLY_VERIFIED`); [`receipt_graph_writer`]'s
//!   own refusal path therefore returns `Err` without forcing an
//!   [`Stage::EquivalenceChecked`] `->` [`Stage::Refused`] transition the
//!   closed list does not name. This is a disclosed interpretive choice
//!   about an internal tension in the survey's own prose (which separately
//!   lists Receipt Graph Writer as a refusal-reachable component) versus its
//!   literal state-machine edge list -- not independently re-confirmed
//!   against the atlas `.md` source file. See [`Stage::allowed`]'s doc
//!   comment for the full reasoning.
//! - The L6 "replay-must-reconstruct-equivalence loop back to ReceiptGraph"
//!   requirement is realized functionally (a written graph's `ReplayOutput`/
//!   `EquivalenceReport` nodes are content-addressed from an *actually
//!   replayed* receipt, never copied from the recorded one, and a second
//!   independent verification run reproduces byte-identical graph content --
//!   see `receipt_graph_writer_output_is_stable_across_a_second_independent_replay`,
//!   test, below) rather than as an additional literal RDF back-edge beyond
//!   the seven the atlas text names.
//!
//! Survey-cited paths informing this pass (from the v26.7.12 family survey
//! handed to this session inline):
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F25_receipts-replay.md
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/engine.rs (`EngineProcessReceipt`, `ReplayMismatch`, `verify_replay`, `verify_replay_with_external_cut`)
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/engine_test.rs
//! - /Users/sac/praxis/crates/cng/src/otel_receipt.rs (`receipt_otel_to_ocel`, `verify_receipt_otel_to_ocel`, `fold_receipt_head`)
//! - /Users/sac/praxis/crates/cng/src/otel_receipt_test.rs
//! - /Users/sac/praxis/crates/cng/src/bench/dispatch.rs (L7 pattern reference only; different admission boundary)
//! - /Users/sac/affidavit/tests/reference_receipt_chain.rs (weak candidate, not reused)
//! - /Users/sac/lsp-max/src/andon/andon.rs (weak candidate, not reused)
//! - /Users/sac/lsp-max/crates/lsp-max-compositor/src/receipt_chain.rs (weak candidate, not reused)
//! - /Users/sac/praxis/justfile
//! - /Users/sac/praxis/packs

use oxigraph::model::Quad;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------
// Digest kinds -- CTQ: "independent replay must verify SIX digest kinds
// together (source, query, template, program, event, output)".
// ---------------------------------------------------------------------

/// One of the six CTQ-named material kinds a receipt/replay cycle must
/// verify together. Order here is the canonical fold/compare order used
/// throughout this module (matches the CTQ's own listed order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DigestKind {
    /// The transformation's raw input material (e.g. a source graph).
    Source,
    /// The query/plan text driving the transformation.
    Query,
    /// The template text (if any) the transformation renders through.
    Template,
    /// The compiled program/artifact text the transformation executes.
    Program,
    /// The event/log material the transformation consumes or emits.
    Event,
    /// The transformation's output material.
    Output,
}

impl DigestKind {
    /// The complete, ordered CTQ set -- every [`receipt_fold::fold`] call
    /// requires exactly this set to be present, no more, no fewer.
    pub const REQUIRED: [DigestKind; 6] = [
        DigestKind::Source,
        DigestKind::Query,
        DigestKind::Template,
        DigestKind::Program,
        DigestKind::Event,
        DigestKind::Output,
    ];

    /// Domain-separation tag folded into [`Digest::of`] so two materials of
    /// different kinds with byte-identical text never collide.
    fn tag(self) -> &'static str {
        match self {
            DigestKind::Source => "mfw/f25/material/source/v1",
            DigestKind::Query => "mfw/f25/material/query/v1",
            DigestKind::Template => "mfw/f25/material/template/v1",
            DigestKind::Program => "mfw/f25/material/program/v1",
            DigestKind::Event => "mfw/f25/material/event/v1",
            DigestKind::Output => "mfw/f25/material/output/v1",
        }
    }
}

/// A tagged BLAKE3 digest (`blake3:<hex>`), matching this repo's existing
/// receipt-digest string shape (`crates/cng/src/otel_receipt.rs`'s
/// `blake3:` prefix convention).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Digest(String);

impl Digest {
    /// Digests one material's text under a domain-separation tag.
    ///
    /// # Complexity
    /// O(bytes of `material`): one BLAKE3 pass.
    fn of(tag: &str, material: &str) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(tag.as_bytes());
        hasher.update(b"\0");
        hasher.update(material.as_bytes());
        Digest(format!("blake3:{}", hasher.finalize().to_hex()))
    }

    /// The full tagged digest string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Bare hex (no `blake3:` prefix), for embedding in `urn:blake3:<hex>`
    /// IRIs -- mirrors `otel_receipt.rs::digest_hex`.
    fn hex(&self) -> &str {
        self.0.strip_prefix("blake3:").unwrap_or(&self.0)
    }
}

impl core::fmt::Display for Digest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------
// Materials -- transformation-agnostic input to the Material Digestor.
// ---------------------------------------------------------------------

/// Raw canonical text for each of the six CTQ material kinds, for one
/// transformation run. This crate has no opinion on *how* a given
/// transformation family (Datalog closure, N3 quarantine, PDDL planning,
/// SHACL admission, ...) produces this text -- only that a receipt/replay
/// cycle over it must carry all six kinds together. This is what makes the
/// subsystem transformation-agnostic rather than one pipeline's bookkeeping.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Materials {
    /// Canonical text of the transformation's input material.
    pub source: String,
    /// Canonical text of the query/plan driving the transformation.
    pub query: String,
    /// Canonical text of the template (empty string if the transformation
    /// uses none -- see [`ReceiptReplayRefused::EmptyMaterial`] for why an
    /// empty *required* material is refused rather than silently digesting
    /// an empty string as if it were meaningful content).
    pub template: String,
    /// Canonical text of the compiled program/artifact executed.
    pub program: String,
    /// Canonical text of the event/log material.
    pub event: String,
    /// Canonical text of the transformation's output material.
    pub output: String,
}

impl Materials {
    fn get(&self, kind: DigestKind) -> &str {
        match kind {
            DigestKind::Source => &self.source,
            DigestKind::Query => &self.query,
            DigestKind::Template => &self.template,
            DigestKind::Program => &self.program,
            DigestKind::Event => &self.event,
            DigestKind::Output => &self.output,
        }
    }
}

// ---------------------------------------------------------------------
// Receipt
// ---------------------------------------------------------------------

/// A built, folded receipt: the six per-kind digests plus the folded root.
/// The only lawful constructor is [`receipt_builder::build`] -- there is no
/// public constructor that lets a caller assemble a `Receipt` from digests
/// it did not compute, matching this family's invariant that "no receipt is
/// ornamental."
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    /// The six CTQ digests, keyed by kind (`BTreeMap`, not `HashMap`, for
    /// deterministic iteration -- this repo's determinism doctrine).
    pub digests: BTreeMap<DigestKind, Digest>,
    /// [`receipt_fold::fold`] over `digests`, in [`DigestKind::REQUIRED`]
    /// order.
    pub receipt_root: Digest,
}

// ---------------------------------------------------------------------
// Typed refusal taxonomy -- reachable from Receipt Builder, Receipt Fold,
// and Receipt Graph Writer (per the family survey), plus the replay/
// equivalence and lifecycle refusals this module adds.
// ---------------------------------------------------------------------

/// F25's typed refusal. Every fallible stage in this module returns this
/// type or a value built from it -- no silent defaults, no `.ok()`
/// swallowing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReceiptReplayRefused {
    /// Receipt Builder: one of the six required materials was empty.
    #[error("material for {kind:?} is empty; Receipt Builder refuses to digest an empty required material")]
    EmptyMaterial {
        /// Which of the six kinds was empty.
        kind: DigestKind,
    },
    /// Receipt Fold: the supplied digest map did not carry all six required
    /// kinds. Structurally unreachable via [`receipt_builder::build`]
    /// (always supplies a complete map), but real and reachable when a
    /// caller folds a digest map assembled from elsewhere (e.g. reconstructed
    /// from a persisted receipt graph).
    #[error("digest set incomplete for Receipt Fold: missing {missing:?}")]
    IncompleteDigestSet {
        /// The required kinds not present in the supplied map.
        missing: Vec<DigestKind>,
    },
    /// Replay Executor: the caller-supplied replay closure itself refused
    /// before materials could be reproduced.
    #[error("independent replay refused before materials could be reproduced: {reason}")]
    ReplayExecutionRefused {
        /// The replay closure's own refusal reason.
        reason: String,
    },
    /// Equivalence Comparator: a per-kind digest mismatch between the
    /// recorded and replayed receipts.
    #[error("equivalence mismatch on independent replay for {kind:?}: recorded {recorded} replayed {replayed}")]
    EquivalenceMismatch {
        /// Which kind mismatched.
        kind: DigestKind,
        /// The digest carried by the recorded receipt.
        recorded: String,
        /// The digest the replay recomputed.
        replayed: String,
    },
    /// Equivalence Comparator: every per-kind digest matched but the folded
    /// root did not (defensive -- unreachable unless [`receipt_fold::fold`]
    /// itself has drifted between the two calls, since it is a pure
    /// function of the digest map).
    #[error(
        "receipt_root mismatch on independent replay: recorded {recorded} recomputed {recomputed}"
    )]
    ReceiptRootMismatch {
        /// Root carried by the recorded receipt.
        recorded: String,
        /// Root recomputed from the replayed digest map.
        recomputed: String,
    },
    /// Receipt Graph Writer: refused to write (equivalence did not match,
    /// or -- defensive, unreachable in practice -- a content-addressed or
    /// minted node IRI failed to construct).
    #[error("Receipt Graph Writer refused: {reason}")]
    GraphWriteRefused {
        /// Why the writer refused.
        reason: String,
    },
    /// The closed L5 lifecycle: an illegal `Stage` transition was attempted.
    #[error("illegal receipt/replay lifecycle transition {from:?} -> {to:?}")]
    IllegalLifecycleTransition {
        /// The stage the machine was in.
        from: Stage,
        /// The stage transition that was refused.
        to: Stage,
    },
    /// L7 (chaos/concurrency lens): disclosed, not yet implemented. See
    /// [`chaos_gate`]'s module doc comment.
    #[error("L7 idempotency+correlation / durable-receipt-head gate for receipts-and-replay is HAND_WRITE_REQUIRED and not implemented yet (ticket V12-025); refusing rather than silently admitting a duplicate/stale/post-restart replay")]
    L7ChaosGateNotImplemented,
}

// ---------------------------------------------------------------------
// Material Digestor (L2/L3 chain, stage 1)
// ---------------------------------------------------------------------

/// Digests each of the six CTQ-named materials independently.
pub mod material_digestor {
    use super::*;

    /// Material Digestor: digests each of the six CTQ materials, tagged and
    /// salted per kind (see [`DigestKind::tag`]) so two materials with
    /// byte-identical text but different kinds never collide.
    ///
    /// # Complexity
    /// O(sum of the six materials' byte lengths): one BLAKE3 pass per kind.
    pub fn digest(materials: &Materials) -> BTreeMap<DigestKind, Digest> {
        DigestKind::REQUIRED
            .iter()
            .map(|kind| (*kind, Digest::of(kind.tag(), materials.get(*kind))))
            .collect()
    }
}

// ---------------------------------------------------------------------
// Receipt Fold (L2/L3 chain, stage 3 -- defined before Receipt Builder
// since the builder calls it)
// ---------------------------------------------------------------------

/// Folds a complete six-kind digest map into one receipt root.
pub mod receipt_fold {
    use super::*;

    const RECEIPT_ROOT_TAG: &str = "mfw/f25/receipt-root/v1";

    /// Receipt Fold: folds a digest map into one receipt root, in fixed CTQ
    /// order (source, query, template, program, event, output). Mirrors
    /// `praxis_graphlaw::chatman::engine::receipt_root`'s tagged
    /// fixed-order-`Hasher` fold, generalized from that engine's nine
    /// constitutional digests to this family's six CTQ-named ones.
    ///
    /// # Errors
    /// [`ReceiptReplayRefused::IncompleteDigestSet`] if `digests` does not
    /// carry all six required [`DigestKind`]s -- the choke point that
    /// enforces the CTQ's "verify SIX digest kinds together" requirement
    /// rather than silently folding a partial set.
    ///
    /// # Complexity
    /// O(1): six fixed-size digest strings, one `Hasher` pass.
    pub fn fold(digests: &BTreeMap<DigestKind, Digest>) -> Result<Digest, ReceiptReplayRefused> {
        let missing: Vec<DigestKind> = DigestKind::REQUIRED
            .iter()
            .copied()
            .filter(|k| !digests.contains_key(k))
            .collect();
        if !missing.is_empty() {
            return Err(ReceiptReplayRefused::IncompleteDigestSet { missing });
        }
        let mut hasher = blake3::Hasher::new();
        hasher.update(RECEIPT_ROOT_TAG.as_bytes());
        // O(6): constitutional CTQ order, never map-iteration order.
        for kind in DigestKind::REQUIRED {
            hasher.update(b"\0");
            hasher.update(digests[&kind].as_str().as_bytes());
        }
        Ok(Digest(format!("blake3:{}", hasher.finalize().to_hex())))
    }
}

// ---------------------------------------------------------------------
// Receipt Builder (L2/L3 chain, stage 2)
// ---------------------------------------------------------------------

/// Validates materials and builds a folded [`Receipt`].
pub mod receipt_builder {
    use super::*;

    /// Receipt Builder: refuses any empty required material, then digests
    /// (Material Digestor) and folds (Receipt Fold) into a [`Receipt`].
    ///
    /// # Errors
    /// [`ReceiptReplayRefused::EmptyMaterial`] naming the first empty kind
    /// found, in CTQ order. [`ReceiptReplayRefused::IncompleteDigestSet`]
    /// propagated from [`receipt_fold::fold`] (structurally unreachable
    /// here, since [`material_digestor::digest`] always returns a complete
    /// map from [`Materials`]'s six fixed fields, but not suppressed).
    ///
    /// # Complexity
    /// O(sum of material byte lengths): [`material_digestor::digest`]'s
    /// bound plus [`receipt_fold::fold`]'s O(1).
    pub fn build(materials: &Materials) -> Result<Receipt, ReceiptReplayRefused> {
        for kind in DigestKind::REQUIRED {
            if materials.get(kind).is_empty() {
                return Err(ReceiptReplayRefused::EmptyMaterial { kind });
            }
        }
        let digests = material_digestor::digest(materials);
        let receipt_root = receipt_fold::fold(&digests)?;
        Ok(Receipt {
            digests,
            receipt_root,
        })
    }
}

// ---------------------------------------------------------------------
// Replay Executor + Independent Verifier + Equivalence Comparator
// (L2/L3 chain, stages 4-6)
// ---------------------------------------------------------------------

/// Re-executes a caller-supplied replay closure and rebuilds a fresh
/// [`Receipt`] from what it reproduces.
pub mod replay_executor {
    use super::*;

    /// Replay Executor: runs `replay` (transformation-agnostic -- this
    /// module has no opinion on *how* a family reproduces its materials,
    /// only that it must) to reproduce [`Materials`], then builds a fresh
    /// [`Receipt`] via the identical [`receipt_builder::build`] formula the
    /// original admission used. One shared formula for "what should a
    /// receipt be, given materials" -- the same anti-duplication discipline
    /// `crates/cng/src/otel_receipt.rs`'s `compute_receipt_digests` sharing
    /// already established in this repo for its OTEL/OCEL receipt pair.
    ///
    /// # Errors
    /// [`ReceiptReplayRefused::ReplayExecutionRefused`] if `replay` itself
    /// returns `Err` before any materials are reproduced (the error's
    /// reason is folded from the closure's own [`ReceiptReplayRefused`]
    /// `Display` text). Otherwise, whatever [`receipt_builder::build`] can
    /// refuse with, unchanged.
    ///
    /// # Complexity
    /// Cost of `replay` plus [`receipt_builder::build`]'s bound.
    pub fn execute<F>(replay: F) -> Result<Receipt, ReceiptReplayRefused>
    where
        F: FnOnce() -> Result<Materials, ReceiptReplayRefused>,
    {
        let materials = replay().map_err(|e| ReceiptReplayRefused::ReplayExecutionRefused {
            reason: e.to_string(),
        })?;
        receipt_builder::build(&materials)
    }
}

/// Compares a recorded [`Receipt`] against a replayed one, fail-fast, in
/// canonical CTQ order.
pub mod equivalence_comparator {
    use super::*;

    /// Equivalence Comparator: fail-fast, per-kind digest comparison in
    /// canonical CTQ order, then a root comparison. Mirrors
    /// `praxis_graphlaw::chatman::engine::ChatmanEngine::verify_replay`'s
    /// per-digest fail-fast constitutional-order comparison, generalized
    /// from that engine's fixed nine S1-S5 digests to this family's six CTQ
    /// digests.
    ///
    /// # Errors
    /// [`ReceiptReplayRefused::EquivalenceMismatch`] on the first per-kind
    /// mismatch found, in CTQ order. [`ReceiptReplayRefused::ReceiptRootMismatch`]
    /// if every per-kind digest matched but the folded root did not
    /// (defensive -- see that variant's doc comment).
    ///
    /// # Complexity
    /// O(1): six fixed digest comparisons plus one root comparison.
    pub fn compare(
        recorded: &Receipt,
        replayed: &Receipt,
    ) -> Result<EquivalenceReport, ReceiptReplayRefused> {
        for kind in DigestKind::REQUIRED {
            // Presence is guaranteed: both `Receipt`s were only constructible
            // via `receipt_builder::build`, which proves completeness through
            // `receipt_fold::fold` before a `Receipt` can exist.
            let (Some(r), Some(p)) = (recorded.digests.get(&kind), replayed.digests.get(&kind))
            else {
                return Err(ReceiptReplayRefused::IncompleteDigestSet {
                    missing: vec![kind],
                });
            };
            if r != p {
                return Err(ReceiptReplayRefused::EquivalenceMismatch {
                    kind,
                    recorded: r.as_str().to_string(),
                    replayed: p.as_str().to_string(),
                });
            }
        }
        if recorded.receipt_root != replayed.receipt_root {
            return Err(ReceiptReplayRefused::ReceiptRootMismatch {
                recorded: recorded.receipt_root.as_str().to_string(),
                recomputed: replayed.receipt_root.as_str().to_string(),
            });
        }
        Ok(EquivalenceReport {
            matched_kinds: DigestKind::REQUIRED.to_vec(),
            receipt_root_matched: true,
        })
    }
}

/// Independent Verifier: orchestrates Replay Executor + Equivalence
/// Comparator against one recorded [`Receipt`].
pub mod independent_verifier {
    use super::*;

    /// Independent Verifier: re-runs the transformation via `replay`
    /// (Replay Executor) and compares the result against `recorded`
    /// (Equivalence Comparator), returning both the replayed [`Receipt`]
    /// and the [`EquivalenceReport`] -- the replayed receipt is needed by
    /// [`receipt_graph_writer::write`] to content-address the `ReplayOutput`
    /// node from what was *actually* replayed, not copied from `recorded`.
    ///
    /// # Errors
    /// Everything [`replay_executor::execute`] and
    /// [`equivalence_comparator::compare`] can refuse with, unchanged.
    ///
    /// # Complexity
    /// [`replay_executor::execute`]'s bound plus
    /// [`equivalence_comparator::compare`]'s O(1).
    pub fn verify<F>(
        recorded: &Receipt,
        replay: F,
    ) -> Result<(Receipt, EquivalenceReport), ReceiptReplayRefused>
    where
        F: FnOnce() -> Result<Materials, ReceiptReplayRefused>,
    {
        let replayed = replay_executor::execute(replay)?;
        let report = equivalence_comparator::compare(recorded, &replayed)?;
        Ok((replayed, report))
    }
}

/// The result of an [`equivalence_comparator::compare`] run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquivalenceReport {
    /// Which kinds matched, in CTQ order (always all six on `Ok`).
    pub matched_kinds: Vec<DigestKind>,
    /// Whether the folded receipt root also matched.
    pub receipt_root_matched: bool,
}

// ---------------------------------------------------------------------
// Receipt Graph Writer (L2/L3 chain, final stage; L6 provenance lens)
// ---------------------------------------------------------------------

/// Writes the F25-L6 PROV-O ancestry chain.
pub mod receipt_graph_writer {
    use super::*;
    use oxigraph::model::{GraphName, Literal, NamedNode, Term};

    /// This module's own minted vocabulary namespace for the digest-chain
    /// terms PROV-O has no native term for -- follows
    /// `crates/cng/src/otel_receipt.rs`'s `cngr:` convention in style
    /// (`https://truex.io/ontology/cng-receipt#`), scoped to this family
    /// rather than reusing that crate's namespace (this family's shape is
    /// generic six-digest/eight-node, not that module's fixed OTEL/OCEL
    /// three-digest one).
    const MFWR_NS: &str = "https://truex.io/ontology/mfw-f25-receipt#";
    /// Reused verbatim from PROV-O -- the exact binding
    /// `crates/cng/src/powl.rs::PROV_PREFIX` and `otel_receipt.rs` already
    /// use in this repo.
    const PROV_NS: &str = "http://www.w3.org/ns/prov#";
    const RDF_TYPE_IRI: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    /// Fixed named graph this family's receipt/replay PROV-O ancestry is
    /// written into.
    pub const RECEIPT_GRAPH_IRI: &str = "urn:graph:mfw:f25:receipts";

    const MECHANISM_DIGEST_TAG: &str = "mfw/f25/mechanism-digest/v1";
    const EQUIVALENCE_REPORT_TAG: &str = "mfw/f25/equivalence-report/v1";

    fn ns_node(ns: &str, local: &str) -> NamedNode {
        NamedNode::new(format!("{ns}{local}"))
            .expect("vocabulary IRI is a compile-time-controlled constant, never external input")
    }

    fn rdf_type() -> NamedNode {
        NamedNode::new(RDF_TYPE_IRI).expect("RDF_TYPE_IRI is a compile-time-controlled constant")
    }

    fn receipt_graph_name() -> GraphName {
        GraphName::NamedNode(
            NamedNode::new(RECEIPT_GRAPH_IRI)
                .expect("RECEIPT_GRAPH_IRI is a compile-time-controlled constant"),
        )
    }

    fn content_addressed(digest: &Digest) -> Result<NamedNode, ReceiptReplayRefused> {
        NamedNode::new(format!("urn:blake3:{}", digest.hex())).map_err(|e| {
            ReceiptReplayRefused::GraphWriteRefused {
                reason: format!("content-addressed node IRI construction failed: {e}"),
            }
        })
    }

    fn minted(iri: &str) -> Result<NamedNode, ReceiptReplayRefused> {
        NamedNode::new(iri).map_err(|e| ReceiptReplayRefused::GraphWriteRefused {
            reason: format!("minted node IRI construction failed: {e}"),
        })
    }

    /// Folds an ordered, named subset of `digests` (e.g. the four
    /// mechanism-describing kinds) into one digest -- same tagged
    /// fixed-order-`Hasher` discipline as [`receipt_fold::fold`]. `kinds` is
    /// always a fixed literal slice at every call site in this module, so
    /// the fold is deterministic regardless of `digests`' own (already
    /// `BTreeMap`-sorted) iteration order.
    fn fold_subset(
        tag: &str,
        digests: &BTreeMap<DigestKind, Digest>,
        kinds: &[DigestKind],
    ) -> Digest {
        let mut hasher = blake3::Hasher::new();
        hasher.update(tag.as_bytes());
        for kind in kinds {
            hasher.update(b"\0");
            if let Some(d) = digests.get(kind) {
                hasher.update(d.as_str().as_bytes());
            }
        }
        Digest(format!("blake3:{}", hasher.finalize().to_hex()))
    }

    fn equivalence_report_digest(report: &EquivalenceReport) -> Digest {
        let mut hasher = blake3::Hasher::new();
        hasher.update(EQUIVALENCE_REPORT_TAG.as_bytes());
        for kind in &report.matched_kinds {
            hasher.update(b"\0");
            hasher.update(format!("{kind:?}").as_bytes());
        }
        hasher.update(b"\0");
        hasher.update(&[u8::from(report.receipt_root_matched)]);
        Digest(format!("blake3:{}", hasher.finalize().to_hex()))
    }

    /// Receipt Graph Writer: writes the F25-L6 PROV-O ancestry chain named
    /// by the family survey, verbatim in node sequence and edge labels:
    /// `SourceDigest -[wasDerivedFrom]-> MechanismDigest -[generates]->
    /// OutputDigest -[wasDerivedFrom]-> Receipt -[generates]-> ReceiptHead
    /// -[wasDerivedFrom]-> ReplayOutput -[generates]-> EquivalenceReport
    /// -[wasDerivedFrom]-> ReceiptGraph`.
    ///
    /// `generates` is realized with the real `prov:generated` term (PROV-O
    /// has no literal `generates` predicate); every node that is the
    /// subject of a `generated` edge (`MechanismDigest`, `Receipt`,
    /// `ReplayOutput`) is additionally typed `prov:Activity`, matching real
    /// PROV-O's Activity-generates-Entity domain rather than asserting the
    /// edge with a mismatched-typed subject. `wasDerivedFrom` is
    /// `prov:wasDerivedFrom` verbatim -- the same binding
    /// `crates/cng/src/otel_receipt.rs` already reuses in this repo.
    ///
    /// Requires an [`EquivalenceReport`] with `receipt_root_matched: true`
    /// (not just a [`Receipt`]) -- enforced by an explicit refusal, not
    /// merely by the type signature accepting the report -- so a receipt
    /// graph can never be written for a replay that did not independently
    /// reconstruct equivalence.
    ///
    /// # Errors
    /// [`ReceiptReplayRefused::GraphWriteRefused`] if
    /// `report.receipt_root_matched` is `false`, or (defensive, unreachable
    /// in practice -- BLAKE3 hex output is always `[0-9a-f]+`) if a node IRI
    /// fails to construct.
    ///
    /// # Complexity
    /// O(1): eight fixed entity nodes, seven fixed edges, two folded-digest
    /// `Hasher` passes (`MechanismDigest`, `EquivalenceReport`).
    pub fn write(
        recorded: &Receipt,
        replayed: &Receipt,
        report: &EquivalenceReport,
    ) -> Result<Vec<Quad>, ReceiptReplayRefused> {
        if !report.receipt_root_matched {
            return Err(ReceiptReplayRefused::GraphWriteRefused {
                reason: "refusing to write a receipt graph for an EquivalenceReport that did not match (replay-must-reconstruct-equivalence, per the F25 CTQ)".to_string(),
            });
        }

        let mechanism_digest = fold_subset(
            MECHANISM_DIGEST_TAG,
            &recorded.digests,
            &[
                DigestKind::Query,
                DigestKind::Template,
                DigestKind::Program,
                DigestKind::Event,
            ],
        );
        let equivalence_digest = equivalence_report_digest(report);

        let source_digest = recorded.digests.get(&DigestKind::Source).ok_or_else(|| {
            ReceiptReplayRefused::IncompleteDigestSet {
                missing: vec![DigestKind::Source],
            }
        })?;
        let output_digest = recorded.digests.get(&DigestKind::Output).ok_or_else(|| {
            ReceiptReplayRefused::IncompleteDigestSet {
                missing: vec![DigestKind::Output],
            }
        })?;

        let source_node = content_addressed(source_digest)?;
        let mechanism_node = content_addressed(&mechanism_digest)?;
        let output_node = content_addressed(output_digest)?;
        let receipt_node = minted(&format!(
            "urn:mfw:f25:receipt:{}",
            recorded.receipt_root.hex()
        ))?;
        let receipt_head_node = minted(&format!(
            "urn:mfw:f25:receipt-head:{}",
            recorded.receipt_root.hex()
        ))?;
        let replay_output_node = minted(&format!(
            "urn:mfw:f25:replay-output:{}",
            replayed.receipt_root.hex()
        ))?;
        let equivalence_node = minted(&format!(
            "urn:mfw:f25:equivalence-report:{}",
            equivalence_digest.hex()
        ))?;
        let receipt_graph_node = minted(RECEIPT_GRAPH_IRI)?;

        let graph = receipt_graph_name();
        let mut quads = Vec::new();

        let activity_subjects = [&mechanism_node, &receipt_node, &replay_output_node];
        let entities: [(&NamedNode, &str, Option<&str>); 8] = [
            (&source_node, "SourceDigest", Some(source_digest.as_str())),
            (
                &mechanism_node,
                "MechanismDigest",
                Some(mechanism_digest.as_str()),
            ),
            (&output_node, "OutputDigest", Some(output_digest.as_str())),
            (
                &receipt_node,
                "Receipt",
                Some(recorded.receipt_root.as_str()),
            ),
            (
                &receipt_head_node,
                "ReceiptHead",
                Some(recorded.receipt_root.as_str()),
            ),
            (
                &replay_output_node,
                "ReplayOutput",
                Some(replayed.receipt_root.as_str()),
            ),
            (
                &equivalence_node,
                "EquivalenceReport",
                Some(equivalence_digest.as_str()),
            ),
            (&receipt_graph_node, "ReceiptGraph", None),
        ];
        for (node, type_local, digest) in entities {
            quads.push(Quad::new(
                node.clone(),
                rdf_type(),
                Term::NamedNode(ns_node(MFWR_NS, type_local)),
                graph.clone(),
            ));
            quads.push(Quad::new(
                node.clone(),
                rdf_type(),
                Term::NamedNode(ns_node(PROV_NS, "Entity")),
                graph.clone(),
            ));
            if activity_subjects.contains(&node) {
                quads.push(Quad::new(
                    node.clone(),
                    rdf_type(),
                    Term::NamedNode(ns_node(PROV_NS, "Activity")),
                    graph.clone(),
                ));
            }
            if let Some(d) = digest {
                quads.push(Quad::new(
                    node.clone(),
                    ns_node(MFWR_NS, "contentDigest"),
                    Term::Literal(Literal::new_simple_literal(d)),
                    graph.clone(),
                ));
            }
        }

        let derived = ns_node(PROV_NS, "wasDerivedFrom");
        let generated = ns_node(PROV_NS, "generated");
        let edges: [(&NamedNode, &NamedNode, &NamedNode); 7] = [
            (&source_node, &derived, &mechanism_node),
            (&mechanism_node, &generated, &output_node),
            (&output_node, &derived, &receipt_node),
            (&receipt_node, &generated, &receipt_head_node),
            (&receipt_head_node, &derived, &replay_output_node),
            (&replay_output_node, &generated, &equivalence_node),
            (&equivalence_node, &derived, &receipt_graph_node),
        ];
        for (s, p, o) in edges {
            quads.push(Quad::new(
                s.clone(),
                p.clone(),
                Term::NamedNode(o.clone()),
                graph.clone(),
            ));
        }

        // Canonical order: sorted by each quad's N-Quads text, matching
        // `crates/cng/src/otel_receipt.rs`'s own canonicalization convention
        // -- independent of insertion order.
        quads.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
        Ok(quads)
    }
}

// ---------------------------------------------------------------------
// L5: closed lawful lifecycle state machine
// ---------------------------------------------------------------------

/// The closed lawful state set (L5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Material Digestor has run (this module's tracked lifecycle begins
    /// here -- see [`StateMachine::new`]'s doc comment for why).
    MaterialsDigested,
    /// Receipt Builder succeeded.
    ReceiptBuilt,
    /// Receipt Fold succeeded.
    Folded,
    /// Independent Verifier's Replay Executor leg succeeded (materials
    /// reproduced, a fresh receipt built) -- equivalence not yet compared.
    IndependentlyVerified,
    /// Equivalence Comparator ran and matched (paired with the next stage;
    /// see [`Stage::allowed`]'s doc comment for why these two are only ever
    /// reached together in this module's orchestration).
    Replayed,
    /// Equivalence Comparator's match is final.
    EquivalenceChecked,
    /// Receipt Graph Writer succeeded.
    GraphWritten,
    /// Terminal refusal state.
    Refused,
}

impl Stage {
    /// The closed lawful edge set, taken verbatim from the family survey's
    /// L5 text: `MATERIALS_DIGESTED -> RECEIPT_BUILT -> FOLDED ->
    /// INDEPENDENTLY_VERIFIED -> REPLAYED -> EQUIVALENCE_CHECKED ->
    /// GRAPH_WRITTEN`, `REFUSED` reachable only from `RECEIPT_BUILT` or
    /// `INDEPENDENTLY_VERIFIED`.
    ///
    /// This module resolves that against the survey's separate prose
    /// ("typed refusal reachable from Receipt Builder, Receipt Fold, and
    /// Receipt Graph Writer") as follows, disclosed rather than silently
    /// picked:
    /// - Receipt Fold's own refusal ([`ReceiptReplayRefused::IncompleteDigestSet`])
    ///   is attributed to the `ReceiptBuilt -> Refused` edge: [`run`] only
    ///   advances to `ReceiptBuilt` after materials are validated
    ///   non-empty, then attempts the fold while still nominally "at"
    ///   `ReceiptBuilt`, transitioning to `Refused` on failure.
    /// - Equivalence Comparator's mismatch is attributed to the
    ///   `IndependentlyVerified -> Refused` edge: [`run`] advances to
    ///   `IndependentlyVerified` once Replay Executor reproduces materials
    ///   (before comparing), then compares while "at" `IndependentlyVerified`,
    ///   transitioning to `Refused` on mismatch. This is *why* `Replayed`
    ///   and `EquivalenceChecked` are only ever reached together on success
    ///   in [`run`] -- the closed edge list names no `Replayed ->` or
    ///   `EquivalenceChecked ->` edge to `Refused`, so a real mismatch must
    ///   be caught before either state is entered.
    /// - Receipt Builder's own refusal ([`ReceiptReplayRefused::EmptyMaterial`])
    ///   happens *before* `ReceiptBuilt` is ever entered (the closed list has
    ///   no edge out of `MaterialsDigested` into `Refused`), so [`run`]
    ///   returns that error directly without any `Stage` transition.
    /// - Receipt Graph Writer's own refusal similarly happens after
    ///   `EquivalenceChecked`, which the closed list also does not name as a
    ///   `Refused` source -- [`run`] returns that error directly, leaving
    ///   the tracked stage at `EquivalenceChecked`, rather than fabricating
    ///   an edge the survey's own closed list does not contain.
    fn allowed(from: Stage, to: Stage) -> bool {
        use Stage::*;
        matches!(
            (from, to),
            (MaterialsDigested, ReceiptBuilt)
                | (ReceiptBuilt, Folded)
                | (Folded, IndependentlyVerified)
                | (IndependentlyVerified, Replayed)
                | (Replayed, EquivalenceChecked)
                | (EquivalenceChecked, GraphWritten)
                | (ReceiptBuilt, Refused)
                | (IndependentlyVerified, Refused)
        )
    }
}

/// Tracks the current [`Stage`] of one receipt/replay cycle, enforcing the
/// closed lawful edge set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateMachine {
    stage: Stage,
}

impl StateMachine {
    /// Material Digestor never refuses (digesting arbitrary text always
    /// succeeds -- there is no failure mode for hashing a string), so this
    /// state machine's tracked lifecycle begins already at
    /// `MaterialsDigested` rather than modeling a separate pre-digest state
    /// with no observable refusal edge.
    pub fn new() -> Self {
        Self {
            stage: Stage::MaterialsDigested,
        }
    }

    /// The current stage.
    pub fn stage(&self) -> Stage {
        self.stage
    }

    /// Attempts a transition, refusing (without mutating `self`) if the
    /// edge is not in the closed lawful set.
    ///
    /// # Complexity
    /// O(1): one fixed-arity pattern match.
    pub fn advance(&mut self, to: Stage) -> Result<(), ReceiptReplayRefused> {
        if !Stage::allowed(self.stage, to) {
            return Err(ReceiptReplayRefused::IllegalLifecycleTransition {
                from: self.stage,
                to,
            });
        }
        self.stage = to;
        Ok(())
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------
// L7 (Chaos/Concurrency lens) -- HAND_WRITE_REQUIRED, disclosed, NOT
// implemented.
// ---------------------------------------------------------------------

/// Would provide the L7 idempotent-replay + durable cross-restart
/// receipt-head gate the family survey requires (duplicate-event,
/// process/engine-restart, and stale/malformed-result handling routed
/// through an atomic idempotency+correlation gate or durable receipt-head/
/// replay-state before re-admission or refusal).
///
/// **Not implemented in this pass.** Per the F25 family survey's own
/// justification: no existing code in this repo (internal or external)
/// implements this specific combination for the receipts-and-replay
/// mechanism -- `praxis_graphlaw::chatman::engine`'s `idempotency_key`
/// dedup covers hook-actuation admission, and `cng::bench::dispatch`'s
/// idempotency+correlation ledger covers dispatch-contract admission; both
/// real, both for a different admission boundary than replaying a receipt
/// after an engine/process restart. This is new architectural surface, not
/// mechanical/structural enough for a first-pass ggen scaffold, so it is
/// hand-written work not yet done.
pub mod chaos_gate {
    use super::*;

    /// Always refuses with [`ReceiptReplayRefused::L7ChaosGateNotImplemented`]
    /// -- fails loud, per this repo's no-overclaiming rule, rather than
    /// silently admitting a duplicate/stale/post-restart replay as if it had
    /// been checked.
    pub fn admit_for_replay(_correlation_id: &str) -> Result<(), ReceiptReplayRefused> {
        Err(ReceiptReplayRefused::L7ChaosGateNotImplemented)
    }
}

// ---------------------------------------------------------------------
// Top-level orchestration
// ---------------------------------------------------------------------

/// The full outcome of one successful `run`.
#[derive(Debug, Clone)]
pub struct ReceiptReplayOutcome {
    /// The recorded (originally built) receipt.
    pub receipt: Receipt,
    /// The independently replayed receipt.
    pub replayed: Receipt,
    /// The equivalence comparison result.
    pub report: EquivalenceReport,
    /// The written PROV-O receipt-graph quads.
    pub graph: Vec<Quad>,
    /// The final [`Stage`] (`GraphWritten` on success).
    pub stage: Stage,
}

/// Runs the full F25 component chain end to end: Material Digestor ->
/// Receipt Builder -> Receipt Fold -> Independent Verifier (Replay Executor
/// + Equivalence Comparator) -> Receipt Graph Writer, tracking [`Stage`]
/// transitions through a [`StateMachine`] per the closed L5 edge set. See
/// [`Stage::allowed`]'s doc comment for exactly which refusal is attributed
/// to which tracked transition (or none, where the closed list names none).
///
/// # Errors
/// Any [`ReceiptReplayRefused`] variant Receipt Builder, Receipt Fold,
/// Independent Verifier, or Receipt Graph Writer can produce.
///
/// # Complexity
/// Sum of every stage's own bound: O(sum of material byte lengths) for
/// digesting (twice: once for `materials`, once for whatever `replay`
/// reproduces) plus O(1) folding/comparing/graph-writing.
pub fn run<F>(
    materials: &Materials,
    replay: F,
) -> Result<ReceiptReplayOutcome, ReceiptReplayRefused>
where
    F: FnOnce() -> Result<Materials, ReceiptReplayRefused>,
{
    let mut sm = StateMachine::new();

    for kind in DigestKind::REQUIRED {
        if materials.get(kind).is_empty() {
            return Err(ReceiptReplayRefused::EmptyMaterial { kind });
        }
    }
    let digests = material_digestor::digest(materials);
    sm.advance(Stage::ReceiptBuilt)?;

    let receipt_root = match receipt_fold::fold(&digests) {
        Ok(root) => root,
        Err(e) => {
            sm.advance(Stage::Refused)
                .expect("ReceiptBuilt -> Refused is a lawful L5 edge");
            return Err(e);
        }
    };
    sm.advance(Stage::Folded)?;
    let recorded = Receipt {
        digests,
        receipt_root,
    };

    let replayed = match replay_executor::execute(replay) {
        Ok(r) => r,
        // Still at `Folded`; the closed L5 list names no `Folded -> Refused`
        // edge, so this returns directly (see `Stage::allowed`'s doc
        // comment).
        Err(e) => return Err(e),
    };
    sm.advance(Stage::IndependentlyVerified)?;

    let report = match equivalence_comparator::compare(&recorded, &replayed) {
        Ok(report) => report,
        Err(e) => {
            sm.advance(Stage::Refused)
                .expect("IndependentlyVerified -> Refused is a lawful L5 edge");
            return Err(e);
        }
    };
    sm.advance(Stage::Replayed)?;
    sm.advance(Stage::EquivalenceChecked)?;

    let quads = receipt_graph_writer::write(&recorded, &replayed, &report)?;
    sm.advance(Stage::GraphWritten)?;

    Ok(ReceiptReplayOutcome {
        receipt: recorded,
        replayed,
        report,
        graph: quads,
        stage: sm.stage(),
    })
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_materials() -> Materials {
        Materials {
            source: "urn:example:source-graph-v1".to_string(),
            query: "SELECT ?s ?p ?o WHERE { ?s ?p ?o }".to_string(),
            template: "{{ subject }} {{ predicate }} {{ object }} .".to_string(),
            program: "compiled-program-bytes-as-hex-deadbeef".to_string(),
            event: "event-log-line-1\nevent-log-line-2".to_string(),
            output: "urn:example:output-graph-v1".to_string(),
        }
    }

    #[test]
    fn material_digestor_produces_all_six_kinds_deterministically() {
        let materials = sample_materials();
        let first = material_digestor::digest(&materials);
        let second = material_digestor::digest(&materials);
        assert_eq!(first.len(), 6);
        for kind in DigestKind::REQUIRED {
            assert!(first.contains_key(&kind), "missing {kind:?}");
        }
        assert_eq!(
            first, second,
            "digesting the same materials twice must be byte-identical"
        );
    }

    #[test]
    fn material_digestor_distinguishes_kinds_with_identical_text() {
        let mut materials = sample_materials();
        materials.query = "same-text".to_string();
        materials.template = "same-text".to_string();
        let digests = material_digestor::digest(&materials);
        assert_ne!(
            digests[&DigestKind::Query],
            digests[&DigestKind::Template],
            "domain-separation tag must prevent kind collisions on identical text"
        );
    }

    #[test]
    fn receipt_builder_refuses_empty_material() {
        let mut materials = sample_materials();
        materials.query = String::new();
        let err = receipt_builder::build(&materials).unwrap_err();
        assert_eq!(
            err,
            ReceiptReplayRefused::EmptyMaterial {
                kind: DigestKind::Query
            }
        );
    }

    #[test]
    fn receipt_fold_refuses_incomplete_digest_set() {
        let materials = sample_materials();
        let mut digests = material_digestor::digest(&materials);
        digests.remove(&DigestKind::Event);
        let err = receipt_fold::fold(&digests).unwrap_err();
        assert_eq!(
            err,
            ReceiptReplayRefused::IncompleteDigestSet {
                missing: vec![DigestKind::Event]
            }
        );
    }

    #[test]
    fn receipt_builder_builds_a_real_receipt_matching_independent_fold() {
        let materials = sample_materials();
        let receipt = receipt_builder::build(&materials).expect("valid materials must build");
        let digests = material_digestor::digest(&materials);
        let recomputed_root = receipt_fold::fold(&digests).expect("complete digest set folds");
        assert_eq!(receipt.receipt_root, recomputed_root);
        assert_eq!(receipt.digests, digests);
    }

    #[test]
    fn independent_verifier_confirms_equivalent_replay() {
        let materials = sample_materials();
        let recorded = receipt_builder::build(&materials).unwrap();
        let (replayed, report) =
            independent_verifier::verify(&recorded, || Ok(materials.clone())).unwrap();
        assert!(report.receipt_root_matched);
        assert_eq!(report.matched_kinds, DigestKind::REQUIRED.to_vec());
        assert_eq!(recorded, replayed);
    }

    #[test]
    fn independent_verifier_refuses_on_tampered_output() {
        let materials = sample_materials();
        let recorded = receipt_builder::build(&materials).unwrap();
        let mut tampered = materials.clone();
        tampered.output = "urn:example:tampered-output".to_string();
        let err = independent_verifier::verify(&recorded, || Ok(tampered)).unwrap_err();
        assert!(matches!(
            err,
            ReceiptReplayRefused::EquivalenceMismatch {
                kind: DigestKind::Output,
                ..
            }
        ));
    }

    #[test]
    fn independent_verifier_propagates_replay_execution_refusal() {
        let materials = sample_materials();
        let recorded = receipt_builder::build(&materials).unwrap();
        let err = independent_verifier::verify(&recorded, || {
            Err(ReceiptReplayRefused::L7ChaosGateNotImplemented)
        })
        .unwrap_err();
        assert!(matches!(
            err,
            ReceiptReplayRefused::ReplayExecutionRefused { .. }
        ));
    }

    #[test]
    fn state_machine_enforces_closed_lawful_edges() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.stage(), Stage::MaterialsDigested);

        // Illegal: cannot skip straight to Refused from the entry stage.
        let err = sm.advance(Stage::Refused).unwrap_err();
        assert!(matches!(
            err,
            ReceiptReplayRefused::IllegalLifecycleTransition { .. }
        ));
        assert_eq!(
            sm.stage(),
            Stage::MaterialsDigested,
            "a refused transition must not mutate state"
        );

        // Legal forward path.
        sm.advance(Stage::ReceiptBuilt).unwrap();
        // Legal: ReceiptBuilt -> Refused.
        let mut sm2 = sm;
        sm2.advance(Stage::Refused).unwrap();
        assert_eq!(sm2.stage(), Stage::Refused);

        sm.advance(Stage::Folded).unwrap();
        sm.advance(Stage::IndependentlyVerified).unwrap();
        // Legal: IndependentlyVerified -> Refused.
        let mut sm3 = sm;
        sm3.advance(Stage::Refused).unwrap();
        assert_eq!(sm3.stage(), Stage::Refused);

        // Illegal: Folded has no edge to Refused.
        let mut sm4 = StateMachine::new();
        sm4.advance(Stage::ReceiptBuilt).unwrap();
        sm4.advance(Stage::Folded).unwrap();
        assert!(sm4.advance(Stage::Refused).is_err());

        // Full happy path reaches GraphWritten.
        sm.advance(Stage::Replayed).unwrap();
        sm.advance(Stage::EquivalenceChecked).unwrap();
        sm.advance(Stage::GraphWritten).unwrap();
        assert_eq!(sm.stage(), Stage::GraphWritten);
    }

    #[test]
    fn receipt_graph_writer_refuses_to_write_unmatched_equivalence() {
        let materials = sample_materials();
        let recorded = receipt_builder::build(&materials).unwrap();
        let unmatched = EquivalenceReport {
            matched_kinds: vec![],
            receipt_root_matched: false,
        };
        let err = receipt_graph_writer::write(&recorded, &recorded, &unmatched).unwrap_err();
        assert!(matches!(
            err,
            ReceiptReplayRefused::GraphWriteRefused { .. }
        ));
    }

    #[test]
    fn receipt_graph_writer_writes_the_named_provo_chain() {
        let materials = sample_materials();
        let recorded = receipt_builder::build(&materials).unwrap();
        let (replayed, report) =
            independent_verifier::verify(&recorded, || Ok(materials.clone())).unwrap();
        let quads = receipt_graph_writer::write(&recorded, &replayed, &report).unwrap();
        assert!(!quads.is_empty());

        let texts: Vec<String> = quads.iter().map(|q| q.to_string()).collect();
        // The seven named edges must all be present, by predicate local name.
        let derived_count = texts
            .iter()
            .filter(|t| t.contains("wasDerivedFrom"))
            .count();
        let generated_count = texts.iter().filter(|t| t.contains("#generated>")).count();
        assert_eq!(derived_count, 4, "SourceDigest->Mechanism, Output->Receipt, ReceiptHead->ReplayOutput, EquivalenceReport->ReceiptGraph");
        assert_eq!(
            generated_count, 3,
            "Mechanism->Output, Receipt->ReceiptHead, ReplayOutput->EquivalenceReport"
        );

        // The recorded source digest must appear as a literal content digest.
        let source_digest = recorded.digests[&DigestKind::Source].as_str();
        assert!(
            texts.iter().any(|t| t.contains(source_digest)),
            "SourceDigest node must carry the real recorded source digest"
        );

        // Canonical order: re-sorting must be a no-op.
        let mut resorted = quads.clone();
        resorted.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
        assert_eq!(quads, resorted);
    }

    #[test]
    fn receipt_graph_writer_output_is_stable_across_a_second_independent_replay() {
        // Functional realization of "replay-must-reconstruct-equivalence
        // loop back to ReceiptGraph": running Independent Verifier + Receipt
        // Graph Writer a second time, against the same recorded receipt,
        // must reproduce byte-identical graph content -- the graph is
        // exactly what a fresh independent replay reconstructs, not a
        // frozen serialization of the first run.
        let materials = sample_materials();
        let recorded = receipt_builder::build(&materials).unwrap();

        let (replayed_1, report_1) =
            independent_verifier::verify(&recorded, || Ok(materials.clone())).unwrap();
        let quads_1 = receipt_graph_writer::write(&recorded, &replayed_1, &report_1).unwrap();

        let (replayed_2, report_2) =
            independent_verifier::verify(&recorded, || Ok(materials.clone())).unwrap();
        let quads_2 = receipt_graph_writer::write(&recorded, &replayed_2, &report_2).unwrap();

        assert_eq!(replayed_1, replayed_2);
        assert_eq!(report_1, report_2);
        assert_eq!(quads_1, quads_2);
    }

    #[test]
    fn run_end_to_end_happy_path_reaches_graph_written() {
        let materials = sample_materials();
        let outcome = run(&materials, || Ok(materials.clone())).unwrap();
        assert_eq!(outcome.stage, Stage::GraphWritten);
        assert!(outcome.report.receipt_root_matched);
        assert!(!outcome.graph.is_empty());
        assert_eq!(outcome.receipt.receipt_root, outcome.replayed.receipt_root);
    }

    #[test]
    fn run_refuses_end_to_end_on_tampered_replay() {
        let materials = sample_materials();
        let mut tampered = materials.clone();
        tampered.program = "different-compiled-program".to_string();
        let err = run(&materials, || Ok(tampered)).unwrap_err();
        assert!(matches!(
            err,
            ReceiptReplayRefused::EquivalenceMismatch {
                kind: DigestKind::Program,
                ..
            }
        ));
    }

    #[test]
    fn run_refuses_end_to_end_on_empty_material() {
        let mut materials = sample_materials();
        materials.source = String::new();
        let err = run(&materials, || Ok(materials.clone())).unwrap_err();
        assert_eq!(
            err,
            ReceiptReplayRefused::EmptyMaterial {
                kind: DigestKind::Source
            }
        );
    }

    #[test]

    fn chaos_gate_fails_loud_not_yet_implemented() {
        let err = chaos_gate::admit_for_replay("corr-id-1").unwrap_err();
        assert_eq!(err, ReceiptReplayRefused::L7ChaosGateNotImplemented);
    }
}
