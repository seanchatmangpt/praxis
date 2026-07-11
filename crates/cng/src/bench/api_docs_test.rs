#![cfg(test)]

//! OpenAPI/AsyncAPI capability-document render verification tests: the
//! function-level digest-comparison law (matching/mismatched/absent), and
//! the real `engine::engine_serve` call-site wiring (matching docs let the
//! engine start, a tampered doc refuses CNG_R11 before any poll, absent
//! docs do not block engine start — the honest skip-if-absent path).

use std::fs;
use std::path::PathBuf;

use chicago_tdd_tools::prelude::*;

use super::{verify_api_docs_render_digest, verify_api_docs_render_digest_if_present};
use crate::bench::engine::{engine_serve, EngineBundle};
use crate::powl::CngRefusal;

/// Scratch root for this test file. O(1).
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/api_docs")
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// Seeds `<root>/generated/{engine-openapi,engine-asyncapi}.yaml` and a
/// `.ggen-v2/receipt.json` recording their true BLAKE3 digests — the exact
/// seam `packs/arazzo-pack/README.md`'s "Downstream verification seam"
/// describes, mirrored from the sibling Arazzo document family.
fn seed_matching_api_docs(root: &std::path::Path) -> (String, String) {
    fs::create_dir_all(root.join("generated")).expect("generated dir");
    fs::create_dir_all(root.join(".ggen-v2")).expect(".ggen-v2 dir");
    let openapi_bytes: &[u8] = b"openapi: 3.1.0\ninfo:\n  title: test\n";
    let asyncapi_bytes: &[u8] = b"asyncapi: 3.0.0\ninfo:\n  title: test\n";
    fs::write(root.join("generated/engine-openapi.yaml"), openapi_bytes).expect("write openapi");
    fs::write(root.join("generated/engine-asyncapi.yaml"), asyncapi_bytes).expect("write asyncapi");
    let openapi_digest = blake3::hash(openapi_bytes).to_hex().to_string();
    let asyncapi_digest = blake3::hash(asyncapi_bytes).to_hex().to_string();
    let receipt = format!(
        "{{\"payload\":{{\"outputs\":{{\"generated/engine-openapi.yaml\":\"{openapi_digest}\",\
         \"generated/engine-asyncapi.yaml\":\"{asyncapi_digest}\"}}}}}}"
    );
    fs::write(root.join(".ggen-v2/receipt.json"), receipt).expect("write receipt");
    (openapi_digest, asyncapi_digest)
}

test!(matching_api_docs_verify_both_documents, {
    // Arrange: a scratch ggen project root with both capability documents
    // and a receipt recording their true digests.
    let root = scratch_dir("function_match");
    let (openapi_digest, asyncapi_digest) = seed_matching_api_docs(&root);

    // Act.
    let result = verify_api_docs_render_digest(&root);

    // Assert: both documents verified, in the fixed (openapi, asyncapi)
    // order, carrying the recomputed digests.
    let verifications = result.expect("matching digests verify");
    assert_eq!(verifications.len(), 2);
    assert_eq!(
        verifications[0].output_path,
        "generated/engine-openapi.yaml"
    );
    assert_eq!(verifications[0].digest, openapi_digest);
    assert_eq!(
        verifications[1].output_path,
        "generated/engine-asyncapi.yaml"
    );
    assert_eq!(verifications[1].digest, asyncapi_digest);
});

test!(tampered_api_doc_render_refuses_cng_r11_audit_mismatch, {
    // Arrange: same shape as the matching case, but the OpenAPI render
    // on disk is corrupted (one byte flipped) after the receipt
    // recorded the original digest — a tampered/stale render.
    let root = scratch_dir("function_mismatch");
    seed_matching_api_docs(&root);
    let path = root.join("generated/engine-openapi.yaml");
    let mut bytes = fs::read(&path).expect("read openapi render");
    bytes[0] ^= 0xFF;
    fs::write(&path, &bytes).expect("write corrupted openapi render");

    // Act.
    let result = verify_api_docs_render_digest(&root);

    // Assert: typed CNG_R11 refusal naming the mismatch.
    match result {
        Err(CngRefusal::AuditMismatch(msg)) => {
            assert!(
                msg.contains("digest mismatch"),
                "message names the mismatch, got {msg}"
            );
            assert_eq!(CngRefusal::AuditMismatch(msg).code(), "CNG_R11");
        }
        Err(other) => panic!("expected AuditMismatch, got {other:?}"),
        Ok(_) => {
            panic!("expected AuditMismatch for a corrupted render, but verification succeeded")
        }
    }
});

test!(absent_api_docs_verify_returns_none, {
    // Arrange: a scratch root with neither the generated docs nor a
    // receipt — the common case today (arazzo-pack not yet synced against
    // this project root).
    let root = scratch_dir("function_absent");
    fs::create_dir_all(&root).expect("root dir");

    // Act.
    let result = verify_api_docs_render_digest_if_present(&root);

    // Assert: an honest, silent skip — not a refusal.
    assert_eq!(result.expect("absence is not a refusal"), None);
});

test!(engine_serve_proceeds_when_api_docs_present_and_matching, {
    // Arrange: an engine bundle whose project_root carries matching
    // capability docs + receipt (root == the engine_serve `root` argument,
    // the same project_root convention as arazzo's adapter.project_root()).
    let root = scratch_dir("wiring_match");
    seed_matching_api_docs(&root);
    let bundle = EngineBundle::new(&root, "H").expect("bundle constructs");

    // Act: bounded serve with an empty inbox — the poll budget ends it.
    let report = engine_serve(&root, "H", 42, 1, None);

    // Assert: the capability-doc verification passed silently and the
    // serve loop ran to completion (poll budget exhausted, nothing to do).
    let report = report.expect("serve proceeds when api docs match their receipt");
    assert_eq!(report.polls, 1);
    assert_eq!(report.contracts_executed, 0);
    assert!(!report.quiesced);
    // The bundle really was constructed under this root (sanity check the
    // fixture wiring, not a re-assertion of engine_serve internals).
    assert!(bundle.dir().starts_with(&root));
});

test!(engine_serve_refuses_cng_r11_when_api_doc_render_tampered, {
    // Arrange: same project_root shape, but the AsyncAPI render is
    // tampered after the receipt recorded its true digest.
    let root = scratch_dir("wiring_mismatch");
    seed_matching_api_docs(&root);
    let path = root.join("generated/engine-asyncapi.yaml");
    let mut bytes = fs::read(&path).expect("read asyncapi render");
    bytes[0] ^= 0xFF;
    fs::write(&path, &bytes).expect("write corrupted asyncapi render");

    // Act: engine_serve must refuse BEFORE entering the poll loop —
    // an engine advertising a tampered capability document must not
    // start serving.
    let result = engine_serve(&root, "H", 42, 5, None);

    // Assert: typed CNG_R11 AuditMismatch, naming the asyncapi output.
    match result {
        Err(CngRefusal::AuditMismatch(msg)) => {
            assert!(
                msg.contains("engine-asyncapi.yaml"),
                "refusal names the tampered document, got {msg}"
            );
            assert_eq!(CngRefusal::AuditMismatch(msg).code(), "CNG_R11");
        }
        Err(other) => panic!("expected AuditMismatch, got {other:?}"),
        Ok(_) => panic!(
            "expected engine_serve to refuse on a tampered capability \
                 document, but it proceeded"
        ),
    }
});

test!(engine_serve_proceeds_when_api_docs_absent, {
    // Arrange: an engine bundle whose project_root carries NEITHER the
    // capability docs nor a receipt — the common case today. This is the
    // honest skip-if-absent path: absence must not be a false refusal.
    let root = scratch_dir("wiring_absent");
    let bundle = EngineBundle::new(&root, "H").expect("bundle constructs");

    // Act.
    let report = engine_serve(&root, "H", 42, 1, None);

    // Assert: the engine starts and runs normally, exactly as it did
    // before this capability-doc check existed.
    let report = report.expect("absent api docs do not block engine start");
    assert_eq!(report.polls, 1);
    assert_eq!(report.contracts_executed, 0);
    assert!(!report.quiesced);
    assert!(bundle.dir().starts_with(&root));
});
