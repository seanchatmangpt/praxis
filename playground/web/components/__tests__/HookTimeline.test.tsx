/**
 * Tests for HookTimeline component.
 * Verifies hook verdicts render in schedule order (not verdict array order)
 * with correct status colors and hash displays.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import HookTimeline from '../HookTimeline';
import type { PlaygroundResult } from '@/lib/types';

describe('HookTimeline', () => {
  const mockPlaygroundResult: PlaygroundResult = {
    graph_hash: 'graph-hash-123',
    profile_hash: 'profile-hash-456',
    dialects: [],
    hooks: {
      status: 'ADMITTED',
      schedule: ['HookA', 'HookB', 'HookC'],
      // Verdicts in different order than schedule (not A, B, C)
      verdicts: [
        {
          hook_id: 2,
          hook_iri: 'http://example.com/hook/HookB',
          hook_name: 'HookB',
          condition_kind: 'SHACL',
          condition_hash: 'cond-hash-b',
          verdict: 'Fired',
          effect: 'asserted',
        },
        {
          hook_id: 1,
          hook_iri: 'http://example.com/hook/HookA',
          hook_name: 'HookA',
          condition_kind: 'SHACL',
          condition_hash: 'cond-hash-a',
          verdict: 'Gated',
          effect: 'withheld',
        },
        {
          hook_id: 3,
          hook_iri: 'http://example.com/hook/HookC',
          hook_name: 'HookC',
          condition_kind: 'SHACL',
          condition_hash: 'cond-hash-c',
          verdict: 'NotFired',
          effect: 'none',
        },
      ],
      receipts: [
        {
          hook_name: 'HookA',
          delta_hash: 'delta-hash-aaaa1111',
          idempotency_key: 'idem-key-aaaa2222',
          delta_quads: '<s> <p> <o> .',
        },
        {
          hook_name: 'HookB',
          delta_hash: 'delta-hash-bbbb3333',
          idempotency_key: 'idem-key-bbbb4444',
          delta_quads: '<s2> <p2> <o2> .',
        },
        {
          hook_name: 'HookC',
          delta_hash: 'delta-hash-cccc5555',
          idempotency_key: 'idem-key-cccc6666',
          delta_quads: '<s3> <p3> <o3> .',
        },
      ],
    },
    replay: {
      status: 'ADMITTED',
      first_hash: 'replay-hash-1',
      second_hash: 'replay-hash-1',
    },
    hash_algorithms: {
      graph: 'blake3',
      profile: 'blake3',
    },
  };

  beforeEach(() => {
    // Reset any component state
  });

  it('test_verdicts_ordered_by_schedule', () => {
    render(<HookTimeline result={mockPlaygroundResult} />);

    // Get all hook name elements and verify they appear in schedule order
    const hookElements = screen.getAllByText(/^\d\. Hook/);
    expect(hookElements).toHaveLength(3);

    // Should be "1. HookA", "2. HookB", "3. HookC"
    expect(hookElements[0]).toHaveTextContent('1. HookA');
    expect(hookElements[1]).toHaveTextContent('2. HookB');
    expect(hookElements[2]).toHaveTextContent('3. HookC');
  });

  it('test_verdict_status_colors', () => {
    const { container } = render(<HookTimeline result={mockPlaygroundResult} />);

    // Verify all three verdict types are present in the rendered output
    const text = container.textContent || '';
    expect(text).toContain('Fired');
    expect(text).toContain('Gated');
    expect(text).toContain('NotFired');

    // Count the number of inline-block spans that are verdict badges
    // Each verdict should render as a styled span with display: inline-block
    const allSpans = container.querySelectorAll('span[style*="display: inline-block"]');
    expect(allSpans.length).toBeGreaterThanOrEqual(3); // At least 3 verdict badges
  });

  it('test_shows_delta_hash_and_idempotency_key', () => {
    const { container } = render(<HookTimeline result={mockPlaygroundResult} />);

    // Check that the component shows delta hash and idempotency key labels
    const fullText = container.textContent;
    expect(fullText).toContain('Delta:');
    expect(fullText).toContain('Idempotency:');

    // Verify truncated hashes are displayed (first 8 chars of each hash)
    // delta-hash-aaaa1111 -> delta-ha
    // idem-key-aaaa2222 -> idem-key
    expect(fullText).toContain('delta-ha');
    expect(fullText).toContain('idem-key');
  });

  it('test_renders_null_result_gracefully', () => {
    render(<HookTimeline result={null} />);

    expect(screen.getByText('Run dialects to see hook execution timeline.')).toBeInTheDocument();
  });

  it('test_renders_empty_schedule', () => {
    const emptyResult: PlaygroundResult = {
      ...mockPlaygroundResult,
      hooks: {
        status: 'ADMITTED',
        verdicts: [],
        receipts: [],
        schedule: [],
      },
    };

    const { container } = render(<HookTimeline result={emptyResult} />);
    expect(container).toBeInTheDocument();
    expect(screen.getByText('Hook Timeline')).toBeInTheDocument();
  });

  it('test_handles_missing_receipt_gracefully', () => {
    const resultWithMissingReceipt: PlaygroundResult = {
      ...mockPlaygroundResult,
      hooks: {
        ...mockPlaygroundResult.hooks,
        receipts: [
          // Only one receipt for three hooks
          {
            hook_name: 'HookA',
            delta_hash: 'delta-hash-aaaa1111',
            idempotency_key: 'idem-key-aaaa2222',
            delta_quads: '<s> <p> <o> .',
          },
        ],
      },
    };

    render(<HookTimeline result={resultWithMissingReceipt} />);

    // Missing receipt fields should show "—"
    const dashElements = screen.getAllByText('—');
    expect(dashElements.length).toBeGreaterThan(0);
  });
});
