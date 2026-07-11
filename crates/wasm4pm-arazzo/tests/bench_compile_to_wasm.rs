use bumpalo::collections::{String as BumpString, Vec as BumpVec};
use bumpalo::Bump;
use std::fmt::Write;
use std::time::Instant;
use wasm4pm_arazzo::air::{AirAction, AirProgram, AirStep, AirTarget, AirWorkflow};
use wasm4pm_arazzo::compile::AirCompiler;

fn huge_program(bump: &Bump, step_count: usize) -> AirProgram<'_> {
    let mut steps = BumpVec::with_capacity_in(step_count, bump);
    for i in 0..step_count {
        let mut name = BumpString::with_capacity_in(32, bump);
        write!(&mut name, "step_{}", i).unwrap();

        steps.push(AirStep {
            name,
            target: AirTarget {
                url: BumpString::from_str_in("http://example.com", bump),
                method: BumpString::from_str_in("GET", bump),
            },
            action: AirAction {
                inputs: BumpVec::new_in(bump),
                outputs: BumpVec::new_in(bump),
            },
            on_success: BumpVec::new_in(bump),
            on_failure: BumpVec::new_in(bump),
        });
    }

    AirProgram {
        workflows: bumpalo::vec![in bump;
            AirWorkflow {
                name: BumpString::from_str_in("huge_wf", bump),
                steps,
            }
        ],
    }
}

/// `compile_to_wasm` has no adaptive/history-dependent codegen path (that was removed because
/// it made output depend on global call-count state, violating this repo's determinism
/// invariant — see crates/wasm4pm-arazzo/src/compile.rs). This benchmark instead checks the
/// two properties that actually matter for a large workflow: it compiles at all, and repeated
/// compilation of the same program is byte-identical.
#[test]
fn bench_compile_to_wasm_10k_steps_is_deterministic() {
    let bump = Bump::new();
    let program = huge_program(&bump, 10_000);

    let start = Instant::now();
    let wasm1 = AirCompiler::compile_to_wasm(&program).unwrap();
    let duration = start.elapsed();
    println!("Compiling 10k steps to WASM took: {:?}", duration);
    println!("WASM bytecode size: {} bytes", wasm1.len());

    let wasm2 = AirCompiler::compile_to_wasm(&program).unwrap();
    assert_eq!(
        wasm1, wasm2,
        "compile_to_wasm must be deterministic across repeated calls"
    );
}
