'use client';

import React from 'react';

interface ToolbarProps {
  onLoadCase?: () => void;
  onRunAllDialects?: () => void;
  onRunHooks?: () => void;
  onReplay?: () => void;
  onGenerateReport?: () => void;
  isLoading?: boolean;
}

export default function Toolbar({
  onLoadCase,
  onRunAllDialects,
  onRunHooks,
  onReplay,
  onGenerateReport,
  isLoading = false,
}: ToolbarProps) {
  const buttonStyle = (disabled: boolean = false) => ({
    padding: '10px 16px',
    marginRight: '8px',
    marginBottom: '8px',
    border: '1px solid #0070f3',
    borderRadius: '4px',
    backgroundColor: disabled ? '#f0f0f0' : '#0070f3',
    color: disabled ? '#999' : '#fff',
    cursor: disabled ? 'not-allowed' : 'pointer',
    fontSize: '14px',
    fontWeight: '600',
    transition: 'all 0.2s',
  });

  return (
    <div
      style={{
        padding: '1rem',
        backgroundColor: '#fafafa',
        borderBottom: '1px solid #ddd',
        display: 'flex',
        gap: '8px',
        flexWrap: 'wrap',
      }}
    >
      <button onClick={onLoadCase} style={buttonStyle(isLoading)} disabled={isLoading}>
        📂 Load Case
      </button>

      <button onClick={onRunAllDialects} style={buttonStyle(isLoading)} disabled={isLoading}>
        ▶ Run All Dialects
      </button>

      <button onClick={onRunHooks} style={buttonStyle(isLoading)} disabled={isLoading}>
        🪝 Run Hooks
      </button>

      <button onClick={onReplay} style={buttonStyle(isLoading)} disabled={isLoading}>
        🔄 Replay
      </button>

      <button onClick={onGenerateReport} style={buttonStyle(isLoading)} disabled={isLoading}>
        📊 Report
      </button>

      {isLoading && (
        <div style={{ display: 'flex', alignItems: 'center', marginLeft: 'auto', color: '#0070f3' }}>
          <span style={{ marginRight: '8px' }}>⏳</span>
          <span style={{ fontSize: '14px' }}>Processing...</span>
        </div>
      )}
    </div>
  );
}
