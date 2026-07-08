/**
 * example-usage.js
 * ----------------
 * Example usage of fixture generation and red-team testing system.
 * Demonstrates the full workflow from Turtle → fixtures → mutators → report.
 */

import {
  generateFixtures,
  indexFixtures,
  executeAllMutators,
  summarizeMutatorResults,
  createReport,
  exportReportMarkdown,
} from './index.js';

// Example base Turtle (valid hooks + SHACL shapes)
const EXAMPLE_TURTLE = `
@prefix : <http://example.org/> .
@prefix hook: <http://hook.example.org/> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Sample hooks
hook:hook_1 a hook:Hook ;
  hook:name "First Hook" ;
  hook:order 1 ;
  hook:enabled true .

hook:hook_2 a hook:Hook ;
  hook:name "Second Hook" ;
  hook:order 2 ;
  hook:enabled true .

hook:hook_3 a hook:Hook ;
  hook:name "Third Hook" ;
  hook:order 3 ;
  hook:enabled false .

# SHACL shapes
:HookShape a sh:NodeShape ;
  sh:targetClass hook:Hook ;
  sh:property [
    sh:path hook:name ;
    sh:datatype xsd:string ;
    sh:minCount 1 ;
  ] ;
  sh:property [
    sh:path hook:order ;
    sh:datatype xsd:integer ;
    sh:minCount 1 ;
  ] ;
  sh:property [
    sh:path hook:enabled ;
    sh:datatype xsd:boolean ;
  ] .
`;

/**
 * Example validation function (stub — replace with your actual validator).
 * @param {string} turtle - Turtle to validate
 * @returns {Promise<{status: string}>}
 */
async function exampleValidateFn(turtle) {
  // Simulate validation with a small delay
  await new Promise((resolve) => setTimeout(resolve, 100));

  // Check for basic syntax
  if (!turtle.includes('@prefix')) {
    return { status: 'SyntaxError', error: 'Missing @prefix declaration' };
  }

  // Check for too many hooks
  const hookMatches = turtle.match(/hook:hook_\d+/g) || [];
  if (hookMatches.length > 12) {
    return { status: 'OverflowError', error: 'Too many hooks (max 12)' };
  }

  // Check for unknown predicates
  if (/hook:[a-z_]*Predicate/i.test(turtle)) {
    return { status: 'UnknownPredicateError', error: 'Unknown predicate' };
  }

  // Check for SHACL properties
  if (!turtle.includes('sh:minCount') && turtle.includes('sh:targetClass')) {
    return { status: 'ValidationFailed', error: 'Missing sh:minCount' };
  }

  // Default to success
  return { status: 'Success' };
}

/**
 * Run the full example workflow.
 */
export async function runExampleWorkflow() {
  console.log('=== Fixture Generation & Red-Team Testing Example ===\n');

  // Step 1: Generate fixtures
  console.log('Step 1: Generating fixtures from base Turtle...');
  const fixtures = generateFixtures(EXAMPLE_TURTLE);
  const indexed = indexFixtures(fixtures);
  console.log(`✓ Generated ${indexed.total} fixture variants`);
  console.log(`  Categories: ${Object.keys(indexed.byCategory).join(', ')}\n`);

  // Step 2: Show fixture summary
  console.log('Step 2: Fixture Summary');
  for (const [category, variants] of Object.entries(indexed.byCategory)) {
    console.log(`  ${category}: ${variants.length} variant(s)`);
    for (const v of variants) {
      console.log(`    - ${v.name} (expects: ${v.expectedStatus})`);
    }
  }
  console.log();

  // Step 3: Run mutators
  console.log('Step 3: Running red-team mutators...');
  const mutatorResults = await executeAllMutators(EXAMPLE_TURTLE, exampleValidateFn);
  const summary = summarizeMutatorResults(mutatorResults);
  console.log(`✓ Ran ${summary.totalMutators} mutators`);
  console.log(`  Passed: ${summary.passed}`);
  console.log(`  Failed: ${summary.failed}`);
  console.log(`  Pass Rate: ${summary.passRate}\n`);

  // Step 4: Show mutator results
  console.log('Step 4: Mutator Results');
  for (const result of mutatorResults) {
    const icon = result.passed ? '✓' : '✗';
    console.log(`  ${icon} ${result.name}`);
    console.log(`     Expected: ${result.expectedStatus}, Actual: ${result.actualStatus}`);
    if (result.errorMessage) {
      console.log(`     Error: ${result.errorMessage}`);
    }
  }
  console.log();

  // Step 5: Generate report
  console.log('Step 5: Generating report...');
  const report = createReport({
    title: 'Praxis Red-Team Validation Report (Example)',
    mutatorResults,
    hookTests: [],
    timestamp: new Date().toISOString(),
  });
  console.log(`✓ Report generated with hash: ${report.hash}\n`);

  // Step 6: Show report
  console.log('Step 6: Report Markdown');
  console.log('---');
  console.log(report.markdown);
  console.log('---\n');

  // Step 7: Export
  console.log('Step 7: Export Options');
  console.log(`  - JSON export available`);
  console.log(`  - Markdown export available`);
  console.log(`  - Report hash: ${report.hash}`);
  console.log();

  return {
    fixtures: indexed,
    mutatorResults,
    summary,
    report,
  };
}

/**
 * Example: Load a fixture and run mutators on it.
 */
export async function runFixtureExample(fixtureVariant) {
  console.log(`\n=== Running Fixture: ${fixtureVariant.name} ===\n`);

  const result = await exampleValidateFn(fixtureVariant.turtle);
  console.log(`Turtle length: ${fixtureVariant.turtle.length} chars`);
  console.log(`Expected Status: ${fixtureVariant.expectedStatus}`);
  console.log(`Actual Status: ${result.status}`);
  console.log(`Match: ${result.status === fixtureVariant.expectedStatus ? '✓' : '✗'}`);

  if (result.error) {
    console.log(`Error: ${result.error}`);
  }

  return result;
}

// Export for testing
export { EXAMPLE_TURTLE, exampleValidateFn };
