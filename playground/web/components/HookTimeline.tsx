'use client';

import React from 'react';
import type { PlaygroundResult } from '@/lib/types';

interface HookTimelineProps {
  result?: PlaygroundResult | null;
}

function VerdictBadge({ verdict }: { verdict: string }) {
  const colors: Record<string, { bg: string; color: string }> = {
    FIRED: { bg: '#22c55e', color: '#fff' },
    GATED: { bg: '#f59e0b', color: '#fff' },
    NOT_FIRED: { bg: '#d1d5db', color: '#666' },
  };

  const style = colors[verdict] || { bg: '#f0f0f0', color: '#000' };

  return (
    <span
      style={{
        display: 'inline-block',
        padding: '4px 8px',
        borderRadius: '3px',
        backgroundColor: style.bg,
        color: style.color,
        fontSize: '12px',
        fontWeight: 'bold',
      }}
    >
      {verdict}
    </span>
  );
}

export default function HookTimeline({ result }: HookTimelineProps) {
  if (!result) {
    return (
      <div style={{ padding: '1rem', color: '#999' }}>
        Run dialects to see hook execution timeline.
      </div>
    );
  }

  const { schedule, verdicts } = result.hooks;
  const verdictMap = new Map(verdicts.map((v) => [v.hook_iri.split('/').pop() || v.hook_iri, v]));

  return (
    <div style={{ padding: '1rem', border: '1px solid #ddd', borderRadius: '4px', backgroundColor: '#fafafa' }}>
      <h3 style={{ marginTop: 0 }}>Hook Timeline</h3>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
        {schedule.map((hookName, idx) => {
          const verdict = verdictMap.get(hookName);
          const deltaHash = result.hooks.receipts[idx]?.delta_hash || '—';
          const idempotencyKey = result.hooks.receipts[idx]?.idempotency_key || '—';

          return (
            <div
              key={`${hookName}-${idx}`}
              style={{
                padding: '12px',
                border: '1px solid #ddd',
                borderRadius: '4px',
                backgroundColor: '#fff',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
              }}
            >
              <div>
                <div style={{ fontWeight: '600', marginBottom: '4px' }}>
                  {idx + 1}. {hookName}
                </div>
                <div style={{ fontSize: '12px', color: '#666' }}>
                  <div>Delta: <code>{deltaHash.substring(0, 8)}</code></div>
                  <div>Idempotency: <code>{idempotencyKey.substring(0, 8)}</code></div>
                </div>
              </div>
              <div>
                {verdict ? (
                  <VerdictBadge verdict={verdict.verdict} />
                ) : (
                  <span style={{ color: '#999', fontSize: '12px' }}>no verdict</span>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
