use wasm4pm_arazzo::air::{AirProgram, AirWorkflow, AirStep, AirTarget, AirAction};
use wasm4pm_arazzo::compile::AirCompiler;
use std::time::Instant;

#[test]
fn bench_million_steps() {
    let mut steps = Vec::with_capacity(1_000_000);
    for i in 0..1_000_000 {
        steps.push(AirStep {
            name: format!("step_{}", i),
            target: AirTarget {
                url: "http://example.com".to_string(),
                method: "GET".to_string(),
            },
            action: AirAction {
                inputs: vec![],
                outputs: vec![],
            }
        });
    }

    let program = AirProgram {
        workflows: vec![
            AirWorkflow {
                name: "huge_wf".to_string(),
                steps,
            }
        ]
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
