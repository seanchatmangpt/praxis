/**
 * Unit tests for lib/engine.ts
 *
 * engine.ts routes every operation through a real Comlink-wrapped Web Worker
 * (graphlaw.worker.ts) that loads the actual GraphLaw WASM module — there is
 * no mock/random data left in engine.ts itself (see
 * .claude/rules/no-overclaiming-js.md). Neither a Worker nor a WASM instance
 * is available in the jsdom/vitest environment, so this suite mocks the
 * `comlink` boundary (`wrap`) and stubs the global `Worker` constructor,
 * then asserts engine.ts correctly assembles a PlaygroundResult from the
 * (fake, in this test) engine's responses — the real WASM-backed behavior
 * is exercised by `crates/praxis-graphlaw-wasm/tests/core.rs` and the
 * Playwright specs in `tests/`, not here.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { PlaygroundResult } from '../types';

const mockValidateAll = vi.fn();
const mockRunHooks = vi.fn();
const mockGraphHash = vi.fn();

vi.mock('comlink', () => ({
  wrap: () => ({
    validateAll: mockValidateAll,
    runHooks: mockRunHooks,
    graphHash: mockGraphHash,
  }),
}));

// jsdom has no Worker implementation; engine.ts only needs a constructible
// stand-in since the real RPC target is replaced by the comlink mock above.
class FakeWorker {
  constructor(_url: URL, _opts?: unknown) {}
}
vi.stubGlobal('Worker', FakeWorker);

describe('engine.ts', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockValidateAll.mockResolvedValue({
      timestamp: new Date().toISOString(),
      conforms: true,
      violations: [],
      detail: 'Validated',
    });
    mockRunHooks.mockResolvedValue({
      receiptHash: 'abc123',
      receipts: [
        {
          hookName: 'ex:hook1',
          deltaQuads: '',
          deltaHash: 'deadbeef',
          idempotencyKey: 'idem-1',
        },
      ],
      errors: [],
    });
    mockGraphHash.mockResolvedValue('feedface1234');
  });

  describe('runAllDialects()', () => {
    it('returns a Promise resolving to a well-formed PlaygroundResult', async () => {
      const { runAllDialects } = await import('../engine');
      const sampleTurtle = '@prefix ex: <http://example.com/> .\n';

      const result = runAllDialects(sampleTurtle);
      expect(result).toBeInstanceOf(Promise);

      const resolved: PlaygroundResult = await result;
      expect(resolved.graph_hash).toBe('feedface1234');
      expect(resolved.dialects.every((d) => d.status !== 'HASH_MISMATCH')).toBe(true);
      expect(mockValidateAll).toHaveBeenCalledWith(sampleTurtle);
      expect(mockGraphHash).toHaveBeenCalledWith(sampleTurtle);
    });

    it('maps validateAll conforms=false to a REFUSED dialect status', async () => {
      mockValidateAll.mockResolvedValueOnce({
        timestamp: new Date().toISOString(),
        conforms: false,
        violations: [
          {
            focusNode: 'graph',
            resultPath: 'SHACL',
            resultMessage: 'missing required property',
            severity: 'Violation',
          },
        ],
        detail: 'refused',
      });

      const { runAllDialects } = await import('../engine');
      const result = await runAllDialects('@prefix ex: <http://example.com/> .\n');

      expect(result.dialects[0].status).toBe('REFUSED');
    });
  });

  describe('runHooks()', () => {
    it('surfaces real hook receipts from the worker, not placeholder data', async () => {
      const { runHooks } = await import('../engine');
      const result = await runHooks('@prefix ex: <http://example.com/> .\n');

      expect(result.hooks.receipts).toHaveLength(1);
      expect(result.hooks.receipts[0].hook_name).toBe('ex:hook1');
      expect(result.hooks.receipts[0].delta_hash).toBe('deadbeef');
      expect(mockRunHooks).toHaveBeenCalled();
    });

    it('maps a non-empty errors array to REFUSED status', async () => {
      mockRunHooks.mockResolvedValueOnce({
        receiptHash: '',
        receipts: [],
        errors: ['hook pack exceeds 12 hooks'],
      });

      const { runHooks } = await import('../engine');
      const result = await runHooks('@prefix ex: <http://example.com/> .\n');

      expect(result.hooks.status).toBe('REFUSED');
    });
  });

  describe('replayVerify()', () => {
    it('reports ADMITTED when two independent graphHash calls match', async () => {
      const { replayVerify } = await import('../engine');
      const result = await replayVerify('@prefix ex: <http://example.com/> .\n');

      expect(result.replay.status).toBe('ADMITTED');
      expect(result.replay.first_hash).toBe(result.replay.second_hash);
      expect(mockGraphHash).toHaveBeenCalledTimes(3); // 2 for replay + 1 inside buildPlaygroundResult
    });

    it('reports REPLAY_MISMATCH when the two graphHash calls disagree', async () => {
      mockGraphHash.mockResolvedValueOnce('hash-a').mockResolvedValueOnce('hash-b');

      const { replayVerify } = await import('../engine');
      const result = await replayVerify('@prefix ex: <http://example.com/> .\n');

      expect(result.replay.status).toBe('REPLAY_MISMATCH');
      expect(result.replay.first_hash).not.toBe(result.replay.second_hash);
    });
  });
});
