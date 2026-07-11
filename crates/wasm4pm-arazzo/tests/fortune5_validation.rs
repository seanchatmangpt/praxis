use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::vec;
use bumpalo::Bump;
use wasm4pm_arazzo::air::{AirAction, AirExpr, AirProgram, AirStep, AirTarget, AirWorkflow};
use wasm4pm_arazzo::compile::AirCompiler;

#[test]
fn test_fortune5_workflow_compilation() {
    let bump = Bump::new();
    let program = AirProgram {
        workflows: vec![in &bump;
            AirWorkflow {
                name: BumpString::from_str_in("fortune5_global_supply_chain", &bump),
                steps: vec![in &bump;
                    AirStep {
                        name: BumpString::from_str_in("validate_order", &bump),
                        target: AirTarget {
                            url: BumpString::from_str_in("https://api.fortune5.com/orders/v1/validate", &bump),
                            method: BumpString::from_str_in("POST", &bump),
                        },
                        action: AirAction {
                            inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("order_payload", &bump))],
                            outputs: vec![in &bump; AirExpr::Literal(BumpString::from_str_in("order_valid", &bump))],
                        },
                        on_success: BumpVec::new_in(&bump),
                        on_failure: BumpVec::new_in(&bump),
                    },
                    AirStep {
                        name: BumpString::from_str_in("check_inventory", &bump),
                        target: AirTarget {
                            url: BumpString::from_str_in("https://api.fortune5.com/inventory/v1/reserve", &bump),
                            method: BumpString::from_str_in("POST", &bump),
                        },
                        action: AirAction {
                            inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("order_id", &bump))],
                            outputs: vec![in &bump; AirExpr::Literal(BumpString::from_str_in("inventory_reserved", &bump))],
                        },
                        on_success: BumpVec::new_in(&bump),
                        on_failure: BumpVec::new_in(&bump),
                    },
                    AirStep {
                        name: BumpString::from_str_in("dispatch_freight", &bump),
                        target: AirTarget {
                            url: BumpString::from_str_in("https://api.fortune5.com/logistics/v1/dispatch", &bump),
                            method: BumpString::from_str_in("POST", &bump),
                        },
                        action: AirAction {
                            inputs: vec![in &bump;
                                AirExpr::Variable(BumpString::from_str_in("inventory_id", &bump)),
                                AirExpr::Variable(BumpString::from_str_in("destination", &bump)),
                            ],
                            outputs: vec![in &bump; AirExpr::Literal(BumpString::from_str_in("freight_id", &bump))],
                        },
                        on_success: BumpVec::new_in(&bump),
                        on_failure: BumpVec::new_in(&bump),
                    },
                    AirStep {
                        name: BumpString::from_str_in("finalize_invoice", &bump),
                        target: AirTarget {
                            url: BumpString::from_str_in("https://api.fortune5.com/finance/v1/invoice", &bump),
                            method: BumpString::from_str_in("POST", &bump),
                        },
                        action: AirAction {
                            inputs: vec![in &bump; AirExpr::Variable(BumpString::from_str_in("freight_id", &bump))],
                            outputs: vec![in &bump; AirExpr::Literal(BumpString::from_str_in("invoice_status", &bump))],
                        },
                        on_success: BumpVec::new_in(&bump),
                        on_failure: BumpVec::new_in(&bump),
                    },
                ],
            }
        ],
    };

    let result = AirCompiler::compile(&program);
    assert!(
        result.is_ok(),
        "Fortune 5 workflow compilation failed: {:?}",
        result.unwrap_err()
    );
}
