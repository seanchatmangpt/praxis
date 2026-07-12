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

/// PROJ-754 capstone (adversarial-review coverage gap): the complete real Rail A/B chain --
/// parse, resolve, lower -- run over a real Arazzo document carrying one intentional, realistic
/// defect (a `type: retry` failure action declaring `retryLimit: 0`, which permits zero retry
/// attempts and can therefore never fire as declared). Before this test, `MalformedRetryPolicy`
/// was only exercised directly against a hand-built `FailureAction`/`Step` in `lower.rs`'s own
/// unit tests (see `refuses_retry_action_with_zero_retry_limit`); this proves the refusal is
/// actually reachable starting from real JSON admitted through `DocumentIndex::add_document`,
/// exactly like the other capstones in this file, not just from `lower::validate_retry_policy`
/// called in isolation.
#[test]
fn malformed_retry_policy_is_refused_end_to_end_through_the_real_pipeline() {
    let doc_json = r##"{
      "arazzo": "1.1.0",
      "info": {
        "title": "PROJ-754 negative fixture: malformed retry policy",
        "version": "1.0.0"
      },
      "sourceDescriptions": [
        { "name": "orders-api", "url": "openapi/orders.yaml", "type": "openapi" }
      ],
      "workflows": [
        {
          "workflowId": "distinctive-proj754-retry-workflow",
          "steps": [
            {
              "stepId": "distinctive-validate-step",
              "operationId": "urn:test:proj754/validate_order",
              "onFailure": [
                {
                  "name": "distinctive-retry-validate",
                  "type": "retry",
                  "retryAfter": 1.5,
                  "retryLimit": 0
                }
              ]
            }
          ]
        }
      ]
    }"##;

    // 1. Parse: real strict serde_json admission via this crate's own DocumentIndex.
    // `retryLimit: 0` is structurally well-formed Arazzo JSON (the field is a plain, in-range
    // integer) -- it must sail through parsing unrefused.
    let mut index = DocumentIndex::new();
    index
        .add_document(doc_json, "https://example.com/test/proj754/malformed-retry")
        .expect(
            "well-formed Arazzo 1.1.0 document must parse even though a later stage refuses it",
        );

    // 2. Resolve: real URI normalization; retryLimit/retryAfter are not URIs, so this stage has
    // nothing to say about them either.
    normalize_uris(&mut index).expect("URI resolution must succeed over this fixture");

    let doc = index
        .documents
        .get("https://example.com/test/proj754/malformed-retry")
        .expect("document was inserted under this exact base URI");
    assert_eq!(
        doc.workflows[0].steps[0]
            .on_failure
            .iter()
            .filter_map(|a| match a {
                wasm4pm_compat::arazzo::FailureActionOrReference::Action(action) =>
                    action.retry_limit,
                wasm4pm_compat::arazzo::FailureActionOrReference::Reference(_) => None,
            })
            .next(),
        Some(0),
        "retryLimit: 0 must survive parse and resolve unchanged"
    );

    // 3. Lower: `validate_retry_policy` (called from `lower_step` for every type=retry failure
    // action) is where the whole chain must refuse -- specifically, not generically.
    let bump = Bump::new();
    let result = lower_description(doc, &bump);
    match result {
        Err(wasm4pm_arazzo::Refusal::MalformedRetryPolicy(msg)) => {
            assert!(
                msg.contains("distinctive-retry-validate"),
                "refusal message must name the offending failure action, traced from the real \
                 document, got: {msg}"
            );
            assert!(
                msg.contains("retryLimit: 0"),
                "refusal message must name the actual invalid field/value, got: {msg}"
            );
        }
        other => panic!(
            "expected Refusal::MalformedRetryPolicy for a type=retry failure action with \
             retryLimit: 0, got {other:?}"
        ),
    }
}

/// PROJ-784 capstone: `AIR_PARSE_REFUSED` (`docs/jira/v26.7.11/PRD.md` sec. 18). A document
/// missing the required `info` object is not a hypothetical -- it is a real defect
/// `DocumentIndex::add_document`'s own `serde_json` admission catches before any later pipeline
/// stage (resolve/lower/normalize/compile) ever runs. This is the earliest possible refusal
/// point in the parse -> resolve -> lower -> normalize -> compile chain, so unlike the other
/// PROJ-784 capstones below, the "pipeline" here is just the one stage that can ever run.
#[test]
fn malformed_arazzo_document_is_refused_at_parse_before_any_later_stage_runs() {
    let doc_json = r#"{
      "arazzo": "1.1.0",
      "sourceDescriptions": [],
      "workflows": [
        {
          "workflowId": "wf_missing_info",
          "steps": [
            { "stepId": "step_1", "operationId": "urn:test:proj784/op1" }
          ]
        }
      ]
    }"#;

    let mut index = DocumentIndex::new();
    let result = index.add_document(doc_json, "https://example.com/test/proj784/malformed");
    match result {
        Err(wasm4pm_arazzo::Refusal::Parse(msg)) => {
            assert!(
                msg.contains("info"),
                "parse refusal must name the missing/offending construct, got: {msg}"
            );
        }
        other => panic!(
            "expected Refusal::Parse for a document missing the required `info` object, got \
             {other:?}"
        ),
    }
    assert!(
        index.documents.is_empty(),
        "a document that fails to parse must never be admitted into the index"
    );
}

/// PROJ-784 capstone: `AIR_EXPRESSION_UNSUPPORTED`. The complete real Rail A/B chain -- parse,
/// resolve, lower -- run over a real Arazzo document whose step declares a `Selector`-shaped
/// output (a structured JSONPath selector object) rather than the spec's plain
/// runtime-expression string. Before PROJ-784, `lower::lower_step` read only the *keys* of
/// `Step.outputs` and never inspected the value, so this exact document shape would have lowered
/// silently with the selector discarded; PROJ-784's `classify_output_value` closes that gap.
#[test]
fn selector_shaped_step_output_is_refused_end_to_end_through_the_real_pipeline() {
    let doc_json = r##"{
      "arazzo": "1.1.0",
      "info": {
        "title": "PROJ-784 negative fixture: selector-shaped output",
        "version": "1.0.0"
      },
      "sourceDescriptions": [
        { "name": "orders-api", "url": "openapi/orders.yaml", "type": "openapi" }
      ],
      "workflows": [
        {
          "workflowId": "distinctive-proj784-selector-workflow",
          "steps": [
            {
              "stepId": "distinctive-validate-step",
              "operationId": "urn:test:proj784/validate_order",
              "outputs": {
                "order_id": {
                  "context": "$response.body",
                  "selector": "$.id",
                  "type": "jsonpath"
                }
              }
            }
          ]
        }
      ]
    }"##;

    // 1. Parse: a selector-shaped output is structurally well-formed Arazzo JSON; it must sail
    // through admission unrefused.
    let mut index = DocumentIndex::new();
    index
        .add_document(doc_json, "https://example.com/test/proj784/selector-output")
        .expect(
            "well-formed Arazzo 1.1.0 document must parse even though a later stage refuses it",
        );

    // 2. Resolve: the selector's `context`/`selector` fields are not URIs; this stage has
    // nothing to say about them either.
    normalize_uris(&mut index).expect("URI resolution must succeed over this fixture");

    let doc = index
        .documents
        .get("https://example.com/test/proj784/selector-output")
        .expect("document was inserted under this exact base URI");
    assert_eq!(doc.workflows[0].steps[0].outputs.len(), 1);

    // 3. Lower: PROJ-784's new `classify_output_value` check is where the chain must refuse.
    let bump = Bump::new();
    let result = lower_description(doc, &bump);
    match result {
        Err(wasm4pm_arazzo::Refusal::UnsupportedExpression(msg)) => {
            assert!(
                msg.contains("order_id"),
                "refusal message must name the offending output, got: {msg}"
            );
            assert!(
                msg.contains("distinctive-validate-step"),
                "refusal message must name the step declaring the offending output, got: {msg}"
            );
        }
        other => panic!(
            "expected Refusal::UnsupportedExpression for a Selector-shaped step output, got \
             {other:?}"
        ),
    }
}

/// PROJ-784 capstone: `AIR_CRITERION_UNSUPPORTED`, this code's first full-pipeline (not just
/// `lower::lower_description`-direct, as in `lower.rs`'s own
/// `refuses_unsupported_criterion_expression_shape` unit test) coverage -- a real document
/// carrying a `jsonpath`-typed success-action criterion, admitted through `DocumentIndex`
/// exactly like the other capstones in this file.
#[test]
fn jsonpath_criterion_is_refused_end_to_end_through_the_real_pipeline() {
    let doc_json = r##"{
      "arazzo": "1.1.0",
      "info": {
        "title": "PROJ-784 negative fixture: jsonpath criterion",
        "version": "1.0.0"
      },
      "sourceDescriptions": [
        { "name": "orders-api", "url": "openapi/orders.yaml", "type": "openapi" }
      ],
      "workflows": [
        {
          "workflowId": "distinctive-proj784-criterion-workflow",
          "steps": [
            {
              "stepId": "distinctive-validate-step",
              "operationId": "urn:test:proj784/validate_order",
              "onSuccess": [
                {
                  "name": "go_next",
                  "type": "end",
                  "criteria": [
                    {
                      "context": "$response.body",
                      "condition": "$.distinctiveOrders[?(@.id == 1)]",
                      "type": "jsonpath"
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }"##;

    let mut index = DocumentIndex::new();
    index
        .add_document(
            doc_json,
            "https://example.com/test/proj784/jsonpath-criterion",
        )
        .expect(
            "well-formed Arazzo 1.1.0 document must parse even though a later stage refuses it",
        );

    normalize_uris(&mut index).expect("URI resolution must succeed over this fixture");

    let doc = index
        .documents
        .get("https://example.com/test/proj784/jsonpath-criterion")
        .expect("document was inserted under this exact base URI");

    let bump = Bump::new();
    let result = lower_description(doc, &bump);
    match result {
        Err(wasm4pm_arazzo::Refusal::UnsupportedCriterion(msg)) => {
            assert!(
                msg.contains("distinctiveOrders"),
                "refusal message must name the offending criterion, traced from the real \
                 document, got: {msg}"
            );
        }
        other => panic!(
            "expected Refusal::UnsupportedCriterion for a jsonpath-typed criterion, got \
             {other:?}"
        ),
    }
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

/// PROJ-784 capstone (adversarial-review correction): the complete real Rail A/B chain --
/// parse, resolve, lower, normalize -- run over a real Arazzo document where `distinctive-ship`
/// is declared *first* but `dependsOn: ["distinctive-validate"]` (declared *second*), and
/// `distinctive-ship`'s requestBody payload references `distinctive-validate`'s declared
/// output. This is a legitimate use of `depends_on` to declare non-textual execution order --
/// exactly what the field exists for per the Arazzo spec -- and it is referentially sound and
/// acyclic. Before this correction, `lower_workflow` lowered steps in raw JSON declaration
/// order, so `temporal::ReferenceResolver::resolve` (which only ever treats array order as
/// "earlier step" order) would wrongly refuse this exact document as an
/// `UnresolvableReference`, even though nothing about it is actually invalid. This test proves
/// the full real pipeline now resolves it instead.
#[test]
fn out_of_textual_order_depends_on_reference_resolves_through_the_real_pipeline() {
    let doc_json = r##"{
      "arazzo": "1.1.0",
      "info": {
        "title": "PROJ-784 fixture: depends_on declares non-textual execution order",
        "version": "1.0.0"
      },
      "sourceDescriptions": [
        { "name": "orders-api", "url": "openapi/orders.yaml", "type": "openapi" }
      ],
      "workflows": [
        {
          "workflowId": "distinctive-proj784-order-workflow",
          "steps": [
            {
              "stepId": "distinctive-ship",
              "operationId": "urn:test:proj784/ship_order",
              "dependsOn": ["distinctive-validate"],
              "requestBody": {
                "payload": "$steps.distinctive-validate.outputs.order_id"
              }
            },
            {
              "stepId": "distinctive-validate",
              "operationId": "urn:test:proj784/validate_order",
              "outputs": {
                "order_id": "$response.body#/id"
              }
            }
          ]
        }
      ]
    }"##;

    // 1. Parse: real strict serde_json admission. `distinctive-ship` declared before
    // `distinctive-validate` (its own dependency) is structurally well-formed Arazzo JSON.
    let mut index = DocumentIndex::new();
    index
        .add_document(doc_json, "https://example.com/test/proj784/depends-order")
        .expect("well-formed Arazzo 1.1.0 document must parse");

    // 2. Resolve: real URI normalization; unaffected by step order.
    normalize_uris(&mut index).expect("URI resolution must succeed over this fixture");

    let doc = index
        .documents
        .get("https://example.com/test/proj784/depends-order")
        .expect("document was inserted under this exact base URI");
    assert_eq!(doc.workflows[0].steps[0].step_id, "distinctive-ship");
    assert_eq!(doc.workflows[0].steps[1].step_id, "distinctive-validate");

    // 3. Lower: validate_step_dependencies confirms the graph is acyclic and sound, then
    // topological_sort_step_indices reorders lowering so distinctive-validate (the real
    // dependency) is lowered before distinctive-ship (the dependent), regardless of their
    // textual position in the source document.
    let bump = Bump::new();
    let mut program =
        lower_description(doc, &bump).expect("acyclic, referentially sound depends_on must lower");
    assert_eq!(program.workflows[0].steps[0].name, "distinctive-validate");
    assert_eq!(program.workflows[0].steps[1].name, "distinctive-ship");

    // 4. Normalize: must resolve, not wrongly refuse, now that step order matches dependency
    // order.
    ArazzoNormalizer::normalize(&mut program, &bump).expect(
        "distinctive-ship's reference to distinctive-validate's real output must resolve, not \
         be wrongly refused as unresolvable, once steps are lowered in dependency order",
    );
    match &program.workflows[0].steps[1].action.inputs[0] {
        wasm4pm_arazzo::air::AirExpr::Literal(l) => assert_eq!(l, "order_id"),
        wasm4pm_arazzo::air::AirExpr::Variable(_) => {
            panic!("normalization must resolve the out-of-textual-order reference into a Literal")
        }
    }
}
