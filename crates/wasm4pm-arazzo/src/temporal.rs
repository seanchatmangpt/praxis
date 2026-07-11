use crate::air::{AirProgram, AirExpr};
use bumpalo::Bump;
use std::collections::HashMap;

/// Predictive Temporal Parsing Engine
///
/// Implements "retrocausal" pre-collapsing of future Arazzo reference states.
/// By statically analyzing the workflow graph, it propagates known future literal
/// states backwards in the compilation timeline, substituting dynamic variables
/// with their inevitable deterministic constants before execution.
pub struct TemporalPredictor;

impl TemporalPredictor {
    /// Pre-collapses references in the AIR program.
    pub fn pre_collapse<'bump>(program: &mut AirProgram<'bump>, _bump: &'bump Bump) {
        for wf in program.workflows.iter_mut() {
            // A theoretical "tachyonic" state map holding future inevitabilities
            let mut retro_state: HashMap<&str, &str> = HashMap::new();

            for step in wf.steps.iter_mut() {
                // Pre-collapse inputs based on retro-state
                for input in step.action.inputs.iter_mut() {
                    if let AirExpr::Variable(var_name) = input {
                        if let Some(collapsed_val) = retro_state.get(var_name.as_str()) {
                            // The future has collapsed; the variable is now a literal
                            // We use unsafe to cast the string slice to a bump string to avoid reallocation
                            // since the string already lives in the same bump arena.
                            // However, to strictly avoid unsafe, we can just use the bump allocator.
                            *input = AirExpr::Literal(bumpalo::collections::String::from_str_in(collapsed_val, _bump));
                        }
                    }
                }

                // Register outputs into the retro-state for future steps
                // In Arazzo, step outputs often map to variables used in subsequent steps.
                for output in step.action.outputs.iter() {
                    if let AirExpr::Literal(val) = output {
                        // For the sake of this theoretical engine, we assume the output literal
                        // also defines a variable binding of the same name for downstream steps.
                        retro_state.insert(val.as_str(), val.as_str());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::air::{AirWorkflow, AirStep, AirTarget, AirAction};
    use bumpalo::collections::{String as BumpString, Vec as BumpVec};
    use bumpalo::vec;

    #[test]
    fn test_temporal_retrocausality() {
        let bump = Bump::new();
        let mut program = AirProgram {
            workflows: vec![in &bump;
                AirWorkflow {
                    name: BumpString::from_str_in("retro_wf", &bump),
                    steps: vec![in &bump;
                        AirStep {
                            name: BumpString::from_str_in("step_1", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                inputs: BumpVec::new_in(&bump),
                                // Step 1 emits literal "order_id"
                                outputs: vec![in &bump; AirExpr::Literal(BumpString::from_str_in("order_id", &bump))],
                            }
                        },
                        AirStep {
                            name: BumpString::from_str_in("step_2", &bump),
                            target: AirTarget {
                                url: BumpString::from_str_in("http://example.com/2", &bump),
                                method: BumpString::from_str_in("POST", &bump),
                            },
                            action: AirAction {
                                // Step 2 takes variable "order_id"
                                inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("order_id", &bump))],
                                outputs: BumpVec::new_in(&bump),
                            }
                        }
                    ],
                }
            ]
        };

        // Pre-collapse
        TemporalPredictor::pre_collapse(&mut program, &bump);

        // Verify Step 2 input collapsed from Variable to Literal
        let step2_input = &program.workflows[0].steps[1].action.inputs[0];
        match step2_input {
            AirExpr::Literal(val) => assert_eq!(val, "order_id"),
            _ => panic!("Temporal predictor failed to pre-collapse the future state!"),
        }
    }
}
