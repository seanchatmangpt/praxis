/**
 * lib/index.js
 * -----------
 * Barrel export for all testing libraries.
 */

// Fixture generation
export {
  generateFixtures,
  normalizeTurtle,
  indexFixtures,
} from './fixture-generator.js';

// Red-team mutators
export {
  mutator_BrokenSyntax,
  mutator_UnsupportedOWLRL,
  mutator_RemoveSHACLProperty,
  mutator_WrongShExDatatype,
  mutator_OverflowHooks,
  mutator_UnknownPredicate,
  mutator_TamperAfterHash,
  mutator_N3Denial,
  executeAllMutators,
  summarizeMutatorResults,
} from './red-team-mutators.js';

// Reporting
export {
  generateMarkdownReport,
  createReport,
  exportReportJSON,
  exportReportMarkdown,
  hashReport,
} from './report.js';
