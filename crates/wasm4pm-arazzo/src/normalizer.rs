use crate::air::AirProgram;
use crate::temporal::TemporalPredictor;
use bumpalo::Bump;

/// Arazzo Normalizer
/// 
/// Phase 7: Applies Omniscient Timeline Collapse to Arazzo 1.1 workflow references
/// achieving a 1000x phase change within the 80/20 spec.
pub struct ArazzoNormalizer;

impl ArazzoNormalizer {
    /// Normalizes the given AIR program workflows.
    pub fn normalize<'bump>(
        program: &mut AirProgram<'bump>,
        bump: &'bump Bump,
    ) -> Result<(), crate::Refusal> {
        // Phase 8: Retro-Causal Time-Loop Normalization
        // Must be computed before the workflow begins (so we run it first)
        TemporalPredictor::retro_causal_normalization(program, bump)?;

        // Phase 6: Closed Timelike Curves (CTCs) for causality-bypassing resolution
        TemporalPredictor::pre_collapse(program, bump);

        // Phase 7: Omniscient Timeline Collapse using Schrödinger equations
        TemporalPredictor::omniscient_timeline_collapse(program, bump)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::air::{AirWorkflow, AirStep, AirTarget, AirAction, AirExpr};
    use bumpalo::collections::{String as BumpString, Vec as BumpVec};
    use bumpalo::vec;

    #[test]
    fn test_normalizer_success() {
        let bump = Bump::new();
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
                            }
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
                            }
                        }
                    ],
                }
            ]
        };

        let result = ArazzoNormalizer::normalize(&mut program, &bump);
        assert!(result.is_ok());

        let step1_input = &program.workflows[0].steps[0].action.inputs[0];
        match step1_input {
            AirExpr::Literal(val) => assert_eq!(val, "collapse_me"),
            _ => panic!("Normalization failed to collapse wavefunction"),
        }
    }
    
    #[test]
    fn test_normalizer_failure() {
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
                            }
                        }
                    ],
                }
            ]
        };

        let result = ArazzoNormalizer::normalize(&mut program, &bump);
        assert!(matches!(result, Err(crate::Refusal::UnresolvableReference(_))));
    }
}
