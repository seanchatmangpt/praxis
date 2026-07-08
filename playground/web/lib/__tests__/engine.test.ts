/**
 * Unit tests for lib/engine.ts
 *
 * Tests the mock engine abstraction layer that provides a consistent interface
 * for playground operations. Verifies that PlaygroundResult structures are
 * well-formed and async methods behave correctly.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  createMockResult,
  runAllDialects,
  runHooks,
  replayVerify,
} from '../engine';
import { PlaygroundResult, Status, HookRunResult } from '../types';

describe('engine.ts', () => {
  describe('createMockResult()', () => {
    it('test_mock_engine_returns_valid_playground_result', () => {
      const result = createMockResult();

      // Verify top-level structure
      expect(result).toHaveProperty('graph_hash');
      expect(result).toHaveProperty('profile_hash');
      expect(result).toHaveProperty('dialects');
      expect(result).toHaveProperty('hooks');
      expect(result).toHaveProperty('replay');
      expect(result).toHaveProperty('hash_algorithms');

      // Verify graph_hash is non-empty string
      expect(typeof result.graph_hash).toBe('string');
      expect(result.graph_hash.length).toBeGreaterThan(0);

      // Verify profile_hash is non-empty string
      expect(typeof result.profile_hash).toBe('string');
      expect(result.profile_hash.length).toBeGreaterThan(0);

      // Verify dialects is an array
      expect(Array.isArray(result.dialects)).toBe(true);
      expect(result.dialects.length).toBeGreaterThan(0);

      // Verify each dialect has required fields
      result.dialects.forEach((dialect) => {
        expect(dialect).toHaveProperty('dialect');
        expect(dialect).toHaveProperty('status');
        expect(dialect).toHaveProperty('detail');
        expect(dialect).toHaveProperty('triples_out');
        expect(typeof dialect.dialect).toBe('string');
        expect(typeof dialect.detail).toBe('string');
        expect(typeof dialect.triples_out).toBe('number');
      });

      // Verify hooks structure
      expect(result.hooks).toHaveProperty('status');
      const validStatuses: Status[] = [
        'ADMITTED',
        'REFUSED',
        'UNSUPPORTED',
        'REPLAY_MISMATCH',
        'HASH_MISMATCH',
        'PROFILE_NOT_ADMITTED',
      ];
      expect(validStatuses).toContain(result.hooks.status);

      // Verify hash_algorithms
      expect(typeof result.hash_algorithms).toBe('object');
      expect(result.hash_algorithms).toHaveProperty('BLAKE3');
    });

    it('test_mock_engine_hook_run_result', () => {
      const result = createMockResult();
      const hookResult: HookRunResult = result.hooks;

      // Verify HookRunResult structure
      expect(hookResult).toHaveProperty('status');
      expect(hookResult).toHaveProperty('verdicts');
      expect(hookResult).toHaveProperty('schedule');
      expect(hookResult).toHaveProperty('receipts');

      // Verify status is a valid Status value
      const validStatuses: Status[] = [
        'ADMITTED',
        'REFUSED',
        'UNSUPPORTED',
        'REPLAY_MISMATCH',
        'HASH_MISMATCH',
        'PROFILE_NOT_ADMITTED',
      ];
      expect(validStatuses).toContain(hookResult.status);

      // Verify verdicts is an array
      expect(Array.isArray(hookResult.verdicts)).toBe(true);

      // Verify each verdict has required fields
      hookResult.verdicts.forEach((verdict) => {
        expect(verdict).toHaveProperty('hook_id');
        expect(verdict).toHaveProperty('hook_iri');
        expect(verdict).toHaveProperty('hook_name');
        expect(verdict).toHaveProperty('condition_kind');
        expect(verdict).toHaveProperty('condition_hash');
        expect(verdict).toHaveProperty('verdict');
        expect(verdict).toHaveProperty('effect');
        expect(typeof verdict.hook_id).toBe('number');
        expect(typeof verdict.hook_iri).toBe('string');
        expect(typeof verdict.hook_name).toBe('string');
      });

      // Verify schedule is an array of strings
      expect(Array.isArray(hookResult.schedule)).toBe(true);
      hookResult.schedule.forEach((item) => {
        expect(typeof item).toBe('string');
      });

      // Verify receipts is an array
      expect(Array.isArray(hookResult.receipts)).toBe(true);
      hookResult.receipts.forEach((receipt) => {
        expect(receipt).toHaveProperty('hook_name');
        expect(receipt).toHaveProperty('delta_hash');
        expect(receipt).toHaveProperty('idempotency_key');
        expect(receipt).toHaveProperty('delta_quads');
        expect(typeof receipt.hook_name).toBe('string');
        expect(typeof receipt.delta_hash).toBe('string');
        expect(typeof receipt.idempotency_key).toBe('string');
        expect(typeof receipt.delta_quads).toBe('string');
      });
    });

    it('test_mock_engine_custom_hash_parameter', () => {
      const customHash = 'deadbeef1234567890';
      const result = createMockResult(customHash);

      expect(result.graph_hash).toBe(customHash);
    });

    it('test_mock_engine_custom_dialects_parameter', () => {
      const customDialects = ['OWL_RL', 'SHACL'];
      const result = createMockResult('hash123', customDialects);

      expect(result.dialects.length).toBe(customDialects.length);
      result.dialects.forEach((dialect, index) => {
        expect(dialect.dialect).toBe(customDialects[index]);
      });
    });
  });

  describe('runAllDialects()', () => {
    it('test_engine_methods_are_async_runAllDialects', async () => {
      const sampleTurtle = '@prefix ex: <http://example.com/> .\n';
      const result = runAllDialects(sampleTurtle);

      // Verify it returns a Promise
      expect(result).toBeInstanceOf(Promise);

      // Await and verify the result is a PlaygroundResult
      const resolved = await result;
      expect(resolved).toHaveProperty('graph_hash');
      expect(resolved).toHaveProperty('hooks');
      expect(resolved).toHaveProperty('dialects');
    });

    it('test_runAllDialects_accepts_turtle_content', async () => {
      const turtleContent =
        '@prefix ex: <http://example.com/> .\nex:subject ex:predicate ex:object .';
      const result = await runAllDialects(turtleContent);

      expect(result).toHaveProperty('graph_hash');
      expect(typeof result.graph_hash).toBe('string');
    });
  });

  describe('runHooks()', () => {
    it('test_engine_methods_are_async_runHooks', async () => {
      const sampleTurtle = '@prefix ex: <http://example.com/> .\n';
      const result = runHooks(sampleTurtle);

      // Verify it returns a Promise
      expect(result).toBeInstanceOf(Promise);

      // Await and verify the result has expected structure
      const resolved = await result;
      expect(resolved).toHaveProperty('hooks');
      expect(resolved.hooks).toHaveProperty('status');
      expect(resolved.hooks).toHaveProperty('verdicts');
      expect(resolved.hooks).toHaveProperty('schedule');
    });

    it('test_mock_engine_hook_run_result_from_runHooks', async () => {
      const result = await runHooks('@prefix ex: <http://example.com/> .\n');
      const hookResult = result.hooks;

      // Verify HookRunResult has required structure
      expect(hookResult).toHaveProperty('status');
      expect(Array.isArray(hookResult.verdicts)).toBe(true);
      expect(Array.isArray(hookResult.schedule)).toBe(true);
      expect(Array.isArray(hookResult.receipts)).toBe(true);
    });
  });

  describe('replayVerify()', () => {
    it('test_replayVerify_returns_promise', async () => {
      const sampleTurtle = '@prefix ex: <http://example.com/> .\n';
      const result = replayVerify(sampleTurtle);

      // Verify it returns a Promise
      expect(result).toBeInstanceOf(Promise);

      // Await and verify the result has replay field
      const resolved = await result;
      expect(resolved).toHaveProperty('replay');
      expect(resolved.replay).toHaveProperty('status');
      expect(resolved.replay).toHaveProperty('first_hash');
      expect(resolved.replay).toHaveProperty('second_hash');
    });

    it('test_replayVerify_status_is_admitted_or_mismatch', async () => {
      const result = await replayVerify('@prefix ex: <http://example.com/> .\n');

      // Status should be either ADMITTED or REPLAY_MISMATCH
      const validReplayStatuses = ['ADMITTED', 'REPLAY_MISMATCH'];
      expect(validReplayStatuses).toContain(result.replay.status);

      // Hashes should be strings
      expect(typeof result.replay.first_hash).toBe('string');
      expect(typeof result.replay.second_hash).toBe('string');
    });
  });

  describe('async behavior across all methods', () => {
    it('test_all_engine_methods_return_promises', async () => {
      const sampleTurtle = '@prefix ex: <http://example.com/> .\n';

      const p1 = runAllDialects(sampleTurtle);
      const p2 = runHooks(sampleTurtle);
      const p3 = replayVerify(sampleTurtle);

      expect(p1).toBeInstanceOf(Promise);
      expect(p2).toBeInstanceOf(Promise);
      expect(p3).toBeInstanceOf(Promise);

      // All should resolve successfully
      const [r1, r2, r3] = await Promise.all([p1, p2, p3]);
      expect(r1).toHaveProperty('graph_hash');
      expect(r2).toHaveProperty('hooks');
      expect(r3).toHaveProperty('replay');
    });

    it('test_engine_methods_simulate_latency', async () => {
      const sampleTurtle = '@prefix ex: <http://example.com/> .\n';

      // Measure runAllDialects (500ms simulated)
      const start1 = Date.now();
      await runAllDialects(sampleTurtle);
      const elapsed1 = Date.now() - start1;
      expect(elapsed1).toBeGreaterThanOrEqual(400); // Allow some variance

      // Measure runHooks (300ms simulated)
      const start2 = Date.now();
      await runHooks(sampleTurtle);
      const elapsed2 = Date.now() - start2;
      expect(elapsed2).toBeGreaterThanOrEqual(200); // Allow some variance

      // Measure replayVerify (800ms simulated)
      const start3 = Date.now();
      await replayVerify(sampleTurtle);
      const elapsed3 = Date.now() - start3;
      expect(elapsed3).toBeGreaterThanOrEqual(700); // Allow some variance
    });
  });

  describe('PlaygroundResult type validation', () => {
    it('test_playground_result_has_all_required_fields', async () => {
      const result = await runAllDialects('@prefix ex: <http://example.com/> .\n');

      // Type-check via runtime assertions
      const requiredFields: (keyof PlaygroundResult)[] = [
        'graph_hash',
        'profile_hash',
        'dialects',
        'hooks',
        'replay',
        'hash_algorithms',
      ];

      requiredFields.forEach((field) => {
        expect(result).toHaveProperty(field);
      });
    });
  });
});
