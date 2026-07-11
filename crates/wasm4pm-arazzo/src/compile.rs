use crate::air::{AirExpr, AirProgram, AirRouting, AirRoutingOutcome};
use crate::Refusal;

/// Deterministic BLAKE3 digest over an `AirProgram`'s canonical content (see [`canonical_bytes`]).
/// Identical programs always produce identical digests; no other property is claimed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirDigest(pub [u8; 32]);

/// The main compiler harness for Arazzo Intermediate Representation.
#[derive(Debug)]
pub struct AirCompiler;

/// Serializes every workflow/step/target/expression/routing-rule in `program`, in declaration
/// order, into a length-prefixed canonical byte buffer. Each field is prefixed with its length
/// (u32 LE) so distinct field boundaries never collide under concatenation. This is the single
/// source of truth for both [`AirCompiler::digest_program`] and the `air-canonical-v1` custom
/// section emitted by [`AirCompiler::compile_to_wasm`] — both are computed from these exact
/// bytes, and both therefore change if `on_success`/`on_failure` routing content changes, not
/// just `target`/`action`.
fn canonical_bytes(program: &AirProgram) -> Vec<u8> {
    let mut buf = Vec::new();
    push_field(&mut buf, b"AIR_CANONICAL_V1");
    for wf in program.workflows.iter() {
        push_field(&mut buf, wf.name.as_bytes());
        for step in wf.steps.iter() {
            push_field(&mut buf, step.name.as_bytes());
            push_field(&mut buf, step.target.url.as_bytes());
            push_field(&mut buf, step.target.method.as_bytes());
            for input in step.action.inputs.iter() {
                push_expr(&mut buf, input);
            }
            for output in step.action.outputs.iter() {
                push_expr(&mut buf, output);
            }
            for routing in step.on_success.iter() {
                push_routing(&mut buf, b"ON_SUCCESS", routing);
            }
            for routing in step.on_failure.iter() {
                push_routing(&mut buf, b"ON_FAILURE", routing);
            }
        }
    }
    buf
}

fn push_expr(buf: &mut Vec<u8>, expr: &AirExpr) {
    match expr {
        AirExpr::Literal(l) => {
            push_field(buf, b"LIT");
            push_field(buf, l.as_bytes());
        }
        AirExpr::Variable(v) => {
            push_field(buf, b"VAR");
            push_field(buf, v.as_bytes());
        }
    }
}

fn push_routing(buf: &mut Vec<u8>, kind: &[u8], routing: &AirRouting) {
    push_field(buf, kind);
    push_field(buf, routing.name.as_bytes());
    match &routing.outcome {
        AirRoutingOutcome::End => push_field(buf, b"END"),
        AirRoutingOutcome::Retry => push_field(buf, b"RETRY"),
        AirRoutingOutcome::GotoStep(s) => {
            push_field(buf, b"GOTO_STEP");
            push_field(buf, s.as_bytes());
        }
        AirRoutingOutcome::GotoWorkflow(s) => {
            push_field(buf, b"GOTO_WORKFLOW");
            push_field(buf, s.as_bytes());
        }
    }
    for c in routing.criteria.iter() {
        push_expr(buf, c);
    }
}

fn push_field(buf: &mut Vec<u8>, field: &[u8]) {
    buf.extend_from_slice(&(field.len() as u32).to_le_bytes());
    buf.extend_from_slice(field);
}

impl AirCompiler {
    /// Computes the [`AirDigest`] of `program`'s canonical content.
    pub fn digest_program(program: &AirProgram) -> Result<AirDigest, Refusal> {
        Self::compile(program)?;
        Ok(AirDigest(
            *blake3::hash(&canonical_bytes(program)).as_bytes(),
        ))
    }

    /// Compiles an `AirProgram` into its target representation.
    ///
    /// Ensures that all invariants of the program hold before generating output.
    pub fn compile(program: &AirProgram) -> Result<(), Refusal> {
        if program.workflows.is_empty() {
            return Err(Refusal::InvalidWorkflow(
                "Program has no workflows".to_string(),
            ));
        }

        for wf in program.workflows.iter() {
            if wf.name.is_empty() {
                return Err(Refusal::InvalidWorkflow(
                    "Workflow name cannot be empty".to_string(),
                ));
            }
            if wf.steps.is_empty() {
                return Err(Refusal::InvalidWorkflow(format!(
                    "Workflow '{}' has no steps",
                    wf.name
                )));
            }

            // Fast path: autovectorizable check for any invalid steps without creating error strings
            let has_error = wf.steps.chunks(125_000).any(|chunk| {
                let mut err = false;
                let mut iter = chunk.chunks_exact(8);
                for c in &mut iter {
                    let l1 = c[0].name.len().min(c[0].target.url.len());
                    let l2 = c[1].name.len().min(c[1].target.url.len());
                    let l3 = c[2].name.len().min(c[2].target.url.len());
                    let l4 = c[3].name.len().min(c[3].target.url.len());
                    let l5 = c[4].name.len().min(c[4].target.url.len());
                    let l6 = c[5].name.len().min(c[5].target.url.len());
                    let l7 = c[6].name.len().min(c[6].target.url.len());
                    let l8 = c[7].name.len().min(c[7].target.url.len());

                    if l1.min(l2).min(l3).min(l4).min(l5).min(l6).min(l7).min(l8) == 0 {
                        err = true;
                        break;
                    }
                }
                if !err {
                    for step in iter.remainder() {
                        if step.name.is_empty() || step.target.url.is_empty() {
                            err = true;
                            break;
                        }
                    }
                }
                err
            });

            // Slow path: generate exactly the right error string if we found an error
            if has_error {
                for step in wf.steps.iter() {
                    if step.name.is_empty() {
                        return Err(Refusal::InvalidWorkflow(format!(
                            "Workflow '{}' has a step with no name",
                            wf.name
                        )));
                    }
                    if step.target.url.is_empty() {
                        return Err(Refusal::InvalidWorkflow(format!(
                            "Step '{}' has no target URL",
                            step.name
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Deterministically compiles an `AirProgram` into a placeholder WASM module.
    ///
    /// This crate has no WASM host-import layer or runtime dispatch yet: function bodies here
    /// are NOT a semantic execution of the workflow (no HTTP calls, no `target`/`action`
    /// evaluation at runtime). See `docs/jira/v26.7.11/PRD.md` sections 7.6-7.9 for the planned
    /// AIR execution architecture (Erlang transition core, OTP/AtomVM runners) — none of that
    /// exists in this repo today, and nothing currently instantiates or executes this module's
    /// output.
    ///
    /// Each exported function's body is exactly `wf.steps.len()` `nop` instructions: a
    /// placeholder whose only guaranteed properties are determinism and a size proportional to
    /// the workflow. To avoid losing workflow content in the compiled artifact, the module also
    /// carries two custom sections built from [`canonical_bytes`]:
    /// - `air-canonical-v1`: the canonical serialized bytes of every workflow/step/target/expr
    /// - `air-digest-v1`: the 32-byte BLAKE3 digest of those canonical bytes
    ///
    /// # Complexity
    /// O(total step count) time and space.
    pub fn compile_to_wasm(program: &AirProgram) -> Result<Vec<u8>, Refusal> {
        Self::compile(program)?;

        use wasm_encoder::{
            CodeSection, CustomSection, ExportSection, Function, FunctionSection, Instruction,
            Module, TypeSection,
        };

        let mut module = Module::new();
        let mut types = TypeSection::new();
        let mut functions = FunctionSection::new();
        let mut exports = ExportSection::new();
        let mut codes = CodeSection::new();

        for (i, wf) in program.workflows.iter().enumerate() {
            types.ty().function(vec![], vec![]);
            functions.function(i as u32);
            exports.export(&wf.name, wasm_encoder::ExportKind::Func, i as u32);

            let mut func = Function::new(vec![]);
            for _ in 0..wf.steps.len() {
                func.instruction(&Instruction::Nop);
            }
            func.instruction(&Instruction::End);
            codes.function(&func);
        }

        module.section(&types);
        module.section(&functions);
        module.section(&exports);
        module.section(&codes);

        let canonical = canonical_bytes(program);
        module.section(&CustomSection {
            name: std::borrow::Cow::Borrowed("air-canonical-v1"),
            data: std::borrow::Cow::Borrowed(&canonical),
        });
        let digest = blake3::hash(&canonical);
        module.section(&CustomSection {
            name: std::borrow::Cow::Borrowed("air-digest-v1"),
            data: std::borrow::Cow::Borrowed(digest.as_bytes()),
        });

        Ok(module.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::air::*;
    use bumpalo::collections::{String as BumpString, Vec as BumpVec};
    use bumpalo::vec;
    use bumpalo::Bump;

    fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
        !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
    }

    #[test]
    fn test_empty_program_refused() {
        let bump = Bump::new();
        let program = AirProgram {
            workflows: BumpVec::new_in(&bump),
        };
        assert_eq!(
            AirCompiler::compile(&program),
            Err(Refusal::InvalidWorkflow(
                "Program has no workflows".to_string()
            ))
        );
    }

    #[test]
    fn test_empty_workflow_name_refused() {
        let bump = Bump::new();
        let program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("", &bump),
                    steps: BumpVec::new_in(&bump),
                }
            ],
        };
        assert_eq!(
            AirCompiler::compile(&program),
            Err(Refusal::InvalidWorkflow(
                "Workflow name cannot be empty".to_string()
            ))
        );
    }

    #[test]
    fn test_empty_steps_refused() {
        let bump = Bump::new();
        let program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("test_wf", &bump),
                    steps: BumpVec::new_in(&bump),
                }
            ],
        };
        assert_eq!(
            AirCompiler::compile(&program),
            Err(Refusal::InvalidWorkflow(
                "Workflow 'test_wf' has no steps".to_string()
            ))
        );
    }

    #[test]
    fn test_empty_step_name_refused() {
        let bump = Bump::new();
        let program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("test_wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com", &bump),
                                method: BumpString::from_str_in("GET", &bump),
                            },
                            action: AirAction {
                                inputs: BumpVec::new_in(&bump),
                                outputs: BumpVec::new_in(&bump),
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        }
                    ],
                }
            ],
        };
        assert_eq!(
            AirCompiler::compile(&program),
            Err(Refusal::InvalidWorkflow(
                "Workflow 'test_wf' has a step with no name".to_string()
            ))
        );
    }

    #[test]
    fn test_empty_target_url_refused() {
        let bump = Bump::new();
        let program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("test_wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("", &bump),
                                method: BumpString::from_str_in("GET", &bump),
                            },
                            action: AirAction {
                                inputs: BumpVec::new_in(&bump),
                                outputs: BumpVec::new_in(&bump),
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        }
                    ],
                }
            ],
        };
        assert_eq!(
            AirCompiler::compile(&program),
            Err(Refusal::InvalidWorkflow(
                "Step 'step_1' has no target URL".to_string()
            ))
        );
    }

    #[test]
    fn test_valid_program() {
        let bump = Bump::new();
        let program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("test_wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com", &bump),
                                method: BumpString::from_str_in("GET", &bump),
                            },
                            action: AirAction {
                                inputs: BumpVec::new_in(&bump),
                                outputs: BumpVec::new_in(&bump),
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        }
                    ],
                }
            ],
        };
        assert!(AirCompiler::compile(&program).is_ok());
    }

    fn sample_program(bump: &Bump) -> AirProgram<'_> {
        AirProgram {
            workflows: vec![in bump;
                AirWorkflow {
                    name: BumpString::from_str_in("test_wf", bump),
                    steps: vec![in bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1", bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://distinctive-host.example/orders", bump),
                                method: BumpString::from_str_in("GET", bump),
                            },
                            action: AirAction {
                                inputs: vec![in bump; AirExpr::Literal(BumpString::from_str_in("a", bump))],
                                outputs: vec![in bump; AirExpr::Variable(BumpString::from_str_in("b", bump))],
                            },
                            on_success: BumpVec::new_in(bump),
                            on_failure: BumpVec::new_in(bump),
                        }
                    ],
                }
            ],
        }
    }

    #[test]
    fn test_digest_program_deterministic() {
        let bump = Bump::new();
        let program = sample_program(&bump);

        let digest1 = AirCompiler::digest_program(&program).expect("Failed to digest");
        let digest2 = AirCompiler::digest_program(&program).expect("Failed to digest");
        assert_eq!(digest1, digest2, "Digest must be purely deterministic");
    }

    #[test]
    fn test_compile_to_wasm_deterministic() {
        let bump = Bump::new();
        let program = sample_program(&bump);

        let wasm1 = AirCompiler::compile_to_wasm(&program).expect("Failed to compile");
        let wasm2 = AirCompiler::compile_to_wasm(&program).expect("Failed to compile");
        assert_eq!(
            wasm1, wasm2,
            "compile_to_wasm must be byte-identical across repeated calls on the same input"
        );
    }

    #[test]
    fn test_compile_to_wasm_preserves_step_content() {
        let bump = Bump::new();
        let program = sample_program(&bump);

        let wasm = AirCompiler::compile_to_wasm(&program).expect("Failed to compile");
        assert!(
            contains_subslice(&wasm, b"http://distinctive-host.example/orders"),
            "compiled module must not silently drop the step's target URL"
        );
        assert!(
            contains_subslice(&wasm, b"test_wf"),
            "compiled module must not silently drop the workflow name"
        );
    }

    #[test]
    fn test_compile_to_wasm_preserves_routing_content() {
        let bump = Bump::new();
        let program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("routed_wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/1", &bump),
                                method: BumpString::from_str_in("operationId", &bump),
                            },
                            action: AirAction {
                                inputs: BumpVec::new_in(&bump),
                                outputs: BumpVec::new_in(&bump),
                            },
                            on_success: vec![in &bump; AirRouting {
                                name: BumpString::from_str_in("distinctive-routing-name", &bump),
                                outcome: AirRoutingOutcome::GotoStep(BumpString::from_str_in("step_2", &bump)),
                                criteria: BumpVec::new_in(&bump),
                            }],
                            on_failure: BumpVec::new_in(&bump),
                        }
                    ],
                }
            ],
        };

        let wasm = AirCompiler::compile_to_wasm(&program).expect("Failed to compile");
        assert!(
            contains_subslice(&wasm, b"distinctive-routing-name"),
            "compiled module must not silently drop on_success routing content"
        );
        assert!(
            contains_subslice(&wasm, b"step_2"),
            "compiled module must not silently drop a goto-step routing target"
        );

        let mut without_routing = program.clone();
        without_routing.workflows[0].steps[0].on_success = BumpVec::new_in(&bump);
        let digest_with_routing = AirCompiler::digest_program(&program).expect("Failed to digest");
        let digest_without_routing =
            AirCompiler::digest_program(&without_routing).expect("Failed to digest");
        assert_ne!(
            digest_with_routing, digest_without_routing,
            "routing content must be part of the canonical digest, not silently ignored"
        );
    }
}
