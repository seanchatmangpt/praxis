use wasm4pm_arazzo::air::{AirProgram, AirWorkflow, AirStep, AirTarget, AirAction};
use wasm4pm_arazzo::compile::AirCompiler;
use bumpalo::Bump;
use bumpalo::collections::{String as BumpString, Vec as BumpVec};
use std::time::Instant;
use std::fmt::Write;

#[test]
fn bench_million_steps_wasm() {
    let bump = Bump::new();
    let mut steps = BumpVec::with_capacity_in(1_000_000, &bump);
    for i in 0..1_000_000 {
        let mut name = BumpString::with_capacity_in(32, &bump);
        write!(&mut name, "step_{}", i).unwrap();
        
        steps.push(AirStep {
            name,
            target: AirTarget {
                url: BumpString::from_str_in("http://example.com", &bump),
                method: BumpString::from_str_in("GET", &bump),
            },
            action: AirAction {
                inputs: BumpVec::new_in(&bump),
                outputs: BumpVec::new_in(&bump),
            }
        });
    }

    let program = AirProgram {
        workflows: bumpalo::vec![in &bump;
            AirWorkflow {
                name: BumpString::from_str_in("huge_wf", &bump),
                steps,
            }
        ]
    };

    // Warmup validation
    let _ = AirCompiler::compile(&program);

    let start = Instant::now();
    let wasm_bytes = AirCompiler::compile_to_wasm(&program).expect("WASM generation failed");
    let duration = start.elapsed();
    
    println!("Emitting 1M steps to WASM took: {:?}", duration);
    println!("WASM bytecode size: {} bytes", wasm_bytes.len());

    assert!(!wasm_bytes.is_empty(), "WASM module should not be empty");
}
