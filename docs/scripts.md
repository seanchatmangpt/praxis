# Operational Scripts (`scripts/`)

The [`scripts/`](file:///Users/sac/praxis/scripts/) directory houses the operational scripts used to execute integration tests, walkthrough checks, and verification routines.

## Command Reference

### [`walkthrough.sh`](file:///Users/sac/praxis/scripts/walkthrough.sh)
The main test harness verifying system capabilities end-to-end. It runs the entire pipeline, simulates a single-byte tamper attack to ensure detection, and confirms log conformance.

### [`membrane_demo.sh`](file:///Users/sac/praxis/scripts/membrane_demo.sh)
Spawns the Model Context Protocol (MCP) server, sends raw JSON-RPC operations (e.g., `propose_revenue`, `plan_solve`, `admit`), and verifies execution of a transaction through the membrane.

### [`trustless_replay.sh`](file:///Users/sac/praxis/scripts/trustless_replay.sh)
Runs the Python-based validator. It copies the ledger manifests and ontology files to a clean temporary directory, verifying transactions using only `python3` and `b3sum`.
