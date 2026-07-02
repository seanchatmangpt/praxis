#!/usr/bin/env bash
#
# membrane_demo.sh — prove an external agent with ONLY membrane access can
# complete a receipted revenue mission.
#
# This drives the COMPLETE Genesis Day 2 revenue pipe through the praxis MCP
# server over raw JSON-RPC on stdio — no repo access, no CLI, only the tools the
# membrane exposes:
#
#   initialize → tools/list → propose_revenue → propose_goal → plan_solve
#              → judge → admit → receipt → whoami (+ fleet_status)
#
# Every response is asserted. The run ends by printing the receipt chain_hash
# and the session's final AgentByte, which must carry RECEIPTED (0x40).
#
# The server is spoken to exactly as any MCP client would: newline-delimited
# JSON-RPC frames on the server's stdin/stdout, framed by a small Python driver
# (Python is used only for robust subprocess pipe handling + JSON, never to
# reach around the membrane).
#
# Usage:  ./scripts/membrane_demo.sh
# Env:    PRAXIS_MCP_BIN  — path to a prebuilt server binary (skips the build).
# Exit:   0 = the external agent completed a receipted mission through the
#             membrane alone; nonzero = an assertion failed (message names it).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

BIN="${PRAXIS_MCP_BIN:-}"
if [[ -z "${BIN}" ]]; then
    echo "==> Building mcp_lawobject_server (--features mcp,proposer) ..." >&2
    cargo build --quiet --features mcp,proposer --bin mcp_lawobject_server
    BIN="${REPO_ROOT}/target/debug/mcp_lawobject_server"
fi
if [[ ! -x "${BIN}" ]]; then
    echo "membrane_demo: server binary not found/executable: ${BIN}" >&2
    exit 3
fi

echo "==> Driving the Day 2 revenue pipe through the membrane at: ${BIN}" >&2

PRAXIS_MCP_BIN="${BIN}" REPO_ROOT="${REPO_ROOT}" python3 - <<'PYEOF'
import json, os, subprocess, sys

BIN = os.environ["PRAXIS_MCP_BIN"]
ROOT = os.environ["REPO_ROOT"]

# Fixed timestamp → stable receipt chain_hash across runs.
DEMO_TS_NS = 1_751_328_000_000_000_000

# The observed revenue snapshot (mirrors crates/praxis-proposer's rank fixture:
# acct-1 at procurement with full evidence is the top-ranked, closeable deal).
FIXTURE_STATE = {
    "accounts": [
        {"id": "acct-1", "stage": "procurement", "amount_cents": 2_500_000,
         "security_review_done": True, "legal_approved": True, "exec_sponsor": True,
         "days_in_stage": 12},
        {"id": "acct-2", "stage": "qualified", "amount_cents": 800_000,
         "security_review_done": True, "legal_approved": False, "exec_sponsor": True,
         "days_in_stage": 45},
        {"id": "acct-3", "stage": "lead", "amount_cents": 150_000,
         "security_review_done": False, "legal_approved": False, "exec_sponsor": False,
         "days_in_stage": 120},
    ]
}

def die(msg, resp=None):
    print(f"\nFAIL: {msg}", file=sys.stderr)
    if resp is not None:
        print(json.dumps(resp, indent=2)[:2000], file=sys.stderr)
    sys.exit(1)

proc = subprocess.Popen(
    [BIN],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    text=True, bufsize=1, cwd=ROOT,
)

_next_id = [0]
def _id():
    _next_id[0] += 1
    return _next_id[0]

def send(obj):
    proc.stdin.write(json.dumps(obj) + "\n")
    proc.stdin.flush()

def request(method, params=None, rid=None):
    rid = rid if rid is not None else _id()
    send({"jsonrpc": "2.0", "id": rid, "method": method, "params": params or {}})
    # Skip any server-initiated notifications; return the reply with our id.
    while True:
        line = proc.stdout.readline()
        if not line:
            err = proc.stderr.read()
            die(f"server closed stdout waiting for {method} (stderr below)\n{err}")
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("id") == rid:
            return msg

def notify(method, params=None):
    send({"jsonrpc": "2.0", "method": method, "params": params or {}})

def call_tool(name, arguments):
    """Invoke an MCP tool; return the parsed JSON body of its text content."""
    resp = request("tools/call", {"name": name, "arguments": arguments})
    if "error" in resp:
        die(f"tool {name} returned a JSON-RPC error", resp)
    result = resp["result"]
    if result.get("isError"):
        die(f"tool {name} reported is_error=true", result)
    content = result.get("content", [])
    text = next((c["text"] for c in content if c.get("type") == "text"), None)
    if text is None:
        die(f"tool {name} returned no text content", result)
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        die(f"tool {name} text content is not JSON: {text[:400]}")

transcript = {}

# ── 0. initialize (the MCP handshake) ──────────────────────────────────────
init = request("initialize", {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {"name": "membrane-demo", "version": "0"},
})
if "result" not in init:
    die("initialize did not return a result", init)
notify("notifications/initialized")
srv = init["result"].get("serverInfo", {})
print(f"[ok] initialize            → server {srv.get('name','?')} {srv.get('version','?')}")

# ── tools/list: the membrane must expose the whole Day 2 pipe ───────────────
listed = request("tools/list")
tools = sorted(t["name"] for t in listed["result"]["tools"])
required = {"propose_revenue", "propose_goal", "plan_solve", "judge", "admit",
           "receipt", "whoami", "fleet_status"}
missing = required - set(tools)
if missing:
    die(f"tools/list is missing required tools: {sorted(missing)}", {"tools": tools})
print(f"[ok] tools/list            → {len(tools)} tools; pipe coverage complete")
print(f"     tools: {', '.join(tools)}")
transcript["tools"] = tools

objective = json.load(open(os.path.join(ROOT, "crates/praxis-proposer/revenue_objective.json")))

# ── 1. observe → propose_revenue ────────────────────────────────────────────
propose_payload = json.dumps({"state": FIXTURE_STATE, "objective": objective})
rev = call_tool("propose_revenue", {"payload_json": propose_payload})
if rev.get("status") != "proposed" or rev.get("count", 0) < 1:
    die("propose_revenue did not rank any lawful candidate", rev)
print(f"[ok] propose_revenue       → {rev['count']} ranked proposals (observation, not authority)")
transcript["proposals"] = rev["count"]

# ── 2. propose → goal (top-ranked) ──────────────────────────────────────────
goal = call_tool("propose_goal", {"payload_json": propose_payload})
if goal.get("status") != "proposed" or "goal" not in goal:
    die("propose_goal did not emit a goal atom", goal)
goal_atom = goal["goal"]
proposal_hash = goal["proposal_hash"]
print(f"[ok] propose_goal          → goal {goal_atom}  (proposal_hash {proposal_hash[:16]}…)")
transcript["goal"] = goal_atom
transcript["proposal_hash"] = proposal_hash

# ── 3. goal → plan_solve (splice the goal into the shipped domain) ──────────
pddl = open(os.path.join(ROOT, "ontology/revenue.pddl")).read()
FIXTURE_GOAL = "(stage acct-1 closed-won)"
if FIXTURE_GOAL not in pddl:
    die("ontology/revenue.pddl fixture goal line drifted; cannot splice")
spliced = pddl.replace(FIXTURE_GOAL, goal_atom)
solved = call_tool("plan_solve", {"payload_json": json.dumps({"domain": spliced, "mode": "classical"})})
if not solved.get("admitted") or solved.get("plan_len", 0) < 1:
    die("plan_solve found no plan for the proposed goal", solved)
plan = [op.get("label", op) if isinstance(op, dict) else op for op in
        (solved.get("plan", {}).get("ops", []) if isinstance(solved.get("plan"), dict) else [])]
print(f"[ok] plan_solve            → plan_len {solved['plan_len']} (goal reachable in shipped domain)")
transcript["plan_len"] = solved["plan_len"]

# ── 4. plan → judge → admit (through the admission gate) ────────────────────
mission_value = {
    "mission": "revenue-physics-day2",
    "proposal_hash": proposal_hash,
    "goal": goal_atom,
    "target_account": goal.get("target_account"),
    "target_stage": goal.get("target_stage"),
}
law_payload = json.dumps({
    "value": mission_value,
    "obligations": [{"type": "evidence_required", "evidence_type": "legal_approved"}],
    "evidence": ["legal_approved"],
    "instruction_id": 1,
    "ts_ns": DEMO_TS_NS,
})

judged = call_tool("judge", {"payload_json": law_payload})
if judged.get("verdict") != "validated":
    die("judge did not validate the mission payload", judged)
print(f"[ok] judge                 → verdict {judged['verdict']}")

admitted = call_tool("admit", {"payload_json": law_payload})
if admitted.get("status") != "admitted":
    die("admit did not admit the mission payload", admitted)
print(f"[ok] admit                 → status {admitted['status']}")

# ── 5. admission → receipt (BLAKE3 chain, binds proposal_hash) ──────────────
receipted = call_tool("receipt", {"payload_json": law_payload})
if receipted.get("status") != "receipted":
    die("receipt was not issued", receipted)
chain_hash = receipted["chain_hash"]
if len(chain_hash) != 64:
    die(f"receipt chain_hash is not 64 hex chars: {chain_hash}", receipted)
print(f"[ok] receipt               → status receipted; chain_hash {chain_hash}")
transcript["chain_hash"] = chain_hash

# ── whoami: the session's resident AgentByte must now carry RECEIPTED ────────
me = call_tool("whoami", {})
byte = me["byte"]
RECEIPTED = 0x40
if not (byte & RECEIPTED):
    die(f"session AgentByte {byte:#04x} ({me['flags']}) does NOT carry RECEIPTED", me)
print(f"[ok] whoami                → byte {byte:#04x} flags {me['flags']} select {me['select']} (RECEIPTED set)")
transcript["final_byte"] = byte
transcript["final_flags"] = me["flags"]
transcript["final_select"] = me["select"]

# ── fleet_status: sweep a small fleet incl. this session's byte ─────────────
fleet = call_tool("fleet_status", {"fleet_json": json.dumps({"agents": [byte, 0x6F, 0x00, 0x40]})})
st = fleet["stats"]
if st["admitted"] != 2:
    die("fleet_status admitted-count unexpected (want 2)", fleet)
print(f"[ok] fleet_status          → {st['admitted']}/{st['total']} admitted, "
      f"{st['receipted']} receipted (SWAR popcount kernel)")
transcript["fleet_stats"] = st

# ── tear down ────────────────────────────────────────────────────────────
try:
    proc.stdin.close()
    proc.terminate()
    proc.wait(timeout=5)
except Exception:
    proc.kill()

print()
print("=== MEMBRANE DEMO: an external agent completed a receipted mission "
      "using ONLY the MCP membrane ===")
print(json.dumps({
    "mission": "revenue-physics-day2",
    "goal": transcript["goal"],
    "proposal_hash": transcript["proposal_hash"],
    "plan_len": transcript["plan_len"],
    "chain_hash": transcript["chain_hash"],
    "final_agent_byte_hex": f"{transcript['final_byte']:#04x}",
    "final_agent_flags": transcript["final_flags"],
    "final_select": transcript["final_select"],
    "fleet_stats": transcript["fleet_stats"],
}, indent=2))
PYEOF
