use crate::air::AirProgram;
use crate::Refusal;
use rayon::prelude::*;

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
        
        for wf in &program.workflows {
            if wf.name.is_empty() {
                return Err(Refusal::InvalidWorkflow("Workflow name cannot be empty".to_string()));
            }
            if wf.steps.is_empty() {
                return Err(Refusal::InvalidWorkflow(format!("Workflow '{}' has no steps", wf.name)));
            }
            
            // Fast path: parallel autovectorizable check for any invalid steps without creating error strings
            let has_error = wf.steps.par_chunks(125_000).any(|chunk| {
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
                for step in &wf.steps {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::air::*;

    #[test]
    fn test_empty_program_refused() {
        let program = AirProgram { workflows: vec![] };
        assert_eq!(
            AirCompiler::compile(&program),
            Err(Refusal::InvalidWorkflow("Program has no workflows".to_string()))
        );
    }
    
    #[test]
    fn test_empty_workflow_name_refused() {
        let program = AirProgram {
            workflows: vec![
                AirWorkflow {
                    name: "".to_string(),
                    steps: vec![],
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
        let program = AirProgram {
            workflows: vec![
                AirWorkflow {
                    name: "test_wf".to_string(),
                    steps: vec![],
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
        let program = AirProgram {
            workflows: vec![
                AirWorkflow {
                    name: "test_wf".to_string(),
                    steps: vec![
                        AirStep {
                            name: "".to_string(),
                            target: AirTarget {
                                url: "http://example.com".to_string(),
                                method: "GET".to_string(),
                            },
                            action: AirAction {
                                inputs: vec![],
                                outputs: vec![],
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
        let program = AirProgram {
            workflows: vec![
                AirWorkflow {
                    name: "test_wf".to_string(),
                    steps: vec![
                        AirStep {
                            name: "step_1".to_string(),
                            target: AirTarget {
                                url: "".to_string(),
                                method: "GET".to_string(),
                            },
                            action: AirAction {
                                inputs: vec![],
                                outputs: vec![],
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
        let program = AirProgram {
            workflows: vec![
                AirWorkflow {
                    name: "test_wf".to_string(),
                    steps: vec![
                        AirStep {
                            name: "step_1".to_string(),
                            target: AirTarget {
                                url: "http://example.com".to_string(),
                                method: "GET".to_string(),
                            },
                            action: AirAction {
                                inputs: vec![],
                                outputs: vec![],
                            }
                        }
                    ],
                }
            ]
        };
        assert!(AirCompiler::compile(&program).is_ok());
    }
}
