'use client';

/**
 * Demo page showcasing Monaco Turtle editor with syntax highlighting and completions.
 * This file demonstrates the exit condition for the Monaco Turtle editor integration:
 * - Syntax highlighting (Monarch grammar)
 * - Completions (prefix-aware predicates)
 * - Diagnostics (validation via GraphLaw engine)
 */

import React, { useState } from 'react';
import TurtleEditor from '@/components/TurtleEditor';
import type { ValidationResult } from '@/lib/graphlaw-wasm';

const DEMO_TURTLE = `@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

# Sample facts demonstrating Turtle syntax highlighting
ex:Alice a ex:Person ;
  ex:name "Alice" ;
  ex:age 30 ;
  ex:email "alice@example.org" .

ex:PersonShape
  a sh:NodeShape ;
  sh:targetClass ex:Person ;
  sh:property [
    sh:path ex:name ;
    sh:datatype xsd:string ;
    sh:minCount 1 ;
    sh:maxCount 1 ;
  ] ;
  sh:property [
    sh:path ex:age ;
    sh:datatype xsd:integer ;
    sh:minInclusive 0 ;
    sh:maxInclusive 150 ;
  ] .

# Knowledge hook that derives facts
kh:AgeValidation
  hook:triggeredWhen [
    sh:path ex:age ;
    sh:datatype xsd:integer ;
  ] ;
  hook:derivesFact ex:validAge ;
  hook:withPriority 100 .

# SPARQL CONSTRUCT rule for inference
CONSTRUCT {
  ?person ex:isAdult true .
} WHERE {
  ?person a ex:Person ;
           ex:age ?age .
  FILTER (?age >= 18)
}
`;

interface EditorDemoProps {
  // Optional: callback when validation result changes
  onValidationChange?: (result: ValidationResult | null) => void;
}

export default function EditorDemo({ onValidationChange }: EditorDemoProps) {
  const [content, setContent] = useState(DEMO_TURTLE);
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [graphHash, setGraphHash] = useState<string | null>(null);

  const handleValidationChange = (result: ValidationResult | null) => {
    setValidationResult(result);
    onValidationChange?.(result);
  };

  const handleGraphHashChange = (hash: string) => {
    setGraphHash(hash);
  };

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      height: '100vh',
      backgroundColor: '#f5f5f5',
      fontFamily: 'system-ui, -apple-system, sans-serif',
    }}>
      {/* Header */}
      <header style={{
        backgroundColor: 'linear-gradient(to right, #1e40af, #1e3a8a)',
        color: 'white',
        padding: '20px 24px',
        boxShadow: '0 2px 8px rgba(0,0,0,0.1)',
      }}>
        <h1 style={{ margin: '0 0 8px 0', fontSize: 28, fontWeight: 700 }}>
          Monaco Turtle Editor Integration Demo
        </h1>
        <p style={{ margin: 0, opacity: 0.9, fontSize: 14 }}>
          Demonstrates Monarch grammar syntax highlighting, prefix-aware completions, and real-time diagnostics
        </p>
      </header>

      {/* Main content */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden', gap: 16, padding: 16 }}>
        {/* Editor pane */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          <div style={{
            marginBottom: 8,
            fontSize: 13,
            fontWeight: 600,
            color: '#1f2937',
          }}>
            Turtle Editor (with Monarch Syntax Highlighting & Completions)
          </div>
          <TurtleEditor
            content={content}
            onChange={setContent}
            language="turtle"
            height="100%"
            onValidationChange={handleValidationChange}
            onGraphHashChange={handleGraphHashChange}
            showToolbar={true}
          />
        </div>

        {/* Info pane */}
        <div style={{
          width: 300,
          display: 'flex',
          flexDirection: 'column',
          gap: 12,
          overflow: 'auto',
          backgroundColor: 'white',
          borderRadius: 8,
          padding: 16,
          boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
        }}>
          <section>
            <h3 style={{ margin: '0 0 8px 0', fontSize: 13, fontWeight: 600, color: '#1f2937' }}>
              Features
            </h3>
            <ul style={{ margin: 0, paddingLeft: 16, fontSize: 12, lineHeight: 1.6, color: '#4b5563' }}>
              <li>✓ Syntax highlighting (Monarch grammar)</li>
              <li>✓ Prefix-aware completions (kh:, hook:, sh:, xsd:, rdf:, fibo:, etc.)</li>
              <li>✓ Real-time diagnostics</li>
              <li>✓ Bracket pair navigation</li>
              <li>✓ IRI and literal syntax support</li>
            </ul>
          </section>

          <hr style={{ margin: '12px 0', border: 'none', borderTop: '1px solid #e5e7eb' }} />

          <section>
            <h3 style={{ margin: '0 0 8px 0', fontSize: 13, fontWeight: 600, color: '#1f2937' }}>
              Validation Status
            </h3>
            {validationResult ? (
              <div style={{
                fontSize: 12,
                color: validationResult.conforms ? '#059669' : '#dc2626',
                fontWeight: 500,
              }}>
                {validationResult.conforms ? '✓ All constraints satisfied' : '✗ Validation failed'}
              </div>
            ) : (
              <div style={{ fontSize: 12, color: '#666' }}>No validation yet</div>
            )}
            {validationResult?.violations && validationResult.violations.length > 0 && (
              <div style={{ fontSize: 11, color: '#7f1d1d', marginTop: 8 }}>
                {validationResult.violations.length} violation(s) found
              </div>
            )}
          </section>

          <hr style={{ margin: '12px 0', border: 'none', borderTop: '1px solid #e5e7eb' }} />

          <section>
            <h3 style={{ margin: '0 0 8px 0', fontSize: 13, fontWeight: 600, color: '#1f2937' }}>
              Graph Hash
            </h3>
            {graphHash ? (
              <div style={{
                fontSize: 10,
                fontFamily: 'monospace',
                backgroundColor: '#f3f4f6',
                padding: 8,
                borderRadius: 4,
                color: '#374151',
                wordBreak: 'break-all',
              }}>
                {/* Hash value displayed here */}
                {graphHash}
              </div>
            ) : (
              <div style={{ fontSize: 12, color: '#666' }}>Click &quot;Hash&quot; to compute</div>
            )}
          </section>

          <hr style={{ margin: '12px 0', border: 'none', borderTop: '1px solid #e5e7eb' }} />

          <section>
            <h3 style={{ margin: '0 0 8px 0', fontSize: 13, fontWeight: 600, color: '#1f2937' }}>
              Syntax Examples
            </h3>
            <div style={{ fontSize: 11, color: '#4b5563', lineHeight: 1.6 }}>
              <p style={{ margin: '0 0 4px 0' }}>
                <code style={{ backgroundColor: '#f3f4f6', padding: '2px 4px' }}>@prefix kh:</code>
              </p>
              <p style={{ margin: '0 0 4px 0' }}>
                <code style={{ backgroundColor: '#f3f4f6', padding: '2px 4px' }}>ex:property</code>
              </p>
              <p style={{ margin: '0 0 4px 0' }}>
                <code style={{ backgroundColor: '#f3f4f6', padding: '2px 4px' }}>&lt;http://...&gt;</code>
              </p>
              <p style={{ margin: '0 0 4px 0' }}>
                <code style={{ backgroundColor: '#f3f4f6', padding: '2px 4px' }}>&quot;string&quot;@en</code>
              </p>
              <p style={{ margin: 0 }}>
                <code style={{ backgroundColor: '#f3f4f6', padding: '2px 4px' }}>_:blank</code>
              </p>
            </div>
          </section>
        </div>
      </div>

      {/* Footer */}
      <footer style={{
        backgroundColor: '#f3f4f6',
        borderTop: '1px solid #d1d5db',
        padding: '12px 24px',
        fontSize: 12,
        color: '#6b7280',
        textAlign: 'center',
      }}>
        Monaco Editor + Turtle Language Support + GraphLaw WASM Engine Integration
      </footer>
    </div>
  );
}
