/**
 * Tests for ReceiptPanel component.
 * Verifies display of hash algorithms, graph/profile hashes, and replay status.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ReceiptPanel from '../ReceiptPanel';
import type { PlaygroundResult } from '@/lib/types';

describe('ReceiptPanel', () => {
  const mockPlaygroundResult: PlaygroundResult = {
    graph_hash: 'graph-hash-blake3-abc123def456789',
    profile_hash: 'profile-hash-blake3-xyz987uvw654321',
    dialects: [],
    hooks: {
      status: 'ADMITTED',
      verdicts: [],
      receipts: [],
      schedule: [],
    },
    replay: {
      status: 'ADMITTED',
      first_hash: 'replay-hash-1-abc123',
      second_hash: 'replay-hash-1-abc123',
    },
    hash_algorithms: {
      graph: 'blake3',
      condition: 'sha256',
      profile: 'blake3',
    },
  };

  beforeEach(() => {
    // Reset any component state
  });

  it('test_renders_hash_algorithms_field', () => {
    render(<ReceiptPanel result={mockPlaygroundResult} />);

    // Verify hash algorithms are visible by checking for the section header
    expect(screen.getByText('Hash Algorithms')).toBeInTheDocument();

    // Check that the algorithm names are in the document
    expect(screen.getByText('graph')).toBeInTheDocument();
    expect(screen.getByText('condition')).toBeInTheDocument();
    expect(screen.getByText('profile')).toBeInTheDocument();

    // Check for at least one blake3 algorithm version (there may be multiple)
    const blake3Elements = screen.getAllByText('blake3');
    expect(blake3Elements.length).toBeGreaterThan(0);

    // Check for sha256
    const sha256Elements = screen.getAllByText('sha256');
    expect(sha256Elements.length).toBeGreaterThan(0);
  });

  it('test_shows_graph_and_profile_hash', () => {
    render(<ReceiptPanel result={mockPlaygroundResult} />);

    // Verify hash values are displayed
    expect(screen.getByText('graph-hash-blake3-abc123def456789')).toBeInTheDocument();
    expect(screen.getByText('profile-hash-blake3-xyz987uvw654321')).toBeInTheDocument();

    // Verify hash labels are displayed
    expect(screen.getByText('Graph Hash')).toBeInTheDocument();
    expect(screen.getByText('Profile Hash')).toBeInTheDocument();
  });

  it('test_replay_badge_stable', () => {
    const { container } = render(<ReceiptPanel result={mockPlaygroundResult} />);

    // Verify replay status is displayed
    const fullText = container.textContent;
    expect(fullText).toContain('Replay Verification:');
    expect(fullText).toContain('ADMITTED');
  });

  it('test_replay_badge_mismatch', () => {
    const resultWithMismatch: PlaygroundResult = {
      ...mockPlaygroundResult,
      replay: {
        status: 'HASH_MISMATCH',
        first_hash: 'replay-hash-1-abc123',
        second_hash: 'replay-hash-2-def456',
      },
    };

    const { container } = render(<ReceiptPanel result={resultWithMismatch} />);

    // Verify mismatch status is displayed
    const fullText = container.textContent;
    expect(fullText).toContain('Replay Verification:');
    expect(fullText).toContain('HASH_MISMATCH');
  });

  it('test_renders_null_result_gracefully', () => {
    render(<ReceiptPanel result={null} />);

    expect(screen.getByText('Run dialects to view receipt information.')).toBeInTheDocument();
  });

  it('test_show_raw_json_button_toggles', () => {
    render(<ReceiptPanel result={mockPlaygroundResult} />);

    // Initially, raw JSON should not be visible
    expect(screen.queryByText(/graph_hash/)).not.toBeInTheDocument();

    // Click "Show Raw JSON" button
    const showButton = screen.getByText('Show Raw JSON');
    fireEvent.click(showButton);

    // Raw JSON should now be visible
    expect(screen.getByText(/graph_hash/)).toBeInTheDocument();

    // Click "Hide Raw JSON" button
    const hideButton = screen.getByText('Hide Raw JSON');
    fireEvent.click(hideButton);

    // Raw JSON should be hidden again
    expect(screen.queryByText(/graph_hash/)).not.toBeInTheDocument();
  });

  it('test_hash_algorithms_section_label', () => {
    render(<ReceiptPanel result={mockPlaygroundResult} />);

    // Verify section header is present
    expect(screen.getByText('Hash Algorithms')).toBeInTheDocument();
  });

  it('test_renders_receipt_information_header', () => {
    render(<ReceiptPanel result={mockPlaygroundResult} />);

    expect(screen.getByText('Receipt Information')).toBeInTheDocument();
  });

  it('test_all_hash_algorithms_types_displayed', () => {
    const resultWithMultipleAlgos: PlaygroundResult = {
      ...mockPlaygroundResult,
      hash_algorithms: {
        graph: 'blake3',
        condition: 'sha256',
        profile: 'blake3',
        fact_set: 'blake3',
      },
    };

    render(<ReceiptPanel result={resultWithMultipleAlgos} />);

    // Verify all algorithm types are displayed
    expect(screen.getByText('graph')).toBeInTheDocument();
    expect(screen.getByText('condition')).toBeInTheDocument();
    expect(screen.getByText('profile')).toBeInTheDocument();
    expect(screen.getByText('fact_set')).toBeInTheDocument();

    // Verify algorithm versions are displayed (may appear multiple times)
    const blake3Elements = screen.getAllByText('blake3');
    expect(blake3Elements.length).toBeGreaterThanOrEqual(3);

    const sha256Elements = screen.getAllByText('sha256');
    expect(sha256Elements.length).toBeGreaterThan(0);
  });
});
