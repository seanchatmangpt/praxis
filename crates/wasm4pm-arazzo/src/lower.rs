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
//!   exact `Literal` vs. `Variable` classification rule. `RequestBody.replacements` (targeted
//!   JSON-pointer overrides within a payload) are not lowered: an unimplemented feature, not a
//!   silently dropped one -- nothing in this crate claims replacements are applied.
//! - **Parameter/component `$ref`s** (`ParameterOrReference::Reference`): skipped, not
//!   fabricated as an input. Dereferencing `#/components/parameters/<name>` is a real gap, left
//!   for a follow-up ticket rather than invented here.
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
//! - **Workflow-level `successActions`/`failureActions`, `depends_on`, and workflow `inputs`**
//!   are not lowered: this ticket scopes AIR at the step level (routing, expression
//!   compilation *per step*); workflow-level defaults are a distinct concern left to a
//!   follow-up ticket.
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
//! - **Timeout / retry policy** (PROJ-754): [`validate_step_timeout`] refuses a step declaring
//!   `timeout: 0`; [`validate_retry_policy`] refuses a `type: retry` failure action whose
//!   `retryLimit` is `0` or whose `retryAfter` is negative or non-finite. Both fields remain
//!   optional per spec -- only an explicitly present, unsatisfiable value is refused, with
//!   [`Refusal::MalformedRetryPolicy`].
//!
//! # Complexity
//! O(w + s + p + a + d) where w = workflow count, s = total step count, p = total parameter +
//! output count, a = total success/failure routing-rule count, d = total `depends_on` edge
//! count. One linear pass over `doc` plus one `O(steps + edges)` dependency-graph traversal per
//! workflow (`validate_step_dependencies`, iterative DFS, each node/edge visited once); no
//! sorting is ever needed (`Step.outputs` / `Components.*` are already `BTreeMap`s).

use crate::air::{
    AirAction, AirExpr, AirProgram, AirRouting, AirRoutingOutcome, AirStep, AirTarget, AirWorkflow,
};
use crate::Refusal;
use bumpalo::collections::{String as BumpString, Vec as BumpVec};
use bumpalo::Bump;
use serde_json::Value;
use std::collections::HashMap;
use wasm4pm_compat::arazzo::{
    ArazzoDescription, Components, Criterion, ExpressionKind, ExpressionTypeOrKind, FailureAction,
    FailureActionOrReference, FailureActionType, ParameterOrReference, ReusableObject, Step,
    SuccessAction, SuccessActionOrReference, SuccessActionType, Workflow,
};

/// Lowers every workflow in `doc` into an [`AirProgram`] allocated in `bump`. See the module
/// doc comment for the exact field-by-field mapping and its documented scope boundaries.
///
/// # Errors
/// [`Refusal::InvalidWorkflow`] if `doc` declares zero workflows, or a workflow declares zero
/// steps. [`Refusal::MissingIdentity`] if a workflow/step has an empty id, a step names none of
/// `operationId`/`operationPath`/`channelPath`/`workflowId`, or a `goto` routing action names
/// neither `stepId` nor `workflowId`. [`Refusal::UnresolvableReference`] if a success/failure
/// action reference cannot be dereferenced against this document's own `components` (see
/// module doc for the local-only dereferencing boundary).
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
    validate_step_dependencies(wf)?;
    let mut steps = BumpVec::with_capacity_in(wf.steps.len(), bump);
    for step in &wf.steps {
        steps.push(lower_step(step, components, bump)?);
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

fn lower_step<'bump>(
    step: &Step,
    components: Option<&Components>,
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
        if let ParameterOrReference::Parameter(p) = param {
            inputs.push(lower_json_value(&p.value, bump));
        }
        // Reference(ReusableObject) parameters: local component dereferencing is out of
        // scope for this bridge (see module doc) -- skipped, not fabricated.
    }
    if let Some(body) = &step.request_body {
        if let Some(payload) = &body.payload {
            inputs.push(lower_json_value(payload, bump));
        }
    }

    let mut outputs = BumpVec::new_in(bump);
    for name in step.outputs.keys() {
        outputs.push(AirExpr::Literal(BumpString::from_str_in(name, bump)));
    }

    let on_success = lower_success_routing(&step.on_success, components, bump)?;
    let on_failure = lower_failure_routing(&step.on_failure, components, bump)?;

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

fn lower_success_routing<'bump>(
    actions: &[SuccessActionOrReference],
    components: Option<&Components>,
    bump: &'bump Bump,
) -> Result<BumpVec<'bump, AirRouting<'bump>>, Refusal> {
    let mut out = BumpVec::with_capacity_in(actions.len(), bump);
    for action in actions {
        let resolved: SuccessAction = match action {
            SuccessActionOrReference::Action(a) => a.clone(),
            SuccessActionOrReference::Reference(r) => resolve_success_reference(r, components)?,
        };
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

fn lower_failure_routing<'bump>(
    actions: &[FailureActionOrReference],
    components: Option<&Components>,
    bump: &'bump Bump,
) -> Result<BumpVec<'bump, AirRouting<'bump>>, Refusal> {
    let mut out = BumpVec::with_capacity_in(actions.len(), bump);
    for action in actions {
        let resolved: FailureAction = match action {
            FailureActionOrReference::Action(a) => a.clone(),
            FailureActionOrReference::Reference(r) => resolve_failure_reference(r, components)?,
        };
        if resolved.action_type == FailureActionType::Retry {
            validate_retry_policy(&resolved)?;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
