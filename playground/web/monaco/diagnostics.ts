/**
 * Monaco diagnostics provider for Turtle language.
 *
 * Transforms ValidationResult from GraphLaw engine into Monaco editor markers.
 * Maps violations to (line, column) positions for inline error display.
 */

// Type-only: importing monaco-editor as a value here would statically pull
// the full editor bundle into the webpack/Next.js graph, which fails to
// resolve monaco's internal AMD loader chunk (`vs/nls.messages-loader`)
// outside a dedicated monaco webpack plugin. `@monaco-editor/react`'s own
// loader already supplies a working, already-instantiated monaco object via
// `onMount`/`beforeMount`; every function here that needs the live API
// takes that instance as a parameter instead of importing the package.
import type * as Monaco from 'monaco-editor';
import type { ValidationResult, ShaclViolation } from '../lib/graphlaw-wasm';

export type { ValidationResult, ShaclViolation };

/**
 * Severity levels as understood by Monaco.
 * Maps from our domain to Monaco's MarkerSeverity enum.
 */
export enum DiagnosticSeverity {
  /** Critical validation failure; blocks semantic reasoning */
  Error = 8,

  /** Violates best practices but doesn't block reasoning */
  Warning = 4,

  /** Informational hint */
  Information = 2,

  /** Hint for potential optimizations */
  Hint = 1,
}

/**
 * Converts DiagnosticSeverity to Monaco's MarkerSeverity.
 *
 * Both enums use the same numeric values:
 * - Error: 8
 * - Warning: 4
 * - Information: 2
 * - Hint: 1
 */
export function convertDiagnosticToMarkerSeverity(
  monaco: typeof Monaco,
  diagnostic: DiagnosticSeverity
): Monaco.MarkerSeverity {
  switch (diagnostic) {
    case DiagnosticSeverity.Error:
      return monaco.MarkerSeverity.Error;
    case DiagnosticSeverity.Warning:
      return monaco.MarkerSeverity.Warning;
    case DiagnosticSeverity.Information:
      return monaco.MarkerSeverity.Info;
    case DiagnosticSeverity.Hint:
      return monaco.MarkerSeverity.Hint;
    default:
      return monaco.MarkerSeverity.Error;
  }
}

/**
 * Parses a diagnostic detail string to extract line and column.
 *
 * Expected formats:
 * - "line 5, column 10"
 * - "L5:C10"
 * - "5:10"
 *
 * Returns { line, column } or null if parsing fails.
 */
export function parseDetailLocation(detail: string): { line: number; column: number } | null {
  // Try "line X, column Y" format
  const lineColMatch = detail.match(/line\s+(\d+),\s*column\s+(\d+)/i);
  if (lineColMatch) {
    return {
      line: parseInt(lineColMatch[1], 10),
      column: parseInt(lineColMatch[2], 10),
    };
  }

  // Try "L5:C10" format
  const lcMatch = detail.match(/L(\d+):C(\d+)/);
  if (lcMatch) {
    return {
      line: parseInt(lcMatch[1], 10),
      column: parseInt(lcMatch[2], 10),
    };
  }

  // Try "5:10" format (line:column)
  const colonMatch = detail.match(/^(\d+):(\d+)/);
  if (colonMatch) {
    return {
      line: parseInt(colonMatch[1], 10),
      column: parseInt(colonMatch[2], 10),
    };
  }

  return null;
}

/**
 * Converts a ValidationResult to Monaco IMarker array.
 *
 * Each SHACL violation becomes a marker with:
 * - range: derived from focusNode and resultPath (or from detail string)
 * - message: resultMessage
 * - severity: Error (Violation) / Warning / Info
 * - source: "Turtle/SHACL" for filtering
 */
export function validationResultToMarkers(
  monaco: typeof Monaco,
  result: ValidationResult
): Monaco.editor.IMarker[] {
  const markers: Monaco.editor.IMarker[] = [];

  // Convert SHACL violations
  if (result.violations) {
    result.violations.forEach((violation) => {
      const severity =
        violation.severity === 'Violation'
          ? DiagnosticSeverity.Error
          : violation.severity === 'Warning'
            ? DiagnosticSeverity.Warning
            : DiagnosticSeverity.Information;

      // Try to extract line/column from detail or use a default range
      const location = parseDetailLocation(
        `${violation.focusNode} ${violation.resultPath}`
      ) || { line: 1, column: 1 };

      markers.push({
        owner: 'praxis-shacl',
        resource: monaco.Uri.parse('memory://praxis/validation'),
        startLineNumber: location.line,
        startColumn: location.column,
        endLineNumber: location.line,
        endColumn: Math.max(location.column + 10, location.column + 1),
        message: `${violation.resultMessage} (at ${violation.focusNode})`,
        severity: convertDiagnosticToMarkerSeverity(monaco, severity),
        source: 'SHACL Validation',
        code: violation.resultPath,
      });
    });
  }

  // Convert denial rule violations (from negation-as-failure)
  if (result.denials) {
    result.denials.forEach((denial, index) => {
      // Denial violations don't have line info; use incrementing lines
      markers.push({
        owner: 'praxis-validation',
        resource: monaco.Uri.parse('memory://praxis/validation'),
        startLineNumber: 1 + index,
        startColumn: 1,
        endLineNumber: 1 + index,
        endColumn: 80,
        message: `Denial rule violated: ${denial.ruleId} (${denial.triggeredFacts.join(', ')})`,
        severity: convertDiagnosticToMarkerSeverity(monaco, DiagnosticSeverity.Error),
        source: 'Datalog Denial',
        code: denial.ruleId,
      });
    });
  }

  // Convert ShEx validation failures
  if (result.shexFailures) {
    result.shexFailures.forEach((failure) => {
      const location = parseDetailLocation(failure.nodeId) || { line: 1, column: 1 };

      markers.push({
        owner: 'praxis-validation',
        resource: monaco.Uri.parse('memory://praxis/validation'),
        startLineNumber: location.line,
        startColumn: location.column,
        endLineNumber: location.line,
        endColumn: location.column + 20,
        message: `ShEx validation failed: ${failure.reason} (expected ${failure.shapeLabel})`,
        severity: convertDiagnosticToMarkerSeverity(monaco, DiagnosticSeverity.Warning),
        source: 'ShEx Validation',
        code: failure.shapeLabel,
      });
    });
  }

  // If detail contains structured error info, add as additional marker
  if (result.detail && !result.conforms) {
    const location = parseDetailLocation(result.detail) || { line: 1, column: 1 };
    markers.push({
      owner: 'praxis-validation',
      resource: monaco.Uri.parse('memory://praxis/validation'),
      startLineNumber: location.line,
      startColumn: location.column,
      endLineNumber: location.line,
      endColumn: location.column + 20,
      message: result.detail,
      severity: convertDiagnosticToMarkerSeverity(monaco, DiagnosticSeverity.Error),
      source: 'Validation',
    });
  }

  return markers;
}

/**
 * Watches a Monaco editor model for changes and runs diagnostics via the GraphLaw engine.
 *
 * Usage:
 *   const editor = monaco.editor.create(...);
 *   const model = editor.getModel()!;
 *   const cleanup = watchTurtleDiagnostics(model, engine);
 *
 *   // Later:
 *   cleanup(); // Stop watching
 */
export function watchTurtleDiagnostics(
  monaco: typeof Monaco,
  model: Monaco.editor.ITextModel,
  engine: any, // GraphlawEngineInterface from graphlaw-wasm.ts
  debounceMs: number = 1000
): () => void {
  let timeoutId: NodeJS.Timeout | null = null;
  let lastValidationId = 0;

  /**
   * Runs validation and updates markers.
   * Debounced to avoid excessive engine calls.
   */
  async function runValidation(): Promise<void> {
    const source = model.getValue();
    const validationId = ++lastValidationId;

    try {
      const result = await engine.validateAll(source);

      // Ignore results from stale validation calls
      if (validationId !== lastValidationId) return;

      const markers = validationResultToMarkers(monaco, result);
      monaco.editor.setModelMarkers(model, 'turtle-validator', markers);
    } catch (error) {
      console.error('Validation error:', error);
      // Set a single error marker on the first line if validation fails
      monaco.editor.setModelMarkers(model, 'turtle-validator', [
        {
          startLineNumber: 1,
          startColumn: 1,
          endLineNumber: 1,
          endColumn: 80,
          message:
            error instanceof Error
              ? error.message
              : 'Validation failed (see console)',
          severity: convertDiagnosticToMarkerSeverity(monaco, DiagnosticSeverity.Error),
          source: 'Validation Engine',
        },
      ]);
    }
  }

  /**
   * Event handler for model changes.
   * Debounced to reduce engine load.
   */
  function onDidChangeContent(): void {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }
    timeoutId = setTimeout(() => {
      runValidation();
    }, debounceMs);
  }

  // Subscribe to model changes
  const disposable = model.onDidChangeContent(() => onDidChangeContent());

  // Run initial validation
  runValidation();

  // Return cleanup function
  return () => {
    disposable.dispose();
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }
  };
}

/**
 * One-shot validation: validate a model without watching for changes.
 *
 * Usage:
 *   const result = await validateTurtleOnce(model, engine);
 */
export async function validateTurtleOnce(
  monaco: typeof Monaco,
  model: Monaco.editor.ITextModel,
  engine: any // GraphlawEngineInterface
): Promise<ValidationResult | null> {
  const source = model.getValue();

  try {
    const result = await engine.validateAll(source);
    const markers = validationResultToMarkers(monaco, result);
    monaco.editor.setModelMarkers(model, 'turtle-validator', markers);
    return result;
  } catch (error) {
    console.error('Validation error:', error);
    return null;
  }
}

/**
 * Clears all validation markers from a model.
 */
export function clearDiagnostics(monaco: typeof Monaco, model: Monaco.editor.ITextModel): void {
  monaco.editor.setModelMarkers(model, 'turtle-validator', []);
}
