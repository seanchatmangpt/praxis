/**
 * Unit tests for lib/types.ts type definitions.
 * Verifies that Status enum, PlaygroundResult, and related types are correctly structured.
 */

import { describe, it, expect } from 'vitest';
import type {
  Status,
  DialectResult,
  PlaygroundResult,
  HookRunResult,
  ReplayResult,
  HookVerdictRecord,
  HookReceipt,
} from '../types';

describe('types.ts', () => {
  describe('Status enum coverage', () => {
    it('test_status_enum_all_values_are_strings', () => {
      const allStatusValues: Status[] = [
        'ADMITTED',
        'REFUSED',
        'UNSUPPORTED',
        'REPLAY_MISMATCH',
        'HASH_MISMATCH',
        'PROFILE_NOT_ADMITTED',
      ];

      expect(allStatusValues).toHaveLength(6);
      allStatusValues.forEach((status) => {
        expect(typeof status).toBe('string');
        expect(status.length).toBeGreaterThan(0);
      });
    });

    it('test_status_enum_distinct_values', () => {
      const statusValues = [
        'ADMITTED',
        'REFUSED',
        'UNSUPPORTED',
        'REPLAY_MISMATCH',
        'HASH_MISMATCH',
        'PROFILE_NOT_ADMITTED',
      ];
      const uniqueValues = new Set(statusValues);
      expect(uniqueValues.size).toBe(6);
    });

    it('test_status_enum_no_duplicates', () => {
      const statusValues = [
        'ADMITTED',
        'REFUSED',
        'UNSUPPORTED',
        'REPLAY_MISMATCH',
        'HASH_MISMATCH',
        'PROFILE_NOT_ADMITTED',
        'ADMITTED', // Duplicate
      ];
      const uniqueValues = new Set(statusValues);
      expect(uniqueValues.size).toBe(6);
    });
  });

  describe('PlaygroundResult structure', () => {
    it('test_playground_result_structure_complete', () => {
      const dialectResult: DialectResult = {
        dialect: 'OWL-RL',
        status: 'ADMITTED',
        detail: 'Successfully processed',
        triples_out: 42,
      };

      const hookReceipt: HookReceipt = {
        hook_name: 'test-hook',
        delta_hash: 'abc123def456',
        idempotency_key: 'idem-key-1',
        delta_quads: '<s> <p> <o> .',
      };

      const hookVerdictRecord: HookVerdictRecord = {
        hook_id: 1,
        hook_iri: 'http://example.com/hook/1',
        hook_name: 'TestHook',
        condition_kind: 'SHACL',
        condition_hash: 'hash123',
        verdict: 'Fired',
        effect: 'asserted',
      };

      const hookRunResult: HookRunResult = {
        status: 'ADMITTED',
        verdicts: [hookVerdictRecord],
        receipts: [hookReceipt],
        schedule: ['TestHook'],
      };

      const replayResult: ReplayResult = {
        status: 'ADMITTED',
        first_hash: 'hash1',
        second_hash: 'hash2',
      };

      const playgroundResult: PlaygroundResult = {
        graph_hash: 'graph-hash-abc',
        profile_hash: 'profile-hash-def',
        dialects: [dialectResult],
        hooks: hookRunResult,
        replay: replayResult,
        hash_algorithms: {
          graph: 'blake3',
          profile: 'blake3',
        },
      };

      // Verify structure by checking all required fields exist and have correct types
      expect(playgroundResult.graph_hash).toBe('graph-hash-abc');
      expect(playgroundResult.profile_hash).toBe('profile-hash-def');
      expect(playgroundResult.dialects).toHaveLength(1);
      expect(playgroundResult.dialects[0].status).toBe('ADMITTED');
      expect(playgroundResult.hooks.status).toBe('ADMITTED');
      expect(playgroundResult.replay.status).toBe('ADMITTED');
      expect(playgroundResult.hash_algorithms.graph).toBe('blake3');
    });

    it('test_playground_result_all_status_values_allowed', () => {
      const statusValues: Status[] = [
        'ADMITTED',
        'REFUSED',
        'UNSUPPORTED',
        'REPLAY_MISMATCH',
        'HASH_MISMATCH',
        'PROFILE_NOT_ADMITTED',
      ];

      statusValues.forEach((status) => {
        const result: PlaygroundResult = {
          graph_hash: 'hash1',
          profile_hash: 'hash2',
          dialects: [],
          hooks: {
            status,
            verdicts: [],
            receipts: [],
            schedule: [],
          },
          replay: {
            status,
            first_hash: 'h1',
            second_hash: 'h2',
          },
          hash_algorithms: {},
        };

        expect(result.hooks.status).toBe(status);
        expect(result.replay.status).toBe(status);
      });
    });

    it('test_playground_result_with_empty_collections', () => {
      const playgroundResult: PlaygroundResult = {
        graph_hash: 'hash',
        profile_hash: 'hash',
        dialects: [],
        hooks: {
          status: 'ADMITTED',
          verdicts: [],
          receipts: [],
          schedule: [],
        },
        replay: {
          status: 'ADMITTED',
          first_hash: 'h1',
          second_hash: 'h2',
        },
        hash_algorithms: {},
      };

      expect(playgroundResult.dialects).toHaveLength(0);
      expect(playgroundResult.hooks.verdicts).toHaveLength(0);
      expect(playgroundResult.hooks.receipts).toHaveLength(0);
      expect(playgroundResult.hooks.schedule).toHaveLength(0);
    });
  });

  describe('DialectResult structure', () => {
    it('test_dialect_result_required_fields', () => {
      const dialectResult: DialectResult = {
        dialect: 'SHACL',
        status: 'ADMITTED',
        detail: 'All shapes satisfied',
        triples_out: 100,
      };

      expect(dialectResult.dialect).toBeTruthy();
      expect(typeof dialectResult.dialect).toBe('string');
      expect(['ADMITTED', 'REFUSED', 'UNSUPPORTED']).toContain(dialectResult.status);
      expect(typeof dialectResult.detail).toBe('string');
      expect(typeof dialectResult.triples_out).toBe('number');
    });
  });

  describe('HookRunResult structure', () => {
    it('test_hook_run_result_with_verdicts', () => {
      const verdict: HookVerdictRecord = {
        hook_id: 1,
        hook_iri: 'http://example.com/hook/1',
        hook_name: 'FirstHook',
        condition_kind: 'SHACL',
        condition_hash: 'cond-hash',
        verdict: 'Fired',
        effect: 'asserted',
      };

      const hookRunResult: HookRunResult = {
        status: 'ADMITTED',
        verdicts: [verdict],
        receipts: [],
        schedule: ['FirstHook'],
      };

      expect(hookRunResult.verdicts).toHaveLength(1);
      expect(hookRunResult.verdicts[0].verdict).toBe('Fired');
      expect(hookRunResult.schedule).toContain('FirstHook');
    });
  });

  describe('ReplayResult structure', () => {
    it('test_replay_result_hash_equality', () => {
      const replayResult: ReplayResult = {
        status: 'ADMITTED',
        first_hash: 'abc123',
        second_hash: 'abc123',
      };

      expect(replayResult.first_hash).toBe(replayResult.second_hash);
    });

    it('test_replay_result_hash_mismatch', () => {
      const replayResult: ReplayResult = {
        status: 'HASH_MISMATCH',
        first_hash: 'abc123',
        second_hash: 'def456',
      };

      expect(replayResult.first_hash).not.toBe(replayResult.second_hash);
      expect(replayResult.status).toBe('HASH_MISMATCH');
    });
  });

  describe('Type structure validation', () => {
    it('test_playground_result_type_safety_with_all_fields', () => {
      const result: PlaygroundResult = {
        graph_hash: 'graph-hash',
        profile_hash: 'profile-hash',
        dialects: [
          {
            dialect: 'OWL-RL',
            status: 'ADMITTED',
            detail: 'OK',
            triples_out: 10,
          },
        ],
        hooks: {
          status: 'ADMITTED',
          verdicts: [
            {
              hook_id: 1,
              hook_iri: 'http://example.com/h1',
              hook_name: 'H1',
              condition_kind: 'SHACL',
              condition_hash: 'ch1',
              verdict: 'Fired',
              effect: 'asserted',
            },
          ],
          receipts: [
            {
              hook_name: 'H1',
              delta_hash: 'dh1',
              idempotency_key: 'ik1',
              delta_quads: '<s> <p> <o> .',
            },
          ],
          schedule: ['H1'],
        },
        replay: {
          status: 'ADMITTED',
          first_hash: 'h1',
          second_hash: 'h1',
        },
        hash_algorithms: {
          graph: 'blake3',
          profile: 'blake3',
          hooks: 'blake3',
        },
      };

      // Verify it's well-formed and all fields are present
      expect(result).toBeDefined();
      expect(result.dialects).toBeDefined();
      expect(result.hooks).toBeDefined();
      expect(result.replay).toBeDefined();
      expect(result.hash_algorithms).toBeDefined();
    });
  });
});
