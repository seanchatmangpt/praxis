/**
 * report.js
 * ---------
 * Render markdown reports for hook verdicts and fixture testing.
 * Compute report hash (stable-stringify + BLAKE3 equivalent).
 *
 * Report shape:
 *   { title, sections, capabilityMatrix, hookVerdicts, hash, generated }
 */

/**
 * Stable JSON stringify for hashing (sorted keys, no whitespace).
 * @param {any} obj
 * @returns {string}
 */
function stableStringify(obj) {
  if (obj === null || obj === undefined) return 'null';
  if (typeof obj !== 'object') {
    if (typeof obj === 'string') return `"${obj.replace(/"/g, '\\"')}"`;
    return String(obj);
  }
  if (Array.isArray(obj)) {
    return '[' + obj.map(stableStringify).join(',') + ']';
  }
  const keys = Object.keys(obj).sort();
  return '{' + keys.map((k) => `"${k}":${stableStringify(obj[k])}`).join(',') + '}';
}

/**
 * Compute a simple hash of a string (simulating BLAKE3).
 * In production, use actual BLAKE3. This is a standin that produces a hex digest.
 * @param {string} data
 * @returns {string} hex digest
 */
function simpleHash(data) {
  let hash = 0;
  for (let i = 0; i < data.length; i++) {
    const char = data.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32-bit integer
  }
  return Math.abs(hash).toString(16).padStart(16, '0');
}

/**
 * Capability matrix: which validator features are present and working.
 * @param {object} results - from red-team mutators and fixture tests
 * @returns {Array<object>}
 */
function buildCapabilityMatrix(results) {
  const matrix = [
    {
      capability: 'Syntax Validation',
      tested: results.mutators?.some((m) => m.id === 'mutator-syntax-break') ?? false,
      passed: results.mutators?.find((m) => m.id === 'mutator-syntax-break')?.passed ?? false,
      description: 'Detect Turtle syntax errors (missing periods, broken N-Triples)',
    },
    {
      capability: 'OWL RL Profile Enforcement',
      tested: results.mutators?.some((m) => m.id === 'mutator-unsupported-owl-rl') ?? false,
      passed: results.mutators?.find((m) => m.id === 'mutator-unsupported-owl-rl')?.passed ?? false,
      description: 'Reject unsupported OWL RL features (intersectionOf, etc.)',
    },
    {
      capability: 'SHACL Validation',
      tested: results.mutators?.some((m) => m.id === 'mutator-missing-shacl-property') ?? false,
      passed: results.mutators?.find((m) => m.id === 'mutator-missing-shacl-property')?.passed ?? false,
      description: 'Enforce required SHACL shape properties (minCount, datatype, path)',
    },
    {
      capability: 'Type Checking',
      tested: results.mutators?.some((m) => m.id === 'mutator-wrong-shex-datatype') ?? false,
      passed: results.mutators?.find((m) => m.id === 'mutator-wrong-shex-datatype')?.passed ?? false,
      description: 'Detect datatype mismatches (xsd:integer vs xsd:string)',
    },
    {
      capability: 'Overflow Detection',
      tested: results.mutators?.some((m) => m.id === 'mutator-overflow-hooks') ?? false,
      passed: results.mutators?.find((m) => m.id === 'mutator-overflow-hooks')?.passed ?? false,
      description: 'Enforce maximum hook count (12)',
    },
    {
      capability: 'Vocabulary Enforcement',
      tested: results.mutators?.some((m) => m.id === 'mutator-unknown-predicate') ?? false,
      passed: results.mutators?.find((m) => m.id === 'mutator-unknown-predicate')?.passed ?? false,
      description: 'Reject unknown hook: and kh: predicates',
    },
    {
      capability: 'Receipt Verification',
      tested: results.mutators?.some((m) => m.id === 'mutator-hash-mismatch') ?? false,
      passed: results.mutators?.find((m) => m.id === 'mutator-hash-mismatch')?.passed ?? false,
      description: 'Detect tampering via hash mismatch',
    },
    {
      capability: 'N3 Denial Rules',
      tested: results.mutators?.some((m) => m.id === 'mutator-n3-denial') ?? false,
      passed: results.mutators?.find((m) => m.id === 'mutator-n3-denial')?.passed ?? false,
      description: 'Enforce N3 denial rule constraints',
    },
  ];

  return matrix;
}

/**
 * Hook verdicts: pass/fail summary for each hook test.
 * @param {Array<object>} hookTests - test results
 * @returns {Array<object>}
 */
function buildHookVerdicts(hookTests = []) {
  return (hookTests || []).map((test) => ({
    hook: test.hook || test.id || 'unknown',
    status: test.passed ? 'PASS' : 'FAIL',
    reason: test.reason || (test.passed ? 'Validated successfully' : 'Failed validation'),
    details: test.details || null,
  }));
}

/**
 * Generate markdown report.
 * @param {object} data - { title, mutatorResults, hookTests, timestamp }
 * @returns {string} Markdown
 */
export function generateMarkdownReport(data) {
  const title = data.title || 'Praxis Red-Team Validation Report';
  const timestamp = data.timestamp || new Date().toISOString();
  const mutatorResults = data.mutatorResults || [];
  const hookTests = data.hookTests || [];

  const capabilityMatrix = buildCapabilityMatrix({ mutators: mutatorResults });
  const hookVerdicts = buildHookVerdicts(hookTests);

  const mutatorsPassCount = mutatorResults.filter((m) => m.passed).length;
  const mutatorsTotal = mutatorResults.length;

  let md = `# ${title}\n\n`;
  md += `Generated: ${timestamp}\n\n`;

  // Executive summary
  md += `## Executive Summary\n\n`;
  md += `| Metric | Result |\n`;
  md += `|--------|--------|\n`;
  md += `| Mutators Passed | ${mutatorsPassCount}/${mutatorsTotal} (${((mutatorsPassCount / mutatorsTotal) * 100).toFixed(1)}%) |\n`;
  md += `| Hooks Validated | ${hookVerdicts.length} |\n`;
  md += `| Status | ${mutatorsPassCount === mutatorsTotal ? '✓ All Clear' : '✗ Failures Detected'} |\n\n`;

  // Capability Matrix
  md += `## Capability Matrix\n\n`;
  md += `| Capability | Status | Description |\n`;
  md += `|------------|--------|-------------|\n`;
  for (const cap of capabilityMatrix) {
    const status = cap.tested
      ? cap.passed
        ? '✓ Working'
        : '✗ Failed'
      : '○ Not Tested';
    md += `| ${cap.capability} | ${status} | ${cap.description} |\n`;
  }
  md += '\n';

  // Hook Verdicts
  if (hookVerdicts.length > 0) {
    md += `## Hook Verdicts\n\n`;
    md += `| Hook | Status | Reason |\n`;
    md += `|------|--------|--------|\n`;
    for (const verdict of hookVerdicts) {
      md += `| ${verdict.hook} | ${verdict.status} | ${verdict.reason} |\n`;
    }
    md += '\n';
  }

  // Detailed Mutator Results
  if (mutatorResults.length > 0) {
    md += `## Mutator Results (Detailed)\n\n`;
    for (const result of mutatorResults) {
      const icon = result.passed ? '✓' : '✗';
      md += `### ${icon} ${result.name}\n\n`;
      md += `- **ID**: ${result.id}\n`;
      md += `- **Mutation**: ${result.mutation}\n`;
      md += `- **Expected Status**: ${result.expectedStatus}\n`;
      md += `- **Actual Status**: ${result.actualStatus}\n`;
      if (result.errorMessage) {
        md += `- **Error**: ${result.errorMessage}\n`;
      }
      md += '\n';
    }
  }

  return md;
}

/**
 * Create a full report object with metadata and hash.
 * @param {object} data - { title, mutatorResults, hookTests, timestamp }
 * @returns {object} Report with { markdown, hash, data }
 */
export function createReport(data) {
  const markdown = generateMarkdownReport(data);
  const dataStr = stableStringify(data);
  const hash = simpleHash(dataStr + markdown);

  return {
    title: data.title || 'Praxis Red-Team Validation Report',
    markdown,
    hash,
    generatedAt: data.timestamp || new Date().toISOString(),
    data,
  };
}

/**
 * Export report as JSON (for storage/sharing).
 * @param {object} report
 * @returns {string} JSON
 */
export function exportReportJSON(report) {
  return JSON.stringify({
    title: report.title,
    generatedAt: report.generatedAt,
    hash: report.hash,
    markdown: report.markdown,
    data: report.data,
  }, null, 2);
}

/**
 * Export report as Markdown.
 * @param {object} report
 * @returns {string} Markdown
 */
export function exportReportMarkdown(report) {
  let text = report.markdown;
  text += '\n\n---\n\n';
  text += `**Report Hash**: \`${report.hash}\`\n`;
  text += `**Generated**: ${report.generatedAt}\n`;
  return text;
}

/**
 * Compute hash of report markdown (deterministic).
 * @param {string} markdown
 * @returns {string} hex digest
 */
export function hashReport(markdown) {
  return simpleHash(markdown);
}
