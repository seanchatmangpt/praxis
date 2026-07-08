/**
 * ReportViewer.jsx
 * ----------------
 * Display markdown report with report hash.
 * Features:
 *   - Render markdown as HTML
 *   - Display report hash (stable, deterministic)
 *   - Copy/export buttons
 */

import { useState } from 'react';
import { exportReportJSON, exportReportMarkdown } from '../lib/report.js';

const mono = "'JetBrains Mono', ui-monospace, monospace";
const sans = "'Space Grotesk', system-ui, sans-serif";

/**
 * Simple markdown to HTML converter (handles basic tables and formatting).
 */
function markdownToHtml(markdown) {
  let html = markdown;

  // Headers
  html = html.replace(/^### (.*?)$/gm, '<h3>$1</h3>');
  html = html.replace(/^## (.*?)$/gm, '<h2>$1</h2>');
  html = html.replace(/^# (.*?)$/gm, '<h1>$1</h1>');

  // Bold and italic
  html = html.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
  html = html.replace(/\*(.*?)\*/g, '<em>$1</em>');
  html = html.replace(/`(.*?)`/g, '<code>$1</code>');

  // Lists
  html = html.replace(/^- (.*?)$/gm, '<li>$1</li>');
  html = html.replace(/(<li>.*<\/li>)/s, '<ul>$1</ul>');
  html = html.replace(/^1\. (.*?)$/gm, '<li>$1</li>');
  html = html.replace(/(<li>.*<\/li>)/s, '<ol>$1</ol>');

  // Tables
  html = html.replace(
    /^\| (.*) \|$/gm,
    (match, row) => {
      const cells = row.split('|').map((c) => c.trim());
      // Skip header separator rows
      if (cells[0] === '' || cells[0].includes('---')) {
        return '';
      }
      const isSeparator = cells.every((c) => c === '' || c.match(/^-+$/));
      if (isSeparator) return '';
      return '<tr><td>' + cells.filter((c) => c).join('</td><td>') + '</td></tr>';
    }
  );
  html = html.replace(/(<tr>.*<\/tr>)/s, '<table>$1</table>');

  // Paragraphs
  html = html
    .split('\n\n')
    .map((para) => {
      if (
        para.startsWith('<') ||
        para.startsWith('---') ||
        para.includes('<table') ||
        para.includes('<h')
      ) {
        return para;
      }
      return '<p>' + para + '</p>';
    })
    .join('\n\n');

  // Line breaks
  html = html.replace(/\n/g, '<br/>');

  return html;
}

export default function ReportViewer({ report = null, onExport = null }) {
  const [copyStatus, setCopyStatus] = useState(null);

  if (!report) {
    return (
      <div
        style={{
          padding: 16,
          background: 'rgba(10, 15, 35, 0.4)',
          border: '1px solid rgba(100, 200, 255, 0.3)',
          borderRadius: 8,
          fontFamily: sans,
          textAlign: 'center',
          color: '#999',
          fontSize: 12,
        }}
      >
        No report generated yet. Run mutators or fixtures to generate a report.
      </div>
    );
  }

  const handleCopyHash = () => {
    navigator.clipboard.writeText(report.hash);
    setCopyStatus('Hash copied!');
    setTimeout(() => setCopyStatus(null), 2000);
  };

  const handleExportJSON = () => {
    const json = exportReportJSON(report);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `report-${report.hash}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleExportMarkdown = () => {
    const md = exportReportMarkdown(report);
    const blob = new Blob([md], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `report-${report.hash}.md`;
    a.click();
    URL.revokeObjectURL(url);
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
          {report.title}
        </h3>
        <p style={{ margin: 0, fontSize: 11, color: '#999' }}>
          Generated {new Date(report.generatedAt).toLocaleString()}
        </p>
      </div>

      {/* Report Hash */}
      <div
        style={{
          padding: 12,
          background: 'rgba(50, 100, 150, 0.1)',
          border: '1px solid rgba(100, 150, 200, 0.3)',
          borderRadius: 4,
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          fontSize: 11,
          fontFamily: mono,
        }}
      >
        <div>
          <strong style={{ color: '#aaa' }}>Report Hash:</strong>
          <div style={{ color: '#4cccff', marginTop: 4, fontSize: 10 }}>{report.hash}</div>
        </div>
        <button
          onClick={handleCopyHash}
          style={{
            padding: '6px 12px',
            background: '#4cccff',
            border: 'none',
            borderRadius: 3,
            fontSize: 10,
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
          {copyStatus || 'Copy'}
        </button>
      </div>

      {/* Export Buttons */}
      <div style={{ display: 'flex', gap: 8 }}>
        <button
          onClick={handleExportJSON}
          style={{
            flex: 1,
            padding: 8,
            background: '#9c27b0',
            border: 'none',
            borderRadius: 4,
            fontSize: 11,
            fontWeight: 600,
            color: '#fff',
            cursor: 'pointer',
            transition: 'all 0.2s',
          }}
          onMouseEnter={(e) => {
            e.target.style.background = '#b635d4';
          }}
          onMouseLeave={(e) => {
            e.target.style.background = '#9c27b0';
          }}
        >
          Export as JSON
        </button>
        <button
          onClick={handleExportMarkdown}
          style={{
            flex: 1,
            padding: 8,
            background: '#ff9800',
            border: 'none',
            borderRadius: 4,
            fontSize: 11,
            fontWeight: 600,
            color: '#fff',
            cursor: 'pointer',
            transition: 'all 0.2s',
          }}
          onMouseEnter={(e) => {
            e.target.style.background = '#ffb74d';
          }}
          onMouseLeave={(e) => {
            e.target.style.background = '#ff9800';
          }}
        >
          Export as Markdown
        </button>
      </div>

      {/* Markdown Report */}
      <div
        style={{
          padding: 16,
          background: 'rgba(20, 30, 60, 0.3)',
          border: '1px solid rgba(100, 150, 200, 0.2)',
          borderRadius: 4,
          overflowY: 'auto',
          maxHeight: 500,
          fontSize: 12,
          lineHeight: 1.6,
          color: '#ccc',
          fontFamily: sans,
        }}
      >
        {/* Simple markdown rendering */}
        {report.markdown.split('\n\n').map((section, idx) => (
          <div key={idx} style={{ marginBottom: 16 }}>
            {section.split('\n').map((line, lineIdx) => {
              // Headers
              if (line.startsWith('### ')) {
                return (
                  <h4
                    key={lineIdx}
                    style={{
                      fontSize: 13,
                      fontWeight: 600,
                      margin: '12px 0 6px 0',
                      color: '#fff',
                    }}
                  >
                    {line.slice(4)}
                  </h4>
                );
              }
              if (line.startsWith('## ')) {
                return (
                  <h3
                    key={lineIdx}
                    style={{
                      fontSize: 14,
                      fontWeight: 600,
                      margin: '12px 0 6px 0',
                      color: '#fff',
                    }}
                  >
                    {line.slice(3)}
                  </h3>
                );
              }
              if (line.startsWith('# ')) {
                return (
                  <h2
                    key={lineIdx}
                    style={{
                      fontSize: 16,
                      fontWeight: 600,
                      margin: '12px 0 6px 0',
                      color: '#fff',
                    }}
                  >
                    {line.slice(2)}
                  </h2>
                );
              }

              // Tables
              if (line.includes('|')) {
                const cells = line.split('|').slice(1, -1).map((c) => c.trim());
                return (
                  <div
                    key={lineIdx}
                    style={{
                      display: 'flex',
                      gap: 8,
                      fontSize: 11,
                      padding: 6,
                      background: 'rgba(50, 100, 150, 0.1)',
                      borderBottom: '1px solid rgba(100, 150, 200, 0.2)',
                    }}
                  >
                    {cells.map((cell, cellIdx) => (
                      <div key={cellIdx} style={{ flex: 1, fontFamily: mono }}>
                        {cell}
                      </div>
                    ))}
                  </div>
                );
              }

              // Bullets and text
              if (line.startsWith('- ')) {
                return (
                  <div key={lineIdx} style={{ marginLeft: 12, marginBottom: 4 }}>
                    • {line.slice(2)}
                  </div>
                );
              }

              // Bold text
              if (line.includes('**')) {
                const parts = line.split(/\*\*(.*?)\*\*/g);
                return (
                  <div key={lineIdx} style={{ marginBottom: 4 }}>
                    {parts.map((part, partIdx) => (
                      <span
                        key={partIdx}
                        style={{
                          fontWeight: partIdx % 2 === 1 ? 600 : 400,
                          color: partIdx % 2 === 1 ? '#4cccff' : '#ccc',
                        }}
                      >
                        {part}
                      </span>
                    ))}
                  </div>
                );
              }

              // Regular text
              if (line.trim()) {
                return (
                  <div key={lineIdx} style={{ marginBottom: 4 }}>
                    {line}
                  </div>
                );
              }

              return null;
            })}
          </div>
        ))}
      </div>
    </div>
  );
}
