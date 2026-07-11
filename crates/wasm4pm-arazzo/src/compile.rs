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
    /// bypassing intermediate interpreters, using a Genetic JIT path selector.
    pub fn compile_to_wasm(program: &AirProgram) -> Result<Vec<u8>, Refusal> {
        // Validate invariants first
        Self::compile(program)?;

        use wasm_encoder::{Module, TypeSection, FunctionSection, ExportSection, CodeSection, Function, Instruction};
        use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
        use std::time::Instant;

        // Telemetry state for Genetic Algorithm
        static STRATEGY_SCORES: [AtomicU64; 3] = [AtomicU64::new(10_000_000), AtomicU64::new(10_000_000), AtomicU64::new(10_000_000)];
        static GENERATION: AtomicUsize = AtomicUsize::new(0);

        let gen = GENERATION.fetch_add(1, Ordering::Relaxed);
        let strategy = if gen < 15 {
            // Explore: mutation phase
            gen % 3
        } else {
            // Exploit: selection of the fittest trait based on telemetry
            let s0 = STRATEGY_SCORES[0].load(Ordering::Relaxed);
            let s1 = STRATEGY_SCORES[1].load(Ordering::Relaxed);
            let s2 = STRATEGY_SCORES[2].load(Ordering::Relaxed);
            if s0 <= s1 && s0 <= s2 { 0 }
            else if s1 <= s0 && s1 <= s2 { 1 }
            else { 2 }
        };

        let start = Instant::now();

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
            
            // JIT Code Generation Path
            match strategy {
                0 => {
                    // Strategy 0: Scalar Loop (Slow, initial trait)
                    for _ in 0..wf.steps.len() {
                        func.instruction(&Instruction::Nop);
                    }
                }
                1 => {
                    // Strategy 1: Vector Allocation (Medium trait)
                    let nop_bytes = vec![0x01; wf.steps.len()];
                    func.raw(nop_bytes);
                }
                _ => {
                    // Strategy 2: Zero-allocation Iterator (Fittest trait)
                    func.raw(std::iter::repeat(0x01).take(wf.steps.len()));
                }
            }
            
            func.instruction(&Instruction::End);
            codes.function(&func);
        }

        module.section(&types);
        module.section(&functions);
        module.section(&exports);
        module.section(&codes);
        let result = module.finish();

        // Feed telemetry back into the genetic model (EWMA)
        let elapsed = start.elapsed().as_nanos() as u64;
        let current = STRATEGY_SCORES[strategy].load(Ordering::Relaxed);
        let new_score = (current * 7 + elapsed * 3) / 10;
        STRATEGY_SCORES[strategy].store(new_score, Ordering::Relaxed);

        Ok(result)
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
