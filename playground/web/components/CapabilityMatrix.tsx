'use client';

import React from 'react';
import { PlaygroundResult, Status } from '@/lib/types';

interface CapabilityMatrixProps {
  result?: PlaygroundResult | null;
  isLoading?: boolean;
}

const CAPABILITIES = ['Turtle', 'OWL-RL', 'SHACL', 'ShEx', 'Datalog', 'N3 Denials', 'Hooks'];

const STATUS_DISPLAY: Record<Status, string> = {
  ADMITTED: 'Admitted',
  REFUSED: 'Refused',
  UNSUPPORTED: 'Unsupported',
  REPLAY_MISMATCH: 'Replay Mismatch',
  HASH_MISMATCH: 'Hash Mismatch',
  PROFILE_NOT_ADMITTED: 'Profile Not Admitted',
};

function StatusChip({ status, label }: { status: Status; label: string }) {
  const statusConfig: Record<Status, { color: string; bgColor: string }> = {
    ADMITTED: { color: '#fff', bgColor: '#22c55e' },
    REFUSED: { color: '#fff', bgColor: '#ef4444' },
    UNSUPPORTED: { color: '#000', bgColor: '#eab308' },
    REPLAY_MISMATCH: { color: '#fff', bgColor: '#f97316' },
    HASH_MISMATCH: { color: '#fff', bgColor: '#ef4444' },
    PROFILE_NOT_ADMITTED: { color: '#fff', bgColor: '#f97316' },
  };

  const config = statusConfig[status];

  return (
    <span
      style={{
        display: 'inline-block',
        padding: '4px 8px',
        borderRadius: '3px',
        backgroundColor: config.bgColor,
        color: config.color,
        fontSize: '12px',
        fontWeight: 'bold',
      }}
    >
      {label}
    </span>
  );
}

export default function CapabilityMatrix({ result, isLoading }: CapabilityMatrixProps) {
  return (
    <div style={{ padding: '1rem', border: '1px solid #ddd', borderRadius: '4px', backgroundColor: '#fafafa' }}>
      <h3 style={{ marginTop: 0 }}>Capability Matrix</h3>
      {isLoading && <p style={{ color: '#666' }}>Running dialects...</p>}
      
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr style={{ borderBottom: '2px solid #ddd' }}>
            <th style={{ textAlign: 'left', padding: '8px', fontWeight: 'bold' }}>Capability</th>
            <th style={{ textAlign: 'center', padding: '8px', fontWeight: 'bold' }}>Status</th>
            <th style={{ textAlign: 'center', padding: '8px', fontWeight: 'bold' }}>Triples</th>
            <th style={{ textAlign: 'center', padding: '8px', fontWeight: 'bold' }}>Hash</th>
          </tr>
        </thead>
        <tbody>
          {CAPABILITIES.map((cap) => {
            const dialectData = result?.dialects.find(
              (d) => d.dialect.toUpperCase().replace(/-/g, '_') === cap.toUpperCase().replace(/ /g, '_')
            );

            return (
              <tr key={cap} style={{ borderBottom: '1px solid #eee' }}>
                <td style={{ padding: '8px', fontWeight: '500' }}>{cap}</td>
                <td style={{ padding: '8px', textAlign: 'center' }}>
                  {dialectData ? (
                    <StatusChip status={dialectData.status} label={STATUS_DISPLAY[dialectData.status]} />
                  ) : (
                    <span style={{ color: '#999' }}>–</span>
                  )}
                </td>
                <td style={{ padding: '8px', textAlign: 'center' }}>
                  {dialectData ? (
                    <code style={{ fontSize: '11px', color: '#666' }}>{dialectData.triples_out}</code>
                  ) : (
                    <span style={{ color: '#999' }}>–</span>
                  )}
                </td>
                <td style={{ padding: '8px', textAlign: 'center' }}>
                  {result?.graph_hash ? (
                    <code style={{ fontSize: '11px', color: '#666' }}>
                      {result.graph_hash.substring(0, 8)}…
                    </code>
                  ) : (
                    <span style={{ color: '#999' }}>–</span>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
