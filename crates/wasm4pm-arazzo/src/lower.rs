//! Bridges a resolved `wasm4pm_compat::arazzo::ArazzoDescription` (real output of
//! `parse::DocumentIndex::add_document` + `resolve::normalize_uris`) into this crate's own
//! `air::AirProgram` (PROJ-753, PRD.md sec.7.6 / Rail B: "Arazzo -> wasm4pm parser -> AIR").
//!
//! Before this module, every `AirProgram` in this crate was a hand-built test fixture -- no
//! function anywhere converted a real parsed Arazzo document into AIR
//! (`docs/jira/v26.7.11/RAIL_A_B_STATUS.md`, "Rail B ... disconnected halves"). This module is
//! that bridge's first real implementation.
//!
//! Pipeline position: `lower_description` runs *before* `normalizer::ArazzoNormalizer::normalize`
//! (which resolves `AirExpr::Variable` cross-step references via `temporal::ReferenceResolver`)
//! and before `compile::AirCompiler::compile_to_wasm`. This module only restructures the
//! document into AIR shape; it does not resolve references itself.
//!
//! # Scope and its boundaries (documented, not silent)
//!
//! - **Step target** (`AirTarget`): an Arazzo Step Object identifies what it invokes via
//!   exactly one of `operationId` / `operationPath` / `channelPath` / `workflowId`. This crate
//!   has no OpenAPI/AsyncAPI operation resolver, so `AirTarget::url` carries that identifying
//!   string verbatim (not a resolved HTTP URL) and `AirTarget::method` carries which of the
//!   four fields it came from (`"operationId"` etc.) -- a real fact about the source document,
//!   never a fabricated HTTP method. A step declaring none of the four is refused with
//!   [`Refusal::MissingIdentity`].
//! - **Parameters and request bodies** (`AirAction::inputs`): each `Parameter.value` /
//!   `RequestBody.payload` is lowered by [`lower_json_value`] -- see its doc comment for the
//!   exact `Literal` vs. `Variable` classification rule. A non-empty `RequestBody.replacements`
//!   (PROJ-810: targeted JSON-pointer overrides within a payload, Arazzo's "Payload Replacement
//!   Object") is applied by [`apply_payload_replacements`] before the payload is lowered: each
//!   replacement's `target` is an RFC 6901 JSON Pointer resolved (and set) within the payload,
//!   in declaration order, each replacement seeing the previous one's result. Only the spec's
//!   default target shape (`targetSelectorType` omitted, or explicitly `jsonpointer`) is
//!   implemented; a non-jsonpointer target shape is refused with
//!   [`Refusal::UnsupportedExpression`] (the same selector-shape boundary
//!   [`classify_criterion`]/[`classify_output_value`] enforce elsewhere), and a `target` that
//!   does not resolve within the payload (an absent object key, an out-of-bounds array index) is
//!   refused with [`Refusal::UnresolvableReference`] rather than silently no-op'd. Replacements
//!   declared with no `payload` to apply them to are refused with
//!   [`Refusal::UnsupportedFeature`]: this bridge has no OpenAPI operation resolver to source the
//!   implicit default payload the replacements would otherwise apply against (PROJ-753 had
//!   refused every non-empty `replacements` unconditionally; PROJ-810 narrows that refusal to
//!   only this genuinely-out-of-scope case).
//! - **Parameter/component `$ref`s** (`ParameterOrReference::Reference`): dereferencing
//!   `#/components/parameters/<name>` (PROJ-810) is resolved by [`resolve_parameter_reference`]
//!   against `Components.parameters`, the same local-only boundary
//!   [`resolve_success_reference`]/[`resolve_failure_reference`] already enforce for routing-rule
//!   references: a cross-document reference, or a name absent from this document's own
//!   `components.parameters`, is refused with [`Refusal::UnresolvableReference`] (PROJ-753 had
//!   refused every `$ref`-shaped parameter unconditionally with
//!   [`Refusal::UnsupportedFeature`]; PROJ-810 replaces that with real dereferencing, keeping the
//!   refusal only for a genuinely dangling or cross-document reference).
//! - **Outputs** (`AirAction::outputs`): one `AirExpr::Literal` per key of `Step.outputs`
//!   (`BTreeMap`, so already sorted -- no `HashMap` iteration-order risk), carrying the bare
//!   output name. This matches `temporal::ReferenceResolver`'s existing (pre-PROJ-753,
//!   already-tested) flat single-workflow variable namespace: it resolves references by bare
//!   name against earlier steps' declared output names, not step-qualified paths.
//! - **Success/failure routing** (`AirStep::on_success` / `on_failure`): inline
//!   `SuccessAction`/`FailureAction` objects lower directly (see [`lower_success_routing`] /
//!   [`lower_failure_routing`]). `Reference(ReusableObject)` entries are dereferenced only
//!   against this *same* document's own `components.successActions` /
//!   `components.failureActions` (a bounded, local, deterministic lookup); a reference to
//!   another document, or to a name absent from `components`, is refused with
//!   [`Refusal::UnresolvableReference`] rather than silently dropped.
//! - **Workflow-level `successActions`/`failureActions`** (as opposed to `Step.onSuccess`/
//!   `onFailure`, which do lower -- see "Success/failure routing" above): PROJ-810 implements
//!   the Arazzo spec's own semantics for these fields ("applicable for all steps... can be
//!   overridden at the step level but cannot be removed there", mirrored on `Step.onSuccess`/
//!   `onFailure`: "the new definition will override [a workflow-level action of the same name]
//!   but can never remove it"). [`lower_workflow`] resolves the workflow's own
//!   `successActions`/`failureActions` once (dereferencing any `Reference` entries against
//!   `components`, same as step-level routing) and [`lower_step`] merges them with each step's
//!   own via [`merge_success_actions`]/[`merge_failure_actions`]: a step-level action whose
//!   `name` matches a workflow-level one replaces it in place; a step-level action with a new
//!   name is appended; a workflow-level action the step doesn't mention passes through
//!   unchanged. A step declaring no actions of its own gets exactly the workflow-level list.
//!   (PROJ-753 had refused any workflow declaring either field at all with
//!   [`Refusal::UnsupportedFeature`]; PROJ-810 replaces that with this real merge -- there is no
//!   longer a "genuinely invalid" shape of this construct to refuse, since every
//!   `SuccessActionOrReference`/`FailureActionOrReference` entry is validated the same way at
//!   both the workflow and step level.)
//! - **Workflow-level `depends_on` and workflow `inputs`** (distinct fields from the
//!   workflow-level routing above, and from `Step.depends_on`/step parameters, both of which do
//!   lower or refuse -- see above): still genuinely out of this bridge's scope, and still not
//!   lowered. Unlike the three constructs above, PROJ-753's adversarial re-review did not flag
//!   these as silently-skipped-with-no-disclosure -- they remain a disclosed, pre-existing gap
//!   (the same category as `Step.success_criteria`, noted below), left to a follow-up ticket
//!   rather than addressed here.
//! - **Step-level `depends_on`** (PROJ-754): validated for referential and structural
//!   soundness by [`validate_step_dependencies`] before any step in the workflow is lowered --
//!   a `depends_on` entry naming a step id absent from the workflow is refused with
//!   [`Refusal::UnresolvableReference`]; a cycle in the dependency graph (including a step
//!   naming itself) is refused with [`Refusal::CyclicStepDependency`]. This is admission-time
//!   soundness checking only: the validated graph is not yet carried into `AirStep` as a
//!   schedulable field (execution-time dependency readiness is PROJ-756's scope, Rail C).
//! - **Criterion expression shape** (PROJ-754): [`classify_criterion`], called from
//!   [`lower_criteria`], refuses a `Criterion` whose `type` requests a selector shape (JSONPath,
//!   XPath, regex, or a versioned selector object) this bridge has no evaluator for, with
//!   [`Refusal::UnsupportedCriterion`]. Only the spec default (`type` omitted, or explicitly
//!   `simple`) lowers to an AIR-evaluable literal condition. This check is wired only into
//!   `SuccessAction.criteria` / `FailureAction.criteria` (routing-rule gating, the only
//!   `Criterion` call site this bridge has ever lowered). `Step.success_criteria`
//!   (step-level pass/fail gating, a distinct field) is not read anywhere in this module,
//!   before or after PROJ-754 -- a real, pre-existing gap, not silently misrepresented as
//!   covered by this check.
//! - **Output expression shape** (PROJ-784): [`classify_output_value`], called from
//!   [`lower_step`] for every entry of `Step.outputs`, refuses a `Selector`-shaped output value
//!   (`wasm4pm_compat::arazzo::OutputValue::Selector` -- a structured JSONPath/XPath/JSONPointer
//!   selector object) with [`Refusal::UnsupportedExpression`]. Before PROJ-784, only the
//!   `BTreeMap`'s *keys* were read (see "Outputs" above); the value was never inspected, so a
//!   `Selector`-shaped output was silently accepted with no record it was even present -- the
//!   same silent-acceptance gap PROJ-754's `classify_criterion` closed for `Criterion.type`.
//!   Only the spec's plain runtime-expression string (`OutputValue::Expression`) lowers.
//! - **Timeout / retry policy** (PROJ-754): [`validate_step_timeout`] refuses a step declaring
//!   `timeout: 0`; [`validate_retry_policy`] refuses a `type: retry` failure action whose
//!   `retryLimit` is `0` or whose `retryAfter` is negative or non-finite. Both fields remain
//!   optional per spec -- only an explicitly present, unsatisfiable value is refused, with
//!   [`Refusal::MalformedRetryPolicy`].
//!
//! - **Step declaration order vs. `depends_on` order** (PROJ-784 correction): before this fix,
//!   `AirWorkflow.steps` were lowered in raw source declaration order, but
//!   `temporal::ReferenceResolver::resolve` (which runs later, in `normalizer::normalize`)
//!   treats array order as the only valid "earlier step" order -- so a step using `depends_on`
//!   to declare a legitimate non-textual execution order (e.g. declared first, but depending on
//!   and referencing a step declared second) would lower successfully here and then be *wrongly*
//!   refused at normalization as an `UnresolvableReference`, even though the dependency graph
//!   itself was sound. [`lower_workflow`] now calls [`topological_sort_step_indices`] after
//!   [`validate_step_dependencies`] confirms the graph is acyclic, and lowers steps in that
//!   topological order (ties broken by original declaration index, so a workflow with no
//!   `depends_on` edges at all lowers in exactly its original order, byte-for-byte unchanged).
//!
//! # Complexity
//! O(w + s + p + a + d + r) where w = workflow count, s = total step count, p = total parameter +
//! output count, a = total success/failure routing-rule count (workflow-level plus step-level),
//! d = total `depends_on` edge count, r = total `RequestBody.replacements` entry count. One
//! linear pass over `doc` plus, per workflow: one `O(steps + edges)` dependency-graph traversal
//! (`validate_step_dependencies`, iterative DFS, each node/edge visited once) and one
//! `O(steps + edges + log(steps))` topological sort (`topological_sort_step_indices`, Kahn's
//! algorithm over a binary-heap ready set); no `HashMap` iteration ever drives output order or
//! content (`Step.outputs` / `Components.*` are already `BTreeMap`s; the id -> index `HashMap`s
//! built by both dependency-graph passes are only ever looked up by key, never iterated).
//! PROJ-810 adds, per step: `O(workflow_defaults * step_actions)` for
//! [`merge_success_actions`]/[`merge_failure_actions`] (a linear name-scan, not a `HashMap` --
//! both lists are single digits in real documents) and `O(replacements * pointer_depth)` for
//! [`apply_payload_replacements`] (each replacement walks its own JSON Pointer once via
//! `serde_json::Value::pointer_mut`).

use crate::air::{
    AirAction, AirExpr, AirProgram, AirRouting, AirRoutingOutcome, AirStep, AirTarget, AirWorkflow,
};
use crate::Refusal;
use bumpalo::collections::{String as BumpString, Vec as BumpVec};
use bumpalo::Bump;
use serde_json::Value;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use wasm4pm_compat::arazzo::{
    ArazzoDescription, Components, Criterion, ExpressionKind, ExpressionTypeOrKind, FailureAction,
    FailureActionOrReference, FailureActionType, OutputValue, Parameter, ParameterOrReference,
    PayloadReplacement, ReusableObject, SelectorKind, SelectorType, Step, SuccessAction,
    SuccessActionOrReference, SuccessActionType, Workflow,
};

/// Lowers every workflow in `doc` into an [`AirProgram`] allocated in `bump`. See the module
/// doc comment for the exact field-by-field mapping and its documented scope boundaries.
///
/// # Errors
/// [`Refusal::InvalidWorkflow`] if `doc` declares zero workflows, or a workflow declares zero
/// steps. [`Refusal::MissingIdentity`] if a workflow/step has an empty id, a step names none of
/// `operationId`/`operationPath`/`channelPath`/`workflowId`, or a `goto` routing action names
/// neither `stepId` nor `workflowId`. [`Refusal::UnresolvableReference`] if a success/failure
/// action reference, a step parameter reference, or a request-body payload replacement target
/// cannot be dereferenced/resolved (see module doc for the exact per-construct boundary).
/// [`Refusal::UnsupportedExpression`] if a payload replacement declares a non-jsonpointer
/// `targetSelectorType`. [`Refusal::UnsupportedFeature`] if a document declares
/// `RequestBody.replacements` with no `payload` to apply them to -- the one remaining
/// genuinely-out-of-scope shape of the three constructs PROJ-753 originally refused
/// unconditionally (see module doc).
pub fn lower_description<'bump>(
    doc: &ArazzoDescription,
    bump: &'bump Bump,
) -> Result<AirProgram<'bump>, Refusal> {
    if doc.workflows.is_empty() {
        return Err(Refusal::InvalidWorkflow(
            "Arazzo document declares zero workflows to lower".to_string(),
        ));
    }
    let mut workflows = BumpVec::with_capacity_in(doc.workflows.len(), bump);
    for wf in &doc.workflows {
        workflows.push(lower_workflow(wf, doc.components.as_ref(), bump)?);
    }
    Ok(AirProgram { workflows })
}

fn lower_workflow<'bump>(
    wf: &Workflow,
    components: Option<&Components>,
    bump: &'bump Bump,
) -> Result<AirWorkflow<'bump>, Refusal> {
    if wf.workflow_id.is_empty() {
        return Err(Refusal::MissingIdentity(
            "workflow has an empty workflowId".to_string(),
        ));
    }
    if wf.steps.is_empty() {
        return Err(Refusal::InvalidWorkflow(format!(
            "workflow '{}' declares zero steps",
            wf.workflow_id
        )));
    }
    // Workflow-level `successActions`/`failureActions` (PROJ-810): resolved once per workflow
    // (dereferencing any `Reference` entries against `components`, same as step-level routing),
    // then merged into every step's own routing by `lower_step` -- see the module doc's
    // "Workflow-level successActions/failureActions" section for the exact override-but-cannot-
    // remove semantics.
    let workflow_success_defaults = resolve_success_actions(&wf.success_actions, components)?;
    let workflow_failure_defaults = resolve_failure_actions(&wf.failure_actions, components)?;
    validate_step_dependencies(wf)?;
    let order = topological_sort_step_indices(wf)?;
    let mut steps = BumpVec::with_capacity_in(wf.steps.len(), bump);
    for &idx in &order {
        steps.push(lower_step(
            &wf.steps[idx],
            components,
            &workflow_success_defaults,
            &workflow_failure_defaults,
            bump,
        )?);
    }
    Ok(AirWorkflow {
        name: BumpString::from_str_in(&wf.workflow_id, bump),
        steps,
    })
}

/// Validates that `wf.steps`' `depends_on` graph is referentially sound (every named id is a
/// real step in this workflow) and acyclic (a valid execution order exists). See the module
/// doc's "Step-level `depends_on`" boundary note for what this check does *not* yet do.
///
/// # Complexity
/// O(steps + edges): one `HashMap` build for id -> index lookup (never iterated, so its
/// unordered layout introduces no determinism risk -- same pattern as
/// `temporal::ReferenceResolver::resolve`'s `resolved` map), one adjacency-list build, then one
/// iterative DFS over the whole graph. Each step is pushed/popped from the DFS stack exactly
/// once; each `depends_on` edge is examined exactly once via the per-frame cursor.
fn validate_step_dependencies(wf: &Workflow) -> Result<(), Refusal> {
    let n = wf.steps.len();
    let mut index_of: HashMap<&str, usize> = HashMap::with_capacity(n);
    for (i, step) in wf.steps.iter().enumerate() {
        index_of.insert(step.step_id.as_str(), i);
    }

    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, step) in wf.steps.iter().enumerate() {
        for dep in &step.depends_on {
            let &predecessor = index_of.get(dep.as_str()).ok_or_else(|| {
                Refusal::UnresolvableReference(format!(
                    "step '{}' in workflow '{}' declares depends_on '{}', which is not a step \
                     id in this workflow",
                    step.step_id, wf.workflow_id, dep
                ))
            })?;
            adjacency[i].push(predecessor);
        }
    }

    // 0 = unvisited (white), 1 = on the current DFS path (gray), 2 = fully explored (black).
    let mut color = vec![0u8; n];
    for start in 0..n {
        if color[start] != 0 {
            continue;
        }
        // Iterative DFS with an explicit (node, next-child-cursor) stack -- avoids unbounded
        // recursion depth on an adversarial long dependency chain.
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        color[start] = 1;
        while let Some(&(node, next_idx)) = stack.last() {
            if next_idx < adjacency[node].len() {
                let child = adjacency[node][next_idx];
                if let Some(top) = stack.last_mut() {
                    top.1 += 1;
                }
                match color[child] {
                    0 => {
                        color[child] = 1;
                        stack.push((child, 0));
                    }
                    1 => {
                        return Err(Refusal::CyclicStepDependency(format!(
                            "workflow '{}' has a cyclic step dependency involving step '{}'",
                            wf.workflow_id, wf.steps[child].step_id
                        )));
                    }
                    _ => {} // already fully explored: no cycle reachable through this edge
                }
            } else {
                color[node] = 2;
                stack.pop();
            }
        }
    }
    Ok(())
}

/// Computes a deterministic topological order over `wf.steps`'s `depends_on` graph, so
/// [`lower_workflow`] can lower `AirWorkflow.steps` in dependency-respecting order instead of
/// raw source declaration order (PROJ-784 correction: see the module doc's "Step declaration
/// order vs. `depends_on` order" note for why declaration order alone is not a safe assumption
/// once `depends_on` is used non-textually). Must be called only after
/// [`validate_step_dependencies`] has confirmed the graph is acyclic and referentially sound --
/// this function re-derives referential soundness defensively (see below) rather than trusting
/// that invariant silently, but does not re-detect a cycle by construction; the length check at
/// the end is the cycle-safety net for a graph that somehow reached this function un-validated.
///
/// # Algorithm
/// Kahn's algorithm: `dep_count[i]` starts as the number of prerequisites (`depends_on` entries)
/// step `i` has; a step enters the ready set once its `dep_count` reaches zero. At each step,
/// the *smallest original index* among the currently-ready steps is scheduled next -- ties are
/// only possible among steps with no relative-order constraint between them (including the
/// all-ties case of a workflow with zero `depends_on` edges at all, which this reduces to
/// exactly), and breaking by original declaration index keeps the result deterministic and
/// stable rather than depending on any `HashMap`/`HashSet` iteration order or randomness.
///
/// # Complexity
/// O(steps + edges + steps * log(steps)): one reverse-adjacency (`successors`) build in
/// `O(steps + edges)`, then each step enters and leaves the `BinaryHeap` ready set exactly once
/// (`O(log(steps))` per push/pop).
fn topological_sort_step_indices(wf: &Workflow) -> Result<Vec<usize>, Refusal> {
    let n = wf.steps.len();
    let mut index_of: HashMap<&str, usize> = HashMap::with_capacity(n);
    for (i, step) in wf.steps.iter().enumerate() {
        index_of.insert(step.step_id.as_str(), i);
    }

    // dep_count[i] = number of not-yet-scheduled prerequisites step i still has.
    // successors[p] = steps that name p in their own depends_on (p must be scheduled first).
    let mut dep_count: Vec<usize> = vec![0; n];
    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, step) in wf.steps.iter().enumerate() {
        for dep in &step.depends_on {
            // Referential soundness is already checked by `validate_step_dependencies`, called
            // immediately before this function in `lower_workflow`; this is a defensive
            // re-check (a typed Refusal, not a panic/index-out-of-bounds) in case that
            // invariant is ever violated by a future caller, not new externally-visible
            // behavior.
            let &predecessor = index_of.get(dep.as_str()).ok_or_else(|| {
                Refusal::UnresolvableReference(format!(
                    "step '{}' in workflow '{}' declares depends_on '{}', which is not a step \
                     id in this workflow",
                    step.step_id, wf.workflow_id, dep
                ))
            })?;
            dep_count[i] += 1;
            successors[predecessor].push(i);
        }
    }

    let mut ready: BinaryHeap<Reverse<usize>> = BinaryHeap::with_capacity(n);
    for (i, &count) in dep_count.iter().enumerate() {
        if count == 0 {
            ready.push(Reverse(i));
        }
    }

    let mut order = Vec::with_capacity(n);
    while let Some(Reverse(i)) = ready.pop() {
        order.push(i);
        for &succ in &successors[i] {
            dep_count[succ] -= 1;
            if dep_count[succ] == 0 {
                ready.push(Reverse(succ));
            }
        }
    }

    if order.len() != n {
        // Unreachable in practice: `lower_workflow` only calls this after
        // `validate_step_dependencies` has confirmed the graph is acyclic, and a finite acyclic
        // graph always fully drains through Kahn's algorithm. Fails loud with a typed Refusal
        // rather than silently returning a truncated/wrong order or panicking on an
        // out-of-bounds index later, should that precondition ever be violated.
        return Err(Refusal::CyclicStepDependency(format!(
            "workflow '{}' has a cyclic step dependency (topological sort could not order all \
             {n} steps, only {})",
            wf.workflow_id,
            order.len()
        )));
    }

    Ok(order)
}

fn lower_step<'bump>(
    step: &Step,
    components: Option<&Components>,
    workflow_success_defaults: &[SuccessAction],
    workflow_failure_defaults: &[FailureAction],
    bump: &'bump Bump,
) -> Result<AirStep<'bump>, Refusal> {
    if step.step_id.is_empty() {
        return Err(Refusal::MissingIdentity(
            "step has an empty stepId".to_string(),
        ));
    }
    validate_step_timeout(step)?;
    let target = lower_target(step, bump)?;

    let mut inputs = BumpVec::new_in(bump);
    for param in &step.parameters {
        match param {
            ParameterOrReference::Parameter(p) => {
                inputs.push(lower_json_value(&p.value, bump));
            }
            ParameterOrReference::Reference(r) => {
                // Local `#/components/parameters/<name>` dereferencing (PROJ-810): resolved
                // against `components`, the same local-only boundary
                // `resolve_success_reference`/`resolve_failure_reference` already enforce for
                // routing-rule references. A dangling or cross-document reference still refuses
                // (see `resolve_parameter_reference`); a real one now lowers instead of vanishing
                // from `AirAction::inputs` with no error (PROJ-753's original gap).
                let resolved = resolve_parameter_reference(r, components)?;
                inputs.push(lower_json_value(&resolved.value, bump));
            }
        }
    }
    if let Some(body) = &step.request_body {
        match (&body.payload, body.replacements.is_empty()) {
            (Some(payload), true) => {
                inputs.push(lower_json_value(payload, bump));
            }
            (Some(payload), false) => {
                // `RequestBody.replacements` (PROJ-810: targeted JSON-pointer overrides within a
                // payload) applied before lowering -- see `apply_payload_replacements` for the
                // exact target-resolution and refusal rules (PROJ-753's original unconditional
                // refusal is now narrowed to only the genuinely out-of-scope no-payload case,
                // below).
                let replaced =
                    apply_payload_replacements(&step.step_id, payload, &body.replacements)?;
                inputs.push(lower_json_value(&replaced, bump));
            }
            (None, true) => {}
            (None, false) => {
                // Replacements with no base `payload` to apply them onto: this bridge has no
                // OpenAPI operation resolver to source the implicit default payload the Arazzo
                // spec would otherwise let the runtime supply, so there is nothing to apply the
                // replacement to. A genuinely out-of-scope construct, refused rather than
                // silently dropped.
                return Err(Refusal::UnsupportedFeature(format!(
                    "step '{}' request body declares {} payload replacement(s) but no payload \
                     for this bridge (which has no OpenAPI operation resolver) to apply them \
                     onto",
                    step.step_id,
                    body.replacements.len()
                )));
            }
        }
    }

    let mut outputs = BumpVec::new_in(bump);
    for (name, value) in step.outputs.iter() {
        classify_output_value(&step.step_id, name, value)?;
        outputs.push(AirExpr::Literal(BumpString::from_str_in(name, bump)));
    }

    // Workflow-level `successActions`/`failureActions` (PROJ-810): merged with this step's own
    // per the Arazzo spec's override-but-cannot-remove semantics -- see
    // `merge_success_actions`/`merge_failure_actions` and the module doc's "Workflow-level
    // successActions/failureActions" section.
    let step_success_actions = resolve_success_actions(&step.on_success, components)?;
    let merged_success = merge_success_actions(workflow_success_defaults, &step_success_actions);
    let on_success = lower_resolved_success_actions(&merged_success, bump)?;

    let step_failure_actions = resolve_failure_actions(&step.on_failure, components)?;
    let merged_failure = merge_failure_actions(workflow_failure_defaults, &step_failure_actions);
    for action in &merged_failure {
        if action.action_type == FailureActionType::Retry {
            validate_retry_policy(action)?;
        }
    }
    let on_failure = lower_resolved_failure_actions(&merged_failure, bump)?;

    Ok(AirStep {
        name: BumpString::from_str_in(&step.step_id, bump),
        target,
        action: AirAction { inputs, outputs },
        on_success,
        on_failure,
    })
}

/// Maps a Step Object's single identity field to `AirTarget`. Preference order (fixed and
/// documented, not incidental): operationId, then operationPath, then channelPath, then
/// workflowId -- operationId is the common case and the one PROJ-752's manufactured documents
/// use exclusively. See the module doc for why `method` carries the identity kind rather than
/// an HTTP method.
fn lower_target<'bump>(step: &Step, bump: &'bump Bump) -> Result<AirTarget<'bump>, Refusal> {
    if let Some(op_id) = &step.operation_id {
        return Ok(AirTarget {
            url: BumpString::from_str_in(op_id, bump),
            method: BumpString::from_str_in("operationId", bump),
        });
    }
    if let Some(op_path) = &step.operation_path {
        return Ok(AirTarget {
            url: BumpString::from_str_in(op_path, bump),
            method: BumpString::from_str_in("operationPath", bump),
        });
    }
    if let Some(chan_path) = &step.channel_path {
        return Ok(AirTarget {
            url: BumpString::from_str_in(chan_path, bump),
            method: BumpString::from_str_in("channelPath", bump),
        });
    }
    if let Some(wf_id) = &step.workflow_id {
        return Ok(AirTarget {
            url: BumpString::from_str_in(wf_id, bump),
            method: BumpString::from_str_in("workflowId", bump),
        });
    }
    Err(Refusal::MissingIdentity(format!(
        "step '{}' declares none of operationId/operationPath/channelPath/workflowId",
        step.step_id
    )))
}

/// Recognizes the one Arazzo runtime-expression shape this bridge resolves as a cross-step
/// reference: `$steps.<step_id>.outputs.<name>` (spec section "Runtime Expressions"). The
/// `<step_id>` component is intentionally discarded (not validated against the actual
/// producing step's id): `temporal::ReferenceResolver` already validates *declaration order* (a
/// reference only resolves if some earlier step declared that output name), which is the real
/// invariant this crate enforces today -- see its own doc comment. Any other value -- a plain
/// literal, or another legitimate runtime expression such as `$inputs.x`, `$response...`,
/// `$statusCode` -- is carried through as an opaque `AirExpr::Literal` of its own text: those
/// other scopes have no AIR representation yet and must not be silently misclassified as an
/// unresolved step reference.
fn lower_json_value<'bump>(value: &Value, bump: &'bump Bump) -> AirExpr<'bump> {
    if let Value::String(s) = value {
        if let Some(name) = step_output_reference_name(s) {
            return AirExpr::Variable(BumpString::from_str_in(name, bump));
        }
        return AirExpr::Literal(BumpString::from_str_in(s, bump));
    }
    // Non-string JSON (object/array/number/bool/null): `serde_json`'s own serialization is a
    // deterministic function of the parsed value (`serde_json::Map` is `BTreeMap`-backed by
    // default in this workspace -- no HashMap iteration order risk), never a hand-typed
    // placeholder.
    let text = value.to_string();
    AirExpr::Literal(BumpString::from_str_in(&text, bump))
}

fn step_output_reference_name(expr: &str) -> Option<&str> {
    let rest = expr.strip_prefix("$steps.")?;
    let (_, name) = rest.split_once(".outputs.")?;
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Rejects a step `timeout` (milliseconds) that can never be satisfied. `timeout: 0` means no
/// operation could ever complete in time; `None` (undeclared) is valid and left to the
/// runtime's own default -- only an explicitly present, unsatisfiable value is refused. The
/// field's type is `Option<u64>`, so a negative value is already impossible to construct from a
/// parsed document; `0` is the one unsatisfiable value the type system cannot rule out.
fn validate_step_timeout(step: &Step) -> Result<(), Refusal> {
    if step.timeout == Some(0) {
        return Err(Refusal::MalformedRetryPolicy(format!(
            "step '{}' declares timeout: 0 (milliseconds), which no operation can satisfy",
            step.step_id
        )));
    }
    Ok(())
}

/// Rejects a `type: retry` failure action whose retry policy can never fire as declared: a
/// `retryLimit` of `0` contradicts the action's own purpose (retry, but permit zero attempts),
/// and a `retryAfter` delay that is negative, `NaN`, or infinite is not a valid wait duration.
/// Both fields are optional per the Arazzo spec (the runtime supplies its own default when
/// absent) -- only an explicitly present, invalid value is refused. Only called for
/// `action_type == Retry`; `end`/`goto` failure actions do not carry retry semantics.
fn validate_retry_policy(action: &FailureAction) -> Result<(), Refusal> {
    if action.retry_limit == Some(0) {
        return Err(Refusal::MalformedRetryPolicy(format!(
            "failure action '{}' is type=retry with retryLimit: 0, which permits zero retry \
             attempts",
            action.name
        )));
    }
    if let Some(after) = action.retry_after {
        if !after.is_finite() || after < 0.0 {
            return Err(Refusal::MalformedRetryPolicy(format!(
                "failure action '{}' is type=retry with retryAfter: {after}, which is not a \
                 finite, non-negative delay",
                action.name
            )));
        }
    }
    Ok(())
}

/// Rejects a `Criterion` whose declared `type` (Arazzo `expressionType`) requests a selector
/// shape this bridge has no evaluator for. Only the spec's default (`type` omitted, or
/// explicitly `simple`) lowers to an AIR-evaluable literal condition; `regex` / `jsonpath` /
/// `xpath` / `jsonpointer`, and any versioned selector `ExpressionType` object, must be refused
/// rather than silently carried through as if they were a plain boolean condition string --
/// nothing downstream in this crate or in AIR evaluates those selector shapes.
fn classify_criterion(c: &Criterion) -> Result<(), Refusal> {
    match &c.expression_type {
        None => Ok(()),
        Some(ExpressionTypeOrKind::Kind(ExpressionKind::Simple)) => Ok(()),
        Some(other) => Err(Refusal::UnsupportedCriterion(format!(
            "criterion '{}' declares expression type {other:?}, which this bridge has no \
             evaluator for (only the default `simple` shape is supported)",
            c.condition
        ))),
    }
}

/// Rejects a `Step.outputs` entry whose declared value is a structured `Selector` object
/// (JSONPath, XPath, or JSONPointer) rather than the spec's plain runtime-expression string.
/// This bridge has no evaluator for any selector shape -- the same boundary
/// [`classify_criterion`] enforces for `Criterion.type` -- so only `OutputValue::Expression`
/// lowers; a `Selector`-shaped value must be refused rather than silently accepted with its
/// shape (and the fact it carries a selector at all) discarded, which is what happened before
/// PROJ-784 added this check (only the map's keys were ever read).
fn classify_output_value(
    step_id: &str,
    output_name: &str,
    value: &OutputValue,
) -> Result<(), Refusal> {
    match value {
        OutputValue::Expression(_) => Ok(()),
        OutputValue::Selector(selector) => Err(Refusal::UnsupportedExpression(format!(
            "step '{step_id}' output '{output_name}' declares a {:?}-typed selector object, \
             which this bridge has no evaluator for (only a plain runtime-expression string is \
             supported)",
            selector.selector_type
        ))),
    }
}

fn lower_criteria<'bump>(
    criteria: &[Criterion],
    bump: &'bump Bump,
) -> Result<BumpVec<'bump, AirExpr<'bump>>, Refusal> {
    let mut out = BumpVec::with_capacity_in(criteria.len(), bump);
    for c in criteria {
        classify_criterion(c)?;
        out.push(AirExpr::Literal(BumpString::from_str_in(
            &c.condition,
            bump,
        )));
    }
    Ok(out)
}

/// Resolves each `SuccessActionOrReference` entry to a concrete, owned `SuccessAction`,
/// dereferencing `Reference` entries against `components` (see [`resolve_success_reference`]).
/// Preserves declaration order. Used for both step-level `Step.onSuccess` and workflow-level
/// `Workflow.successActions` (PROJ-810) -- the two lists are merged by
/// [`merge_success_actions`] before either is lowered to AIR by
/// [`lower_resolved_success_actions`].
fn resolve_success_actions(
    actions: &[SuccessActionOrReference],
    components: Option<&Components>,
) -> Result<Vec<SuccessAction>, Refusal> {
    actions
        .iter()
        .map(|action| match action {
            SuccessActionOrReference::Action(a) => Ok(a.clone()),
            SuccessActionOrReference::Reference(r) => resolve_success_reference(r, components),
        })
        .collect()
}

/// Same contract as [`resolve_success_actions`], for `FailureActionOrReference`.
fn resolve_failure_actions(
    actions: &[FailureActionOrReference],
    components: Option<&Components>,
) -> Result<Vec<FailureAction>, Refusal> {
    actions
        .iter()
        .map(|action| match action {
            FailureActionOrReference::Action(a) => Ok(a.clone()),
            FailureActionOrReference::Reference(r) => resolve_failure_reference(r, components),
        })
        .collect()
}

/// Merges a workflow's default success actions with one step's own (PROJ-810), per the Arazzo
/// spec's Workflow Object `successActions` field: "applicable for all steps ... can be
/// overridden at the step level but cannot be removed there" (mirrored on `Step.onSuccess`:
/// "the new definition will override [a workflow-level action of the same name] but can never
/// remove it"). A step-level action whose `name` matches a workflow-level action replaces it in
/// place (same position in the merged list, so criteria/outcome come from the step's
/// definition); a step-level action with a name the workflow-level list doesn't have is
/// appended after all workflow-level entries, in step declaration order; a workflow-level action
/// the step doesn't mention passes through unchanged. When the step declares no actions of its
/// own, the result is exactly `workflow_defaults`, unchanged (byte-for-byte, same as before
/// PROJ-810 for the overwhelming majority of documents that never use workflow-level actions at
/// all: `workflow_defaults` is then empty, so the merge is a no-op).
///
/// # Complexity
/// O(workflow_defaults * step_actions): a linear name-scan per pairing rather than a `HashMap`
/// keyed by name, because both lists are small (single digits) in real documents and preserving
/// workflow-level declaration order for non-overridden entries matters more here than
/// asymptotic lookup cost.
fn merge_success_actions(
    workflow_defaults: &[SuccessAction],
    step_actions: &[SuccessAction],
) -> Vec<SuccessAction> {
    if step_actions.is_empty() {
        return workflow_defaults.to_vec();
    }
    let mut merged = Vec::with_capacity(workflow_defaults.len() + step_actions.len());
    for wf_action in workflow_defaults {
        match step_actions.iter().find(|s| s.name == wf_action.name) {
            Some(overriding) => merged.push(overriding.clone()),
            None => merged.push(wf_action.clone()),
        }
    }
    for step_action in step_actions {
        if !workflow_defaults.iter().any(|w| w.name == step_action.name) {
            merged.push(step_action.clone());
        }
    }
    merged
}

/// Same contract as [`merge_success_actions`], for `FailureAction`.
fn merge_failure_actions(
    workflow_defaults: &[FailureAction],
    step_actions: &[FailureAction],
) -> Vec<FailureAction> {
    if step_actions.is_empty() {
        return workflow_defaults.to_vec();
    }
    let mut merged = Vec::with_capacity(workflow_defaults.len() + step_actions.len());
    for wf_action in workflow_defaults {
        match step_actions.iter().find(|s| s.name == wf_action.name) {
            Some(overriding) => merged.push(overriding.clone()),
            None => merged.push(wf_action.clone()),
        }
    }
    for step_action in step_actions {
        if !workflow_defaults.iter().any(|w| w.name == step_action.name) {
            merged.push(step_action.clone());
        }
    }
    merged
}

/// Lowers an already-resolved, already-merged success-action list to `AirRouting`. See
/// [`resolve_success_actions`]/[`merge_success_actions`] for how `actions` gets to this state.
fn lower_resolved_success_actions<'bump>(
    actions: &[SuccessAction],
    bump: &'bump Bump,
) -> Result<BumpVec<'bump, AirRouting<'bump>>, Refusal> {
    let mut out = BumpVec::with_capacity_in(actions.len(), bump);
    for resolved in actions {
        let outcome = match resolved.action_type {
            SuccessActionType::End => AirRoutingOutcome::End,
            SuccessActionType::Goto => lower_goto_outcome(
                &resolved.name,
                resolved.step_id.as_deref(),
                resolved.workflow_id.as_deref(),
                bump,
            )?,
        };
        out.push(AirRouting {
            name: BumpString::from_str_in(&resolved.name, bump),
            outcome,
            criteria: lower_criteria(&resolved.criteria, bump)?,
        });
    }
    Ok(out)
}

/// Same contract as [`lower_resolved_success_actions`], for `FailureAction`. Retry-policy
/// validation ([`validate_retry_policy`]) runs in the caller ([`lower_step`]) against the merged
/// list before this function is called, not duplicated here.
fn lower_resolved_failure_actions<'bump>(
    actions: &[FailureAction],
    bump: &'bump Bump,
) -> Result<BumpVec<'bump, AirRouting<'bump>>, Refusal> {
    let mut out = BumpVec::with_capacity_in(actions.len(), bump);
    for resolved in actions {
        let outcome = match resolved.action_type {
            FailureActionType::End => AirRoutingOutcome::End,
            FailureActionType::Retry => AirRoutingOutcome::Retry,
            FailureActionType::Goto => lower_goto_outcome(
                &resolved.name,
                resolved.step_id.as_deref(),
                resolved.workflow_id.as_deref(),
                bump,
            )?,
        };
        out.push(AirRouting {
            name: BumpString::from_str_in(&resolved.name, bump),
            outcome,
            criteria: lower_criteria(&resolved.criteria, bump)?,
        });
    }
    Ok(out)
}

fn lower_goto_outcome<'bump>(
    action_name: &str,
    step_id: Option<&str>,
    workflow_id: Option<&str>,
    bump: &'bump Bump,
) -> Result<AirRoutingOutcome<'bump>, Refusal> {
    if let Some(step_id) = step_id {
        return Ok(AirRoutingOutcome::GotoStep(BumpString::from_str_in(
            step_id, bump,
        )));
    }
    if let Some(workflow_id) = workflow_id {
        return Ok(AirRoutingOutcome::GotoWorkflow(BumpString::from_str_in(
            workflow_id,
            bump,
        )));
    }
    Err(Refusal::MissingIdentity(format!(
        "routing action '{action_name}' is type=goto but names neither stepId nor workflowId"
    )))
}

/// Dereferences a local `#/components/successActions/<name>` reference against `components`.
/// Cross-document references (any other reference shape) are refused, not resolved: this
/// bridge has no `DocumentIndex` in scope to look them up in (see module doc).
fn resolve_success_reference(
    r: &ReusableObject,
    components: Option<&Components>,
) -> Result<SuccessAction, Refusal> {
    let name = r
        .reference
        .strip_prefix("#/components/successActions/")
        .ok_or_else(|| {
            Refusal::UnresolvableReference(format!(
                "success action reference '{}' is not a local #/components/successActions/<name> \
                 reference (cross-document component dereferencing is out of scope for this bridge)",
                r.reference
            ))
        })?;
    components
        .and_then(|c| c.success_actions.get(name))
        .cloned()
        .ok_or_else(|| {
            Refusal::UnresolvableReference(format!(
                "success action reference '{}' has no matching entry in this document's \
                 components.successActions",
                r.reference
            ))
        })
}

/// Dereferences a local `#/components/failureActions/<name>` reference against `components`.
/// Same cross-document boundary as [`resolve_success_reference`].
fn resolve_failure_reference(
    r: &ReusableObject,
    components: Option<&Components>,
) -> Result<FailureAction, Refusal> {
    let name = r
        .reference
        .strip_prefix("#/components/failureActions/")
        .ok_or_else(|| {
            Refusal::UnresolvableReference(format!(
                "failure action reference '{}' is not a local #/components/failureActions/<name> \
                 reference (cross-document component dereferencing is out of scope for this bridge)",
                r.reference
            ))
        })?;
    components
        .and_then(|c| c.failure_actions.get(name))
        .cloned()
        .ok_or_else(|| {
            Refusal::UnresolvableReference(format!(
                "failure action reference '{}' has no matching entry in this document's \
                 components.failureActions",
                r.reference
            ))
        })
}

/// Dereferences a local `#/components/parameters/<name>` reference against `components`
/// (PROJ-810). Same local-only boundary as [`resolve_success_reference`]/
/// [`resolve_failure_reference`]: a cross-document reference, or a name absent from this
/// document's own `components.parameters`, is refused with [`Refusal::UnresolvableReference`]
/// rather than resolved (this bridge has no `DocumentIndex` in scope to look a cross-document
/// reference up in).
fn resolve_parameter_reference(
    r: &ReusableObject,
    components: Option<&Components>,
) -> Result<Parameter, Refusal> {
    let name = r
        .reference
        .strip_prefix("#/components/parameters/")
        .ok_or_else(|| {
            Refusal::UnresolvableReference(format!(
                "parameter reference '{}' is not a local #/components/parameters/<name> \
                 reference (cross-document component dereferencing is out of scope for this bridge)",
                r.reference
            ))
        })?;
    components
        .and_then(|c| c.parameters.get(name))
        .cloned()
        .ok_or_else(|| {
            Refusal::UnresolvableReference(format!(
                "parameter reference '{}' has no matching entry in this document's \
                 components.parameters",
                r.reference
            ))
        })
}

/// Applies `RequestBody.replacements` (PROJ-810: Arazzo's "Payload Replacement Object") to
/// `payload` before it is lowered by [`lower_json_value`]. Each replacement's `target` is a JSON
/// Pointer (RFC 6901) identifying an existing location within `payload`, and `value` is
/// substituted there. Replacements apply in declaration order, each seeing the previous
/// replacement's result, so a document may compose multiple targeted overrides into one payload
/// (e.g. replacing `/id` and then `/nested/id` in the same call).
///
/// Only the spec's default target shape is implemented: `targetSelectorType` omitted, or
/// explicitly `jsonpointer`. An `xpath`/`jsonpath`/versioned target shape has no evaluator in
/// this bridge -- the same selector-shape boundary [`classify_criterion`] and
/// [`classify_output_value`] enforce for `Criterion.type` / `OutputValue::Selector` -- and is
/// refused rather than silently misapplied as a JSON Pointer.
///
/// A replacement whose `target` does not resolve to an existing location within the (possibly
/// already-replaced) payload -- an object key that doesn't exist, or an array index out of
/// bounds -- is refused: RFC 6901 defines pointer resolution as failing in exactly these cases,
/// and silently no-op'ing a declared-but-unappliable replacement would be exactly the kind of
/// silent data loss PROJ-753 exists to close.
///
/// Runtime-expression strings nested inside the (original or replaced) payload's fields are not
/// individually resolved to `AirExpr::Variable` -- the same documented, pre-existing scope
/// boundary [`lower_json_value`] already has for any non-string-root JSON value (an object/array
/// payload always lowers as a single serialized `Literal`); this function only changes *which*
/// JSON value occupies the payload before that existing lowering rule runs.
///
/// # Errors
/// [`Refusal::UnsupportedExpression`] for a non-jsonpointer `targetSelectorType`.
/// [`Refusal::UnresolvableReference`] for a `target` JSON Pointer that does not resolve.
///
/// # Complexity
/// O(replacements * pointer_depth): each replacement walks its own pointer's token list once
/// (`serde_json::Value::pointer_mut`); pointer_depth is small (single digits) in real documents.
fn apply_payload_replacements(
    step_id: &str,
    payload: &Value,
    replacements: &[PayloadReplacement],
) -> Result<Value, Refusal> {
    let mut out = payload.clone();
    for replacement in replacements {
        match &replacement.target_selector_type {
            None | Some(SelectorType::Kind(SelectorKind::Jsonpointer)) => {}
            Some(other) => {
                return Err(Refusal::UnsupportedExpression(format!(
                    "step '{step_id}' request body replacement targeting '{}' declares \
                     targetSelectorType {other:?}, which this bridge has no evaluator for (only \
                     the default JSON Pointer target shape is supported)",
                    replacement.target
                )));
            }
        }
        let slot = out.pointer_mut(&replacement.target).ok_or_else(|| {
            Refusal::UnresolvableReference(format!(
                "step '{step_id}' request body replacement target '{}' does not resolve within \
                 the payload (RFC 6901 JSON Pointer)",
                replacement.target
            ))
        })?;
        *slot = replacement.value.clone();
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalizer::ArazzoNormalizer;
    use serde_json::json;
    use wasm4pm_compat::arazzo::{ArazzoInfo, RequestBody};

    fn minimal_step(step_id: &str, operation_id: &str) -> Step {
        Step {
            description: None,
            step_id: step_id.to_string(),
            operation_id: Some(operation_id.to_string()),
            operation_path: None,
            channel_path: None,
            workflow_id: None,
            parameters: vec![],
            request_body: None,
            success_criteria: vec![],
            on_success: vec![],
            on_failure: vec![],
            outputs: Default::default(),
            timeout: None,
            correlation_id: None,
            action: None,
            depends_on: vec![],
            extensions: Default::default(),
        }
    }

    fn minimal_doc(workflows: Vec<Workflow>) -> ArazzoDescription {
        ArazzoDescription {
            arazzo: "1.1.0".to_string(),
            self_uri: None,
            info: ArazzoInfo {
                title: "test".to_string(),
                summary: None,
                description: None,
                version: "1.0.0".to_string(),
                extensions: Default::default(),
            },
            source_descriptions: vec![],
            workflows,
            components: None,
            extensions: Default::default(),
        }
    }

    #[test]
    fn lowers_operation_id_target_and_bare_literal_and_output() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.parameters = vec![ParameterOrReference::Parameter(
            wasm4pm_compat::arazzo::Parameter {
                name: "amount".to_string(),
                location: None,
                value: json!(42),
                extensions: Default::default(),
            },
        )];
        step.outputs.insert(
            "order_id".to_string(),
            wasm4pm_compat::arazzo::OutputValue::Expression("$response.body#/id".to_string()),
        );
        let wf = Workflow {
            workflow_id: "wf_1".to_string(),
            summary: None,
            description: None,
            inputs: None,
            depends_on: vec![],
            steps: vec![step],
            success_actions: vec![],
            failure_actions: vec![],
            outputs: Default::default(),
            extensions: Default::default(),
        };
        let doc = minimal_doc(vec![wf]);

        let program = lower_description(&doc, &bump).expect("should lower");
        assert_eq!(program.workflows.len(), 1);
        assert_eq!(program.workflows[0].name, "wf_1");
        let air_step = &program.workflows[0].steps[0];
        assert_eq!(air_step.name, "step_1");
        assert_eq!(air_step.target.url, "urn:test:op1");
        assert_eq!(air_step.target.method, "operationId");
        assert_eq!(air_step.action.inputs.len(), 1);
        match &air_step.action.inputs[0] {
            AirExpr::Literal(l) => assert_eq!(l, "42"),
            AirExpr::Variable(_) => panic!("plain JSON number must lower to a Literal"),
        }
        assert_eq!(air_step.action.outputs.len(), 1);
        match &air_step.action.outputs[0] {
            AirExpr::Literal(l) => assert_eq!(l, "order_id"),
            AirExpr::Variable(_) => panic!("declared output name must lower to a Literal"),
        }
    }

    #[test]
    fn lowers_step_output_reference_to_variable() {
        let bump = Bump::new();
        let mut step = minimal_step("step_2", "urn:test:op2");
        step.request_body = Some(RequestBody {
            content_type: None,
            payload: Some(json!("$steps.step_1.outputs.order_id")),
            replacements: vec![],
            extensions: Default::default(),
        });
        let wf = Workflow {
            workflow_id: "wf_1".to_string(),
            summary: None,
            description: None,
            inputs: None,
            depends_on: vec![],
            steps: vec![step],
            success_actions: vec![],
            failure_actions: vec![],
            outputs: Default::default(),
            extensions: Default::default(),
        };
        let doc = minimal_doc(vec![wf]);

        let program = lower_description(&doc, &bump).expect("should lower");
        match &program.workflows[0].steps[0].action.inputs[0] {
            AirExpr::Variable(v) => assert_eq!(v, "order_id"),
            AirExpr::Literal(_) => {
                panic!("$steps.<id>.outputs.<name> must lower to a cross-step Variable")
            }
        }
    }

    #[test]
    fn other_runtime_expressions_lower_as_opaque_literals() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.parameters = vec![ParameterOrReference::Parameter(
            wasm4pm_compat::arazzo::Parameter {
                name: "x".to_string(),
                location: None,
                value: json!("$inputs.user_id"),
                extensions: Default::default(),
            },
        )];
        let wf = Workflow {
            workflow_id: "wf_1".to_string(),
            summary: None,
            description: None,
            inputs: None,
            depends_on: vec![],
            steps: vec![step],
            success_actions: vec![],
            failure_actions: vec![],
            outputs: Default::default(),
            extensions: Default::default(),
        };
        let doc = minimal_doc(vec![wf]);

        let program = lower_description(&doc, &bump).expect("should lower");
        match &program.workflows[0].steps[0].action.inputs[0] {
            AirExpr::Literal(l) => assert_eq!(l, "$inputs.user_id"),
            AirExpr::Variable(_) => {
                panic!("$inputs.* is not a cross-step reference and must not be treated as one")
            }
        }
    }

    #[test]
    fn refuses_step_with_no_identity() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "placeholder");
        step.operation_id = None;
        let wf = Workflow {
            workflow_id: "wf_1".to_string(),
            summary: None,
            description: None,
            inputs: None,
            depends_on: vec![],
            steps: vec![step],
            success_actions: vec![],
            failure_actions: vec![],
            outputs: Default::default(),
            extensions: Default::default(),
        };
        let doc = minimal_doc(vec![wf]);

        let result = lower_description(&doc, &bump);
        assert!(matches!(result, Err(Refusal::MissingIdentity(_))));
    }

    #[test]
    fn refuses_workflow_with_zero_steps() {
        let bump = Bump::new();
        let wf = Workflow {
            workflow_id: "wf_1".to_string(),
            summary: None,
            description: None,
            inputs: None,
            depends_on: vec![],
            steps: vec![],
            success_actions: vec![],
            failure_actions: vec![],
            outputs: Default::default(),
            extensions: Default::default(),
        };
        let doc = minimal_doc(vec![wf]);

        let result = lower_description(&doc, &bump);
        assert!(matches!(result, Err(Refusal::InvalidWorkflow(_))));
    }

    #[test]
    fn refuses_document_with_zero_workflows() {
        let bump = Bump::new();
        let doc = minimal_doc(vec![]);
        let result = lower_description(&doc, &bump);
        assert!(matches!(result, Err(Refusal::InvalidWorkflow(_))));
    }

    #[test]
    fn lowers_inline_goto_step_success_action_with_criteria() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.on_success = vec![SuccessActionOrReference::Action(SuccessAction {
            name: "go_next".to_string(),
            action_type: SuccessActionType::Goto,
            workflow_id: None,
            step_id: Some("step_2".to_string()),
            parameters: vec![],
            criteria: vec![Criterion {
                context: None,
                condition: "$statusCode == 200".to_string(),
                expression_type: None,
                extensions: Default::default(),
            }],
            extensions: Default::default(),
        })];
        let wf = Workflow {
            workflow_id: "wf_1".to_string(),
            summary: None,
            description: None,
            inputs: None,
            depends_on: vec![],
            steps: vec![step],
            success_actions: vec![],
            failure_actions: vec![],
            outputs: Default::default(),
            extensions: Default::default(),
        };
        let doc = minimal_doc(vec![wf]);

        let program = lower_description(&doc, &bump).expect("should lower");
        let routing = &program.workflows[0].steps[0].on_success[0];
        assert_eq!(routing.name, "go_next");
        assert_eq!(routing.criteria.len(), 1);
        match &routing.outcome {
            AirRoutingOutcome::GotoStep(s) => assert_eq!(s, "step_2"),
            other => panic!("expected GotoStep, got {other:?}"),
        }
    }

    #[test]
    fn lowers_success_action_reference_via_local_components() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.on_success = vec![SuccessActionOrReference::Reference(ReusableObject {
            reference: "#/components/successActions/finish".to_string(),
            value: None,
        })];
        let wf = Workflow {
            workflow_id: "wf_1".to_string(),
            summary: None,
            description: None,
            inputs: None,
            depends_on: vec![],
            steps: vec![step],
            success_actions: vec![],
            failure_actions: vec![],
            outputs: Default::default(),
            extensions: Default::default(),
        };
        let mut components = Components {
            inputs: Default::default(),
            parameters: Default::default(),
            success_actions: Default::default(),
            failure_actions: Default::default(),
            extensions: Default::default(),
        };
        components.success_actions.insert(
            "finish".to_string(),
            SuccessAction {
                name: "finish".to_string(),
                action_type: SuccessActionType::End,
                workflow_id: None,
                step_id: None,
                parameters: vec![],
                criteria: vec![],
                extensions: Default::default(),
            },
        );
        let mut doc = minimal_doc(vec![wf]);
        doc.components = Some(components);

        let program = lower_description(&doc, &bump).expect("should lower");
        let routing = &program.workflows[0].steps[0].on_success[0];
        assert_eq!(routing.name, "finish");
        assert_eq!(routing.outcome, AirRoutingOutcome::End);
    }

    #[test]
    fn refuses_cross_document_success_action_reference() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.on_success = vec![SuccessActionOrReference::Reference(ReusableObject {
            reference: "https://example.com/other.json#/components/successActions/finish"
                .to_string(),
            value: None,
        })];
        let wf = Workflow {
            workflow_id: "wf_1".to_string(),
            summary: None,
            description: None,
            inputs: None,
            depends_on: vec![],
            steps: vec![step],
            success_actions: vec![],
            failure_actions: vec![],
            outputs: Default::default(),
            extensions: Default::default(),
        };
        let doc = minimal_doc(vec![wf]);

        let result = lower_description(&doc, &bump);
        assert!(matches!(result, Err(Refusal::UnresolvableReference(_))));
    }

    /// Shorthand for a single-workflow document wrapping the given steps, matching
    /// `minimal_doc`'s field defaults for everything not under test.
    fn wf_with_steps(steps: Vec<Step>) -> Workflow {
        Workflow {
            workflow_id: "wf_1".to_string(),
            summary: None,
            description: None,
            inputs: None,
            depends_on: vec![],
            steps,
            success_actions: vec![],
            failure_actions: vec![],
            outputs: Default::default(),
            extensions: Default::default(),
        }
    }

    // --- PROJ-754: CyclicStepDependency ------------------------------------------------

    #[test]
    fn refuses_two_step_cyclic_dependency() {
        let bump = Bump::new();
        let mut step_a = minimal_step("step_a", "urn:test:a");
        step_a.depends_on = vec!["step_b".to_string()];
        let mut step_b = minimal_step("step_b", "urn:test:b");
        step_b.depends_on = vec!["step_a".to_string()];
        let doc = minimal_doc(vec![wf_with_steps(vec![step_a, step_b])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::CyclicStepDependency(_))),
            "step_a depends on step_b and step_b depends on step_a: expected \
             CyclicStepDependency, got {result:?}"
        );
    }

    #[test]
    fn refuses_three_step_cyclic_dependency() {
        // A -> B -> C -> A: a longer cycle than the 1-node (self) and 2-node cases already
        // covered above, proving the iterative DFS's gray/black coloring detects a cycle that
        // closes through an intermediate node rather than only the immediately-previous one.
        // Traced by hand against `validate_step_dependencies`'s DFS: steps are pushed onto the
        // `index_of`/adjacency arrays in declaration order (step_a=0, step_b=1, step_c=2), so
        // the DFS visits step_a -> step_b -> step_c -> (back-edge to step_a, still gray) and the
        // refusal names step_a, the step whose gray back-edge closes the cycle.
        let bump = Bump::new();
        let mut step_a = minimal_step("step_a", "urn:test:a");
        step_a.depends_on = vec!["step_b".to_string()];
        let mut step_b = minimal_step("step_b", "urn:test:b");
        step_b.depends_on = vec!["step_c".to_string()];
        let mut step_c = minimal_step("step_c", "urn:test:c");
        step_c.depends_on = vec!["step_a".to_string()];
        let doc = minimal_doc(vec![wf_with_steps(vec![step_a, step_b, step_c])]);

        let result = lower_description(&doc, &bump);
        match result {
            Err(Refusal::CyclicStepDependency(msg)) => {
                assert!(
                    msg.contains("step_a"),
                    "3-node cycle A->B->C->A: refusal must name the step whose back-edge \
                     closes the cycle, got: {msg}"
                );
            }
            other => panic!(
                "step_a -> step_b -> step_c -> step_a is a 3-node cycle: expected \
                 CyclicStepDependency, got {other:?}"
            ),
        }
    }

    #[test]
    fn refuses_self_referential_step_dependency() {
        let bump = Bump::new();
        let mut step_a = minimal_step("step_a", "urn:test:a");
        step_a.depends_on = vec!["step_a".to_string()];
        let doc = minimal_doc(vec![wf_with_steps(vec![step_a])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::CyclicStepDependency(_))),
            "a step naming itself in depends_on is a degenerate 1-node cycle, got {result:?}"
        );
    }

    #[test]
    fn accepts_acyclic_three_step_dependency_chain() {
        // step_c depends on step_b, step_b depends on step_a -- a real DAG, not a cycle;
        // proves the new cycle check has no false positives on valid ordering.
        let bump = Bump::new();
        let step_a = minimal_step("step_a", "urn:test:a");
        let mut step_b = minimal_step("step_b", "urn:test:b");
        step_b.depends_on = vec!["step_a".to_string()];
        let mut step_c = minimal_step("step_c", "urn:test:c");
        step_c.depends_on = vec!["step_a".to_string(), "step_b".to_string()];
        let doc = minimal_doc(vec![wf_with_steps(vec![step_a, step_b, step_c])]);

        let program = lower_description(&doc, &bump).expect("acyclic chain must lower");
        assert_eq!(program.workflows[0].steps.len(), 3);
    }

    #[test]
    fn refuses_dangling_step_dependency_reference() {
        let bump = Bump::new();
        let mut step_a = minimal_step("step_a", "urn:test:a");
        step_a.depends_on = vec!["step_nonexistent".to_string()];
        let doc = minimal_doc(vec![wf_with_steps(vec![step_a])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::UnresolvableReference(_))),
            "depends_on naming a step id absent from the workflow must be refused, got {result:?}"
        );
    }

    // --- PROJ-754: UnsupportedCriterion --------------------------------------------------

    #[test]
    fn refuses_unsupported_criterion_expression_shape() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.on_success = vec![SuccessActionOrReference::Action(SuccessAction {
            name: "go_next".to_string(),
            action_type: SuccessActionType::End,
            workflow_id: None,
            step_id: None,
            parameters: vec![],
            criteria: vec![Criterion {
                context: Some("$response.body".to_string()),
                condition: "$.orders[?(@.id == 1)]".to_string(),
                expression_type: Some(ExpressionTypeOrKind::Kind(ExpressionKind::Jsonpath)),
                extensions: Default::default(),
            }],
            extensions: Default::default(),
        })];
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::UnsupportedCriterion(_))),
            "a jsonpath-typed criterion has no evaluator in this bridge, got {result:?}"
        );
    }

    #[test]
    fn accepts_simple_and_untyped_criteria() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.on_success = vec![SuccessActionOrReference::Action(SuccessAction {
            name: "go_next".to_string(),
            action_type: SuccessActionType::End,
            workflow_id: None,
            step_id: None,
            parameters: vec![],
            criteria: vec![
                Criterion {
                    context: None,
                    condition: "$statusCode == 200".to_string(),
                    expression_type: None,
                    extensions: Default::default(),
                },
                Criterion {
                    context: None,
                    condition: "$statusCode == 200".to_string(),
                    expression_type: Some(ExpressionTypeOrKind::Kind(ExpressionKind::Simple)),
                    extensions: Default::default(),
                },
            ],
            extensions: Default::default(),
        })];
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let program = lower_description(&doc, &bump).expect("simple/untyped criteria must lower");
        assert_eq!(
            program.workflows[0].steps[0].on_success[0].criteria.len(),
            2
        );
    }

    // --- PROJ-754: MalformedRetryPolicy (timeout) ----------------------------------------

    #[test]
    fn refuses_zero_millisecond_step_timeout() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.timeout = Some(0);
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::MalformedRetryPolicy(_))),
            "timeout: 0 can never be satisfied, got {result:?}"
        );
    }

    #[test]
    fn accepts_positive_step_timeout_and_undeclared_timeout() {
        let bump = Bump::new();
        let mut step_with_timeout = minimal_step("step_1", "urn:test:op1");
        step_with_timeout.timeout = Some(5000);
        let step_without_timeout = minimal_step("step_2", "urn:test:op2");
        let doc = minimal_doc(vec![wf_with_steps(vec![
            step_with_timeout,
            step_without_timeout,
        ])]);

        let program = lower_description(&doc, &bump).expect("valid timeouts must lower");
        assert_eq!(program.workflows[0].steps.len(), 2);
    }

    // --- PROJ-754: MalformedRetryPolicy (retry) ------------------------------------------

    fn retry_failure_action(retry_limit: Option<u64>, retry_after: Option<f64>) -> FailureAction {
        FailureAction {
            name: "retry_step".to_string(),
            action_type: FailureActionType::Retry,
            workflow_id: None,
            step_id: None,
            parameters: vec![],
            retry_after,
            retry_limit,
            criteria: vec![],
            extensions: Default::default(),
        }
    }

    #[test]
    fn refuses_retry_action_with_zero_retry_limit() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.on_failure = vec![FailureActionOrReference::Action(retry_failure_action(
            Some(0),
            None,
        ))];
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::MalformedRetryPolicy(_))),
            "retryLimit: 0 permits zero retry attempts, got {result:?}"
        );
    }

    #[test]
    fn refuses_retry_action_with_negative_retry_after() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.on_failure = vec![FailureActionOrReference::Action(retry_failure_action(
            Some(3),
            Some(-1.5),
        ))];
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::MalformedRetryPolicy(_))),
            "retryAfter: -1.5 is not a valid wait duration, got {result:?}"
        );
    }

    #[test]
    fn refuses_retry_action_with_non_finite_retry_after() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.on_failure = vec![FailureActionOrReference::Action(retry_failure_action(
            Some(3),
            Some(f64::NAN),
        ))];
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::MalformedRetryPolicy(_))),
            "retryAfter: NaN is not a valid wait duration, got {result:?}"
        );
    }

    #[test]
    fn accepts_well_formed_retry_policy() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.on_failure = vec![FailureActionOrReference::Action(retry_failure_action(
            Some(3),
            Some(1.5),
        ))];
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let program = lower_description(&doc, &bump).expect("well-formed retry policy must lower");
        assert_eq!(
            program.workflows[0].steps[0].on_failure[0].outcome,
            AirRoutingOutcome::Retry
        );
    }

    #[test]
    fn accepts_retry_action_with_unset_retry_fields() {
        // Both fields are optional per spec -- absence must not be treated as malformed.
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.on_failure = vec![FailureActionOrReference::Action(retry_failure_action(
            None, None,
        ))];
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let program = lower_description(&doc, &bump).expect("unset retry fields must lower");
        assert_eq!(
            program.workflows[0].steps[0].on_failure[0].outcome,
            AirRoutingOutcome::Retry
        );
    }

    // --- PROJ-784: UnsupportedExpression (output selector) --------------------------------

    #[test]
    fn refuses_selector_shaped_step_output() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.outputs.insert(
            "order_id".to_string(),
            OutputValue::Selector(wasm4pm_compat::arazzo::Selector {
                context: "$response.body".to_string(),
                selector: "$.id".to_string(),
                selector_type: wasm4pm_compat::arazzo::SelectorType::Kind(
                    wasm4pm_compat::arazzo::SelectorKind::Jsonpath,
                ),
                extensions: Default::default(),
            }),
        );
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::UnsupportedExpression(_))),
            "a Selector-shaped output value has no evaluator in this bridge, got {result:?}"
        );
    }

    #[test]
    fn accepts_expression_shaped_step_output() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.outputs.insert(
            "order_id".to_string(),
            OutputValue::Expression("$response.body#/id".to_string()),
        );
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let program = lower_description(&doc, &bump).expect("plain expression output must lower");
        assert_eq!(program.workflows[0].steps[0].action.outputs.len(), 1);
    }

    // --- PROJ-810: parameter $ref dereferencing, RequestBody.replacements, and workflow-level
    // successActions/failureActions now genuinely implemented (PROJ-753 had refused all three
    // unconditionally as a safety-net interim fix; the tests below prove real, spec-compliant
    // documents using these constructs now lower correctly end-to-end, while a document with a
    // genuinely malformed use of the construct -- e.g. a $ref to a nonexistent component -- is
    // still refused). ------------------------------------------------------------------------

    fn components_with_parameter(name: &str, value: Value) -> Components {
        let mut components = Components {
            inputs: Default::default(),
            parameters: Default::default(),
            success_actions: Default::default(),
            failure_actions: Default::default(),
            extensions: Default::default(),
        };
        components.parameters.insert(
            name.to_string(),
            wasm4pm_compat::arazzo::Parameter {
                name: name.to_string(),
                location: None,
                value,
                extensions: Default::default(),
            },
        );
        components
    }

    #[test]
    fn lowers_step_parameter_reference_via_local_components() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.parameters = vec![ParameterOrReference::Reference(ReusableObject {
            reference: "#/components/parameters/region".to_string(),
            value: None,
        })];
        let mut doc = minimal_doc(vec![wf_with_steps(vec![step])]);
        doc.components = Some(components_with_parameter("region", json!("us-east-1")));

        let program = lower_description(&doc, &bump)
            .expect("a $ref parameter resolving against components.parameters must lower");
        match &program.workflows[0].steps[0].action.inputs[0] {
            AirExpr::Literal(l) => assert_eq!(l, "us-east-1"),
            AirExpr::Variable(_) => panic!("resolved parameter value is a plain string literal"),
        }
    }

    #[test]
    fn refuses_step_parameter_reference_to_nonexistent_component() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.parameters = vec![ParameterOrReference::Reference(ReusableObject {
            reference: "#/components/parameters/region".to_string(),
            value: None,
        })];
        // No `components` declared at all: the $ref cannot resolve.
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::UnresolvableReference(_))),
            "a $ref to a component that doesn't exist must still refuse, got {result:?}"
        );
    }

    #[test]
    fn refuses_cross_document_step_parameter_reference() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.parameters = vec![ParameterOrReference::Reference(ReusableObject {
            reference: "https://example.com/other.json#/components/parameters/region".to_string(),
            value: None,
        })];
        let mut doc = minimal_doc(vec![wf_with_steps(vec![step])]);
        doc.components = Some(components_with_parameter("region", json!("us-east-1")));

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::UnresolvableReference(_))),
            "cross-document parameter dereferencing is out of scope, got {result:?}"
        );
    }

    #[test]
    fn applies_request_body_payload_replacements() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.request_body = Some(RequestBody {
            content_type: None,
            payload: Some(json!({"id": 1, "nested": {"x": 0}})),
            replacements: vec![
                PayloadReplacement {
                    target: "/id".to_string(),
                    target_selector_type: None,
                    value: json!(42),
                    extensions: Default::default(),
                },
                PayloadReplacement {
                    target: "/nested/x".to_string(),
                    target_selector_type: None,
                    value: json!(99),
                    extensions: Default::default(),
                },
            ],
            extensions: Default::default(),
        });
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let program = lower_description(&doc, &bump)
            .expect("payload replacements targeting real JSON Pointer locations must lower");
        let expected = json!({"id": 42, "nested": {"x": 99}}).to_string();
        match &program.workflows[0].steps[0].action.inputs[0] {
            AirExpr::Literal(l) => assert_eq!(l.as_str(), expected),
            AirExpr::Variable(_) => panic!("a replaced JSON object payload lowers as a Literal"),
        }
    }

    #[test]
    fn refuses_request_body_replacement_target_that_does_not_resolve() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.request_body = Some(RequestBody {
            content_type: None,
            payload: Some(json!({"id": 1})),
            replacements: vec![PayloadReplacement {
                target: "/nonexistent/deep".to_string(),
                target_selector_type: None,
                value: json!(42),
                extensions: Default::default(),
            }],
            extensions: Default::default(),
        });
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::UnresolvableReference(_))),
            "a replacement target JSON Pointer that does not resolve within the payload must \
             refuse, got {result:?}"
        );
    }

    #[test]
    fn refuses_request_body_replacement_with_unsupported_target_selector_type() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.request_body = Some(RequestBody {
            content_type: None,
            payload: Some(json!({"id": 1})),
            replacements: vec![PayloadReplacement {
                target: "/id".to_string(),
                target_selector_type: Some(wasm4pm_compat::arazzo::SelectorType::Kind(
                    wasm4pm_compat::arazzo::SelectorKind::Xpath,
                )),
                value: json!(42),
                extensions: Default::default(),
            }],
            extensions: Default::default(),
        });
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::UnsupportedExpression(_))),
            "an xpath-targeted replacement has no evaluator in this bridge, got {result:?}"
        );
    }

    #[test]
    fn refuses_request_body_replacements_without_payload() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.request_body = Some(RequestBody {
            content_type: None,
            payload: None,
            replacements: vec![PayloadReplacement {
                target: "/id".to_string(),
                target_selector_type: None,
                value: json!(42),
                extensions: Default::default(),
            }],
            extensions: Default::default(),
        });
        let doc = minimal_doc(vec![wf_with_steps(vec![step])]);

        let result = lower_description(&doc, &bump);
        assert!(
            matches!(result, Err(Refusal::UnsupportedFeature(_))),
            "replacements with no base payload have no OpenAPI-resolved default to apply onto \
             in this bridge, got {result:?}"
        );
    }

    #[test]
    fn applies_workflow_level_success_action_as_step_default() {
        let bump = Bump::new();
        let step = minimal_step("step_1", "urn:test:op1");
        let mut wf = wf_with_steps(vec![step]);
        wf.success_actions = vec![SuccessActionOrReference::Action(SuccessAction {
            name: "wf_finish".to_string(),
            action_type: SuccessActionType::End,
            workflow_id: None,
            step_id: None,
            parameters: vec![],
            criteria: vec![],
            extensions: Default::default(),
        })];
        let doc = minimal_doc(vec![wf]);

        let program = lower_description(&doc, &bump)
            .expect("workflow-level successActions must lower as a step default");
        let routing = &program.workflows[0].steps[0].on_success[0];
        assert_eq!(routing.name, "wf_finish");
        assert_eq!(routing.outcome, AirRoutingOutcome::End);
    }

    #[test]
    fn applies_workflow_level_failure_action_as_step_default() {
        let bump = Bump::new();
        let step = minimal_step("step_1", "urn:test:op1");
        let mut wf = wf_with_steps(vec![step]);
        wf.failure_actions = vec![FailureActionOrReference::Action(retry_failure_action(
            Some(3),
            Some(1.5),
        ))];
        let doc = minimal_doc(vec![wf]);

        let program = lower_description(&doc, &bump)
            .expect("workflow-level failureActions must lower as a step default");
        assert_eq!(
            program.workflows[0].steps[0].on_failure[0].outcome,
            AirRoutingOutcome::Retry
        );
    }

    #[test]
    fn step_level_success_action_overrides_same_named_workflow_default() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        step.on_success = vec![SuccessActionOrReference::Action(SuccessAction {
            name: "finish".to_string(),
            action_type: SuccessActionType::Goto,
            workflow_id: None,
            step_id: Some("step_2".to_string()),
            parameters: vec![],
            criteria: vec![],
            extensions: Default::default(),
        })];
        let mut wf = wf_with_steps(vec![step]);
        wf.success_actions = vec![SuccessActionOrReference::Action(SuccessAction {
            name: "finish".to_string(),
            action_type: SuccessActionType::End,
            workflow_id: None,
            step_id: None,
            parameters: vec![],
            criteria: vec![],
            extensions: Default::default(),
        })];
        let doc = minimal_doc(vec![wf]);

        let program = lower_description(&doc, &bump)
            .expect("a step-level action overriding a same-named workflow default must lower");
        let routing = &program.workflows[0].steps[0].on_success;
        assert_eq!(
            routing.len(),
            1,
            "the step's own 'finish' must replace, not duplicate, the workflow default"
        );
        match &routing[0].outcome {
            AirRoutingOutcome::GotoStep(s) => assert_eq!(s, "step_2"),
            other => panic!(
                "step-level 'finish' (type=goto) must override the workflow-level 'finish' \
                 (type=end), got {other:?}"
            ),
        }
    }

    #[test]
    fn step_level_success_actions_cannot_remove_unmentioned_workflow_defaults() {
        let bump = Bump::new();
        let mut step = minimal_step("step_1", "urn:test:op1");
        // Step only overrides "a"; "b" is a workflow-level default the step never mentions.
        step.on_success = vec![SuccessActionOrReference::Action(SuccessAction {
            name: "a".to_string(),
            action_type: SuccessActionType::Goto,
            workflow_id: None,
            step_id: Some("step_x".to_string()),
            parameters: vec![],
            criteria: vec![],
            extensions: Default::default(),
        })];
        let mut wf = wf_with_steps(vec![step]);
        wf.success_actions = vec![
            SuccessActionOrReference::Action(SuccessAction {
                name: "a".to_string(),
                action_type: SuccessActionType::End,
                workflow_id: None,
                step_id: None,
                parameters: vec![],
                criteria: vec![],
                extensions: Default::default(),
            }),
            SuccessActionOrReference::Action(SuccessAction {
                name: "b".to_string(),
                action_type: SuccessActionType::End,
                workflow_id: None,
                step_id: None,
                parameters: vec![],
                criteria: vec![],
                extensions: Default::default(),
            }),
        ];
        let doc = minimal_doc(vec![wf]);

        let program = lower_description(&doc, &bump)
            .expect("a workflow-level default the step does not mention must survive the merge");
        let routing = &program.workflows[0].steps[0].on_success;
        assert_eq!(
            routing.len(),
            2,
            "'b' must not be removed just because the step declared its own on_success list, \
             got {routing:?}"
        );
        let a = routing.iter().find(|r| r.name == "a").expect("'a' present");
        match &a.outcome {
            AirRoutingOutcome::GotoStep(s) => assert_eq!(s, "step_x"),
            other => panic!("'a' must be the step's override (goto), got {other:?}"),
        }
        let b = routing.iter().find(|r| r.name == "b").expect("'b' present");
        assert_eq!(
            b.outcome,
            AirRoutingOutcome::End,
            "'b' must pass through unchanged from the workflow-level default"
        );
    }

    // --- PROJ-784 correction: depends_on non-textual order vs. declaration order ----------

    #[test]
    fn depends_on_execution_order_resolves_when_declared_out_of_textual_order() {
        // step_A is declared FIRST but depends on step_B (declared SECOND) and references
        // step_B's output -- a legitimate use of `depends_on` to declare non-textual execution
        // order, exactly what the field exists for per the Arazzo spec. Before PROJ-784's
        // topological-sort fix, `lower_workflow` lowered steps in raw declaration order
        // (step_A before step_B), so `ArazzoNormalizer::normalize`'s single left-to-right scan
        // over that order would wrongly refuse this as an UnresolvableReference even though the
        // dependency graph itself is acyclic and referentially sound.
        let bump = Bump::new();

        let mut step_a = minimal_step("step_A", "urn:test:a");
        step_a.depends_on = vec!["step_B".to_string()];
        step_a.request_body = Some(RequestBody {
            content_type: None,
            payload: Some(json!("$steps.step_B.outputs.thing")),
            replacements: vec![],
            extensions: Default::default(),
        });

        let mut step_b = minimal_step("step_B", "urn:test:b");
        step_b.outputs.insert(
            "thing".to_string(),
            OutputValue::Expression("$response.body#/id".to_string()),
        );

        // Declaration order is [step_A, step_B] -- step_A textually first, even though it
        // depends on step_B.
        let doc = minimal_doc(vec![wf_with_steps(vec![step_a, step_b])]);

        let mut program = lower_description(&doc, &bump).expect(
            "an acyclic, referentially sound depends_on graph must lower even when declared \
             out of textual order",
        );

        // The topological sort must have reordered lowering so step_B (the dependency) comes
        // before step_A (the dependent) in AirWorkflow.steps, regardless of source declaration
        // order.
        assert_eq!(program.workflows[0].steps[0].name, "step_B");
        assert_eq!(program.workflows[0].steps[1].name, "step_A");

        // Normalization -- which assumes array order == "earlier step" order -- must now
        // resolve the reference instead of wrongly refusing it.
        ArazzoNormalizer::normalize(&mut program, &bump).expect(
            "step_A's reference to step_B's output must resolve once steps are lowered in \
             dependency order, not raw declaration order",
        );
        match &program.workflows[0].steps[1].action.inputs[0] {
            AirExpr::Literal(l) => assert_eq!(l, "thing"),
            AirExpr::Variable(_) => {
                panic!("normalization must resolve step_A's reference into a Literal")
            }
        }
    }

    #[test]
    fn depends_on_reorder_is_a_no_op_when_no_dependencies_are_declared() {
        // Ties in the topological sort (the common case: no depends_on edges at all) must
        // break by original declaration index, so a workflow that never uses depends_on lowers
        // in exactly its original order -- no behavior change for the overwhelming majority of
        // documents.
        let bump = Bump::new();
        let step_1 = minimal_step("step_1", "urn:test:1");
        let step_2 = minimal_step("step_2", "urn:test:2");
        let step_3 = minimal_step("step_3", "urn:test:3");
        let doc = minimal_doc(vec![wf_with_steps(vec![step_1, step_2, step_3])]);

        let program = lower_description(&doc, &bump).expect("must lower");
        assert_eq!(program.workflows[0].steps[0].name, "step_1");
        assert_eq!(program.workflows[0].steps[1].name, "step_2");
        assert_eq!(program.workflows[0].steps[2].name, "step_3");
    }
}
