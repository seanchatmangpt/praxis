'use client';

import React, { useState } from 'react';
import { PlaygroundResult } from '@/lib/types';

interface ReceiptPanelProps {
  result?: PlaygroundResult | null;
}

export default function ReceiptPanel({ result }: ReceiptPanelProps) {
  const [showRaw, setShowRaw] = useState(false);

  if (!result) {
    return (
      <div style={{ padding: '1rem', color: '#999' }}>
        Run dialects to view receipt information.
      </div>
    );
  }

  return (
    <div style={{ padding: '1rem', border: '1px solid #ddd', borderRadius: '4px', backgroundColor: '#fafafa' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
        <h3 style={{ margin: 0 }}>Receipt Information</h3>
        <button
          onClick={() => setShowRaw(!showRaw)}
          style={{
            padding: '6px 12px',
            border: '1px solid #ddd',
            borderRadius: '3px',
            backgroundColor: '#fff',
            cursor: 'pointer',
            fontSize: '12px',
          }}
        >
          {showRaw ? 'Hide' : 'Show'} Raw JSON
        </button>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1rem' }}>
        <div style={{ padding: '12px', backgroundColor: '#fff', borderRadius: '4px', border: '1px solid #ddd' }}>
          <div style={{ fontSize: '12px', color: '#999', marginBottom: '4px' }}>Graph Hash</div>
          <div
            style={{
              fontFamily: 'monospace',
              fontSize: '14px',
              fontWeight: 'bold',
              wordBreak: 'break-all',
              color: '#0070f3',
            }}
          >
            {result.graph_hash}
          </div>
        </div>

        <div style={{ padding: '12px', backgroundColor: '#fff', borderRadius: '4px', border: '1px solid #ddd' }}>
          <div style={{ fontSize: '12px', color: '#999', marginBottom: '4px' }}>Profile Hash</div>
          <div
            style={{
              fontFamily: 'monospace',
              fontSize: '14px',
              fontWeight: 'bold',
              wordBreak: 'break-all',
              color: '#0070f3',
            }}
          >
            {result.profile_hash}
          </div>
        </div>
      </div>

      <div style={{ padding: '12px', backgroundColor: '#fff', borderRadius: '4px', border: '1px solid #ddd', marginBottom: '1rem' }}>
        <div style={{ fontSize: '12px', color: '#999', marginBottom: '8px', fontWeight: 'bold' }}>Hash Algorithms</div>
        <div>
          {Object.entries(result.hash_algorithms).map(([algo, version]) => (
            <div key={algo} style={{ fontSize: '13px', display: 'flex', justifyContent: 'space-between' }}>
              <span>{algo}</span>
              <code style={{ color: '#666' }}>{version}</code>
            </div>
          ))}
        </div>
      </div>

      {showRaw && (
        <div style={{ padding: '12px', backgroundColor: '#f0f0f0', borderRadius: '4px', border: '1px solid #ddd' }}>
          <pre
            style={{
              fontSize: '11px',
              overflow: 'auto',
              maxHeight: '300px',
              margin: 0,
              color: '#333',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
            }}
          >
            {JSON.stringify(result, null, 2)}
          </pre>
        </div>
      )}

      <div style={{ marginTop: '1rem', fontSize: '12px', color: '#999', borderTop: '1px solid #ddd', paddingTop: '1rem' }}>
        <strong>Replay Verification:</strong> {result.replay.status}
      </div>
    </div>
  );
}
