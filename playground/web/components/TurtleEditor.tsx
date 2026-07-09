'use client';

/**
 * Enhanced TurtleEditor component: Monaco editor with Turtle syntax highlighting,
 * completions, and real-time diagnostics from GraphLaw engine via WASM.
 */

import React, { useEffect, useRef, useState } from 'react';
import Editor, { Monaco } from '@monaco-editor/react';
// Type-only: never import monaco-editor as a value in Next.js/webpack code
// paths -- it statically pulls the full editor bundle in, which fails to
// resolve monaco's internal AMD loader chunk (`vs/nls.messages-loader`).
// @monaco-editor/react's `Editor` component loads and instantiates monaco
// itself and hands back a live, working instance via `onMount` below.
import type * as monaco from 'monaco-editor';

import { initGraphlawEngine, type GraphlawEngineInterface } from '@/lib/graphlaw-wasm';
import { registerTurtleLanguage, configureTurtleLanguage } from '@/monaco/turtle-language';
import { registerTurtleCompletions } from '@/monaco/turtle-completions';
import {
  watchTurtleDiagnostics,
  validateTurtleOnce,
  type ValidationResult,
} from '@/monaco/diagnostics';

interface TurtleEditorProps {
  content?: string;
  onChange?: (content: string) => void;
  language?: 'turtle' | 'sparql' | 'shacl' | 'shex' | 'n3';
  height?: string;
  onValidationChange?: (result: ValidationResult | null) => void;
  onGraphHashChange?: (hash: string) => void;
  showToolbar?: boolean;
}

export default function TurtleEditor({
  content = '',
  onChange,
  language = 'turtle',
  height = '400px',
  onValidationChange,
  onGraphHashChange,
  showToolbar = false,
}: TurtleEditorProps) {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const [engine, setEngine] = useState<GraphlawEngineInterface | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastValidation, setLastValidation] = useState<ValidationResult | null>(null);
  const cleanupWatchRef = useRef<(() => void) | null>(null);
  const monacoRef = useRef<Monaco | null>(null);

  const languageMap = {
    turtle: 'turtle',
    sparql: 'sparql',
    shacl: 'turtle',
    shex: 'shex',
    n3: 'n3',
  };

  /**
   * Initialize the GraphLaw engine.
   */
  useEffect(() => {
    const initializeEngine = async () => {
      try {
        const graphlawEngine = await initGraphlawEngine();
        setEngine(graphlawEngine);
      } catch (err) {
        console.error('Failed to initialize GraphLaw engine:', err);
        setError(
          err instanceof Error ? err.message : 'Failed to initialize GraphLaw engine'
        );
      } finally {
        setIsLoading(false);
      }
    };

    if (language === 'turtle') {
      initializeEngine();
    } else {
      setIsLoading(false);
    }

    return () => {
      if (cleanupWatchRef.current) {
        cleanupWatchRef.current();
      }
    };
  }, [language]);

  /**
   * Handle editor mount: configure language and set up diagnostics.
   */
  const handleEditorMount = (editor: monaco.editor.IStandaloneCodeEditor, monacoInstance: Monaco) => {
    editorRef.current = editor;
    monacoRef.current = monacoInstance;

    // Register Turtle language features if needed
    if (language === 'turtle') {
      registerTurtleLanguage(monacoInstance);
      configureTurtleLanguage(monacoInstance);
      registerTurtleCompletions(monacoInstance);

      // Set up diagnostics if engine is ready
      if (engine) {
        const model = editor.getModel();
        if (model) {
          cleanupWatchRef.current = watchTurtleDiagnostics(monacoInstance, model, engine);
        }
      }
    }
  };

  /**
   * Handle editor value changes.
   */
  const handleEditorChange = (value: string | undefined) => {
    if (!value) return;
    onChange?.(value);
  };

  /**
   * Validate the current content.
   */
  const handleValidate = async () => {
    if (!engine || !editorRef.current || !monacoRef.current) return;

    const model = editorRef.current.getModel();
    if (!model) return;

    try {
      const result = await validateTurtleOnce(monacoRef.current, model, engine);
      setLastValidation(result);
      onValidationChange?.(result);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : 'Validation failed'
      );
    }
  };

  /**
   * Compute content hash.
   */
  const handleHashGraph = async () => {
    if (!engine || !editorRef.current) return;

    const source = editorRef.current.getValue();

    try {
      const hash = await engine.graphHash(source);
      onGraphHashChange?.(hash);
      console.log('Graph hash:', hash);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : 'Failed to compute hash'
      );
    }
  };

  /**
   * Run knowledge hooks.
   */
  const handleRunHooks = async () => {
    if (!engine || !editorRef.current) return;

    const source = editorRef.current.getValue();

    try {
      const result = await engine.runHooks(source);
      console.log('Hook results:', result);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : 'Failed to run hooks'
      );
    }
  };

  const containerStyle: React.CSSProperties = {
    border: '1px solid #ddd',
    borderRadius: '4px',
    overflow: 'hidden',
    display: 'flex',
    flexDirection: 'column',
  };

  const editorContainerStyle: React.CSSProperties = {
    flex: 1,
    overflow: 'hidden',
  };

  const toolbarStyle: React.CSSProperties = {
    display: 'flex',
    gap: '8px',
    padding: '8px',
    backgroundColor: '#f5f5f5',
    borderBottom: '1px solid #ddd',
  };

  const buttonStyle = (disabled: boolean = false): React.CSSProperties => ({
    padding: '6px 12px',
    fontSize: '12px',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: disabled ? 'default' : 'pointer',
    opacity: disabled ? 0.5 : 1,
  });

  return (
    <div style={containerStyle}>
      {error && (
        <div style={{
          padding: '8px 12px',
          backgroundColor: '#fff3cd',
          color: '#856404',
          fontSize: '12px',
          borderBottom: '1px solid #ffc107',
        }}>
          {error}
        </div>
      )}

      {showToolbar && language === 'turtle' && (
        <div style={toolbarStyle}>
          <button
            onClick={handleValidate}
            disabled={!engine || isLoading}
            style={buttonStyle(!engine || isLoading)}
          >
            {lastValidation ? (lastValidation.conforms ? '✓ Valid' : '✗ Invalid') : 'Validate'}
          </button>
          <button
            onClick={handleHashGraph}
            disabled={!engine || isLoading}
            style={buttonStyle(!engine || isLoading)}
          >
            Hash
          </button>
          <button
            onClick={handleRunHooks}
            disabled={!engine || isLoading}
            style={buttonStyle(!engine || isLoading)}
          >
            Run Hooks
          </button>
        </div>
      )}

      <div style={editorContainerStyle}>
        <Editor
          height={height}
          language={languageMap[language]}
          value={content}
          onChange={handleEditorChange}
          theme="vs-light"
          options={{
            minimap: { enabled: false },
            wordWrap: 'on',
            fontSize: 13,
            scrollBeyondLastLine: false,
            quickSuggestions: language === 'turtle' ? { other: true, comments: false, strings: false } : undefined,
            suggestOnTriggerCharacters: language === 'turtle' ? true : false,
            parameterHints: { enabled: language === 'turtle' },
            wordBasedSuggestions: language === 'turtle' ? 'matchingDocuments' : 'off' as const,
          }}
          onMount={handleEditorMount}
        />
      </div>
    </div>
  );
}
