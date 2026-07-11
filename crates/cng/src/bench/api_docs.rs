//! Verifies the arazzo-pack's per-engine OpenAPI 3.1 / AsyncAPI 3.0
//! capability documents (`generated/engine-openapi.yaml`,
//! `generated/engine-asyncapi.yaml` — the `to:` targets of
//! `packs/arazzo-pack/templates/{engine-openapi,engine-asyncapi}.yaml.tmpl`)
//! against the ggen sync receipt's recorded digest for those outputs
//! (`.ggen-v2/receipt.json`, `payload.outputs[...]` — see
//! `packs/arazzo-pack/README.md`'s "Downstream verification seam"). Same
//! digest-comparison discipline as `arazzo::verify_arazzo_render_digest`
//! (PROJ-745): recompute BLAKE3 over the on-disk file's bytes, compare
//! against the receipt's recorded digest, never re-admit or re-parse the
//! YAML as truth. `CngRefusal::AuditMismatch` (CNG_R11) on any missing/
//! mismatched artifact — no new refusal code, no new receipt format (the
//! `.ggen-v2/receipt.json` shape below is field-for-field the same as
//! `arazzo::GgenReceiptDocument`/`GgenReceiptPayload`, re-declared locally
//! per that module's own rationale: `cng` does not depend on the `ggen`
//! crate for one field lookup).
//!
//! Honesty boundary: these two documents are the DECLARED capability/event
//! contract of the Chatman Engine processes (see the templates' own header
//! comments — `submitWorkflow`/`getExecutionEvidence`/`quiesce`;
//! `workflowAcknowledged`/`workflowResultProduced`); the implemented
//! transport binding is filesystem, not HTTP. Verifying the render digest
//! confirms the declared document was not silently altered or never
//! rendered — it does not and cannot confirm any HTTP surface exists.
//!
//! Wired call site: `engine::engine_serve`, once at startup before the poll
//! loop — but ONLY when both rendered outputs and the receipt are present
//! at the engine's project_root (the common case today is neither exists,
//! since arazzo-pack has not been synced against every engine root);
//! absence is a silent, correct skip, not a refusal (mirrors the
//! "declared-contract mechanism ALIVE / HTTP binding UNVERIFIED" honesty
//! boundary already established for these documents — an engine without
//! pre-generated capability docs is not thereby lying about anything).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::powl::CngRefusal;

/// Relative path (from a ggen project root) of the arazzo-pack's rendered
/// OpenAPI capability document — the `to:` target of
/// `packs/arazzo-pack/templates/engine-openapi.yaml.tmpl`.
const OPENAPI_RENDERED_YAML_REL_PATH: &str = "generated/engine-openapi.yaml";

/// Relative path (from a ggen project root) of the arazzo-pack's rendered
/// AsyncAPI event document — the `to:` target of
/// `packs/arazzo-pack/templates/engine-asyncapi.yaml.tmpl`.
const ASYNCAPI_RENDERED_YAML_REL_PATH: &str = "generated/engine-asyncapi.yaml";

/// One verified ggen-pack render: the output path checked and the BLAKE3
/// digest that was recomputed and confirmed to match the ggen sync receipt.
/// Same shape as `arazzo::ArazzoRenderVerification` (sibling document
/// family, same verification law).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct ApiDocRenderVerification {
    /// Rendered output path, relative to the ggen project root.
    pub(super) output_path: String,
    /// Recomputed BLAKE3 hex digest of the rendered file's bytes.
    pub(super) digest: String,
}

/// Minimal shape of a ggen `.ggen-v2/receipt.json` document — only the
/// `payload.outputs` map this seam reads. Field-for-field identical to
/// `arazzo::GgenReceiptDocument`/`GgenReceiptPayload` (same receipt format;
/// re-declared locally, not imported, so this module stays a standalone
/// unit — `cng` does not depend on the `ggen` crate for one field lookup).
#[derive(Debug, serde::Deserialize)]
struct GgenReceiptDocument {
    payload: GgenReceiptPayload,
}

/// See [`GgenReceiptDocument`].
#[derive(Debug, serde::Deserialize)]
struct GgenReceiptPayload {
    outputs: BTreeMap<String, String>,
}

/// Verifies one rendered output (`rel_path`, relative to `project_root`)
/// against the ggen sync receipt's recorded digest for that output. Exactly
/// the digest-comparison logic of `arazzo::verify_arazzo_render_digest`,
/// parameterized by output path.
///
/// # Errors
/// `CNG_R11 AuditMismatch` when: the rendered file is missing/unreadable,
/// the receipt is missing/unreadable/unparseable, the receipt has no entry
/// for `rel_path`, or the recomputed digest disagrees with the recorded one.
///
/// # Complexity
/// O(rendered file bytes) BLAKE3 hash + O(receipt bytes) JSON parse.
fn verify_one_render_digest(
    project_root: &Path,
    rel_path: &'static str,
) -> Result<ApiDocRenderVerification, CngRefusal> {
    let output_path = project_root.join(rel_path);
    let bytes = fs::read(&output_path).map_err(|e| {
        CngRefusal::AuditMismatch(format!(
            "api doc render not auditable — cannot read {}: {e}",
            output_path.display()
        ))
    })?;
    let recomputed = blake3::hash(&bytes).to_hex().to_string();

    let receipt_path = project_root.join(".ggen-v2").join("receipt.json");
    let receipt_text = fs::read_to_string(&receipt_path).map_err(|e| {
        CngRefusal::AuditMismatch(format!(
            "api doc render not auditable — cannot read ggen receipt {}: {e}",
            receipt_path.display()
        ))
    })?;
    let receipt: GgenReceiptDocument = serde_json::from_str(&receipt_text).map_err(|e| {
        CngRefusal::AuditMismatch(format!(
            "api doc render not auditable — cannot parse ggen receipt {}: {e}",
            receipt_path.display()
        ))
    })?;
    let recorded = receipt.payload.outputs.get(rel_path).ok_or_else(|| {
        CngRefusal::AuditMismatch(format!(
            "ggen receipt {} has no digest recorded for {rel_path}",
            receipt_path.display()
        ))
    })?;
    if &recomputed != recorded {
        return Err(CngRefusal::AuditMismatch(format!(
            "api doc render digest mismatch for {rel_path} — recomputed \
             {recomputed} vs receipt {recorded}"
        )));
    }

    Ok(ApiDocRenderVerification {
        output_path: rel_path.to_string(),
        digest: recomputed,
    })
}

/// Whether BOTH capability documents AND the ggen receipt are present at
/// `project_root`. Absence is the common case today (arazzo-pack has not
/// been synced against every engine root) and is NOT a refusal — see
/// [`verify_api_docs_render_digest_if_present`].
///
/// # Complexity
/// O(1) — three filesystem existence checks.
fn api_docs_present(project_root: &Path) -> bool {
    project_root.join(OPENAPI_RENDERED_YAML_REL_PATH).is_file()
        && project_root
            .join(ASYNCAPI_RENDERED_YAML_REL_PATH)
            .is_file()
        && project_root.join(".ggen-v2").join("receipt.json").is_file()
}

/// Verifies both the OpenAPI and AsyncAPI capability documents against the
/// ggen sync receipt's recorded digests. Always attempts both (missing
/// inputs refuse `CNG_R11`); callers that need the honest skip-if-absent
/// gate use [`verify_api_docs_render_digest_if_present`] instead.
///
/// # Errors
/// `CNG_R11 AuditMismatch` — see [`verify_one_render_digest`].
///
/// # Complexity
/// O(rendered file bytes) BLAKE3 hash x2 + O(receipt bytes) JSON parse x2.
pub(super) fn verify_api_docs_render_digest(
    project_root: &Path,
) -> Result<Vec<ApiDocRenderVerification>, CngRefusal> {
    Ok(vec![
        verify_one_render_digest(project_root, OPENAPI_RENDERED_YAML_REL_PATH)?,
        verify_one_render_digest(project_root, ASYNCAPI_RENDERED_YAML_REL_PATH)?,
    ])
}

/// The gate `engine::engine_serve` actually needs: verifies the OpenAPI/
/// AsyncAPI capability documents ONLY when both rendered outputs and the
/// ggen receipt are present at `project_root`; returns `Ok(None)` (a
/// correct, silent skip — not a refusal) when they are absent, and
/// `Ok(Some(verifications))` when present and verified.
///
/// # Errors
/// `CNG_R11 AuditMismatch` when the documents ARE present but stale/
/// tampered/unreceipted — see [`verify_one_render_digest`].
///
/// # Complexity
/// O(1) presence check; O(rendered bytes) x2 + O(receipt bytes) x2 when
/// present.
pub(super) fn verify_api_docs_render_digest_if_present(
    project_root: &Path,
) -> Result<Option<Vec<ApiDocRenderVerification>>, CngRefusal> {
    if !api_docs_present(project_root) {
        return Ok(None);
    }
    Ok(Some(verify_api_docs_render_digest(project_root)?))
}

#[cfg(test)]
#[path = "api_docs_test.rs"]
mod api_docs_test;
