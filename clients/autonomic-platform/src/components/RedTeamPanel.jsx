/**
 * RedTeamPanel.jsx
 * ----------------
 * UI component for running red-team mutators individually or all at once.
 * Displays:
 *   - List of 8 mutators with descriptions
 *   - Button to run each mutator
 *   - Button to run all mutators
 *   - Live result display (Status, passed/failed, error message)
 *   - Summary table
 */

import { useState } from 'react';
import {
  mutator_BrokenSyntax,
  mutator_UnsupportedOWLRL,
  mutator_RemoveSHACLProperty,
  mutator_WrongShExDatatype,
  mutator_OverflowHooks,
  mutator_UnknownPredicate,
  mutator_TamperAfterHash,
  mutator_N3Denial,
  executeAllMutators,
  summarizeMutatorResults,
} from '../lib/red-team-mutators.js';

const mono = "'JetBrains Mono', ui-monospace, monospace";
const sans = "'Space Grotesk', system-ui, sans-serif";

const MUTATORS = [
  { fn: mutator_BrokenSyntax, label: 'Break Syntax', id: 'mutator-syntax-break' },
  { fn: mutator_UnsupportedOWLRL, label: 'Unsupported OWL RL', id: 'mutator-unsupported-owl-rl' },
  { fn: mutator_RemoveSHACLProperty, label: 'Missing SHACL', id: 'mutator-missing-shacl-property' },
  { fn: mutator_WrongShExDatatype, label: 'Wrong Datatype', id: 'mutator-wrong-shex-datatype' },
  { fn: mutator_OverflowHooks, label: 'Overflow Hooks', id: 'mutator-overflow-hooks' },
  { fn: mutator_UnknownPredicate, label: 'Unknown Predicate', id: 'mutator-unknown-predicate' },
  { fn: mutator_TamperAfterHash, label: 'Hash Mismatch', id: 'mutator-hash-mismatch' },
  { fn: mutator_N3Denial, label: 'N3 Denial', id: 'mutator-n3-denial' },
];

export default function RedTeamPanel({ baseTurtle = '', validateFn = null, onResultsUpdate = null }) {
  const [results, setResults] = useState([]);
  const [running, setRunning] = useState(false);
  const [summary, setSummary] = useState(null);
  const [selectedResult, setSelectedResult] = useState(null);

  const runSingleMutator = async (mutator) => {
    if (!validateFn) {
      console.warn('validateFn not provided to RedTeamPanel');
      return;
    }

    setRunning(true);
    try {
      const mutatorDef = mutator.fn(baseTurtle);
      const actualStatus = await validateFn(mutatorDef.mutated).catch((err) => ({
        error: err.message,
        status: 'ExecutionError',
      }));

      const status = actualStatus.status || actualStatus;
      const passed = status === mutatorDef.expectedStatus;

      const result = {
        id: mutatorDef.id,
        name: mutator.label,
        mutation: mutatorDef.mutation,
        actualStatus: status,
        expectedStatus: mutatorDef.expectedStatus,
        passed,
        errorMessage: actualStatus.error || null,
      };

      setResults((prev) => {
        const updated = prev.filter((r) => r.id !== result.id);
        updated.push(result);
        return updated;
      });

      setSelectedResult(result);
      onResultsUpdate?.(results);
    } finally {
      setRunning(false);
    }
  };

  const runAllMutators = async () => {
    if (!validateFn) {
      console.warn('validateFn not provided to RedTeamPanel');
      return;
    }

    setRunning(true);
    try {
      const allResults = await executeAllMutators(baseTurtle, validateFn);
      setResults(allResults);
      const summaryData = summarizeMutatorResults(allResults);
      setSummary(summaryData);
      setSelectedResult(null);
      onResultsUpdate?.(allResults);
    } finally {
      setRunning(false);
    }
  };

  const passCount = results.filter((r) => r.passed).length;
  const failCount = results.filter((r) => !r.passed).length;

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 16,
        padding: 16,
        background: 'rgba(10, 15, 35, 0.4)',
        border: '1px solid rgba(100, 200, 255, 0.3)',
        borderRadius: 8,
        fontFamily: sans,
      }}
    >
      {/* Header */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h3 style={{ margin: '0 0 4px 0', fontSize: 16, fontWeight: 600, color: '#fff' }}>
            Red-Team Panel
          </h3>
          <p style={{ margin: 0, fontSize: 12, color: '#999' }}>
            Run adversarial mutations; assert expected vs actual Status
          </p>
        </div>
        <button
          onClick={runAllMutators}
          disabled={running || !baseTurtle}
          style={{
            padding: '8px 16px',
            fontSize: 12,
            fontWeight: 600,
            background: running ? '#444' : '#4c9aff',
            color: '#fff',
            border: 'none',
            borderRadius: 4,
            cursor: running || !baseTurtle ? 'not-allowed' : 'pointer',
            opacity: running || !baseTurtle ? 0.5 : 1,
          }}
        >
          {running ? 'Running...' : 'Run All'}
        </button>
      </div>

      {/* Summary */}
      {results.length > 0 && (
        <div
          style={{
            padding: 12,
            background: 'rgba(50, 100, 150, 0.1)',
            border: '1px solid rgba(100, 150, 200, 0.3)',
            borderRadius: 4,
            fontSize: 12,
            color: '#ccc',
          }}
        >
          <div style={{ fontFamily: mono, fontWeight: 600, marginBottom: 8 }}>
            {passCount}/{results.length} passed · {failCount} failed
          </div>
          {passCount === results.length && (
            <div style={{ color: '#4caf50' }}>✓ All mutators detected as expected</div>
          )}
          {failCount > 0 && (
            <div style={{ color: '#ff9800' }}>⚠ {failCount} mutator(s) did not trigger expected status</div>
          )}
        </div>
      )}

      {/* Mutator List */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))',
          gap: 8,
        }}
      >
        {MUTATORS.map((mutator) => {
          const result = results.find((r) => r.id === mutator.id);
          const status = result ? (result.passed ? '✓' : '✗') : '○';
          const statusColor = result
            ? result.passed
              ? '#4caf50'
              : '#ff6b6b'
            : '#999';

          return (
            <button
              key={mutator.id}
              onClick={() => runSingleMutator(mutator)}
              disabled={running}
              style={{
                padding: 12,
                background: result
                  ? result.passed
                    ? 'rgba(76, 175, 80, 0.1)'
                    : 'rgba(255, 107, 107, 0.1)'
                  : 'rgba(100, 100, 100, 0.1)',
                border: `1px solid ${statusColor}`,
                borderRadius: 4,
                cursor: running ? 'not-allowed' : 'pointer',
                opacity: running ? 0.5 : 1,
                textAlign: 'left',
                fontSize: 12,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4 }}>
                <span style={{ fontSize: 14, color: statusColor, fontWeight: 600 }}>{status}</span>
                <span style={{ color: '#fff', fontWeight: 600 }}>{mutator.label}</span>
              </div>
              {result && (
                <div style={{ fontSize: 10, color: '#999' }}>
                  {result.actualStatus}
                </div>
              )}
            </button>
          );
        })}
      </div>

      {/* Selected Result Detail */}
      {selectedResult && (
        <div
          style={{
            padding: 12,
            background: 'rgba(30, 60, 100, 0.2)',
            border: '1px solid rgba(100, 150, 200, 0.3)',
            borderRadius: 4,
            fontSize: 12,
            fontFamily: mono,
          }}
        >
          <div style={{ marginBottom: 8, fontWeight: 600, color: '#fff' }}>
            {selectedResult.name}
          </div>
          <div style={{ marginBottom: 4, color: '#ccc' }}>
            <strong>Mutation:</strong> {selectedResult.mutation}
          </div>
          <div style={{ marginBottom: 4, color: '#ccc' }}>
            <strong>Expected:</strong> {selectedResult.expectedStatus}
          </div>
          <div style={{ marginBottom: 4, color: '#ccc' }}>
            <strong>Actual:</strong> {selectedResult.actualStatus}
          </div>
          {selectedResult.errorMessage && (
            <div style={{ color: '#ff9800', marginTop: 8 }}>
              <strong>Error:</strong> {selectedResult.errorMessage}
            </div>
          )}
        </div>
      )}

      {/* Results Table */}
      {results.length > 0 && (
        <div style={{ overflowX: 'auto', fontSize: 11 }}>
          <table
            style={{
              width: '100%',
              borderCollapse: 'collapse',
              fontFamily: mono,
            }}
          >
            <thead>
              <tr style={{ borderBottom: '1px solid rgba(100, 150, 200, 0.3)' }}>
                <th style={{ padding: 8, textAlign: 'left', color: '#999' }}>Mutator</th>
                <th style={{ padding: 8, textAlign: 'left', color: '#999' }}>Status</th>
                <th style={{ padding: 8, textAlign: 'left', color: '#999' }}>Expected</th>
                <th style={{ padding: 8, textAlign: 'left', color: '#999' }}>Actual</th>
              </tr>
            </thead>
            <tbody>
              {results.map((result) => (
                <tr
                  key={result.id}
                  style={{
                    borderBottom: '1px solid rgba(100, 100, 100, 0.2)',
                    background: result.passed
                      ? 'rgba(76, 175, 80, 0.05)'
                      : 'rgba(255, 107, 107, 0.05)',
                  }}
                >
                  <td style={{ padding: 8, color: '#ccc' }}>{result.name}</td>
                  <td style={{ padding: 8, color: result.passed ? '#4caf50' : '#ff6b6b' }}>
                    {result.passed ? '✓ PASS' : '✗ FAIL'}
                  </td>
                  <td style={{ padding: 8, color: '#aaa' }}>{result.expectedStatus}</td>
                  <td style={{ padding: 8, color: '#aaa' }}>{result.actualStatus}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
