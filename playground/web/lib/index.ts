/**
 * Main exports for the Praxis Playground library.
 * Includes GraphLaw engine integration and Monaco editor setup.
 */

export {
  initGraphlawEngine,
  getNextCallId,
  validateWithCallId,
  runHooksWithCallId,
  graphHashWithCallId,
  type GraphlawEngineInterface,
  type ValidationResult,
  type ShaclViolation,
  type ShExFailure,
  type DenialViolation,
  type HookExecutionResult,
  type HookReceipt,
  type HookMetadata,
  type CallResult,
} from './graphlaw-wasm';

export * from './types';
export * from './utils';
export * from './fixtures';
export * from './engine';
