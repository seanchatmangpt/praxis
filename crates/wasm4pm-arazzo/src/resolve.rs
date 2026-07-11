use crate::{parse::DocumentIndex, Refusal};
use url::Url;
use wasm4pm_compat::arazzo::{
    ArazzoDescription, FailureActionOrReference, ParameterOrReference, ReusableObject,
    SuccessActionOrReference,
};
use rayon::prelude::*;

/// Normalizes all URIs in the given index. Resolves relative references to
/// absolute URIs based on the document's base URI. This does not follow or
/// resolve the cross-references, it only makes them absolute.
pub fn normalize_uris(index: &mut DocumentIndex) -> Result<(), Refusal> {
    index.documents
        .par_iter_mut()
        .try_for_each(|(base_uri_str, doc)| {
            normalize_document_uris(doc, base_uri_str)
        })
}

fn normalize_document_uris(doc: &mut ArazzoDescription, base_uri_str: &str) -> Result<(), Refusal> {
    let base_uri = Url::parse(base_uri_str).map_err(|e| {
        Refusal::UriResolution(format!("Invalid base URI '{}': {}", base_uri_str, e))
    })?;

    // 1. Resolve sourceDescription URLs
    doc.source_descriptions.par_iter_mut().try_for_each(|source| {
        let resolved = base_uri.join(&source.url).map_err(|e| {
            Refusal::UriResolution(format!(
                "Failed to resolve source URL '{}': {}",
                source.url, e
            ))
        })?;
        source.url = resolved.to_string();
        Ok::<(), Refusal>(())
    })?;

    // 2. Resolve references in workflows
    doc.workflows.par_iter_mut().try_for_each(|workflow| {
        workflow.success_actions.par_iter_mut().try_for_each(|action| {
            if let SuccessActionOrReference::Reference(r) = action {
                resolve_reusable_object(r, &base_uri)?;
            }
            Ok::<(), Refusal>(())
        })?;

        workflow.failure_actions.par_iter_mut().try_for_each(|action| {
            if let FailureActionOrReference::Reference(r) = action {
                resolve_reusable_object(r, &base_uri)?;
            }
            Ok::<(), Refusal>(())
        })?;

        workflow.steps.par_iter_mut().try_for_each(|step| {
            step.parameters.par_iter_mut().try_for_each(|param| {
                if let ParameterOrReference::Reference(r) = param {
                    resolve_reusable_object(r, &base_uri)?;
                }
                Ok::<(), Refusal>(())
            })?;

            step.on_success.par_iter_mut().try_for_each(|on_success| {
                if let SuccessActionOrReference::Reference(r) = on_success {
                    resolve_reusable_object(r, &base_uri)?;
                }
                Ok::<(), Refusal>(())
            })?;

            step.on_failure.par_iter_mut().try_for_each(|on_failure| {
                if let FailureActionOrReference::Reference(r) = on_failure {
                    resolve_reusable_object(r, &base_uri)?;
                }
                Ok::<(), Refusal>(())
            })?;
            Ok::<(), Refusal>(())
        })?;
        Ok::<(), Refusal>(())
    })?;

    // 3. Resolve references in components
    if let Some(components) = &mut doc.components {
        components.success_actions.par_iter_mut().try_for_each(|(_, action)| {
            action.parameters.par_iter_mut().try_for_each(|param| {
                if let ParameterOrReference::Reference(r) = param {
                    resolve_reusable_object(r, &base_uri)?;
                }
                Ok::<(), Refusal>(())
            })
        })?;

        components.failure_actions.par_iter_mut().try_for_each(|(_, action)| {
            action.parameters.par_iter_mut().try_for_each(|param| {
                if let ParameterOrReference::Reference(r) = param {
                    resolve_reusable_object(r, &base_uri)?;
                }
                Ok::<(), Refusal>(())
            })
        })?;
    }

    Ok(())
}

fn resolve_reusable_object(r: &mut ReusableObject, base: &Url) -> Result<(), Refusal> {
    // Fast path: if the reference is already absolute (e.g. starts with http:// or https://)
    if r.reference.starts_with("http://") || r.reference.starts_with("https://") {
        return Ok(());
    }

    // Fast path: if the reference is just a fragment, we can just join it quickly.
    // However, url::Url::join handles fragments safely, but we can do a very fast check
    // if we just want a string concatenation, but `Url::join` handles relative resolution correctly.
    let resolved = base.join(&r.reference).map_err(|e| {
        Refusal::UriResolution(format!(
            "Failed to resolve reference '{}': {}",
            r.reference, e
        ))
    })?;
    
    // Instead of allocating a new string via to_string() and potentially dropping the old,
    // we just replace it directly.
    r.reference = resolved.into();
    Ok(())
}
