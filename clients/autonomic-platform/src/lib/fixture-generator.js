/**
 * fixture-generator.js
 * ---------------------
 * Derives test fixture variants from loaded hooks/shapes Turtle data.
 * Each variant represents a different failure mode: missing properties,
 * wrong datatypes, overflow conditions, unknown predicates, etc.
 *
 * FixtureVariant shape:
 *   { id, name, description, turtle, expectedStatus, category }
 *   - id: unique fixture identifier
 *   - name: human-readable name
 *   - description: what the variant tests
 *   - turtle: Turtle syntax (may be broken)
 *   - expectedStatus: the Status enum we expect this to trigger
 *   - category: 'missing-property' | 'wrong-datatype' | 'overflow' | 'unknown-predicate' | 'syntax-error'
 */

/**
 * Generate fixture variants from a valid hooks/shapes Turtle input.
 * @param {string} baseTurtle - Valid Turtle syntax with hooks and SHACL shapes
 * @returns {Array<FixtureVariant>}
 */
export function generateFixtures(baseTurtle) {
  const variants = [];

  // Variant 1: Missing required SHACL property (e.g., minCount)
  variants.push({
    id: 'fixture-missing-required-shape',
    name: 'Missing Required SHACL minCount',
    description: 'Remove sh:minCount from a required property shape',
    turtle: baseTurtle.replace(
      /sh:minCount\s+1\s*;/g,
      '# sh:minCount removed'
    ),
    expectedStatus: 'ValidationFailed',
    category: 'missing-property',
  });

  // Variant 2: Wrong datatype
  variants.push({
    id: 'fixture-wrong-datatype',
    name: 'Wrong Datatype',
    description: 'Change sh:datatype to incompatible type (string instead of integer)',
    turtle: baseTurtle.replace(
      /sh:datatype\s+xsd:integer/g,
      'sh:datatype xsd:string'
    ),
    expectedStatus: 'TypeError',
    category: 'wrong-datatype',
  });

  // Variant 3: Overflow to 13 hooks (when max is 12)
  variants.push({
    id: 'fixture-overflow-hooks',
    name: 'Overflow: 13 Hooks',
    description: 'Define 13 hooks when maximum is 12',
    turtle: baseTurtle + '\n' + Array.from({ length: 13 }, (_, i) =>
      `hook:hook_${i} a hook:Hook ; hook:name "Hook ${i}" ; hook:order ${i} .`
    ).join('\n'),
    expectedStatus: 'OverflowError',
    category: 'overflow',
  });

  // Variant 4: Unknown predicate
  variants.push({
    id: 'fixture-unknown-predicate',
    name: 'Unknown Predicate',
    description: 'Use an unknown hook: or kh: predicate',
    turtle: baseTurtle + '\n' +
      'hook:unknown_hook a hook:Hook ;\n' +
      '  hook:unknownPredicate "not-allowed" ;\n' +
      '  hook:name "Test" .\n',
    expectedStatus: 'UnknownPredicateError',
    category: 'unknown-predicate',
  });

  // Variant 5: Turtle syntax broken (missing trailing period)
  variants.push({
    id: 'fixture-broken-syntax',
    name: 'Broken Turtle Syntax',
    description: 'Remove trailing period from a triple',
    turtle: baseTurtle.replace(/(\S+\s+\S+\s+[^.]+)\./, '$1'),
    expectedStatus: 'SyntaxError',
    category: 'syntax-error',
  });

  // Variant 6: Inject OWL RL feature that profile refuses
  variants.push({
    id: 'fixture-unsupported-owl-rl',
    name: 'Unsupported OWL RL Feature',
    description: 'Use owl:onProperty with restriction the profile does not support',
    turtle: baseTurtle + '\n' +
      'hook:restricted a owl:Class ;\n' +
      '  rdfs:subClassOf [ a owl:Restriction ;\n' +
      '    owl:onProperty hook:forbiddenProperty ;\n' +
      '    owl:maxQualifiedCardinality 5 ;\n' +
      '    owl:onClass hook:ForbiddenClass\n' +
      '  ] .\n',
    expectedStatus: 'UnsupportedFeatureError',
    category: 'unsupported-feature',
  });

  // Variant 7: Remove required SHACL property in a shape
  variants.push({
    id: 'fixture-missing-shape-property',
    name: 'Missing Shape Property',
    description: 'Remove sh:path from a property shape',
    turtle: baseTurtle.replace(
      /sh:path\s+[a-zA-Z:_][a-zA-Z0-9:_-]*\s*;/,
      '# sh:path removed;'
    ),
    expectedStatus: 'ValidationFailed',
    category: 'missing-property',
  });

  // Variant 8: Tamper graph after hashing (HASH_MISMATCH)
  variants.push({
    id: 'fixture-hash-mismatch',
    name: 'Hash Mismatch',
    description: 'Add triple after hash was computed',
    turtle: baseTurtle + '\n' +
      '# This triple was added after hashing was complete\n' +
      'hook:tampered hook:status "altered" .\n',
    expectedStatus: 'HashMismatchError',
    category: 'tampering',
  });

  // Variant 9: N3 denial rule that trips validation
  variants.push({
    id: 'fixture-n3-denial',
    name: 'N3 Denial Rule Violation',
    description: 'Violate an N3 denial rule',
    turtle: baseTurtle + '\n' +
      '{ ?h a hook:Hook ; hook:order ?o1 .\n' +
      '  ?h2 a hook:Hook ; hook:order ?o1 .\n' +
      '  ?h a hook:order ?o1 .\n' +
      '  ?h2 a hook:order ?o1 .\n' +
      '  ?h log:notEqualTo ?h2 .\n' +
      '} => { false } .\n',
    expectedStatus: 'DenialViolationError',
    category: 'n3-denial',
  });

  return variants;
}

/**
 * Load a Turtle file and parse it into a base fixture.
 * @param {string} turtleText - Turtle content
 * @returns {string} Normalized Turtle
 */
export function normalizeTurtle(turtleText) {
  if (!turtleText) return '';
  return turtleText
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith('#'))
    .join('\n');
}

/**
 * Merge multiple fixture variants into a single descriptor.
 * @param {Array<FixtureVariant>} variants
 * @returns {object} Merged descriptor with categories indexed
 */
export function indexFixtures(variants) {
  const byCategory = {};
  for (const v of variants) {
    if (!byCategory[v.category]) byCategory[v.category] = [];
    byCategory[v.category].push(v);
  }
  return {
    total: variants.length,
    byCategory,
    all: variants,
  };
}
