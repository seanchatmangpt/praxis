/**
 * Typed bridge to GraphLaw WASM engine via Web Worker using Comlink.
 *
 * Provides a proxy interface to the WebAssembly-compiled reasoning engine,
 * with call tracking to discard stale results from long-running operations.
 */

import { wrap } from 'comlink';

export interface GraphlawEngineInterface {
  /**
   * Validates a Turtle/Datalog document against all loaded rules and shapes.
   * Returns dialect-specific validation results (SHACL conforms, ShEx validation, etc).
   */
  validateAll(turtleSource: string): Promise<ValidationResult>;

  /**
   * Evaluates knowledge hooks against the given facts.
   * Returns hook receipts (audit trail of derivations).
   */
  runHooks(turtleSource: string): Promise<HookExecutionResult>;

  /**
   * Computes BLAKE3 receipt hash of canonical N-Quads for the given facts.
   * Used to verify determinism and content-addressability.
   */
  graphHash(turtleSource: string): Promise<string>;

  /**
   * Parses Turtle/Datalog and extracts hooks, rules, and shape definitions.
   * Useful for IDE operations: symbol extraction, outline, refactoring.
   */
  extractMetadata(
    turtleSource: string
  ): Promise<{ hooks: HookMetadata[]; rules: string[] }>;

  /**
   * Gracefully terminates the worker and cleans up resources.
   */
  terminate(): Promise<void>;
}

export interface ValidationResult {
  /** ISO 8601 timestamp when validation ran */
  timestamp: string;

  /** True if all constraints satisfied */
  conforms: boolean;

  /** SHACL violation results, if any */
  violations?: ShaclViolation[];

  /** ShEx validation failures, if any */
  shexFailures?: ShExFailure[];

  /** Datalog denial rule violations */
  denials?: DenialViolation[];

  /** Free-form detail string for diagnostics */
  detail?: string;
}

export interface ShaclViolation {
  focusNode: string;
  resultPath: string;
  resultMessage: string;
  severity: 'Violation' | 'Warning' | 'Info';
}

export interface ShExFailure {
  nodeId: string;
  shapeLabel: string;
  reason: string;
}

export interface DenialViolation {
  ruleId: string;
  triggeredFacts: string[];
  message: string;
}

export interface HookExecutionResult {
  /** Deterministic BLAKE3 receipt hash of all derivations */
  receiptHash: string;

  /** Hook audit trail (one entry per triggered hook) */
  receipts: HookReceipt[];

  /** Any semantic errors encountered during evaluation */
  errors?: string[];
}

export interface HookReceipt {
  /** IRI of the triggered hook */
  hookName: string;

  /** N-Quads of derived facts, sorted canonically */
  deltaQuads: string;

  /** BLAKE3 hash of deltaQuads for content-addressability */
  deltaHash: string;

  /** Deterministic key for deduplication/idempotency */
  idempotencyKey: string;
}

export interface HookMetadata {
  /** IRI or local name of the hook */
  name: string;

  /** Either 'kh:' or 'hook:' prefix */
  namespace: string;

  /** Offset into source (line, column) for IDE navigation */
  offset?: { line: number; column: number };
}

/**
 * Call counter to track and discard stale results.
 * Each validateAll/runHooks/graphHash increments the counter;
 * the worker includes its counter in results. If counter doesn't match,
 * the result was from an older operation and should be discarded.
 */
let callCounter = 0;

/**
 * Initializes the GraphLaw engine worker.
 *
 * Spawns a Web Worker running graphlaw.worker.ts and returns a typed proxy
 * via Comlink. The proxy supports call counting to ensure results are fresh.
 */
export async function initGraphlawEngine(): Promise<GraphlawEngineInterface> {
  const worker = new Worker(new URL('./graphlaw.worker.ts', import.meta.url), {
    type: 'module',
  });

  // Wrap the worker with Comlink to get typed proxy
  const engine = wrap<GraphlawEngineInterface>(worker);

  return engine;
}

/**
 * Wraps a validateAll call with call counting to detect stale results.
 *
 * Usage:
 *   const callId = getNextCallId();
 *   const result = await validateWithCallId(engine, source, callId);
 *   if (result.stale) {
 *     console.log('Result is stale; ignoring');
 *   } else {
 *     processResult(result.data);
 *   }
 */
export function getNextCallId(): number {
  return ++callCounter;
}

export interface CallResult<T> {
  data: T;
  callId: number;
  stale: boolean;
}

export async function validateWithCallId(
  engine: GraphlawEngineInterface,
  turtleSource: string,
  callId: number
): Promise<CallResult<ValidationResult>> {
  const result = await engine.validateAll(turtleSource);
  return {
    data: result,
    callId,
    stale: callId !== callCounter,
  };
}

export async function runHooksWithCallId(
  engine: GraphlawEngineInterface,
  turtleSource: string,
  callId: number
): Promise<CallResult<HookExecutionResult>> {
  const result = await engine.runHooks(turtleSource);
  return {
    data: result,
    callId,
    stale: callId !== callCounter,
  };
}

export async function graphHashWithCallId(
  engine: GraphlawEngineInterface,
  turtleSource: string,
  callId: number
): Promise<CallResult<string>> {
  const result = await engine.graphHash(turtleSource);
  return {
    data: result,
    callId,
    stale: callId !== callCounter,
  };
}
