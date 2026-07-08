/**
 * Web Worker that loads and runs the GraphLaw WASM engine.
 *
 * Exports the GraphlawEngine interface for Comlink RPC.
 * All operations run in the worker thread to avoid blocking the UI.
 */

import { expose } from 'comlink';
import type {
  GraphlawEngineInterface,
  ValidationResult,
  HookExecutionResult,
  HookMetadata,
} from './graphlaw-wasm';

/**
 * Lazy-loaded WASM module. Initialized on first engine method call.
 *
 * `null` means "not yet attempted"; an object with `.ready === false` means
 * the import/instantiate step failed and callers must surface a loud error
 * rather than silently falling back to placeholder data (see
 * .claude/rules/no-overclaiming-js.md: stub failures must throw, not
 * resolve success-shaped).
 */
let wasmModule: {
  ready: boolean;
  validate_all?: (
    ttl: string,
    profileTtl: string,
    shaclShapes: string,
    shexSchema: string,
    shexShapeMap: string
  ) => string;
  run_hooks?: (baseTtl: string, eventTtl: string) => string;
  graph_hash?: (ttl: string) => string;
  blake3_hex?: (data: string) => string;
} | null = null;

/**
 * Initialize WASM module if not already loaded.
 *
 * This function handles the async loading and instantiation of the
 * WebAssembly module (built via `wasm-pack build --target web`, which
 * requires calling the default export to instantiate before the exported
 * functions are usable).
 *
 * Throws if the module cannot be loaded/instantiated — callers must not
 * swallow this into a placeholder result.
 */
async function ensureWasmLoaded(): Promise<void> {
  if (wasmModule?.ready) return;

  // Dynamically import the WASM package (praxis-graphlaw-wasm), built via
  // `wasm-pack build --target web` from crates/praxis-graphlaw-wasm.
  const mod = await import('praxis-graphlaw-wasm');
  // wasm-bindgen `--target web` modules export a default init function that
  // must be awaited before any other export is callable.
  await mod.default();
  wasmModule = {
    ready: true,
    validate_all: mod.validate_all,
    run_hooks: mod.run_hooks,
    graph_hash: mod.graph_hash,
    blake3_hex: mod.blake3_hex,
  };
}

/** DTO shapes matching crates/praxis-graphlaw-wasm/src/dto.rs (snake_case, as serialized). */
interface DialectResultDto {
  dialect: string;
  status: string;
  detail: string;
  triples_out: number;
}
interface HookReceiptDto {
  hook_name: string;
  delta_hash: string;
  idempotency_key: string;
  delta_quads: string;
}
interface HookVerdictRecordDto {
  hook_id: number;
  hook_iri: string;
  hook_name: string;
  condition_kind: string;
  condition_hash: string;
  verdict: string;
  effect: string;
  action_iri?: string;
  delta_hash?: string;
  idempotency_key?: string;
}
interface HookRunResultDto {
  status: string;
  verdicts: HookVerdictRecordDto[];
  receipts: HookReceiptDto[];
  schedule: string[];
}
interface PlaygroundResultDto {
  graph_hash: string;
  profile_hash: string;
  dialects: DialectResultDto[];
  hooks: HookRunResultDto;
  replay: { status: string; first_hash: string; second_hash: string };
  hash_algorithms: Record<string, string>;
}

/**
 * Parses a WASM bridge JSON response, throwing if the engine returned
 * `{ "error": "..." }` (the convention used by lib.rs's validate_all/
 * run_hooks/graph_hash wrappers to surface Refusal-style failures across
 * the FFI boundary without relying on WASM exception marshaling).
 */
function parseWasmJson<T>(raw: string): T {
  const parsed = JSON.parse(raw);
  if (parsed && typeof parsed === 'object' && 'error' in parsed) {
    throw new Error(String((parsed as { error: unknown }).error));
  }
  return parsed as T;
}

/**
 * Implementation of the GraphlawEngine interface.
 * All methods are async to support WASM blocking operations.
 */
const engine: GraphlawEngineInterface = {
  async validateAll(turtleSource: string): Promise<ValidationResult> {
    // Jidoka: no try/catch here. A thrown WASM/parse error must stop the
    // line and surface to the caller as a rejected promise (Comlink
    // re-throws across the worker boundary) — never get re-packaged into a
    // conforms:false ValidationResult, which would let a real engine
    // failure look identical to a normal SHACL refusal.
    await ensureWasmLoaded();
    if (!wasmModule?.validate_all) {
      throw new Error('GraphLaw WASM module not loaded: validate_all unavailable');
    }

    // No profile/SHACL/ShEx inputs are wired at this call site (single
    // turtleSource argument only) — OWL RL, SHACL, and ShEx dialects will
    // report PROFILE_NOT_ADMITTED / UNSUPPORTED, which is correct given
    // the inputs, not a placeholder result.
    const raw = wasmModule.validate_all(turtleSource, '', '', '', '');
    const result = parseWasmJson<PlaygroundResultDto>(raw);

    const shaclDialect = result.dialects.find((d) => d.dialect === 'SHACL');
    const n3Dialect = result.dialects.find((d) => d.dialect === 'N3_DENIAL');
    const conforms = result.dialects.every((d) => d.status !== 'REFUSED');

    return {
      timestamp: new Date().toISOString(),
      conforms,
      violations: !conforms && shaclDialect
        ? [
            {
              focusNode: 'graph',
              resultPath: 'SHACL',
              resultMessage: shaclDialect.detail,
              severity: 'Violation',
            },
          ]
        : [],
      detail: [n3Dialect?.detail, shaclDialect?.detail]
        .filter(Boolean)
        .join('; ') || 'Validated',
    };
  },

  async runHooks(turtleSource: string): Promise<HookExecutionResult> {
    // Jidoka: no try/catch. A hook-pack refusal or engine error must throw
    // and stop the line, not be laundered into an `errors: [...]` field on
    // an otherwise success-shaped HookExecutionResult.
    await ensureWasmLoaded();
    if (!wasmModule?.run_hooks || !wasmModule.blake3_hex) {
      throw new Error('GraphLaw WASM module not loaded: run_hooks unavailable');
    }

    // Single-argument interface: treat turtleSource as the base graph with
    // an empty event delta (no incremental facts asserted this call).
    const raw = wasmModule.run_hooks(turtleSource, '');
    const result = parseWasmJson<HookRunResultDto>(raw);

    const receipts = result.receipts.map((r) => ({
      hookName: r.hook_name,
      deltaQuads: r.delta_quads,
      deltaHash: r.delta_hash,
      idempotencyKey: r.idempotency_key,
    }));

    // Canonical order (by hook_name) before hashing, per the determinism
    // invariant: no relying on incidental Vec ordering as canonical.
    const canonical = [...receipts]
      .sort((a, b) => a.hookName.localeCompare(b.hookName))
      .map((r) => `${r.hookName}|${r.deltaHash}|${r.idempotencyKey}`)
      .join('\n');
    const receiptHash = wasmModule.blake3_hex(canonical);

    return { receiptHash, receipts, errors: [] };
  },

  async graphHash(turtleSource: string): Promise<string> {
    await ensureWasmLoaded();
    if (!wasmModule?.graph_hash) {
      throw new Error('GraphLaw WASM module not loaded: graph_hash unavailable');
    }

    const raw = wasmModule.graph_hash(turtleSource);
    // graph_hash returns a bare hex digest on success (not JSON) or
    // `{ "error": "..." }` on failure; only attempt JSON parsing to detect
    // the error case.
    if (raw.startsWith('{')) {
      const parsed = JSON.parse(raw);
      if (parsed && typeof parsed === 'object' && 'error' in parsed) {
        throw new Error(
          `Failed to compute graph hash: ${String((parsed as { error: unknown }).error)}`
        );
      }
    }
    return raw;
  },

  async extractMetadata(
    turtleSource: string
  ): Promise<{ hooks: HookMetadata[]; rules: string[] }> {
    await ensureWasmLoaded();

    try {
      // Parse hooks from source by looking for kh: and hook: IRIs
      // This is a regex-based fallback; ideally WASM would do the parsing
      const hookPattern =
        /(?:kh:|hook:)(\w+)|<http:\/\/seanchatmangpt\.github\.io\/praxis\/(kh|hook)#(\w+)>/g;
      const hooks: HookMetadata[] = [];
      let match;

      // Extract line/column offsets for each hook found
      let line = 1;
      let column = 1;
      for (const char of turtleSource) {
        match = hookPattern.exec(turtleSource);
        if (match) {
          const name = match[1] || match[3];
          const namespace = match[2] || (match[0].startsWith('kh:') ? 'kh' : 'hook');
          hooks.push({
            name,
            namespace,
            offset: { line, column },
          });
        }
        if (char === '\n') {
          line++;
          column = 1;
        } else {
          column++;
        }
      }

      // Extract rules (simplified: look for @prefix and CONSTRUCT queries)
      const rules: string[] = [];
      const rulePattern = /CONSTRUCT\s*{[\s\S]*?}\s*WHERE/gi;
      let ruleMatch;
      while ((ruleMatch = rulePattern.exec(turtleSource)) !== null) {
        rules.push(ruleMatch[0]);
      }

      return { hooks, rules };
    } catch (error) {
      console.error('Failed to extract metadata:', error);
      return { hooks: [], rules: [] };
    }
  },

  async terminate(): Promise<void> {
    wasmModule = null;
    // Worker will self-terminate when this function returns
    self.close();
  },
};

// Export the engine for Comlink
expose(engine);
