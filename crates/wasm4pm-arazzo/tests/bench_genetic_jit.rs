use wasm4pm_arazzo::air::{AirProgram, AirWorkflow, AirStep, AirTarget, AirAction};
use wasm4pm_arazzo::compile::AirCompiler;
use bumpalo::Bump;
use bumpalo::collections::{String as BumpString, Vec as BumpVec};
use std::fmt::Write;
use std::time::Instant;

#[test]
fn bench_genetic_jit() {
    let bump = Bump::new();
    let mut steps = BumpVec::with_capacity_in(10_000, &bump);
    for i in 0..10_000 {
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

    println!("Starting Genetic JIT Execution...");

    // Generation 1-25 (Exploration phase - slow!)
    let start_explore = Instant::now();
    for _ in 0..25 {
        let _ = AirCompiler::compile_to_wasm(&program).unwrap();
    }
    let duration_explore = start_explore.elapsed();
    let avg_explore = duration_explore / 25;
    println!("Exploration Phase (Generations 1-25) average time: {:?}", avg_explore);

    // Generation 26-100 (Exploitation phase - fast! should converge on Strategy 4)
    let start_exploit = Instant::now();
    for _ in 0..75 {
        let _ = AirCompiler::compile_to_wasm(&program).unwrap();
    }
    let duration_exploit = start_exploit.elapsed();
    let avg_exploit = duration_exploit / 75;
    println!("Exploitation Phase (Generations 26-100) average time: {:?}", avg_exploit);

    assert!(avg_exploit < avg_explore, "Genetic evolution should select a strategy faster than the exploration average");
}
