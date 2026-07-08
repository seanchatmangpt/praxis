/**
 * FixtureTestHarness.jsx
 * ----------------------
 * Integrated testing harness combining FixtureExplorer, RedTeamPanel, and ReportViewer.
 * Orchestrates the full fixture → mutator → report workflow.
 *
 * Usage:
 *   <FixtureTestHarness baseTurtle={...} validateFn={...} />
 */

import React, { useState, useCallback } from 'react';
import FixtureExplorer from './FixtureExplorer.jsx';
import RedTeamPanel from './RedTeamPanel.jsx';
import ReportViewer from './ReportViewer.jsx';
import { createReport } from '../lib/report.js';

const sans = "'Space Grotesk', system-ui, sans-serif";

export default function FixtureTestHarness({ baseTurtle = '', validateFn = null }) {
  const [activeTurtle, setActiveTurtle] = useState(baseTurtle);
  const [mutatorResults, setMutatorResults] = useState([]);
  const [report, setReport] = useState(null);
  const [tab, setTab] = useState('fixtures'); // fixtures | mutators | report

  const handleFixtureSelect = useCallback((fixture) => {
    // Show selected fixture; could update visualization here
  }, []);

  const handleLoadFixture = useCallback((fixture) => {
    setActiveTurtle(fixture.turtle);
    setTab('mutators');
  }, []);

  const handleMutatorResults = useCallback((results) => {
    setMutatorResults(results);
  }, []);

  const handleGenerateReport = useCallback(() => {
    const reportData = createReport({
      title: 'Praxis Red-Team Validation Report',
      mutatorResults,
      hookTests: [],
      timestamp: new Date().toISOString(),
    });
    setReport(reportData);
    setTab('report');
  }, [mutatorResults]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16, padding: 16, fontFamily: sans }}>
      {/* Tabs */}
      <div style={{ display: 'flex', gap: 8, borderBottom: '1px solid rgba(100, 150, 200, 0.2)' }}>
        {['fixtures', 'mutators', 'report'].map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            style={{
              padding: '8px 16px',
              background: tab === t ? 'rgba(100, 200, 255, 0.2)' : 'transparent',
              border: 'none',
              borderBottom: tab === t ? '2px solid #4cccff' : '2px solid transparent',
              cursor: 'pointer',
              fontSize: 12,
              fontWeight: 600,
              color: tab === t ? '#4cccff' : '#999',
              transition: 'all 0.2s',
            }}
          >
            {t.charAt(0).toUpperCase() + t.slice(1)}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div style={{ display: tab === 'fixtures' ? 'block' : 'none' }}>
        <FixtureExplorer
          baseTurtle={baseTurtle}
          onFixtureSelect={handleFixtureSelect}
          onLoadFixture={handleLoadFixture}
        />
      </div>

      <div style={{ display: tab === 'mutators' ? 'block' : 'none' }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <RedTeamPanel
            baseTurtle={activeTurtle}
            validateFn={validateFn}
            onResultsUpdate={handleMutatorResults}
          />

          {/* Generate Report Button */}
          {mutatorResults.length > 0 && (
            <button
              onClick={handleGenerateReport}
              style={{
                padding: 12,
                background: '#4caf50',
                border: 'none',
                borderRadius: 4,
                fontSize: 12,
                fontWeight: 600,
                color: '#fff',
                cursor: 'pointer',
                transition: 'all 0.2s',
              }}
              onMouseEnter={(e) => {
                e.target.style.background = '#66bb6a';
              }}
              onMouseLeave={(e) => {
                e.target.style.background = '#4caf50';
              }}
            >
              Generate Report
            </button>
          )}
        </div>
      </div>

      <div style={{ display: tab === 'report' ? 'block' : 'none' }}>
        <ReportViewer report={report} />
      </div>
    </div>
  );
}
