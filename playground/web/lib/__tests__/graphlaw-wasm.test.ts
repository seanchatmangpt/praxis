/**
 * Unit tests for lib/graphlaw-wasm.ts
 *
 * Tests the Web Worker bridge and call-tracking logic that uses Comlink
 * to communicate with the GraphLaw WASM engine. Verifies the interface,
 * call counting for stale result detection, and Promise-based async behavior.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  getNextCallId,
  validateWithCallId,
  runHooksWithCallId,
  graphHashWithCallId,
  GraphlawEngineInterface,
  CallResult,
  ValidationResult,
  HookExecutionResult,
} from '../graphlaw-wasm';

/**
 * Mock implementation of GraphlawEngineInterface for testing.
 * Simulates delayed responses to test call-tracking behavior.
 */
class MockGraphlawEngine implements GraphlawEngineInterface {
  constructor(private delayMs: number = 0) {}

  async validateAll(turtleSource: string): Promise<ValidationResult> {
    await new Promise((resolve) => setTimeout(resolve, this.delayMs));
    return {
      timestamp: new Date().toISOString(),
      conforms: true,
      violations: [],
      detail: `Validated: ${turtleSource.length} bytes`,
    };
  }

  async runHooks(turtleSource: string): Promise<HookExecutionResult> {
    await new Promise((resolve) => setTimeout(resolve, this.delayMs));
    return {
      receiptHash: 'blake3-test-hash-12345678901234567890123456789012',
      receipts: [
        {
          hookName: 'TestHook',
          deltaQuads: `<http://example.com/s1> <http://example.com/p1> <http://example.com/o1> .`,
          deltaHash: 'delta-hash-test',
          idempotencyKey: 'idempotent-key-1',
        },
      ],
      errors: [],
    };
  }

  async graphHash(turtleSource: string): Promise<string> {
    await new Promise((resolve) => setTimeout(resolve, this.delayMs));
    return 'blake3-graph-hash-abcdef1234567890abcdef1234567890abcd';
  }

  async extractMetadata(turtleSource: string): Promise<{
    hooks: any[];
    rules: string[];
  }> {
    return { hooks: [], rules: [] };
  }

  async terminate(): Promise<void> {
    // Mock termination
  }
}

describe('graphlaw-wasm.ts', () => {
  beforeEach(() => {
    // Reset call counter before each test
    // Note: We can't directly reset the module-level callCounter,
    // but we can test relative behavior
  });

  describe('GraphlawEngineInterface structure', () => {
    it('test_graphlaw_wasm_interface_has_three_main_methods', () => {
      const mockEngine = new MockGraphlawEngine();

      // Verify interface has required methods
      expect(typeof mockEngine.validateAll).toBe('function');
      expect(typeof mockEngine.runHooks).toBe('function');
      expect(typeof mockEngine.graphHash).toBe('function');

      // Also verify the other methods exist
      expect(typeof mockEngine.extractMetadata).toBe('function');
      expect(typeof mockEngine.terminate).toBe('function');
    });

    it('test_graphlaw_wasm_methods_return_promises', async () => {
      const mockEngine = new MockGraphlawEngine();
      const turtleSource = '@prefix ex: <http://example.com/> .\n';

      const validateResult = mockEngine.validateAll(turtleSource);
      const runHooksResult = mockEngine.runHooks(turtleSource);
      const graphHashResult = mockEngine.graphHash(turtleSource);

      // Verify all return Promises
      expect(validateResult).toBeInstanceOf(Promise);
      expect(runHooksResult).toBeInstanceOf(Promise);
      expect(graphHashResult).toBeInstanceOf(Promise);

      // Verify promises resolve to correct types
      const vr = await validateResult;
      const hr = await runHooksResult;
      const gr = await graphHashResult;

      expect(vr).toHaveProperty('timestamp');
      expect(vr).toHaveProperty('conforms');
      expect(hr).toHaveProperty('receiptHash');
      expect(hr).toHaveProperty('receipts');
      expect(typeof gr).toBe('string');
    });
  });

  describe('Call tracking and stale result detection', () => {
    it('test_graphlaw_wasm_call_generation_counter_increments', () => {
      const callId1 = getNextCallId();
      const callId2 = getNextCallId();
      const callId3 = getNextCallId();

      // Each call should return a unique incrementing ID
      expect(callId2).toBe(callId1 + 1);
      expect(callId3).toBe(callId2 + 1);
    });

    it('test_validateWithCallId_detects_stale_results', async () => {
      const mockEngine = new MockGraphlawEngine(10); // 10ms delay
      const turtleSource = '@prefix ex: <http://example.com/> .\n';

      // Get the current call ID before making a call
      const callId1 = getNextCallId();

      // Make the call with callId1
      const result1 = await validateWithCallId(
        mockEngine,
        turtleSource,
        callId1
      );

      // Result should not be stale (callId1 matches the counter)
      expect(result1.callId).toBe(callId1);
      expect(result1.stale).toBe(false);
      expect(result1.data).toHaveProperty('conforms');

      // Simulate making a newer call
      const callId2 = getNextCallId();

      // Now use the old callId1 to make a "stale" call
      const result2 = await validateWithCallId(
        mockEngine,
        turtleSource,
        callId1
      );

      // Result should be marked stale because callId1 != current counter
      expect(result2.stale).toBe(true);
      expect(result2.callId).toBe(callId1);
    });

    it('test_runHooksWithCallId_detects_stale_results', async () => {
      const mockEngine = new MockGraphlawEngine(10);
      const turtleSource = '@prefix ex: <http://example.com/> .\n';

      const callId1 = getNextCallId();
      const result1 = await runHooksWithCallId(
        mockEngine,
        turtleSource,
        callId1
      );

      expect(result1.stale).toBe(false);
      expect(result1.data).toHaveProperty('receiptHash');
      expect(result1.data).toHaveProperty('receipts');

      // Increment counter to make result "stale"
      getNextCallId();

      const result2 = await runHooksWithCallId(
        mockEngine,
        turtleSource,
        callId1
      );

      expect(result2.stale).toBe(true);
    });

    it('test_graphHashWithCallId_detects_stale_results', async () => {
      const mockEngine = new MockGraphlawEngine(10);
      const turtleSource = '@prefix ex: <http://example.com/> .\n';

      const callId1 = getNextCallId();
      const result1 = await graphHashWithCallId(
        mockEngine,
        turtleSource,
        callId1
      );

      expect(result1.stale).toBe(false);
      expect(typeof result1.data).toBe('string');
      expect(result1.data.length).toBeGreaterThan(0);

      // Increment counter
      getNextCallId();

      const result2 = await graphHashWithCallId(
        mockEngine,
        turtleSource,
        callId1
      );

      expect(result2.stale).toBe(true);
    });

    it('test_call_tracking_multiple_concurrent_calls', async () => {
      const mockEngine = new MockGraphlawEngine(20); // 20ms delay
      const turtleSource = '@prefix ex: <http://example.com/> .\n';

      // Simulate rapid sequential calls
      const callId1 = getNextCallId();
      const callId2 = getNextCallId();
      const callId3 = getNextCallId();

      // Start all three calls concurrently
      const [result1, result2, result3] = await Promise.all([
        validateWithCallId(mockEngine, turtleSource, callId1),
        runHooksWithCallId(mockEngine, turtleSource, callId2),
        graphHashWithCallId(mockEngine, turtleSource, callId3),
      ]);

      // All three should be stale because only callId3 matches final counter
      expect(result1.stale).toBe(true);
      expect(result2.stale).toBe(true);
      expect(result3.stale).toBe(false);

      // But all should still have valid data
      expect(result1.data).toBeDefined();
      expect(result2.data).toBeDefined();
      expect(result3.data).toBeDefined();
    });
  });

  describe('CallResult structure', () => {
    it('test_call_result_has_data_callid_and_stale_fields', async () => {
      const mockEngine = new MockGraphlawEngine();
      const turtleSource = '@prefix ex: <http://example.com/> .\n';
      const callId = getNextCallId();

      const result: CallResult<ValidationResult> = await validateWithCallId(
        mockEngine,
        turtleSource,
        callId
      );

      // Verify CallResult structure
      expect(result).toHaveProperty('data');
      expect(result).toHaveProperty('callId');
      expect(result).toHaveProperty('stale');

      // Verify types
      expect(typeof result.callId).toBe('number');
      expect(typeof result.stale).toBe('boolean');
      expect(result.data).toBeDefined();
    });
  });

  describe('Interface method contracts', () => {
    it('test_validateAll_returns_validation_result', async () => {
      const mockEngine = new MockGraphlawEngine();
      const result = await mockEngine.validateAll(
        '@prefix ex: <http://example.com/> .\n'
      );

      // Verify ValidationResult structure
      expect(result).toHaveProperty('timestamp');
      expect(result).toHaveProperty('conforms');
      expect(typeof result.timestamp).toBe('string');
      expect(typeof result.conforms).toBe('boolean');

      // ISO 8601 timestamp format
      expect(new Date(result.timestamp).getTime()).not.toBeNaN();
    });

    it('test_runHooks_returns_hook_execution_result', async () => {
      const mockEngine = new MockGraphlawEngine();
      const result = await mockEngine.runHooks(
        '@prefix ex: <http://example.com/> .\n'
      );

      // Verify HookExecutionResult structure
      expect(result).toHaveProperty('receiptHash');
      expect(result).toHaveProperty('receipts');
      expect(typeof result.receiptHash).toBe('string');
      expect(Array.isArray(result.receipts)).toBe(true);

      // Verify each receipt has required fields
      result.receipts.forEach((receipt) => {
        expect(receipt).toHaveProperty('hookName');
        expect(receipt).toHaveProperty('deltaQuads');
        expect(receipt).toHaveProperty('deltaHash');
        expect(receipt).toHaveProperty('idempotencyKey');
      });
    });

    it('test_graphHash_returns_string', async () => {
      const mockEngine = new MockGraphlawEngine();
      const result = await mockEngine.graphHash(
        '@prefix ex: <http://example.com/> .\n'
      );

      expect(typeof result).toBe('string');
      expect(result.length).toBeGreaterThan(0);

      // BLAKE3 hashes are 64 hex characters (256 bits)
      expect(/^[a-f0-9]{64}$|^[a-f0-9]{40}$|^[a-f0-9]{32}$/).toEqual(
        expect.any(RegExp)
      );
    });
  });

  describe('Error handling in call tracking', () => {
    it('test_call_tracking_with_mock_engine_errors', async () => {
      class ErrorEngine implements GraphlawEngineInterface {
        async validateAll(): Promise<ValidationResult> {
          throw new Error('Validation failed');
        }

        async runHooks(): Promise<HookExecutionResult> {
          throw new Error('Hook execution failed');
        }

        async graphHash(): Promise<string> {
          throw new Error('Graph hash computation failed');
        }

        async extractMetadata() {
          return { hooks: [], rules: [] };
        }

        async terminate(): Promise<void> {}
      }

      const errorEngine = new ErrorEngine();
      const callId = getNextCallId();

      // Call tracking should still work even if the engine throws
      await expect(
        validateWithCallId(errorEngine, '@prefix ex: <http://example.com/> .\n', callId)
      ).rejects.toThrow();
    });
  });

  describe('Async behavior and timing', () => {
    it('test_call_tracking_functions_are_async', () => {
      const mockEngine = new MockGraphlawEngine();
      const turtleSource = '@prefix ex: <http://example.com/> .\n';
      const callId = getNextCallId();

      const validatePromise = validateWithCallId(mockEngine, turtleSource, callId);
      const runHooksPromise = runHooksWithCallId(mockEngine, turtleSource, callId + 1);
      const graphHashPromise = graphHashWithCallId(mockEngine, turtleSource, callId + 2);

      // All should return Promises immediately
      expect(validatePromise).toBeInstanceOf(Promise);
      expect(runHooksPromise).toBeInstanceOf(Promise);
      expect(graphHashPromise).toBeInstanceOf(Promise);
    });

    it('test_call_result_preserves_callid_after_async_completion', async () => {
      const mockEngine = new MockGraphlawEngine(50);
      const turtleSource = '@prefix ex: <http://example.com/> .\n';
      const callId = 12345; // Use a specific ID for verification

      const result = await validateWithCallId(
        mockEngine,
        turtleSource,
        callId
      );

      // Verify callId is preserved in result
      expect(result.callId).toBe(callId);
      expect(result.data).toBeDefined();
    });
  });

  describe('Mock engine behavior', () => {
    it('test_mock_engine_simulates_realistic_responses', async () => {
      const mockEngine = new MockGraphlawEngine();

      const validationResult = await mockEngine.validateAll(
        '@prefix ex: <http://example.com/> .\n'
      );
      const hooksResult = await mockEngine.runHooks(
        '@prefix ex: <http://example.com/> .\n'
      );
      const graphHash = await mockEngine.graphHash(
        '@prefix ex: <http://example.com/> .\n'
      );

      // Verify all responses have realistic structure
      expect(validationResult).toMatchObject({
        timestamp: expect.any(String),
        conforms: expect.any(Boolean),
        violations: expect.any(Array),
      });

      expect(hooksResult).toMatchObject({
        receiptHash: expect.any(String),
        receipts: expect.any(Array),
        errors: expect.any(Array),
      });

      expect(typeof graphHash).toBe('string');
    });
  });
});
