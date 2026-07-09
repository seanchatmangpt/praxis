/**
 * Monaco integration exports for Turtle language support.
 * Includes syntax highlighting, completions, and diagnostics.
 */

export {
  registerTurtleLanguage,
  configureTurtleLanguage,
} from './turtle-language';

export {
  TurtleVocabulary,
  registerTurtleCompletions,
} from './turtle-completions';

export {
  DiagnosticSeverity,
  parseDetailLocation,
  validationResultToMarkers,
  watchTurtleDiagnostics,
  validateTurtleOnce,
  clearDiagnostics,
} from './diagnostics';
