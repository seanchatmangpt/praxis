# Ticket: Implement Ocel Event Logging and Build Trace Generation

## Title
Implement Ocel Event Logging and Build Trace Generation

## Description
To enhance observability and track provenance across the compile and test cycles, we must implement Object-Centric Event Log (OCEL 2.0) tracking. Currently, compile metrics are not captured, and test diagnostic events are discarded without structured log aggregation.

This ticket requires:
1. Adding a `build.rs` compile-time hook to `praxis-retrofit` that captures build compilation events (activity: "SubstrateLabor", object: "praxis-retrofit" of type "Artifact") and writes them to a JSONL log file under the cargo build output directory.
2. Extending the `cargo-cicd` trace profile execution command (`cargo cicd trace profile`) to support a `build` profile mapping, which executes `cargo build` and records the trace execution.
3. Integrating the `OcelCollector` diagnostic sink from `chicago-tdd-tools` into the praxis test suites to output test diagnostics, assertions, start/completion events, and checksum receipts to `target/praxis/evidence/events.ocel.json`.
4. Standardizing output directory configuration to ensure build compile logs and test traces consolidate into a globally verifiable log directory.

## Acceptance Criteria
- **Build Hook Implementation**: Create `/Users/sac/praxis/crates/praxis-retrofit/build.rs` with the compile hook logic:
  ```rust
  fn main() {
      println!("cargo:rerun-if-changed=Cargo.toml");
      println!("cargo:rerun-if-changed=src/");

      let out_dir = std::env::var("OUT_DIR").unwrap();
      let now = std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap()
          .as_nanos();
      
      // Log build compilation event (OCEL 2.0 format)
      let build_log = format!(
          "{{\"event_id\":\"compile-{}\",\"activity\":\"SubstrateLabor\",\"timestamp\":\"{}\",\"objects\":[{{\"ocel:objectId\":\"praxis-retrofit\",\"ocel:type\":\"Artifact\"}}]}}",
          now, now
      );
      
      let build_log_path = std::path::Path::new(&out_dir).join("build_event.jsonl");
      std::fs::write(build_log_path, build_log).unwrap();
  }
  ```
- **cargo-cicd Extension**: Refactor `cargo-cicd`'s profile tracing code (specifically in the recipe command parser) to support the `build` profile. Executing `cargo cicd trace profile build` must build the workspace and emit corresponding execution event traces to `.cargo-cicd/ocel/events.jsonl`.
- **Test Observability Integration**: Integrate `chicago_tdd_tools::observability::ocel::OcelCollector` into the integration test suites (e.g., `crates/praxis-retrofit/tests/property_tests.rs` or other integration test blocks). The test runner must initialize the collector pointing to `target/praxis/evidence/events.ocel.json`, register it as the active `DiagnosticSink`, and call `close()` at test completion to write the sealed run receipt.
- **Log Location and Compliance**: Running `cargo test` generates the structured JSON log at `/Users/sac/praxis/target/praxis/evidence/events.ocel.json`. The output must be valid OCEL 2.0 JSON containing both the test events and the registered objects.
- **Compilation Hygiene**: All modified crates and tooling compile cleanly.

## Dependencies
- ticket_003_property_testing

## Verification Mechanism
Execute the following verification steps:
1. Build the `praxis-retrofit` package and confirm compile event logging:
   ```bash
   cargo build --package praxis-retrofit
   find target -name "build_event.jsonl" -exec cat {} \;
   ```
   The command should print a valid JSON string with the compile event details.
2. Run the test suite and confirm OCEL file generation:
   ```bash
   cargo test --package praxis-retrofit
   cat target/praxis/evidence/events.ocel.json
   ```
   Confirm that `events.ocel.json` is created, is non-empty, and parses as valid JSON.
3. Run the extended `cargo-cicd` trace command:
   ```bash
   cargo cicd trace profile build
   cat .cargo-cicd/ocel/events.jsonl
   ```
   Verify that trace execution records are appended.
