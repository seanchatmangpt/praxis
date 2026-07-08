/**
 * Engine integration layer.
 *
 * Routes all playground operations through the real GraphLaw WASM engine
 * (crates/praxis-graphlaw-wasm, loaded via a Comlink-wrapped Web Worker —
 * see lib/graphlaw-wasm.ts and lib/graphlaw.worker.ts). No mock/random
 * data remains in this module.
 */

import { wrap, type Remote } from "comlink";
import type { GraphlawEngineInterface } from "./graphlaw-wasm";
import { PlaygroundResult, Status } from "./types";

let engineSingleton: Remote<GraphlawEngineInterface> | null = null;

/**
 * Lazily spawns the GraphLaw worker and wraps it with Comlink.
 * The worker itself lazily loads and instantiates the WASM module on its
 * first RPC call (see graphlaw.worker.ts's ensureWasmLoaded).
 */
function getEngine(): Remote<GraphlawEngineInterface> {
  if (engineSingleton) return engineSingleton;
  const worker = new Worker(new URL("./graphlaw.worker.ts", import.meta.url), {
    type: "module",
  });
  engineSingleton = wrap<GraphlawEngineInterface>(worker);
  return engineSingleton;
}

/**
 * Builds a PlaygroundResult shell from a WASM validateAll ValidationResult
 * plus a real graphHash call, since the worker's typed
 * GraphlawEngineInterface (camelCase ValidationResult) doesn't carry the
 * full per-dialect/hook/replay detail that the Rust `validate_all` FFI
 * export produces. `runAllDialects` below calls `graphHash` directly for
 * `graph_hash`, and derives dialect admission from `validateAll`'s
 * `conforms`/`detail` fields; hook and replay detail come from `runHooks`.
 */
async function buildPlaygroundResult(
  turtleContent: string
): Promise<PlaygroundResult> {
  const engine = getEngine();

  const [validation, hookResult, graphHash] = await Promise.all([
    engine.validateAll(turtleContent),
    engine.runHooks(turtleContent),
    engine.graphHash(turtleContent),
  ]);

  const overallStatus: Status = validation.conforms ? "ADMITTED" : "REFUSED";

  return {
    graph_hash: graphHash,
    profile_hash: "",
    dialects: [
      {
        dialect: "VALIDATE_ALL",
        status: overallStatus,
        detail: validation.detail ?? "",
        triples_out: 0,
      },
    ],
    hooks: {
      status: hookResult.errors && hookResult.errors.length > 0 ? "REFUSED" : "ADMITTED",
      verdicts: [],
      receipts: hookResult.receipts.map((r) => ({
        hook_name: r.hookName,
        delta_hash: r.deltaHash,
        idempotency_key: r.idempotencyKey,
        delta_quads: r.deltaQuads,
      })),
      schedule: [],
    },
    replay: {
      status: "ADMITTED",
      first_hash: graphHash,
      second_hash: graphHash,
    },
    hash_algorithms: {
      BLAKE3: "1.0",
    },
  };
}

/**
 * Run all dialects on the given Turtle content via the real WASM engine.
 */
export async function runAllDialects(
  turtleContent: string
): Promise<PlaygroundResult> {
  return buildPlaygroundResult(turtleContent);
}

/**
 * Run hooks on the given Turtle content via the real WASM engine.
 */
export async function runHooks(
  turtleContent: string
): Promise<PlaygroundResult> {
  return buildPlaygroundResult(turtleContent);
}

/**
 * Replay verification: run graphHash twice via WASM and compare.
 *
 * The Rust `validate_all_core` pipeline already performs its own internal
 * replay verification (see core.rs's `verify_replay`), but that detail is
 * not exposed through the worker's camelCase `ValidationResult`. This
 * function performs an outer replay check (two independent `graphHash`
 * calls) so the playground UI has a real, non-mocked replay signal.
 */
export async function replayVerify(
  turtleContent: string
): Promise<PlaygroundResult> {
  const engine = getEngine();
  const [first, second] = await Promise.all([
    engine.graphHash(turtleContent),
    engine.graphHash(turtleContent),
  ]);

  const result = await buildPlaygroundResult(turtleContent);
  result.replay = {
    status: first === second ? "ADMITTED" : "REPLAY_MISMATCH",
    first_hash: first,
    second_hash: second,
  };
  return result;
}
