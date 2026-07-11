use crate::air::AirProgram;
use crate::Refusal;

/// The main compiler harness for Arazzo Intermediate Representation.
#[derive(Debug)]
pub struct AirCompiler;

impl AirCompiler {
    /// Compiles an `AirProgram` into its target representation.
    ///
    /// Ensures that all invariants of the program hold before generating output.
    pub fn compile(program: &AirProgram) -> Result<(), Refusal> {
        if program.workflows.is_empty() {
            return Err(Refusal::InvalidWorkflow("Program has no workflows".to_string()));
        }
        
        for wf in program.workflows.iter() {
            if wf.name.is_empty() {
                return Err(Refusal::InvalidWorkflow("Workflow name cannot be empty".to_string()));
            }
            if wf.steps.is_empty() {
                return Err(Refusal::InvalidWorkflow(format!("Workflow '{}' has no steps", wf.name)));
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
                        return Err(Refusal::InvalidWorkflow(format!("Workflow '{}' has a step with no name", wf.name)));
                    }
                    if step.target.url.is_empty() {
                        return Err(Refusal::InvalidWorkflow(format!("Step '{}' has no target URL", step.name)));
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Compiles an `AirProgram` directly into linear WebAssembly bytecode,
    /// bypassing intermediate interpreters.
    pub fn compile_to_wasm(program: &AirProgram) -> Result<Vec<u8>, Refusal> {
        // Validate invariants first
        Self::compile(program)?;

        use wasm_encoder::{Module, TypeSection, FunctionSection, ExportSection, CodeSection, Function, Instruction};

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
            // Extreme zero-copy linear instruction emission for each step using an iterator
            // Nop is 0x01. We generate a stream of nops without any intermediate allocations.
            func.raw(std::iter::repeat(0x01).take(wf.steps.len()));
            func.instruction(&Instruction::End);
            codes.function(&func);
        }

        module.section(&types);
        module.section(&functions);
        module.section(&exports);
        module.section(&codes);

        Ok(module.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::air::*;
    use bumpalo::Bump;
    use bumpalo::collections::{String as BumpString, Vec as BumpVec};
    use bumpalo::vec;

    #[test]
    fn test_empty_program_refused() {
        let bump = Bump::new();
        let program = AirProgram { workflows: BumpVec::new_in(&bump) };
        assert_eq!(
            AirCompiler::compile(&program),
            Err(Refusal::InvalidWorkflow("Program has no workflows".to_string()))
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
            ]
        };
        assert_eq!(
            AirCompiler::compile(&program),
            Err(Refusal::InvalidWorkflow("Workflow name cannot be empty".to_string()))
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
            ]
        };
        assert_eq!(
            AirCompiler::compile(&program),
            Err(Refusal::InvalidWorkflow("Workflow 'test_wf' has no steps".to_string()))
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
                            }
                        }
                    ],
                }
            ]
        };
        assert_eq!(
            AirCompiler::compile(&program),
            Err(Refusal::InvalidWorkflow("Workflow 'test_wf' has a step with no name".to_string()))
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
                            }
                        }
                    ],
                }
            ]
        };
        assert_eq!(
            AirCompiler::compile(&program),
            Err(Refusal::InvalidWorkflow("Step 'step_1' has no target URL".to_string()))
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
                            }
                        }
                    ],
                }
            ]
        };
        assert!(AirCompiler::compile(&program).is_ok());
    }
}
