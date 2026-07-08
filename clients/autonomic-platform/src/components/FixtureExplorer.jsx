/**
 * FixtureExplorer.jsx
 * -------------------
 * UI component for browsing and loading fixture variants.
 * Displays:
 *   - List of fixture categories
 *   - Variants within each category
 *   - Click to load/run a fixture
 *   - Turtle preview
 */

import React, { useState, useMemo } from 'react';
import { generateFixtures, indexFixtures, normalizeTurtle } from '../lib/fixture-generator.js';

const mono = "'JetBrains Mono', ui-monospace, monospace";
const sans = "'Space Grotesk', system-ui, sans-serif";

const CATEGORY_COLORS = {
  'missing-property': '#ff9800',
  'wrong-datatype': '#f44336',
  'overflow': '#e91e63',
  'unknown-predicate': '#9c27b0',
  'syntax-error': '#ff6b6b',
  'unsupported-feature': '#ff9800',
  'tampering': '#e91e63',
  'n3-denial': '#9c27b0',
};

export default function FixtureExplorer({ baseTurtle = '', onFixtureSelect = null, onLoadFixture = null }) {
  const [selectedFixture, setSelectedFixture] = useState(null);
  const [expandedCategories, setExpandedCategories] = useState({});

  const fixtures = useMemo(() => {
    if (!baseTurtle) return { total: 0, byCategory: {}, all: [] };
    const variants = generateFixtures(baseTurtle);
    return indexFixtures(variants);
  }, [baseTurtle]);

  const handleSelectFixture = (fixture) => {
    setSelectedFixture(fixture);
    onFixtureSelect?.(fixture);
  };

  const handleLoadFixture = (fixture) => {
    onLoadFixture?.(fixture);
  };

  const toggleCategory = (category) => {
    setExpandedCategories((prev) => ({
      ...prev,
      [category]: !prev[category],
    }));
  };

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
      <div>
        <h3 style={{ margin: '0 0 4px 0', fontSize: 16, fontWeight: 600, color: '#fff' }}>
          Fixture Explorer
        </h3>
        <p style={{ margin: 0, fontSize: 12, color: '#999' }}>
          {fixtures.total} fixture variants across {Object.keys(fixtures.byCategory).length} categories
        </p>
      </div>

      {/* Categories */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
        {Object.entries(fixtures.byCategory).map(([category, variants]) => (
          <div key={category}>
            {/* Category Header */}
            <button
              onClick={() => toggleCategory(category)}
              style={{
                width: '100%',
                padding: 10,
                background: expandedCategories[category]
                  ? 'rgba(100, 150, 200, 0.15)'
                  : 'rgba(60, 80, 120, 0.1)',
                border: `1px solid ${CATEGORY_COLORS[category] || '#999'}`,
                borderRadius: 4,
                cursor: 'pointer',
                textAlign: 'left',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                fontSize: 12,
                fontWeight: 600,
                color: '#fff',
              }}
            >
              <span>
                <span style={{ color: CATEGORY_COLORS[category] || '#999', marginRight: 6 }}>
                  {expandedCategories[category] ? '▼' : '▶'}
                </span>
                {category}
              </span>
              <span style={{ fontSize: 10, color: '#999' }}>{variants.length}</span>
            </button>

            {/* Variants */}
            {expandedCategories[category] && (
              <div style={{ marginTop: 8, marginLeft: 16, display: 'flex', flexDirection: 'column', gap: 8 }}>
                {variants.map((fixture) => (
                  <button
                    key={fixture.id}
                    onClick={() => handleSelectFixture(fixture)}
                    style={{
                      padding: 10,
                      background:
                        selectedFixture?.id === fixture.id
                          ? 'rgba(100, 200, 255, 0.2)'
                          : 'rgba(50, 60, 100, 0.1)',
                      border: `1px solid ${
                        selectedFixture?.id === fixture.id
                          ? '#4cccff'
                          : 'rgba(100, 150, 200, 0.2)'
                      }`,
                      borderRadius: 4,
                      cursor: 'pointer',
                      textAlign: 'left',
                      fontSize: 11,
                      color: '#ccc',
                      transition: 'all 0.2s',
                    }}
                    onMouseEnter={(e) => {
                      e.target.style.background = 'rgba(100, 150, 200, 0.15)';
                    }}
                    onMouseLeave={(e) => {
                      e.target.style.background =
                        selectedFixture?.id === fixture.id
                          ? 'rgba(100, 200, 255, 0.2)'
                          : 'rgba(50, 60, 100, 0.1)';
                    }}
                  >
                    <div style={{ fontWeight: 600, marginBottom: 4, color: '#fff' }}>
                      {fixture.name}
                    </div>
                    <div style={{ fontSize: 10, color: '#999' }}>
                      {fixture.description}
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Selected Fixture Detail */}
      {selectedFixture && (
        <div
          style={{
            padding: 12,
            background: 'rgba(30, 60, 100, 0.2)',
            border: '1px solid rgba(100, 150, 200, 0.3)',
            borderRadius: 4,
          }}
        >
          <div style={{ marginBottom: 8 }}>
            <div style={{ fontSize: 12, fontWeight: 600, color: '#fff', marginBottom: 4 }}>
              {selectedFixture.name}
            </div>
            <div style={{ fontSize: 11, color: '#999', marginBottom: 8 }}>
              {selectedFixture.description}
            </div>
          </div>

          {/* Expected Status */}
          <div style={{ fontSize: 10, marginBottom: 8, fontFamily: mono }}>
            <strong style={{ color: '#aaa' }}>Expected Status:</strong>
            <div style={{ color: '#4cccff', marginTop: 2 }}>{selectedFixture.expectedStatus}</div>
          </div>

          {/* Turtle Preview */}
          <div style={{ fontSize: 10, marginBottom: 8 }}>
            <strong style={{ color: '#aaa' }}>Turtle Preview:</strong>
            <pre
              style={{
                margin: '4px 0 0 0',
                padding: 8,
                background: 'rgba(0, 0, 0, 0.3)',
                borderRadius: 3,
                borderLeft: '3px solid #4cccff',
                fontSize: 9,
                color: '#4cccff',
                maxHeight: 150,
                overflowY: 'auto',
                fontFamily: mono,
                whiteSpace: 'pre-wrap',
                wordWrap: 'break-word',
              }}
            >
              {selectedFixture.turtle.substring(0, 400)}
              {selectedFixture.turtle.length > 400 ? '\n... (truncated)' : ''}
            </pre>
          </div>

          {/* Load Button */}
          <button
            onClick={() => handleLoadFixture(selectedFixture)}
            style={{
              width: '100%',
              padding: 8,
              background: '#4cccff',
              border: 'none',
              borderRadius: 4,
              fontSize: 11,
              fontWeight: 600,
              color: '#000',
              cursor: 'pointer',
              transition: 'all 0.2s',
            }}
            onMouseEnter={(e) => {
              e.target.style.background = '#6dd4ff';
            }}
            onMouseLeave={(e) => {
              e.target.style.background = '#4cccff';
            }}
          >
            Load & Run Fixture
          </button>
        </div>
      )}

      {/* Empty State */}
      {fixtures.total === 0 && (
        <div
          style={{
            padding: 20,
            textAlign: 'center',
            color: '#999',
            fontSize: 12,
            background: 'rgba(100, 100, 100, 0.05)',
            borderRadius: 4,
            border: '1px dashed rgba(100, 100, 100, 0.3)',
          }}
        >
          {baseTurtle
            ? 'Generating fixtures...'
            : 'Load a base Turtle to generate fixtures'}
        </div>
      )}
    </div>
  );
}
