use crate::air::AirProgram;
use crate::temporal::ReferenceResolver;
use bumpalo::Bump;

/// Arazzo Normalizer
///
/// Resolves `Variable` step-input references against earlier steps' literal outputs within
/// each workflow. See `ReferenceResolver::resolve` for the resolution rule and its complexity.
pub struct ArazzoNormalizer;

impl ArazzoNormalizer {
    /// Normalizes the given AIR program's workflows in place.
    pub fn normalize<'bump>(
        program: &mut AirProgram<'bump>,
        bump: &'bump Bump,
    ) -> Result<(), crate::Refusal> {
        ReferenceResolver::resolve(program, bump)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::air::{AirAction, AirExpr, AirStep, AirTarget, AirWorkflow};
    use bumpalo::collections::{String as BumpString, Vec as BumpVec};
    use bumpalo::vec;

    #[test]
    fn test_normalizer_resolves_earlier_step_output() {
        let bump = Bump::new();
        let mut program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("norm_wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1_producer", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/1", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: BumpVec::new_in(&bump),
                                outputs: vec![in &bump; AirExpr::Literal(BumpString::from_str_in("collapse_me", &bump))],
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
                                inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("collapse_me", &bump))],
                                outputs: BumpVec::new_in(&bump),
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        }
                    ],
                }
            ],
        };

        let result = ArazzoNormalizer::normalize(&mut program, &bump);
        assert!(result.is_ok());

        let step2_input = &program.workflows[0].steps[1].action.inputs[0];
        match step2_input {
            AirExpr::Literal(val) => assert_eq!(val, "collapse_me"),
            _ => panic!("Normalization failed to resolve a reference to an earlier step's output"),
        }
    }

    #[test]
    fn test_normalizer_refuses_forward_reference() {
        let bump = Bump::new();
        // step_1 references an output that only step_2 (declared after it) produces.
        let mut program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("norm_wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/1", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("collapse_me", &bump))],
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
                                outputs: vec![in &bump; AirExpr::Literal(BumpString::from_str_in("collapse_me", &bump))],
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        }
                    ],
                }
            ],
        };

        let result = ArazzoNormalizer::normalize(&mut program, &bump);
        assert!(matches!(
            result,
            Err(crate::Refusal::UnresolvableReference(_))
        ));
    }

    #[test]
    fn test_normalizer_refuses_unknown_reference() {
        let bump = Bump::new();
        let mut program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("norm_fail_wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/1", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("missing", &bump))],
                                outputs: BumpVec::new_in(&bump),
                            },
                            on_success: BumpVec::new_in(&bump),
                            on_failure: BumpVec::new_in(&bump),
                        }
                    ],
                }
            ],
        };

        let result = ArazzoNormalizer::normalize(&mut program, &bump);
        assert!(matches!(
            result,
            Err(crate::Refusal::UnresolvableReference(_))
        ));
    }
}
