'use client';

import React, { useState, useCallback } from 'react';
import EditorTabs from '@/components/EditorTabs';
import TurtleEditor from '@/components/TurtleEditor';
import CapabilityMatrix from '@/components/CapabilityMatrix';
import HookTimeline from '@/components/HookTimeline';
import ReceiptPanel from '@/components/ReceiptPanel';
import Toolbar from '@/components/Toolbar';
import { EditorFile, PlaygroundResult, AppState } from '@/lib/types';
import { runAllDialects, runHooks, replayVerify } from '@/lib/engine';
import { sampleTurtle } from '@/lib/fixtures';

const DEFAULT_FILES: EditorFile[] = [
  {
    name: 'sample.ttl',
    content: sampleTurtle,
    language: 'turtle',
  },
  {
    name: 'rules.n3',
    content: `# N3 rules for inference
@prefix ex: <http://example.com/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix log: <http://www.w3.org/2000/10/swap/log#> .

{ ?x rdfs:subClassOf ?y . ?y rdfs:subClassOf ?z } => { ?x rdfs:subClassOf ?z } .
`,
    language: 'n3',
  },
  {
    name: 'shapes.ttl',
    content: `# SHACL shapes for validation
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.com/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:PersonShape
  a sh:NodeShape ;
  sh:targetClass ex:Person ;
  sh:property [
    sh:path ex:foaf:name ;
    sh:datatype xsd:string ;
    sh:minCount 1 ;
  ] .
`,
    language: 'shacl',
  },
];

export default function App() {
  const [state, setState] = useState<AppState>({
    files: DEFAULT_FILES,
    activeFileIndex: 0,
    result: null,
    isLoading: false,
    error: null,
  });

  const activeFile = state.files[state.activeFileIndex];

  const handleFileChange = useCallback((content: string): void => {
    setState((prev: AppState) => {
      const newFiles = [...prev.files];
      newFiles[prev.activeFileIndex] = {
        ...newFiles[prev.activeFileIndex],
        content,
      };
      return { ...prev, files: newFiles };
    });
  }, []);

  const handleAddFile = useCallback((): void => {
    const newFile: EditorFile = {
      name: `file-${state.files.length}.ttl`,
      content: '',
      language: 'turtle',
    };
    setState((prev: AppState) => ({
      ...prev,
      files: [...prev.files, newFile],
      activeFileIndex: prev.files.length,
    }));
  }, [state.files.length]);

  const handleRemoveFile = useCallback((index: number): void => {
    setState((prev: AppState) => {
      const newFiles = prev.files.filter((_, i) => i !== index);
      const newActiveIndex = Math.min(prev.activeFileIndex, newFiles.length - 1);
      return {
        ...prev,
        files: newFiles,
        activeFileIndex: newActiveIndex,
      };
    });
  }, []);

  const handleLoadCase = useCallback(async (): Promise<void> => {
    setState((prev: AppState) => ({ ...prev, isLoading: true, error: null }));
    try {
      // In a real app, this would load from the Praxis case library
      setState((prev: AppState) => ({ ...prev, isLoading: false }));
    } catch (error) {
      setState((prev: AppState) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      }));
    }
  }, []);

  const handleRunAllDialects = useCallback(async (): Promise<void> => {
    setState((prev: AppState) => ({ ...prev, isLoading: true, error: null }));
    try {
      const result = await runAllDialects(activeFile.content);
      setState((prev: AppState) => ({
        ...prev,
        result,
        isLoading: false,
      }));
    } catch (error) {
      setState((prev: AppState) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      }));
    }
  }, [activeFile.content]);

  const handleRunHooks = useCallback(async (): Promise<void> => {
    setState((prev: AppState) => ({ ...prev, isLoading: true, error: null }));
    try {
      const result = await runHooks(activeFile.content);
      setState((prev: AppState) => ({
        ...prev,
        result,
        isLoading: false,
      }));
    } catch (error) {
      setState((prev: AppState) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      }));
    }
  }, [activeFile.content]);

  const handleReplay = useCallback(async (): Promise<void> => {
    setState((prev: AppState) => ({ ...prev, isLoading: true, error: null }));
    try {
      const result = await replayVerify(activeFile.content);
      setState((prev: AppState) => ({
        ...prev,
        result,
        isLoading: false,
      }));
    } catch (error) {
      setState((prev: AppState) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      }));
    }
  }, [activeFile.content]);

  const handleGenerateReport = useCallback((): void => {
    if (!state.result) {
      setState((prev: AppState) => ({
        ...prev,
        error: 'Run dialects first to generate a report',
      }));
      return;
    }
    const markdown = generateReport(state.result);
    const blob = new Blob([markdown], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'praxis-report.md';
    a.click();
    URL.revokeObjectURL(url);
  }, [state.result]);

  return (
    <main style={{ height: '100vh', display: 'flex', flexDirection: 'column', backgroundColor: '#fff' }}>
      {/* Header */}
      <header style={{ padding: '1rem', borderBottom: '1px solid #ddd', backgroundColor: '#f9f9f9' }}>
        <h1 style={{ margin: '0 0 8px 0', fontSize: '24px', fontWeight: 'bold' }}>
          🔬 Praxis Playground v0.1.0
        </h1>
        <p style={{ margin: 0, color: '#666', fontSize: '14px' }}>
          Interactive reasoning engine for Turtle, OWL-RL, SHACL, ShEx, Datalog, N3, and Hooks.
        </p>
      </header>

      {/* Toolbar */}
      <Toolbar
        onLoadCase={handleLoadCase}
        onRunAllDialects={handleRunAllDialects}
        onRunHooks={handleRunHooks}
        onReplay={handleReplay}
        onGenerateReport={handleGenerateReport}
        isLoading={state.isLoading}
      />

      {/* Error banner */}
      {state.error && (
        <div
          style={{
            padding: '12px 1rem',
            backgroundColor: '#fee2e2',
            borderBottom: '1px solid #fecaca',
            color: '#991b1b',
            fontSize: '14px',
          }}
        >
          ⚠️ {state.error}
        </div>
      )}

      {/* Main content */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Left panel: Editor */}
        <div
          style={{
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            overflow: 'hidden',
            borderRight: '1px solid #ddd',
          }}
        >
          <EditorTabs
            files={state.files}
            activeIndex={state.activeFileIndex}
            onActiveChange={(idx) => setState((prev: AppState) => ({ ...prev, activeFileIndex: idx }))}
            onAddFile={handleAddFile}
            onRemoveFile={handleRemoveFile}
          />
          <div style={{ flex: 1, overflow: 'hidden' }}>
            <TurtleEditor
              content={activeFile.content}
              onChange={handleFileChange}
              language={activeFile.language}
              height="100%"
            />
          </div>
        </div>

        {/* Right panel: Results */}
        <div
          style={{
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            overflow: 'auto',
            padding: '1rem',
            backgroundColor: '#f9f9f9',
            gap: '1rem',
          }}
        >
          <CapabilityMatrix result={state.result} isLoading={state.isLoading} />
          <HookTimeline result={state.result} />
          <ReceiptPanel result={state.result} />
        </div>
      </div>
    </main>
  );
}

/**
 * Generate a markdown report from the playground result.
 */
function generateReport(result: PlaygroundResult): string {
  const lines = [
    '# Praxis Playground Report',
    '',
    `Generated: ${new Date().toISOString()}`,
    '',
    '## Hashes',
    '',
    `- **Graph Hash**: \`${result.graph_hash}\``,
    `- **Profile Hash**: \`${result.profile_hash}\``,
    '',
    '## Dialect Results',
    '',
  ];

  result.dialects.forEach((d) => {
    lines.push(`- **${d.dialect}**: ${d.status} (${d.triples_out} triples)`);
    lines.push(`  - ${d.detail}`);
    lines.push('');
  });

  lines.push('## Hook Execution');
  lines.push('');
  result.hooks.schedule.forEach((hook, idx) => {
    const verdict = result.hooks.verdicts.find((v) => v.hook_iri.includes(hook));
    lines.push(`${idx + 1}. **${hook}**: ${verdict?.verdict || 'NO_VERDICT'}`);
  });

  lines.push('');
  lines.push('## Replay Verification');
  lines.push('');
  lines.push(`- Status: ${result.replay.status}`);
  lines.push(`- First Hash: \`${result.replay.first_hash}\``);
  lines.push(`- Second Hash: \`${result.replay.second_hash}\``);

  return lines.join('\n');
}
