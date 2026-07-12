pub mod air;
pub mod compile;
pub mod lower;
pub mod normalizer;
pub mod parse;
pub mod resolve;
pub mod temporal;

use thiserror::Error;

/// Core Refusal type for wasm4pm-arazzo, following strict core team discipline.
///
/// # PROJ-754 taxonomy mapping (Rail B: Arazzo -> AIR compiler refusal coverage)
///
/// PROJ-754 (`docs/jira/v26.7.11/tickets/index.md`) requires a specific, tested `Refusal`
/// variant for every construct `crate::lower::lower_description` cannot compile. Two of the
/// five required constructs are already covered by pre-existing, pre-PROJ-754 variants and are
/// deliberately reused rather than duplicated (a fresh variant carrying the same meaning would
/// itself violate the "specific, not generic" rule this taxonomy exists to enforce):
///
/// - **missing operationId** (a step declaring none of `operationId`/`operationPath`/
///   `channelPath`/`workflowId`) -> [`Refusal::MissingIdentity`], tested by
///   `lower::tests::refuses_step_with_no_identity` and
///   `tests/end_to_end_lowering.rs::hand_written_production_arazzo_without_operation_identity_is_refused_at_lowering`.
/// - **unresolvable reference** (a success/failure action `$ref` that does not dereference
///   against the document's own `components`, or -- new in PROJ-754 -- a `Step.depends_on`
///   entry naming a step id absent from the workflow; or -- new in PROJ-810 -- a step parameter
///   `$ref` that does not dereference against `components.parameters`, or a
///   `RequestBody.replacements` target JSON Pointer that does not resolve within the payload) ->
///   [`Refusal::UnresolvableReference`].
///
/// The remaining three constructs are new in PROJ-754, added below:
/// [`Refusal::CyclicStepDependency`], [`Refusal::UnsupportedCriterion`],
/// [`Refusal::MalformedRetryPolicy`].
///
/// # PROJ-784 taxonomy mapping (typed refusal catalog: AIR, `docs/jira/v26.7.11/PRD.md`
/// section 18, `AIR_PARSE_REFUSED` / `AIR_REFERENCE_UNRESOLVED` /
/// `AIR_EXPRESSION_UNSUPPORTED` / `AIR_CRITERION_UNSUPPORTED`)
///
/// Three of the four PRD-named codes already had a real, tested, differently-named variant in
/// this enum before PROJ-784; per the same "reuse, don't duplicate" rule PROJ-754 already
/// established above, they are aliased by documentation here rather than renamed (renaming
/// would touch call sites across `lower.rs`, `temporal.rs`, `normalizer.rs`, and every test
/// module that matches on them, for a name-only change with no new information -- see Rule 9,
/// "API Stability Is a Promise", `.claude/rules/rust-agi-core-team.md`) and rather than
/// duplicated under an `Air`-prefixed twin (the crate's own naming convention never prefixes
/// variants with `Air` despite the crate being named for AIR -- `Parse`, `MissingIdentity`,
/// `UnresolvableReference` etc. are already bare):
///
/// - **`AIR_PARSE_REFUSED`** (a document that cannot even be admitted as Arazzo JSON) ->
///   [`Refusal::Parse`], fired by `parse::DocumentIndex::add_document` (and its
///   file/parallel-loading siblings) on a `serde_json` deserialization failure or a duplicate
///   document base URI. This is the earliest possible refusal point in the
///   parse -> resolve -> lower -> normalize -> compile pipeline: nothing downstream ever runs.
///   Tested end-to-end by
///   `tests/end_to_end_lowering.rs::malformed_arazzo_document_is_refused_at_parse_before_any_later_stage_runs`.
/// - **`AIR_REFERENCE_UNRESOLVED`** (a reference that cannot be dereferenced -- success/failure
///   action `$ref`, `Step.depends_on` entry, or cross-step `Variable`) ->
///   [`Refusal::UnresolvableReference`], already documented and tested above (PROJ-753/754) and
///   end-to-end tested by
///   `tests/end_to_end_lowering.rs::dangling_step_dependency_is_refused_end_to_end_through_the_real_pipeline`.
/// - **`AIR_EXPRESSION_UNSUPPORTED`** is genuinely new: before PROJ-784,
///   `lower::lower_step` read only the *keys* of `Step.outputs` (a
///   `BTreeMap<String, wasm4pm_compat::arazzo::OutputValue>`) and silently discarded every
///   value, so a `Selector`-shaped output (a structured JSONPath/XPath/JSONPointer selector
///   object -- the same three selector shapes `classify_criterion` already refuses for
///   criteria) was accepted with no record it was even present. [`Refusal::UnsupportedExpression`]
///   closes that gap: only the spec's plain runtime-expression string
///   (`OutputValue::Expression`) lowers; a `Selector`-shaped output value is refused. Fired by
///   `lower::classify_output_value`, called from `lower::lower_step`. Tested by
///   `lower::tests::refuses_selector_shaped_step_output` and
///   `tests/end_to_end_lowering.rs::selector_shaped_step_output_is_refused_end_to_end_through_the_real_pipeline`.
/// - **`AIR_CRITERION_UNSUPPORTED`** -> [`Refusal::UnsupportedCriterion`], already documented
///   and unit-tested above (PROJ-754); PROJ-784 adds its first full-pipeline (not just
///   `lower::lower_description`-direct) coverage:
///   `tests/end_to_end_lowering.rs::jsonpath_criterion_is_refused_end_to_end_through_the_real_pipeline`.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Refusal {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("URI resolution error: {0}")]
    UriResolution(String),
    #[error("Invalid arazzo version: {0}")]
    InvalidVersion(String),
    #[error("Missing identity: {0}")]
    MissingIdentity(String),
    #[error("Cyclic dependency or unresolvable cross-reference: {0}")]
    UnresolvableReference(String),
    #[error("Invalid workflow: {0}")]
    InvalidWorkflow(String),
    /// A workflow's `Step.depends_on` graph contains a cycle (including a step naming itself):
    /// no valid readiness order exists, so the workflow can never start. Fired by
    /// `lower::validate_step_dependencies`, called from `lower::lower_workflow` before any step
    /// is lowered to AIR.
    #[error("Cyclic step dependency: {0}")]
    CyclicStepDependency(String),
    /// A `Criterion.type` (Arazzo `expressionType`) names a selector shape -- JSONPath, XPath,
    /// regex, or a versioned selector object -- that this bridge has no evaluator for. Only the
    /// spec's default (`simple`, or `type` omitted) lowers to an AIR-evaluable condition; see
    /// `lower::classify_criterion`. Fired at lowering time rather than left to silently
    /// misclassify an unevaluatable selector as a plain boolean condition downstream.
    #[error("Unsupported criterion expression shape: {0}")]
    UnsupportedCriterion(String),
    /// A step's `timeout` (milliseconds) or a `type: retry` failure action's `retryLimit` /
    /// `retryAfter` carries a value that can never be satisfied: a zero-millisecond timeout, a
    /// zero retry limit on an action whose entire purpose is to retry, or a negative/
    /// non-finite `retryAfter` delay. Fired by `lower::validate_step_timeout` /
    /// `lower::validate_retry_policy`.
    #[error("Malformed timeout or retry policy: {0}")]
    MalformedRetryPolicy(String),
    /// A `Step.outputs` entry (`wasm4pm_compat::arazzo::OutputValue`) is a structured
    /// `Selector` object -- JSONPath, XPath, or JSONPointer -- rather than the spec's plain
    /// runtime-expression string. This bridge has no evaluator for any selector shape (the same
    /// boundary [`Refusal::UnsupportedCriterion`] enforces for `Criterion.type`); only
    /// `OutputValue::Expression` lowers. Fired by `lower::classify_output_value`, called from
    /// `lower::lower_step` (PROJ-784, AIR taxonomy code `AIR_EXPRESSION_UNSUPPORTED`).
    #[error("Unsupported output expression shape: {0}")]
    UnsupportedExpression(String),
    /// A document uses an Arazzo construct this bridge has deliberately not implemented lowering
    /// for. Before PROJ-753's own adversarial re-review, three such constructs were silently
    /// skipped/never read with no error surfaced at all -- a direct violation of this repo's
    /// Invariant #1 ("no silent defaults -- every error is a typed `Refusal`"): a
    /// `ParameterOrReference::Reference` (`#/components/parameters/<name>` `$ref`
    /// dereferencing), a non-empty `RequestBody.replacements` (targeted JSON-pointer payload
    /// overrides), and a workflow's own `successActions`/`failureActions` (workflow-level, as
    /// opposed to the step-level `Step.onSuccess`/`onFailure` this bridge does lower). PROJ-753
    /// closed all three with an unconditional refusal (no silent data loss, but no real
    /// end-to-end support either); PROJ-810 replaces that refusal with real implementations for
    /// all three -- see `lower.rs`'s module doc for the exact per-construct lowering. This
    /// variant now fires only for the one case genuinely left unimplemented: a
    /// `RequestBody.replacements` declared with no `payload` for this bridge (which has no
    /// OpenAPI operation resolver) to apply the replacements onto. Fired by `lower::lower_step`.
    #[error("Unsupported Arazzo feature: {0}")]
    UnsupportedFeature(String),
}
