//! PROJ-753 end-to-end proof: a real Arazzo 1.1.0 JSON document, parsed through this crate's
//! own admission path (`parse::DocumentIndex`), resolved (`resolve::normalize_uris`), lowered
//! to AIR (`lower::lower_description` -- the bridge this ticket adds), normalized
//! (`normalizer::ArazzoNormalizer`, which resolves cross-step variable references via
//! `temporal::ReferenceResolver`), and compiled to WASM (`compile::AirCompiler`). Before this
//! ticket, `ArazzoDescription` and `AirProgram` were disconnected islands (see
//! `docs/jira/v26.7.11/RAIL_A_B_STATUS.md`, Rail B) -- every `AirProgram` anywhere in this
//! crate's tests was a hand-built fixture. This test starts from parsed JSON instead, and
//! never constructs an `AirProgram` by hand.
//!
//! The document below exercises real step-output cross-referencing (step_2 references
//! step_1's declared output through a genuine `$steps.<id>.outputs.<name>` runtime
//! expression), a local `#/components/successActions/<name>` routing reference, and a
//! `goto`-typed failure action -- not just the trivial single-leaf case.

use bumpalo::Bump;
use wasm4pm_arazzo::air::AirRoutingOutcome;
use wasm4pm_arazzo::compile::AirCompiler;
use wasm4pm_arazzo::lower::lower_description;
use wasm4pm_arazzo::normalizer::ArazzoNormalizer;
use wasm4pm_arazzo::parse::DocumentIndex;
use wasm4pm_arazzo::resolve::normalize_uris;

const REALISTIC_ARAZZO_DOCUMENT: &str = r##"{
  "arazzo": "1.1.0",
  "info": {
    "title": "PROJ-753 end-to-end fixture",
    "version": "1.0.0"
  },
  "sourceDescriptions": [
    { "name": "orders-api", "url": "openapi/orders.yaml", "type": "openapi" }
  ],
  "components": {
    "successActions": {
      "finish_ok": { "name": "finish_ok", "type": "end" }
    }
  },
  "workflows": [
    {
      "workflowId": "distinctive-order-fulfillment-workflow",
      "steps": [
        {
          "stepId": "distinctive-validate-step",
          "operationId": "urn:test:proj753/validate_order",
          "parameters": [
            { "name": "region", "value": "us-east" }
          ],
          "outputs": {
            "order_id": "$response.body#/id"
          },
          "onSuccess": [
            { "reference": "#/components/successActions/finish_ok" }
          ],
          "onFailure": [
            {
              "name": "retry_validate",
              "type": "retry",
              "retryAfter": 1.5,
              "retryLimit": 3
            }
          ]
        },
        {
          "stepId": "distinctive-ship-step",
          "operationId": "urn:test:proj753/ship_order",
          "requestBody": {
            "payload": "$steps.distinctive-validate-step.outputs.order_id"
          },
          "onFailure": [
            {
              "name": "escalate",
              "type": "goto",
              "stepId": "distinctive-validate-step",
              "criteria": [
                { "context": "$statusCode", "condition": "$statusCode == 503" }
              ]
            }
          ]
        }
      ]
    }
  ]
}"##;

#[test]
fn arazzo_document_parses_resolves_lowers_normalizes_and_compiles_to_wasm() {
    // 1. Parse: real strict serde_json admission via this crate's own DocumentIndex.
    let mut index = DocumentIndex::new();
    index
        .add_document(
            REALISTIC_ARAZZO_DOCUMENT,
            "https://example.com/test/proj753/base",
        )
        .expect("well-formed Arazzo 1.1.0 document must parse");
    assert_eq!(index.documents.len(), 1);

    // 2. Resolve: real URI normalization over the parsed document (in place).
    normalize_uris(&mut index).expect("URI resolution must succeed over this fixture");

    let doc = index
        .documents
        .get("https://example.com/test/proj753/base")
        .expect("document was inserted under this exact base URI");
    assert_eq!(doc.workflows.len(), 1);
    assert_eq!(doc.workflows[0].steps.len(), 2);

    // 3. Lower: the bridge this ticket adds -- a real ArazzoDescription becomes a real
    // AirProgram, not a hand-built fixture.
    let bump = Bump::new();
    let mut program = lower_description(doc, &bump).expect("lowering must succeed");
    assert_eq!(program.workflows.len(), 1);
    let wf = &program.workflows[0];
    assert_eq!(wf.name, "distinctive-order-fulfillment-workflow");
    assert_eq!(wf.steps.len(), 2);
    assert_eq!(wf.steps[0].name, "distinctive-validate-step");
    assert_eq!(wf.steps[0].target.url, "urn:test:proj753/validate_order");
    assert_eq!(wf.steps[0].target.method, "operationId");

    // The local component reference resolved to the real inline action it points at.
    assert_eq!(wf.steps[0].on_success.len(), 1);
    assert_eq!(wf.steps[0].on_success[0].name, "finish_ok");
    assert_eq!(wf.steps[0].on_success[0].outcome, AirRoutingOutcome::End);

    // The retry failure action lowered to a real Retry outcome.
    assert_eq!(wf.steps[0].on_failure.len(), 1);
    assert_eq!(wf.steps[0].on_failure[0].outcome, AirRoutingOutcome::Retry);

    // step_2's requestBody payload is a genuine cross-step reference to step_1's output --
    // still unresolved (a bare `Variable`) at this point, since lowering does not resolve.
    match &wf.steps[1].action.inputs[0] {
        wasm4pm_arazzo::air::AirExpr::Variable(v) => assert_eq!(v, "order_id"),
        wasm4pm_arazzo::air::AirExpr::Literal(_) => {
            panic!("requestBody payload referencing $steps...outputs.order_id must lower to a Variable before normalization")
        }
    }

    // step_2's goto failure action targets step_1 by stepId, with one criterion.
    assert_eq!(wf.steps[1].on_failure.len(), 1);
    assert_eq!(wf.steps[1].on_failure[0].criteria.len(), 1);
    match &wf.steps[1].on_failure[0].outcome {
        AirRoutingOutcome::GotoStep(s) => assert_eq!(s, "distinctive-validate-step"),
        other => panic!("expected GotoStep, got {other:?}"),
    }

    // 4. Normalize: resolve the cross-step Variable against step_1's real declared output.
    ArazzoNormalizer::normalize(&mut program, &bump)
        .expect("step_2's reference to step_1's real output must resolve");
    match &program.workflows[0].steps[1].action.inputs[0] {
        wasm4pm_arazzo::air::AirExpr::Literal(l) => assert_eq!(l, "order_id"),
        wasm4pm_arazzo::air::AirExpr::Variable(_) => {
            panic!("normalization must resolve the Variable into a Literal")
        }
    }

    // 5. Compile: real deterministic WASM emission over the lowered-and-normalized program.
    let wasm = AirCompiler::compile_to_wasm(&program).expect("compilation must succeed");
    let wasm_again = AirCompiler::compile_to_wasm(&program).expect("compilation must succeed");
    assert_eq!(
        wasm, wasm_again,
        "compiling the same lowered program twice must be byte-identical"
    );

    // The compiled module's air-canonical-v1 custom section must carry the real step
    // identifiers/URLs/routing names traced from the original parsed document -- not
    // fixture strings a test typed by hand.
    for needle in [
        "distinctive-order-fulfillment-workflow",
        "distinctive-validate-step",
        "urn:test:proj753/validate_order",
        "distinctive-ship-step",
        "urn:test:proj753/ship_order",
        "finish_ok",
        "escalate",
    ] {
        assert!(
            contains_subslice(&wasm, needle.as_bytes()),
            "compiled WASM must contain real document content {needle:?}, traced from the \
             original parsed Arazzo document through lowering and normalization"
        );
    }
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn hand_written_production_arazzo_without_operation_identity_is_refused_at_lowering() {
    // A step declaring none of operationId/operationPath/channelPath/workflowId is not a
    // hypothetical -- it is a real document shape this bridge must refuse, not silently
    // accept with a fabricated target.
    let doc_json = r#"{
      "arazzo": "1.1.0",
      "info": { "title": "negative fixture", "version": "1.0.0" },
      "sourceDescriptions": [],
      "workflows": [
        {
          "workflowId": "wf_no_identity",
          "steps": [
            { "stepId": "orphan_step" }
          ]
        }
      ]
    }"#;

    let mut index = DocumentIndex::new();
    index
        .add_document(doc_json, "https://example.com/test/proj753/negative")
        .expect("parses: identity omission is a lowering-time, not parse-time, defect");
    let doc = index
        .documents
        .get("https://example.com/test/proj753/negative")
        .unwrap();

    let bump = Bump::new();
    let result = lower_description(doc, &bump);
    assert!(matches!(
        result,
        Err(wasm4pm_arazzo::Refusal::MissingIdentity(_))
    ));
}

/// PROJ-754 capstone: the complete real Rail A/B chain -- parse, resolve, lower -- run over a
/// real Arazzo document carrying one intentional, realistic defect (a step's `dependsOn`
/// naming a step that does not exist anywhere in the workflow), proving the pipeline refuses
/// correctly and specifically, not just that it succeeds on well-formed input. Nothing here is
/// a hand-built `AirProgram`: the document below is genuine JSON, admitted through
/// `DocumentIndex::add_document` exactly like
/// `arazzo_document_parses_resolves_lowers_normalizes_and_compiles_to_wasm` above, and the
/// defect (a dangling cross-step dependency) is exactly the shape PROJ-753's own `lower.rs`
/// had no check for before PROJ-754 added `validate_step_dependencies`.
#[test]
fn dangling_step_dependency_is_refused_end_to_end_through_the_real_pipeline() {
    let doc_json = r##"{
      "arazzo": "1.1.0",
      "info": {
        "title": "PROJ-754 negative fixture: dangling step dependency",
        "version": "1.0.0"
      },
      "sourceDescriptions": [
        { "name": "orders-api", "url": "openapi/orders.yaml", "type": "openapi" }
      ],
      "workflows": [
        {
          "workflowId": "distinctive-proj754-dependency-workflow",
          "steps": [
            {
              "stepId": "distinctive-ship-step",
              "operationId": "urn:test:proj754/ship_order",
              "dependsOn": ["distinctive-nonexistent-validate-step"]
            }
          ]
        }
      ]
    }"##;

    // 1. Parse: real strict serde_json admission via this crate's own DocumentIndex. The
    // defect (a dangling `dependsOn` reference) is not a parse-time or resolve-time concern --
    // it is structurally well-formed JSON/Arazzo and must sail through both stages.
    let mut index = DocumentIndex::new();
    index
        .add_document(
            doc_json,
            "https://example.com/test/proj754/dangling-dependency",
        )
        .expect(
            "well-formed Arazzo 1.1.0 document must parse even though a later stage refuses it",
        );

    // 2. Resolve: real URI normalization; dependsOn is a bare step id, not a URI, so this
    // stage has nothing to say about it either.
    normalize_uris(&mut index).expect("URI resolution must succeed over this fixture");

    let doc = index
        .documents
        .get("https://example.com/test/proj754/dangling-dependency")
        .expect("document was inserted under this exact base URI");
    assert_eq!(
        doc.workflows[0].steps[0].depends_on,
        vec!["distinctive-nonexistent-validate-step".to_string()]
    );

    // 3. Lower: the real bridge (PROJ-753) plus PROJ-754's new dependency-soundness check.
    // This is where the whole chain must refuse -- specifically, not generically.
    let bump = Bump::new();
    let result = lower_description(doc, &bump);
    match result {
        Err(wasm4pm_arazzo::Refusal::UnresolvableReference(msg)) => {
            assert!(
                msg.contains("distinctive-nonexistent-validate-step"),
                "refusal message must name the actual dangling id traced from the source \
                 document, got: {msg}"
            );
            assert!(
                msg.contains("distinctive-ship-step"),
                "refusal message must name the step that declared the dangling dependency, \
                 got: {msg}"
            );
        }
        other => panic!(
            "expected Refusal::UnresolvableReference for a step depending on a nonexistent \
             sibling step, got {other:?}"
        ),
    }
}
