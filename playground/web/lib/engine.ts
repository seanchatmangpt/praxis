/**
 * Engine integration layer.
 * Provides interface to Praxis GraphLaw WASM binding and mock data.
 */

import { PlaygroundResult } from "./types";

/**
 * Mock engine result for development.
 * Replace with actual WASM call when available.
 */
export function createMockResult(
  graphHash: string = "a1b2c3d4",
  dialects: string[] = [
    "OWL_RL",
    "SHACL",
    "ShEx",
    "Datalog",
    "N3_DENIALS",
  ]
): PlaygroundResult {
  return {
    graph_hash: graphHash,
    profile_hash: "profile_e5f6g7h8",
    dialects: dialects.map((d) => ({
      dialect: d,
      status: Math.random() > 0.1 ? "ADMITTED" : "REFUSED",
      detail: `Processed ${d} rules successfully`,
      triples_out: Math.floor(Math.random() * 500) + 50,
    })),
    hooks: {
      status: "ADMITTED",
      verdicts: [
        {
          hook_id: 1,
          hook_iri: "http://example.com/hooks/ValidateShape",
          hook_name: "ValidateShape",
          condition_kind: "shacl",
          condition_hash: "cond_hash_1",
          verdict: "Fired",
          effect: "assert",
          action_iri: "http://example.com/actions/log",
          delta_hash: "delta_a1b2c3d4",
        },
        {
          hook_id: 2,
          hook_iri: "http://example.com/hooks/LogChange",
          hook_name: "LogChange",
          condition_kind: "delta",
          condition_hash: "cond_hash_2",
          verdict: "Fired",
          effect: "assert",
          action_iri: "http://example.com/actions/log",
          delta_hash: "delta_e5f6g7h8",
        },
        {
          hook_id: 3,
          hook_iri: "http://example.com/hooks/NotifyExternal",
          hook_name: "NotifyExternal",
          condition_kind: "sparql",
          condition_hash: "cond_hash_3",
          verdict: "Gated",
          effect: "assert",
        },
      ],
      receipts: [
        {
          hook_name: "ValidateShape",
          delta_hash: "delta_a1b2c3d4",
          idempotency_key: "idempotent_key_1",
          delta_quads: "<http://example.com/s1> <http://example.com/p1> <http://example.com/o1> .",
        },
        {
          hook_name: "LogChange",
          delta_hash: "delta_e5f6g7h8",
          idempotency_key: "idempotent_key_2",
          delta_quads: "<http://example.com/s2> <http://example.com/p2> <http://example.com/o2> .",
        },
      ],
      schedule: [
        "ValidateShape",
        "LogChange",
        "NotifyExternal",
        "UpdateIndex",
      ],
    },
    replay: {
      status: "ADMITTED",
      first_hash: "replay_first_1a2b3c4d5e6f",
      second_hash: "replay_second_1a2b3c4d5e6f",
    },
    hash_algorithms: {
      BLAKE3: "1.0",
    },
  };
}

/**
 * Run all dialects on the given Turtle content.
 * Currently returns mock data; integrate with WASM when available.
 */
export async function runAllDialects(
  _turtleContent: string
): Promise<PlaygroundResult> {
  // Simulate network latency
  await new Promise((resolve) => setTimeout(resolve, 500));
  return createMockResult();
}

/**
 * Run hooks on the given Turtle content.
 * Currently returns mock data; integrate with WASM when available.
 */
export async function runHooks(_turtleContent: string): Promise<PlaygroundResult> {
  // Simulate network latency
  await new Promise((resolve) => setTimeout(resolve, 300));
  return createMockResult();
}

/**
 * Replay verification: run twice and compare hashes.
 */
export async function replayVerify(_turtleContent: string): Promise<PlaygroundResult> {
  // Simulate network latency
  await new Promise((resolve) => setTimeout(resolve, 800));
  const result = createMockResult();
  result.replay.status = result.replay.first_hash === result.replay.second_hash
    ? "ADMITTED"
    : "REPLAY_MISMATCH";
  return result;
}
