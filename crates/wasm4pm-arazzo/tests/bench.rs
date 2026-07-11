use bumpalo::collections::{String as BumpString, Vec as BumpVec};
use bumpalo::vec;
use bumpalo::Bump;
use std::fmt::Write;
use std::time::Instant;
use wasm4pm_arazzo::air::{AirAction, AirProgram, AirStep, AirTarget, AirWorkflow};
use wasm4pm_arazzo::compile::AirCompiler;

#[test]
fn bench_million_steps() {
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
            },
            on_success: BumpVec::new_in(&bump),
            on_failure: BumpVec::new_in(&bump),
        });
    }

    let program = AirProgram {
        workflows: vec![in &bump;
            AirWorkflow {
                name: BumpString::from_str_in("huge_wf", &bump),
                steps,
            }
        ],
    };

    // Warmup
    let _ = AirCompiler::compile(&program);

    let start = Instant::now();
    let res = AirCompiler::compile(&program);
    let duration = start.elapsed();
    assert!(res.is_ok());
    println!("Compilation of 1M steps took: {:?}", duration);
    // write result to a file so we can see it
    std::fs::write("bench_result.txt", format!("{:?}", duration)).unwrap();
}
