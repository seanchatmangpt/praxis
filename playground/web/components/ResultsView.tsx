'use client';

import React from 'react';
import type { PlaygroundResult } from '@/lib/types';

interface ResultsViewProps {
  result: PlaygroundResult | null;
  isLoading: boolean;
  error: string | null;
}

export function ResultsView({
  result,
  isLoading,
  error,
}: ResultsViewProps): React.ReactElement {
  if (isLoading) {
    return <div style={{ padding: '1rem' }}>Loading...</div>;
  }

  if (error) {
    return (
      <div style={{ padding: '1rem', color: 'red' }}>
        <strong>Error:</strong> {error}
      </div>
    );
  }

  if (!result) {
    return (
      <div style={{ padding: '1rem', color: '#666' }}>
        No results yet. Run the playground to see results.
      </div>
    );
  }

  return (
    <div style={{ padding: '1rem' }}>
      <h3>Playground Results</h3>
      <div>
        <p>Graph Hash: <code>{result.graph_hash}</code></p>
        <p>Profile Hash: <code>{result.profile_hash}</code></p>
        <p>Dialects Applied: {result.dialects.length}</p>
        <p>Hooks Executed: {result.hooks.verdicts.length}</p>
      </div>
    </div>
  );
}
