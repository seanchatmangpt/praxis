# Ticket: Implement Impossibility Sensors for Runtime State Invariants

## Title
Implement Impossibility Sensors for Runtime State Invariants (PROJ-101)

## Description
In accordance with Sean Chatman's thesis on **operational physics** and **impossibility harvesting**, this ticket introduces runtime observability hooks, termed "impossibility sensors," into the Praxis testbed environment. 

Rather than allowing the system to experience a catastrophic crash, deadlock, or unhandled exception when architectural or physical invariants are violated, these sensors will actively intercept anomalous states. When an anomaly is detected (e.g., resource deadlocks, state cycles, or anomalous FFI timing drift), the sensor must capture the current in-memory state of the affected components, serialize this context into an Object-Centric Event Log (OCEL 2.0) diagnostic log format, and write the output to the local workspace for analysis and subsequent autonomic resolution.

Specifically, the sensors must:
1. Intercept state cycle dependencies (e.g., cyclic support structures, circular locks, or cyclic token replays).
2. Gather detailed telemetry of the violating entities, including timestamps, thread/component identifiers, and the specific invariant violated.
3. Formulate and serialize the diagnostic trace as an OCEL 2.0 JSON event log.
4. Output the serialized trace to `target/praxis/evidence/impossibility_violations.ocel.json`.

## Acceptance Criteria
- **Sensor Hooks Implementation**: Runtime interceptors must be implemented in the testbed system that can monitor active state graphs.
- **Cycle Detection**: The sensor must be capable of identifying cyclic relationships among state objects (e.g., where object A depends on B, B depends on C, and C depends on A), matching the `ax-support` validation logic.
- **OCEL 2.0 Conformance**: When an invariant violation is intercepted, the system must generate a valid OCEL 2.0 JSON file containing:
  - An event stream with event types: `violation_detected`, `state_captured`, and `sensor_shutdown`.
  - Object types representing the entities involved in the cyclic state or deadlock.
  - Attributes detailing the violation type, timestamps, and diagnostic identifiers.
- **Graceful Degradation**: The sensor must gracefully log the violation and transition the monitored subsystem into a safe, quarantined, or idle state rather than panicking or hanging indefinitely.
- **Target File Path**: The output must be written to `target/praxis/evidence/impossibility_violations.ocel.json` without blocking concurrent writes.

## Dependencies
None.

## Verification Mechanism
Verify the sensor implementation through the following objective tests:
1. Run the dedicated test suite targeting the cycle detection sensors:
   ```bash
   cargo test --test sensor_tests
   ```
2. Inspect the test code `tests/sensor_tests.rs` to ensure it simulates a circular state reference (A -> B -> C -> A), triggers the impossibility sensor, and asserts that:
   - No panic or crash occurs.
   - The file `target/praxis/evidence/impossibility_violations.ocel.json` is created.
   - The generated JSON conforms to the OCEL 2.0 format and contains the events `violation_detected`, `state_captured`, and `sensor_shutdown`.
3. Check the contents of the generated log using a JSON parser to verify the structure:
   ```bash
   cat target/praxis/evidence/impossibility_violations.ocel.json | jq .
   ```
