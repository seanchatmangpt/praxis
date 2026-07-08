/**
 * Tests for CapabilityMatrix component.
 * Verifies that the matrix renders all seven dialects in fixed order
 * with correct status colors and hash display.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import CapabilityMatrix from '../CapabilityMatrix';
import type { PlaygroundResult } from '@/lib/types';

describe('CapabilityMatrix', () => {
  const mockPlaygroundResult: PlaygroundResult = {
    graph_hash: 'abc123def456789',
    profile_hash: 'prof789hash123',
    dialects: [
      {
        dialect: 'Turtle',
        status: 'ADMITTED',
        detail: 'Parsed successfully',
        triples_out: 42,
      },
      {
        dialect: 'OWL_RL',
        status: 'ADMITTED',
        detail: 'Inference complete',
        triples_out: 128,
      },
      {
        dialect: 'SHACL',
        status: 'REFUSED',
        detail: 'Shape violation',
        triples_out: 0,
      },
      {
        dialect: 'ShEx',
        status: 'ADMITTED',
        detail: 'Shape valid',
        triples_out: 42,
      },
      {
        dialect: 'Datalog',
        status: 'ADMITTED',
        detail: 'Rules materialized',
        triples_out: 64,
      },
      {
        dialect: 'N3_Denials',
        status: 'UNSUPPORTED',
        detail: 'Not yet implemented',
        triples_out: 0,
      },
    ],
    hooks: {
      status: 'ADMITTED',
      verdicts: [],
      receipts: [],
      schedule: [],
    },
    replay: {
      status: 'ADMITTED',
      first_hash: 'hash1',
      second_hash: 'hash2',
    },
    hash_algorithms: {
      graph: 'blake3',
      profile: 'blake3',
    },
  };

  beforeEach(() => {
    // Reset any component state
  });

  it('test_renders_seven_rows', () => {
    render(<CapabilityMatrix result={mockPlaygroundResult} />);

    // Check all seven capability names are present
    expect(screen.getByText('Turtle')).toBeInTheDocument();
    expect(screen.getByText('OWL-RL')).toBeInTheDocument();
    expect(screen.getByText('SHACL')).toBeInTheDocument();
    expect(screen.getByText('ShEx')).toBeInTheDocument();
    expect(screen.getByText('Datalog')).toBeInTheDocument();
    expect(screen.getByText('N3 Denials')).toBeInTheDocument();
    expect(screen.getByText('Hooks')).toBeInTheDocument();

    // Verify we have 7 rows plus header (8 tr elements total)
    const rows = screen.getAllByRole('row');
    expect(rows).toHaveLength(8);
  });

  it('test_status_chip_colors', () => {
    render(<CapabilityMatrix result={mockPlaygroundResult} />);

    // ADMITTED status should render with green background (#22c55e)
    const admittedChips = screen.getAllByText('Admitted');
    expect(admittedChips.length).toBeGreaterThan(0);
    admittedChips.forEach((chip) => {
      expect(chip).toHaveStyle({ backgroundColor: '#22c55e' });
    });

    // REFUSED status should render with red background (#ef4444)
    const refusedChip = screen.getByText('Refused');
    expect(refusedChip).toHaveStyle({ backgroundColor: '#ef4444' });

    // UNSUPPORTED status should render with yellow background (#eab308)
    const unsupportedChip = screen.getByText('Unsupported');
    expect(unsupportedChip).toHaveStyle({ backgroundColor: '#eab308' });
  });

  it('test_rows_in_fixed_order', () => {
    render(<CapabilityMatrix result={mockPlaygroundResult} />);

    const capabilities = ['Turtle', 'OWL-RL', 'SHACL', 'ShEx', 'Datalog', 'N3 Denials', 'Hooks'];
    const rows = screen.getAllByRole('row');

    // Skip header row (index 0), then check data rows
    capabilities.forEach((cap, idx) => {
      const row = rows[idx + 1];
      expect(row).toHaveTextContent(cap);
    });
  });

  it('test_shows_graph_hash_truncated', () => {
    render(<CapabilityMatrix result={mockPlaygroundResult} />);

    // Hash should be truncated to first 8 chars with ellipsis
    // Use getAllByText because the hash appears multiple times in the table
    const hashElements = screen.getAllByText('abc123de…');
    expect(hashElements.length).toBeGreaterThan(0);
  });

  it('test_renders_null_result_gracefully', () => {
    const { container } = render(<CapabilityMatrix result={null} />);

    // Component should render without crashing
    expect(container).toBeInTheDocument();
    expect(screen.getByText('Capability Matrix')).toBeInTheDocument();
  });

  it('test_renders_loading_state', () => {
    render(<CapabilityMatrix result={null} isLoading={true} />);

    expect(screen.getByText('Running dialects...')).toBeInTheDocument();
  });
});
