/**
 * Unit tests for TurtleEditor component.
 * Verifies editor initialization, content display, and onChange callback.
 */

import React from 'react';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import TurtleEditor from '../TurtleEditor';

// Mock @monaco-editor/react
vi.mock('@monaco-editor/react', () => ({
  default: ({
    value,
    onChange,
    onMount,
  }: {
    value?: string;
    onChange?: (value?: string) => void;
    onMount?: (editor: any, monaco: any) => void;
  }) => {
    // Trigger onMount callback with mock editor and monaco
    React.useEffect(() => {
      const mockEditor = {
        getModel: vi.fn(() => ({
          uri: { path: '/test.ttl' },
          getValue: vi.fn(() => value || ''),
        })),
        getValue: vi.fn(() => value || ''),
      };

      const mockMonaco = {
        editor: {
          defineTheme: vi.fn(),
        },
        languages: {
          register: vi.fn(),
          setMonarchTokensProvider: vi.fn(),
          registerCompletionItemProvider: vi.fn(),
        },
      };

      if (onMount) {
        onMount(mockEditor, mockMonaco);
      }
    }, [value, onMount]);

    return (
      <div data-testid="monaco-editor-container" style={{ position: 'relative' }}>
        <div data-testid="editor-content" style={{ whiteSpace: 'pre-wrap' }}>
          {value || ''}
        </div>
        <input
          data-testid="editor-input"
          type="text"
          value={value || ''}
          onChange={(e) => onChange?.(e.target.value)}
          placeholder="Editor input"
          style={{ display: 'none' }}
        />
      </div>
    );
  },
}));

// Mock graphlaw-wasm module
vi.mock('@/lib/graphlaw-wasm', () => ({
  initGraphlawEngine: vi.fn(async () => ({
    graphHash: vi.fn(async () => 'mock-hash-123'),
    runHooks: vi.fn(async () => ({ status: 'ADMITTED', verdicts: [] })),
  })),
}));

// Mock turtle language configuration modules
vi.mock('@/monaco/turtle-language', () => ({
  registerTurtleLanguage: vi.fn(),
  configureTurtleLanguage: vi.fn(),
}));

vi.mock('@/monaco/turtle-completions', () => ({
  registerTurtleCompletions: vi.fn(),
}));

vi.mock('@/monaco/diagnostics', () => ({
  watchTurtleDiagnostics: vi.fn(() => vi.fn()),
  validateTurtleOnce: vi.fn(async () => ({ conforms: true })),
}));

describe('TurtleEditor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('test_editor_renders', () => {
    it('renders editor container when component mounts', async () => {
      render(<TurtleEditor />);

      await waitFor(() => {
        expect(screen.getByTestId('monaco-editor-container')).toBeTruthy();
      });
    });

    it('renders with default language turtle', async () => {
      const sampleTurtle = `@prefix ex: <http://example.org/> .
ex:subject ex:predicate ex:object .`;

      render(<TurtleEditor content={sampleTurtle} language="turtle" />);

      await waitFor(() => {
        expect(screen.getByTestId('editor-content')).toBeTruthy();
      });
    });

    it('renders without error when language is not turtle', async () => {
      render(<TurtleEditor language="sparql" />);

      await waitFor(() => {
        expect(screen.getByTestId('monaco-editor-container')).toBeTruthy();
      });
    });

    it('accepts height prop and renders container', async () => {
      render(<TurtleEditor height="600px" />);

      await waitFor(() => {
        expect(screen.getByTestId('monaco-editor-container')).toBeTruthy();
      });
    });
  });

  describe('test_displays_initial_value', () => {
    it('displays initial content passed via content prop', async () => {
      const sampleContent = '@prefix ex: <http://example.org/> .\nex:s ex:p ex:o .';

      render(<TurtleEditor content={sampleContent} />);

      await waitFor(() => {
        const editorContent = screen.getByTestId('editor-content');
        expect(editorContent.textContent).toContain(sampleContent);
      });
    });

    it('displays empty string by default', async () => {
      render(<TurtleEditor />);

      await waitFor(() => {
        const editorContent = screen.getByTestId('editor-content');
        expect(editorContent.textContent).toBe('');
      });
    });

    it('updates displayed content when content prop changes', async () => {
      const { rerender } = render(<TurtleEditor content="initial" />);

      await waitFor(() => {
        expect(screen.getByTestId('editor-content').textContent).toContain('initial');
      });

      rerender(<TurtleEditor content="updated" />);

      await waitFor(() => {
        expect(screen.getByTestId('editor-content').textContent).toContain('updated');
      });
    });
  });

  describe('test_on_change_callback', () => {
    it('calls onChange callback when editor content changes', async () => {
      const onChange = vi.fn();

      render(<TurtleEditor onChange={onChange} content="" />);

      await waitFor(() => {
        expect(screen.getByTestId('editor-input')).toBeTruthy();
      });

      const input = screen.getByTestId('editor-input') as HTMLInputElement;
      fireEvent.change(input, { target: { value: 'new content' } });

      expect(onChange).toHaveBeenCalled();
    });

    it('onChange receives the updated content as parameter', async () => {
      const onChange = vi.fn();

      render(<TurtleEditor onChange={onChange} content="" />);

      await waitFor(() => {
        const input = screen.getByTestId('editor-input') as HTMLInputElement;
        fireEvent.change(input, { target: { value: 'updated turtle content' } });

        expect(onChange).toHaveBeenCalledWith('updated turtle content');
      });
    });

    it('does not call onChange if no callback provided', async () => {
      render(<TurtleEditor content="" />);

      await waitFor(() => {
        const input = screen.getByTestId('editor-input') as HTMLInputElement;
        // Should not throw or error
        fireEvent.change(input, { target: { value: 'test' } });
      });
    });

    it('onChange is called multiple times for multiple edits', async () => {
      const onChange = vi.fn();

      render(<TurtleEditor onChange={onChange} content="" />);

      await waitFor(() => {
        const input = screen.getByTestId('editor-input') as HTMLInputElement;
        fireEvent.change(input, { target: { value: 'first' } });
        fireEvent.change(input, { target: { value: 'second' } });

        expect(onChange).toHaveBeenCalledTimes(2);
      });
    });
  });
});
