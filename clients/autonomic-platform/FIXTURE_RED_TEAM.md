# Fixture Generation & Red-Team Testing System

## Overview

The fixture generation and red-team testing system validates Praxis hooks/shapes Turtle inputs through adversarial mutation testing. The system generates test variants, runs deliberate failure mutations, and produces deterministic reports with cryptographic hashing.

## Architecture

### 1. Fixture Generator (`lib/fixture-generator.js`)

Derives test fixture variants from a valid Turtle input. Each variant represents a different failure mode:

- **Missing required SHACL property** — Remove `sh:minCount`, `sh:datatype`, or `sh:path`
- **Wrong datatype** — Change `xsd:integer` to `xsd:string` or other incompatible types
- **Overflow to 13+ hooks** — Exceeds maximum of 12 hooks
- **Unknown predicate** — Uses undefined `hook:*` or `kh:*` predicates
- **Broken Turtle syntax** — Removes trailing periods, malformed N-Triples
- **Unsupported OWL RL feature** — Injects `owl:intersectionOf`, `owl:disjointWith`
- **Removed shape property** — Deletes required SHACL constraints
- **Tampering (hash mismatch)** — Adds triples after hash computed
- **N3 denial violation** — Violates denial rule constraints

**Usage:**
```javascript
import { generateFixtures, indexFixtures } from './lib/fixture-generator.js';

const baseTurtle = `...`;
const variants = generateFixtures(baseTurtle);
const indexed = indexFixtures(variants);

console.log(indexed.total);        // 9 variants
console.log(indexed.byCategory);   // Organized by error type
```

### 2. Red-Team Mutators (`lib/red-team-mutators.js`)

Eight adversarial mutators that deliberately break Turtle inputs. Each mutator:
1. Applies a specific transformation
2. Executes validation via `validateFn(mutatedTurtle)`
3. Asserts expected Status vs actual Status

**Status Enum:**
- `Success` — Valid
- `ValidationFailed` — SHACL/ShEx validation error
- `TypeError` — Type mismatch
- `SyntaxError` — Malformed Turtle
- `UnknownPredicateError` — Unknown vocabulary predicate
- `OverflowError` — Exceeds limits (e.g., 13 hooks)
- `UnsupportedFeatureError` — Profile does not support feature
- `HashMismatchError` — Receipt/hash invalid
- `DenialViolationError` — N3 denial rule violated

**API:**
```javascript
import { executeAllMutators, summarizeMutatorResults } from './lib/red-team-mutators.js';

const baseTurtle = `...`;

// Define a validation function (yours)
const validateFn = async (turtle) => {
  // Return: { status: 'Success' | error }
};

const results = await executeAllMutators(baseTurtle, validateFn);
// results: Array<{id, name, mutation, actualStatus, expectedStatus, passed, errorMessage}>

const summary = summarizeMutatorResults(results);
console.log(summary.passRate);     // "87.5%"
console.log(summary.failedMutators);
```

### 3. Report Generation (`lib/report.js`)

Renders markdown reports and computes deterministic report hashes.

**Report Shape:**
```javascript
{
  title: string,
  markdown: string,              // Markdown report
  hash: string,                  // Hex digest (stable-stringify + hash)
  generatedAt: string,           // ISO timestamp
  data: {                        // Source data
    title, mutatorResults, hookTests, timestamp
  }
}
```

**Capability Matrix:**
- Syntax Validation
- OWL RL Profile Enforcement
- SHACL Validation
- Type Checking
- Overflow Detection
- Vocabulary Enforcement
- Receipt Verification
- N3 Denial Rules

**Usage:**
```javascript
import { createReport, exportReportJSON, exportReportMarkdown } from './lib/report.js';

const report = createReport({
  title: 'Praxis Red-Team Validation Report',
  mutatorResults: results,     // From executeAllMutators()
  hookTests: [],
  timestamp: new Date().toISOString(),
});

// Export
const json = exportReportJSON(report);
const md = exportReportMarkdown(report);

console.log(report.hash);      // Deterministic hex digest
```

## React Components

### 1. FixtureExplorer

Browse and load fixture variants.

**Props:**
- `baseTurtle` — Turtle input to generate fixtures from
- `onFixtureSelect(fixture)` — Called when fixture selected
- `onLoadFixture(fixture)` — Called when "Load & Run" clicked

**Features:**
- List fixtures by category
- Preview Turtle for each variant
- Click to load/run fixture

### 2. RedTeamPanel

Run mutators individually or all at once.

**Props:**
- `baseTurtle` — Turtle to mutate
- `validateFn(turtle) => Promise<{status: string}>` — Validation function
- `onResultsUpdate(results)` — Called when mutator runs

**Features:**
- Run individual mutators
- Run all mutators at once
- Live Status display (✓ pass / ✗ fail)
- Summary table
- Detailed result view

### 3. ReportViewer

Display markdown report with hash and export options.

**Props:**
- `report` — Report object from `createReport()`
- `onExport(report)` — Called on export (optional)

**Features:**
- Render markdown with tables
- Display report hash
- Copy hash to clipboard
- Export as JSON or Markdown

### 4. FixtureTestHarness

Integrated testing harness combining all components.

**Props:**
- `baseTurtle` — Turtle input
- `validateFn(turtle) => Promise<{status: string}>` — Validation function

**Workflow:**
1. Load fixtures (Fixtures tab)
2. Click "Load & Run Fixture" → switches to Mutators tab
3. Run mutators individually or all at once
4. Click "Generate Report" → switches to Report tab
5. View/export report with hash

## Integration Example

Add to your app (e.g., in a test harness screen):

```javascript
import FixtureTestHarness from './components/FixtureTestHarness.jsx';

function MyTestScreen() {
  const baseTurtle = `
    @prefix hook: <http://example.org/hook/> .
    hook:example a hook:Hook ; hook:name "Example" .
  `;

  // Your validation function
  const validateTurtle = async (turtle) => {
    try {
      // Call your validator (e.g., GraphLaw, SHACL processor)
      const result = await myValidator(turtle);
      return { status: result.status || 'Success' };
    } catch (err) {
      return { status: 'SyntaxError', error: err.message };
    }
  };

  return (
    <FixtureTestHarness
      baseTurtle={baseTurtle}
      validateFn={validateTurtle}
    />
  );
}
```

## File Locations

```
src/
├── lib/
│   ├── fixture-generator.js    (9 variants)
│   ├── red-team-mutators.js    (8 mutators)
│   ├── report.js               (markdown + hashing)
│   └── index.js                (barrel export)
├── components/
│   ├── FixtureExplorer.jsx     (fixture browser)
│   ├── RedTeamPanel.jsx        (mutator runner)
│   ├── ReportViewer.jsx        (report display)
│   └── FixtureTestHarness.jsx  (integrated harness)
```

## Data Flow

```
Turtle Input
    ↓
FixtureExplorer (browse variants)
    ↓ (select fixture)
    ↓
RedTeamPanel (load → run mutators)
    ├── Mutator 1 (syntax break) → validateFn() → Status
    ├── Mutator 2 (OWL RL) → validateFn() → Status
    ├── ... (8 total)
    ↓
Generate Report (createReport)
    ↓
ReportViewer (markdown + hash)
    ├── Capability Matrix (✓/✗ per feature)
    ├── Hook Verdicts (per-hook status)
    ├── Mutator Results (detailed)
    └── Report Hash (stable)
```

## Deterministic Hashing

The report hash is computed deterministically:
1. Source data normalized via `stableStringify()` (sorted keys, no whitespace)
2. Markdown concatenated
3. `simpleHash()` produces hex digest (portable, no external dependencies)

In production, replace with BLAKE3:
```javascript
import blake3 from 'blake3';

function hashReport(markdown, data) {
  const normalized = stableStringify(data) + markdown;
  return blake3(normalized).toString('hex');
}
```

## Testing Exit Criteria

Click "Generate Report" → see:
- ✓ Markdown report with capability matrix + hook verdicts + hashes
- ✓ Report hash displayed and copyable
- ✓ Export buttons (JSON + Markdown) working

Run a red-team mutator → see:
- ✓ Status changes from expected (e.g., `ValidationFailed`)
- ✓ Mutator marked ✓ (pass) or ✗ (fail)
- ✓ Detailed error message (if applicable)

## Dependencies

- React (no additional npm packages for mutation/report logic)
- Existing codebase validation function (you provide `validateFn`)

## Future Enhancements

1. **Real BLAKE3 hashing** — Replace `simpleHash()` with actual BLAKE3
2. **Fixture persistence** — Save/load fixture collections
3. **Mutator history** — Track results over time
4. **Comparison reports** — Diff two reports
5. **Automated CI integration** — Run on push, gate on mutator pass rate
6. **Profile-specific mutators** — Add mutators for specific OWL RL/SHACL profiles
