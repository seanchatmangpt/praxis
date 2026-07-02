//! Membrane end-to-end test: an external client drives the Day 2 revenue pipe
//! through the `mcp_lawobject_server` binary over raw JSON-RPC on stdio.
//!
//! This is the CI-runnable form of `scripts/membrane_demo.sh` — no Python, no
//! repo access from the "agent" side beyond the tool calls: it spawns the built
//! server binary, speaks newline-delimited JSON-RPC to it, and asserts the
//! whole pipe (propose_revenue → propose_goal → plan_solve → judge → admit →
//! receipt) plus the session's resident AgentByte ending with RECEIPTED set.
//!
//! Only compiled/run under `--features mcp,proposer` (the server binary needs
//! both). Run: `cargo test --features mcp,proposer --test membrane_mcp`.
#![cfg(all(feature = "mcp", feature = "proposer"))]

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

/// A fixed ed25519 seed so the `receipt` tool's fail-closed signing path (under
/// `--features law-signed`, part of `--all-features`) has a key. Harmless when
/// signing is off. Not security-sensitive — test-only.
const TEST_SIGNING_KEY_HEX: &str =
    "3c9d1e2f4a5b6c7d8e9fa0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9cadb";

/// Fixed receipt timestamp → stable chain hash.
const DEMO_TS_NS: u64 = 1_751_328_000_000_000_000;

struct Membrane {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl Membrane {
    fn spawn() -> Self {
        let bin = env!("CARGO_BIN_EXE_mcp_lawobject_server");
        let mut child = Command::new(bin)
            .env("PRAXIS_SIGNING_KEY", TEST_SIGNING_KEY_HEX)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn mcp_lawobject_server");
        let stdin = child.stdin.take().expect("child stdin");
        let stdout = BufReader::new(child.stdout.take().expect("child stdout"));
        Self { child, stdin, stdout, next_id: 0 }
    }

    fn write_msg(&mut self, msg: &Value) {
        self.stdin.write_all(msg.to_string().as_bytes()).expect("write");
        self.stdin.write_all(b"\n").expect("write nl");
        self.stdin.flush().expect("flush");
    }

    /// Send a request and return the reply whose `id` matches (skipping any
    /// server-initiated notifications).
    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        self.write_msg(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        loop {
            let mut line = String::new();
            let n = self.stdout.read_line(&mut line).expect("read_line");
            assert!(n != 0, "server closed stdout while awaiting {method}");
            let Ok(msg) = serde_json::from_str::<Value>(line.trim()) else { continue };
            if msg.get("id").and_then(Value::as_i64) == Some(id) {
                return msg;
            }
        }
    }

    fn notify(&mut self, method: &str) {
        self.write_msg(&json!({"jsonrpc": "2.0", "method": method}));
    }

    /// Invoke an MCP tool; return the parsed JSON body of its text content.
    fn call_tool(&mut self, name: &str, arguments: Value) -> Value {
        let resp = self.request("tools/call", json!({"name": name, "arguments": arguments}));
        assert!(resp.get("error").is_none(), "tool {name} JSON-RPC error: {resp}");
        let result = &resp["result"];
        assert_ne!(result["isError"], json!(true), "tool {name} is_error: {result}");
        let text = result["content"]
            .as_array()
            .and_then(|c| c.iter().find_map(|x| x.get("text")))
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("tool {name} returned no text content: {result}"));
        serde_json::from_str(text).unwrap_or_else(|_| panic!("tool {name} text not JSON: {text}"))
    }

    fn initialize(&mut self) {
        let init = self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "membrane-test", "version": "0"},
            }),
        );
        assert!(init.get("result").is_some(), "initialize failed: {init}");
        self.notify("notifications/initialized");
    }
}

impl Drop for Membrane {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn fixture_state() -> Value {
    json!({
        "accounts": [
            {"id": "acct-1", "stage": "procurement", "amount_cents": 2_500_000,
             "security_review_done": true, "legal_approved": true, "exec_sponsor": true,
             "days_in_stage": 12},
            {"id": "acct-2", "stage": "qualified", "amount_cents": 800_000,
             "security_review_done": true, "legal_approved": false, "exec_sponsor": true,
             "days_in_stage": 45},
            {"id": "acct-3", "stage": "lead", "amount_cents": 150_000,
             "security_review_done": false, "legal_approved": false, "exec_sponsor": false,
             "days_in_stage": 120}
        ]
    })
}

#[test]
fn external_agent_completes_receipted_mission_through_membrane() {
    let objective: Value = serde_json::from_str(include_str!(
        "../crates/praxis-proposer/revenue_objective.json"
    ))
    .expect("objective json");
    let revenue_pddl = include_str!("../ontology/revenue.pddl");
    const FIXTURE_GOAL: &str = "(stage acct-1 closed-won)";

    let mut m = Membrane::spawn();
    m.initialize();

    // tools/list must cover the whole Day 2 pipe.
    let listed = m.request("tools/list", json!({}));
    let tools: Vec<String> = listed["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .map(|t| t["name"].as_str().unwrap_or_default().to_string())
        .collect();
    for required in [
        "propose_revenue",
        "propose_goal",
        "plan_solve",
        "judge",
        "admit",
        "receipt",
        "whoami",
        "fleet_status",
    ] {
        assert!(tools.iter().any(|t| t == required), "tools/list missing {required}: {tools:?}");
    }

    let propose_payload =
        json!({"state": fixture_state(), "objective": objective}).to_string();

    // 1. observe → propose_revenue
    let rev = m.call_tool("propose_revenue", json!({"payload_json": propose_payload}));
    assert_eq!(rev["status"], json!("proposed"));
    assert!(rev["count"].as_u64().unwrap_or(0) >= 1);

    // 2. propose → goal
    let goal = m.call_tool("propose_goal", json!({"payload_json": propose_payload}));
    assert_eq!(goal["status"], json!("proposed"));
    let goal_atom = goal["goal"].as_str().expect("goal atom").to_string();
    let proposal_hash = goal["proposal_hash"].as_str().expect("proposal_hash").to_string();

    // 3. goal → plan_solve (splice the proposer goal into the shipped domain)
    assert!(revenue_pddl.contains(FIXTURE_GOAL), "fixture goal drifted from revenue.pddl");
    let spliced = revenue_pddl.replace(FIXTURE_GOAL, &goal_atom);
    let solved = m.call_tool("plan_solve", json!({"payload_json": json!({"domain": spliced, "mode": "classical"}).to_string()}));
    assert_eq!(solved["admitted"], json!(true), "plan_solve refused: {solved}");
    assert!(solved["plan_len"].as_u64().unwrap_or(0) >= 1);

    // 4/5. judge → admit → receipt of the mission payload (evidence satisfied).
    let law_payload = json!({
        "value": {
            "mission": "revenue-physics-day2",
            "proposal_hash": proposal_hash,
            "goal": goal_atom,
        },
        "obligations": [{"type": "evidence_required", "evidence_type": "legal_approved"}],
        "evidence": ["legal_approved"],
        "instruction_id": 1,
        "ts_ns": DEMO_TS_NS,
    })
    .to_string();

    let judged = m.call_tool("judge", json!({"payload_json": law_payload}));
    assert_eq!(judged["verdict"], json!("validated"), "judge: {judged}");

    let admitted = m.call_tool("admit", json!({"payload_json": law_payload}));
    assert_eq!(admitted["status"], json!("admitted"), "admit: {admitted}");

    let receipted = m.call_tool("receipt", json!({"payload_json": law_payload}));
    assert_eq!(receipted["status"], json!("receipted"), "receipt: {receipted}");
    let chain_hash = receipted["chain_hash"].as_str().expect("chain_hash");
    assert_eq!(chain_hash.len(), 64, "chain_hash not 64 hex chars");

    // The session's resident AgentByte must now carry RECEIPTED (0x40).
    let me = m.call_tool("whoami", json!({}));
    let byte = me["byte"].as_u64().expect("byte") as u8;
    assert_ne!(byte & 0x40, 0, "session byte {byte:#04x} ({}) lacks RECEIPTED", me["flags"]);
    // Full pipe with satisfied evidence lands a fully-granted byte.
    assert_eq!(me["select"], json!("Grant"), "final byte should be Grant: {me}");

    // fleet_status sweeps a fleet including this session's byte.
    let fleet = m.call_tool(
        "fleet_status",
        json!({"fleet_json": json!({"agents": [byte, 0x6F, 0x00, 0x40]}).to_string()}),
    );
    assert_eq!(fleet["stats"]["admitted"], json!(2), "fleet_status: {fleet}");
}
