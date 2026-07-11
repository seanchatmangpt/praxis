# v26.7.11 Product Requirements Document: The Long-Duration Escalation Test

## 1. Objective
To empirically and cryptographically prove (or falsify) the claim that the `praxis` engine can act as an infinite-horizon steward capable of autonomous, unattended self-correction against a strict invariant without systemic drift.

## 2. Rationale
Prior speculative claims suggested that merely removing `--max-polls` and calling `validate_shacl` on every tick would yield an autonomous self-healing agent. This was a category error that conflated liveness with autonomous agency and ignored the massive risk of the sensor-to-graph boundary. 

To provide a mathematically rigorous, honestly scoped test of self-correction, we must isolate the engine from external sensors and rely exclusively on an already-built, deterministic refusal-driven repair path within the system.

## 3. The Concrete Experiment

### 3.1 Test Constraints
- **Unbounded Execution**: Drive `engine serve` thousands of polls past its standard testing ceiling.
- **Internal Fault Injection**: Continuously inject a narrow, deterministic, internally generated failure (e.g., a forced dispatch timeout matching the `deadline_expiry_times_out_and_manufactures_escalation` behavior).
- **No Sensor Polling**: The test must not touch any real external sensor, entirely sidestepping the hallucination/sensor-boundary risk.

### 3.2 Success Criteria per Iteration
After each injected failure, the engine must autonomously:
1. Detect the refusal (e.g., `CNG_R07 RunnerMismatch` or a dispatch timeout).
2. Trigger the pre-existing escalation/compensation workflow.
3. Seal a mathematically valid `EngineProcessReceipt`.
4. Return to a lawful, closed standing state.

### 3.3 The Falsifiable Oracle
The pass/fail oracle for the experiment is the existing ledger chain-verification machinery. 
- **PASS**: If, after $N$ thousands of iterations, the ledger chain stays cryptographically verifiable and standing closes properly, the claim is empirically proven.
- **FAIL**: If the engine deadlocks, the chain stops replaying cleanly, or standing fails to close, the negative result is proven and the failure is isolated for repair.

## 4. Conclusion
This PRD scopes a real, falsifiable answer to the self-correction capability of the `praxis` architecture. It relies purely on the mathematical physics already built into the engine, avoiding extrapolation and maintaining absolute AGI-level Rust core-team discipline.
