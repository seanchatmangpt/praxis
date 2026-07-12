//! Family F14 -- "wasm4pm Arazzo Compiler" (atlas ticket V12-014).
//!
//! Survey verdict: **MIXED**. ALREADY_BUILT for the L1-L6 compiler core
//! (Document Loader -> Identity Index -> URI Resolver -> Reference Resolver ->
//! Feature Validator -> Expression Compiler -> AIR Lowerer -> Compiler Receipt);
//! HAND_WRITE_REQUIRED for L7 (Concurrency/Chaos) and L8 (Verification Claim
//! Ceiling). This file wires the real ALREADY_BUILT half by thinly wrapping
//! `crates/wasm4pm-arazzo` (a real, already-compiled, already-tested 3,577-line
//! crate -- 55/55 unit tests + 8/8 end-to-end integration tests passing at
//! survey time) and honestly stubs the L7/L8 half rather than dressing it up as
//! done. Per `.claude/rules/no-overclaiming.md`, this file's own claim is
//! bounded to what is re-exported/composed here and exercised by this file's
//! own `#[cfg(test)]` module (see below) -- not a repeat of wasm4pm-arazzo's own
//! test results, which this file did not re-run.
//!
//! ## What is real here (L1-L6)
//!
//! [`compile`] composes wasm4pm-arazzo's five real pipeline functions --
//! [`wasm4pm_arazzo::parse::DocumentIndex::add_document`],
//! [`wasm4pm_arazzo::resolve::normalize_uris`],
//! [`wasm4pm_arazzo::lower::lower_description`],
//! [`wasm4pm_arazzo::normalizer::ArazzoNormalizer::normalize`], and
//! [`wasm4pm_arazzo::compile::AirCompiler::digest_program`] -- in the same
//! order and with the same glue shape as the crate's own
//! `tests/end_to_end_lowering.rs::arazzo_document_parses_resolves_lowers_normalizes_and_compiles_to_wasm`
//! and `crates/praxis-core/src/arazzo.rs::render_and_compile` (an independent,
//! already-in-repo precedent for composing this exact chain from production,
//! non-test code). No parsing/resolution/lowering/normalization/compilation
//! logic is reimplemented; every stage is a direct call into wasm4pm-arazzo.
//!
//! The eight-stage topology the atlas names does not exist as eight separate
//! wasm4pm-arazzo modules -- it exists as five real functions whose internal
//! logic covers all eight concerns (see the stage-by-stage mapping below and
//! the survey's `mixed_breakdown` for the full detail). This module documents
//! that mapping rather than inventing eight wrapper functions that would just
//! call through to fewer real stages underneath.
//!
//! | Atlas topology stage    | Real wasm4pm-arazzo implementation |
//! |---|---|
//! | Document Loader          | [`wasm4pm_arazzo::parse::DocumentIndex::add_document`] |
//! | Identity Index           | folded into `DocumentIndex`'s dedup-by-base-URI admission (no separate module -- a disclosed topology gap, not hidden) |
//! | URI Resolver             | [`wasm4pm_arazzo::resolve::normalize_uris`] |
//! | Reference Resolver       | `lower::resolve_success_reference`/`resolve_failure_reference`/`resolve_parameter_reference` (called inside `lower_description`) plus [`wasm4pm_arazzo::normalizer::ArazzoNormalizer::normalize`] (cross-step variable resolution) |
//! | Feature Validator        | `lower::validate_step_dependencies`/`validate_step_timeout`/`validate_retry_policy`/`classify_criterion` (called inside `lower_description`) |
//! | Expression Compiler      | `lower::classify_output_value` (called inside `lower_description`; partial expression-grammar coverage -- most runtime-expression strings lower as opaque literals, not a structured AST) |
//! | AIR Lowerer              | [`wasm4pm_arazzo::lower::lower_description`] |
//! | Compiler Receipt         | [`wasm4pm_arazzo::compile::AirCompiler::digest_program`] |
//!
//! [`ArazzoCompileRefused`] is a type alias for [`wasm4pm_arazzo::Refusal`], not
//! a fresh enum -- the atlas names the refusal type `ArazzoCompileRefused`, but
//! wasm4pm-arazzo's own `Refusal` already carries the fine-grained variants the
//! atlas's L2/L4/L5 require (`MissingIdentity` for the Identity Index branch,
//! `UnresolvableReference`/`CyclicStepDependency` for the Reference Resolver
//! branch, `InvalidWorkflow` for the Compiler Receipt branch). Renaming or
//! duplicating it here would touch nothing real and would violate this crate's
//! own established "reuse, don't duplicate cosmetically" rule (see
//! `wasm4pm_arazzo::Refusal`'s own doc comment, which already applies this
//! reasoning to the PRD's `AIR_*` taxonomy names).
//!
//! ## What is honestly stubbed here (L7/L8)
//!
//! [`durability`] is a HAND_WRITE_REQUIRED stub: this repo has no atomic
//! idempotency/correlation gate, durable receipt head/replay state, or
//! production admission caller for this compiler anywhere today (independently
//! corroborated by `docs/jira/v26.7.11/IMPLEMENTATION_STATUS.md:20` as of the
//! last milestone: "the composed pipeline has no production caller
//! (PROJ-796)"). Its functions return a typed [`durability::NotYetImplemented`]
//! error unconditionally -- never a fake success -- per this session's standing
//! instruction that a Refusal placeholder is only acceptable for genuinely
//! undone work, disclosed as such.
//!
//! Survey-cited paths for F14 (see the survey verdict for full detail):
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F14_wasm4pm-arazzo.md
//! - /Users/sac/praxis/crates/wasm4pm-arazzo/{Cargo.toml,src/*.rs,tests/*.rs}
//! - /Users/sac/wasm4pm-compat/src/arazzo.rs (already an integrated path
//!   dependency of wasm4pm-arazzo -- no separate adaptation needed)
//! - /Users/sac/praxis/docs/jira/v26.7.11/{IMPLEMENTATION_STATUS.md,RAIL_A_B_STATUS.md}
//! - /Users/sac/praxis/crates/praxis-core/src/arazzo.rs (independent in-repo
//!   precedent for composing this exact real pipeline from production code,
//!   read this turn to confirm the composition shape below)

use bumpalo::Bump;
use wasm4pm_arazzo::air::AirProgram;
use wasm4pm_arazzo::compile::{AirCompiler, AirDigest};
use wasm4pm_arazzo::lower::lower_description;
use wasm4pm_arazzo::normalizer::ArazzoNormalizer;
use wasm4pm_arazzo::parse::DocumentIndex;
use wasm4pm_arazzo::resolve::normalize_uris;

/// Alias, not a fresh type -- see this module's own doc comment for why.
pub type ArazzoCompileRefused = wasm4pm_arazzo::Refusal;

/// The real, non-test-only output of [`compile`]: the admitted-and-lowered AIR
/// program (L7's atlas name for this is "Admitted AIR program") plus its
/// [`AirDigest`] (the atlas's "receipt/replay authority" for L1 -- a
/// deterministic BLAKE3 digest, [`wasm4pm_arazzo::compile::AirCompiler::digest_program`]'s
/// own guarantee, not asserted here).
#[derive(Debug)]
pub struct CompiledArazzo<'bump> {
    /// The lowered-and-normalized AIR program (stages: AIR Lowerer + the
    /// Reference-Resolver half that runs during normalization).
    pub program: AirProgram<'bump>,
    /// The Compiler Receipt stage's output: a deterministic BLAKE3 digest over
    /// `program`'s canonical bytes.
    pub digest: AirDigest,
}

/// Runs the real L1-L6 pipeline end to end: admits `document_json` as an
/// Arazzo 1.1.x document (refusing anything else at the earliest possible
/// stage, per this family's invariant that only 1.1.x is admitted), resolves
/// its URIs, lowers it to AIR (running the Feature Validator and Expression
/// Compiler stages as part of lowering -- see this module's stage-mapping
/// table), resolves cross-step references (the remaining half of the
/// Reference Resolver stage), and computes its Compiler Receipt digest.
///
/// `base_uri` is both the fallback base URI [`DocumentIndex::add_document`]
/// assigns the document if it declares no `$self` URI, and the key this
/// function looks the document back up under afterward -- so this function is
/// only correct for documents that do not declare their own `$self` URI (the
/// same constraint `tests/end_to_end_lowering.rs`'s own fixture operates
/// under). A document that does declare `$self` is still parsed and resolved
/// correctly by the underlying calls; it is this function's own post-resolve
/// lookup that would then miss and refuse -- disclosed here rather than
/// silently mishandled.
///
/// # Errors
/// [`ArazzoCompileRefused`] at whichever stage first refuses: parse
/// (malformed JSON, non-1.1.x version, duplicate base URI), URI resolution,
/// lowering (missing identity, unresolvable local reference, cyclic step
/// dependency, unsupported criterion/expression shape, malformed retry
/// policy), or normalization (unresolvable/forward/self cross-step reference).
/// A post-resolve lookup miss (see previous paragraph) refuses
/// [`wasm4pm_arazzo::Refusal::UriResolution`] naming the expected key, not a
/// panic.
///
/// # Complexity
/// Linear in the input document's size: one JSON parse, one URI-resolution
/// pass, one lowering pass, one normalization pass (each documented as such on
/// their own definitions in `wasm4pm_arazzo`), plus [`AirCompiler::digest_program`]'s
/// own linear-in-program-size BLAKE3 pass.
pub fn compile<'bump>(
    document_json: &str,
    base_uri: &str,
    bump: &'bump Bump,
) -> Result<CompiledArazzo<'bump>, ArazzoCompileRefused> {
    // Stage 1 (Document Loader) + Stage 2 (Identity Index, folded in):
    // strict Arazzo 1.1.x admission, refusing malformed JSON, non-1.1.x
    // versions, and duplicate base URIs before anything downstream runs.
    let mut index = DocumentIndex::new();
    index.add_document(document_json, base_uri)?;

    // Stage 3 (URI Resolver): resolves every relative reference in the
    // index to an absolute URI, in place.
    normalize_uris(&mut index)?;

    let doc = index.documents.get(base_uri).ok_or_else(|| {
        wasm4pm_arazzo::Refusal::UriResolution(format!(
            "document admitted under base URI {base_uri:?} but not found there after \
             URI resolution -- it likely declares its own $self URI, which this composed \
             pipeline function does not yet look up by (see compile's own doc comment)"
        ))
    })?;

    // Stage 4 (Reference Resolver, local half) + Stage 5 (Feature Validator)
    // + Stage 6 (Expression Compiler) + Stage 7 (AIR Lowerer): all run
    // inside lower_description -- see this module's stage-mapping table.
    let mut program = lower_description(doc, bump)?;

    // Stage 4 (Reference Resolver, cross-step half): resolves $steps.<id>.
    // outputs.<name> references against earlier steps' declared outputs
    // within the same workflow; forward/self/unknown references refuse.
    ArazzoNormalizer::normalize(&mut program, bump)?;

    // Stage 8 (Compiler Receipt): deterministic BLAKE3 digest over the
    // program's canonical bytes.
    let digest = AirCompiler::digest_program(&program)?;

    Ok(CompiledArazzo { program, digest })
}

/// L7 (Concurrency/Chaos) and L8 (Verification Claim Ceiling) --
/// HAND_WRITE_REQUIRED. See this module's own doc comment for the full
/// disclosure. Every function here always returns
/// [`NotYetImplemented`] -- never a fake success -- because none of this
/// logic exists anywhere in this repo today.
pub mod durability {
    use std::fmt;

    /// Names exactly which piece of L7/L8 has no real implementation yet.
    /// Each variant corresponds to one atlas requirement this repo does not
    /// meet for the wasm4pm Arazzo Compiler as of this wiring pass.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NotYetImplemented {
        /// L7: no atomic idempotency/correlation gate exists to admit or
        /// refuse a duplicate/replayed compile request before re-running
        /// [`super::compile`].
        ConcurrencyIdempotencyGate,
        /// L7/L8: no durable receipt head or replay state store exists --
        /// [`super::CompiledArazzo`] is an in-memory value with no persisted
        /// identity a later process can look up or reconstruct from.
        DurableReceiptHead,
        /// L8: no production caller invokes [`super::compile`] outside this
        /// crate's own tests -- the same reachability gap
        /// `docs/jira/v26.7.11/IMPLEMENTATION_STATUS.md:20` already names for
        /// wasm4pm-arazzo's own composed pipeline (PROJ-796).
        ProductionReachabilityTrace,
        /// L8: no chaos/restart-recovery test harness exists exercising
        /// process/engine restarts, stale/malformed results, or re-admission
        /// after a crash.
        ChaosRecoveryEvidence,
    }

    impl fmt::Display for NotYetImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let msg = match self {
                Self::ConcurrencyIdempotencyGate => {
                    "F14 (V12-014) L7: no atomic idempotency/correlation gate exists yet"
                }
                Self::DurableReceiptHead => {
                    "F14 (V12-014) L7/L8: no durable receipt head/replay state store exists yet"
                }
                Self::ProductionReachabilityTrace => {
                    "F14 (V12-014) L8: no production caller/reachability trace exists yet \
                     (see PROJ-796)"
                }
                Self::ChaosRecoveryEvidence => {
                    "F14 (V12-014) L8: no chaos/restart-recovery evidence exists yet"
                }
            };
            f.write_str(msg)
        }
    }

    impl std::error::Error for NotYetImplemented {}

    /// Would gate re-admission of a duplicate/replayed compile request behind
    /// an atomic correlation check keyed on `correlation_id`. Not built
    /// anywhere in this repo today; always refuses.
    pub fn admit_idempotent(_correlation_id: &str) -> Result<(), NotYetImplemented> {
        Err(NotYetImplemented::ConcurrencyIdempotencyGate)
    }

    /// Would persist a [`super::CompiledArazzo`]'s digest to a durable
    /// receipt head and read it back after a process/engine restart, proving
    /// replay reconstructs an equivalent digest. Not built anywhere in this
    /// repo today; always refuses.
    pub fn persist_receipt_head(_digest_hex: &str) -> Result<(), NotYetImplemented> {
        Err(NotYetImplemented::DurableReceiptHead)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal, single-step, single-workflow Arazzo 1.1.0 document -- every
    /// required field present (`arazzo`, `info.title`, `info.version`,
    /// `sourceDescriptions`, `workflows[].workflowId`,
    /// `workflows[].steps[].stepId`, and one step identity field), matching
    /// the field shapes confirmed against `wasm4pm_compat::arazzo`'s real
    /// `serde` structs (`ArazzoDescription`, `ArazzoInfo`, `SourceDescription`,
    /// `Workflow`, `Step`, `SuccessAction`) this turn.
    const SMOKE_DOCUMENT: &str = r#"{
      "arazzo": "1.1.0",
      "info": { "title": "F14 wiring smoke test", "version": "1.0.0" },
      "sourceDescriptions": [
        { "name": "s", "url": "openapi/s.yaml", "type": "openapi" }
      ],
      "workflows": [
        {
          "workflowId": "f14-smoke-workflow",
          "steps": [
            {
              "stepId": "f14-smoke-step",
              "operationId": "urn:test:f14/smoke",
              "onSuccess": [
                { "name": "finish", "type": "end" }
              ]
            }
          ]
        }
      ]
    }"#;

    /// Proves [`compile`] is real glue, not decoration: a real Arazzo JSON
    /// string goes in, a real lowered `AirProgram` plus a real BLAKE3
    /// [`AirDigest`] come out, through wasm4pm-arazzo's own unmodified
    /// pipeline functions.
    #[test]
    fn compile_admits_a_real_minimal_document_end_to_end() {
        let bump = Bump::new();
        let result = compile(
            SMOKE_DOCUMENT,
            "https://example.com/test/f14/smoke-base",
            &bump,
        )
        .expect("well-formed minimal Arazzo 1.1.0 document must compile");

        assert_eq!(result.program.workflows.len(), 1);
        assert_eq!(result.program.workflows[0].name, "f14-smoke-workflow");
        assert_eq!(result.program.workflows[0].steps.len(), 1);
        assert_eq!(
            result.program.workflows[0].steps[0].target.url,
            "urn:test:f14/smoke"
        );

        // Compiler Receipt (stage 8): deterministic across repeated digesting
        // of the same lowered program -- wasm4pm_arazzo::compile's own
        // guarantee, exercised here through this crate's composed function
        // rather than re-asserted as true by fiat.
        let digest_again = AirCompiler::digest_program(&result.program)
            .expect("digesting the same already-validated program twice must succeed");
        assert_eq!(result.digest, digest_again);
    }

    /// L4 (Refusal/Negative Sequence): a document declaring an unsupported
    /// Arazzo version must refuse at the earliest stage (Document Loader),
    /// never reaching lowering or compilation.
    #[test]
    fn compile_refuses_a_non_1_1_x_document_at_parse() {
        let bump = Bump::new();
        let doc = SMOKE_DOCUMENT.replacen("1.1.0", "1.0.0", 1);
        let result = compile(&doc, "https://example.com/test/f14/smoke-base", &bump);
        assert!(
            matches!(result, Err(wasm4pm_arazzo::Refusal::InvalidVersion(_))),
            "a non-1.1.x document must refuse InvalidVersion, got: {result:?}"
        );
    }

    /// L4/L5: a step declaring none of operationId/operationPath/channelPath/
    /// workflowId must refuse MissingIdentity (the Identity Index refusal
    /// branch) rather than silently lowering with an empty target.
    #[test]
    fn compile_refuses_a_step_with_no_identity() {
        let bump = Bump::new();
        let doc = SMOKE_DOCUMENT.replace(r#""operationId": "urn:test:f14/smoke","#, "");
        let result = compile(&doc, "https://example.com/test/f14/smoke-base", &bump);
        assert!(
            matches!(result, Err(wasm4pm_arazzo::Refusal::MissingIdentity(_))),
            "a step with no identity field must refuse MissingIdentity, got: {result:?}"
        );
    }

    /// L7/L8 stub: every durability function honestly refuses rather than
    /// faking success, and does so with a variant naming which piece is
    /// missing (per this session's standing rule against decorative Refusal
    /// placeholders that hide which gap is real).
    #[test]
    
    fn durability_functions_honestly_refuse_not_yet_implemented() {
        assert_eq!(
            durability::admit_idempotent("any-correlation-id"),
            Err(durability::NotYetImplemented::ConcurrencyIdempotencyGate)
        );
        assert_eq!(
            durability::persist_receipt_head("deadbeef"),
            Err(durability::NotYetImplemented::DurableReceiptHead)
        );
    }
}
