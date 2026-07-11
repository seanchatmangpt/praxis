use crate::air::{AirExpr, AirProgram};
use crate::Refusal;
use bumpalo::Bump;
use std::collections::HashMap;

/// Resolves `Variable` step-input references within a workflow.
pub struct ReferenceResolver;

impl ReferenceResolver {
    /// Resolves each step's `Variable` inputs against the literal outputs declared by *earlier*
    /// steps in the same workflow, in declaration order.
    ///
    /// A step may only reference an output declared by a step before it: forward references (a
    /// step referencing a later step's output), self-references, and unknown references are all
    /// refused with `Refusal::UnresolvableReference`. This matches the constraint that a step
    /// can only observe the outputs of steps that have already run.
    ///
    /// # Complexity
    /// O(total step count) time and space, single forward pass — no fixed-point iteration is
    /// needed because a single order-respecting scan already resolves every valid multi-hop
    /// dependency chain by construction (each step's own outputs become visible to later steps
    /// as soon as it is processed).
    pub fn resolve<'bump>(
        program: &mut AirProgram<'bump>,
        bump: &'bump Bump,
    ) -> Result<(), Refusal> {
        for wf in program.workflows.iter_mut() {
            let mut resolved: HashMap<String, String> = HashMap::new();

            for step in wf.steps.iter_mut() {
                for input in step.action.inputs.iter_mut() {
                    if let AirExpr::Variable(var_name) = input {
                        match resolved.get(var_name.as_str()) {
                            Some(value) => {
                                *input = AirExpr::Literal(
                                    bumpalo::collections::String::from_str_in(value, bump),
                                );
                            }
                            None => {
                                return Err(Refusal::UnresolvableReference(format!(
                                    "Step '{}' in workflow '{}' references '{}', which is not an output of any earlier step",
                                    step.name, wf.name, var_name
                                )));
                            }
                        }
                    }
                }

                for output in step.action.outputs.iter() {
                    if let AirExpr::Literal(val) = output {
                        resolved.insert(val.as_str().to_string(), val.as_str().to_string());
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
    use crate::air::{AirAction, AirStep, AirTarget, AirWorkflow};
    use bumpalo::collections::{String as BumpString, Vec as BumpVec};
    use bumpalo::vec;

    #[test]
    fn test_forward_reference_resolves() {
        let bump = Bump::new();
        let mut program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1_producer", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/1", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: BumpVec::new_in(&bump),
                                outputs: vec![in &bump; AirExpr::Literal(BumpString::from_str_in("order_id", &bump))],
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        },
                        AirStep {
                            name: BumpString::from_str_in("step_2_consumer", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/2", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("order_id", &bump))],
                                outputs: BumpVec::new_in(&bump),
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        }
                    ],
                }
            ],
        };

        ReferenceResolver::resolve(&mut program, &bump).expect("should resolve");

        let step2_input = &program.workflows[0].steps[1].action.inputs[0];
        match step2_input {
            AirExpr::Literal(val) => assert_eq!(val, "order_id"),
            _ => panic!("Reference to an earlier step's output should have resolved"),
        }
    }

    #[test]
    fn test_backward_reference_refused() {
        let bump = Bump::new();
        // step_1 references an output that only step_2 (declared after it) produces.
        let mut program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/1", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("later_artifact", &bump))],
                                outputs: BumpVec::new_in(&bump),
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        },
                        AirStep {
                            name: BumpString::from_str_in("step_2", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/2", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: BumpVec::new_in(&bump),
                                outputs: vec![in &bump; AirExpr::Literal(BumpString::from_str_in("later_artifact", &bump))],
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        }
                    ],
                }
            ],
        };

        let result = ReferenceResolver::resolve(&mut program, &bump);
        assert!(
            matches!(result, Err(Refusal::UnresolvableReference(_))),
            "a step must not be able to resolve a reference to a later step's output"
        );
    }

    #[test]
    fn test_unknown_reference_refused() {
        let bump = Bump::new();
        let mut program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/1", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("missing_var", &bump))],
                                outputs: BumpVec::new_in(&bump),
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        }
                    ],
                }
            ],
        };

        let result = ReferenceResolver::resolve(&mut program, &bump);
        match result {
            Err(Refusal::UnresolvableReference(msg)) => {
                assert_eq!(
                    msg,
                    "Step 'step_1' in workflow 'wf' references 'missing_var', which is not an output of any earlier step"
                );
            }
            _ => panic!("Expected UnresolvableReference Refusal for an unknown variable"),
        }
    }

    #[test]
    fn test_multi_hop_chain_resolves_in_a_single_pass() {
        let bump = Bump::new();
        // step_1 produces x; step_2 consumes x and produces y; step_3 consumes y.
        let mut program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/1", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: BumpVec::new_in(&bump),
                                outputs: vec![in &bump; AirExpr::Literal(BumpString::from_str_in("x", &bump))],
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        },
                        AirStep {
                            name: BumpString::from_str_in("step_2", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/2", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("x", &bump))],
                                outputs: vec![in &bump; AirExpr::Literal(BumpString::from_str_in("y", &bump))],
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        },
                        AirStep {
                            name: BumpString::from_str_in("step_3", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/3", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("y", &bump))],
                                outputs: BumpVec::new_in(&bump),
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        }
                    ],
                }
            ],
        };

        ReferenceResolver::resolve(&mut program, &bump).expect("should resolve the whole chain");

        match &program.workflows[0].steps[1].action.inputs[0] {
            AirExpr::Literal(val) => assert_eq!(val, "x"),
            _ => panic!("step_2 should have resolved x"),
        }
        match &program.workflows[0].steps[2].action.inputs[0] {
            AirExpr::Literal(val) => assert_eq!(val, "y"),
            _ => panic!("step_3 should have resolved y"),
        }
    }
}
