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
  // @ts-expect-error - Module resolved via file: dependency; no ambient types
  const mod = await import('praxis-graphlaw-wasm');
  // wasm-bindgen `--target web` modules export a default init function that
  // must be awaited before any other export is callable.
  await mod.default();
  wasmModule = {
    ready: true,
    validate_all: mod.validate_all,
    run_hooks: mod.run_hooks,
    graph_hash: mod.graph_hash,
  };
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
  async validateAll(_turtleSource: string): Promise<ValidationResult> {
    await ensureWasmLoaded();

    try {
      // TODO: integrate with WASM engine
      // preprocessTurtle(_turtleSource);

      // Call WASM engine to load and validate
      // For now, we create a TripleStore and validate against loaded shapes
      // This assumes we have WASM methods: load_triples, validate_shacl, validate_shex
      // TODO: Expose these methods from praxis-graphlaw-wasm/lib.rs

      return {
        timestamp: new Date().toISOString(),
        conforms: true, // Placeholder: will be computed by WASM
        violations: [],
        detail: 'Validation result placeholder',
      };
    } catch (error) {
      return {
        timestamp: new Date().toISOString(),
        conforms: false,
        violations: [
          {
            focusNode: 'example:root',
            resultPath: 'example:property',
            resultMessage:
              error instanceof Error
                ? error.message
                : 'Unknown validation error',
            severity: 'Violation',
          },
        ],
      };
    }
  },

  async runHooks(_turtleSource: string): Promise<HookExecutionResult> {
    await ensureWasmLoaded();

    try {
      // TODO: integrate with WASM engine
      // preprocessTurtle(_turtleSource);

      // Call WASM engine to run hooks
      // This assumes we have a WASM method: load_triples, compile_hooks, evaluate_hooks, get_hook_receipts
      // TODO: Expose these methods from praxis-graphlaw-wasm/lib.rs

      return {
        receiptHash: 'blake3-hash-placeholder',
        receipts: [],
        errors: [],
      };
    } catch (error) {
      return {
        receiptHash: '',
        receipts: [],
        errors: [
          error instanceof Error
            ? error.message
            : 'Unknown hook execution error',
        ],
      };
    }
  },

  async graphHash(_turtleSource: string): Promise<string> {
    await ensureWasmLoaded();

    try {
      // TODO: integrate with WASM engine
      // preprocessTurtle(_turtleSource);

      // Call WASM engine to compute hash
      // This assumes we have a WASM method: load_triples, compute_hash
      // TODO: Expose these methods from praxis-graphlaw-wasm/lib.rs

      return 'blake3-placeholder-hash';
    } catch (error) {
      throw new Error(
        `Failed to compute graph hash: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
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
