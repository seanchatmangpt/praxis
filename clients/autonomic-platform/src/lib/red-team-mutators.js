/**
 * red-team-mutators.js
 * --------------------
 * Eight adversarial mutators that deliberately break Turtle/hooks/shapes inputs.
 * Each mutator:
 *   1. Applies a specific transformation
 *   2. Executes the validation
 *   3. Asserts expected Status vs actual Status
 *
 * Status enum: Success | ValidationFailed | TypeError | SyntaxError | UnknownPredicateError
 *             | OverflowError | UnsupportedFeatureError | HashMismatchError | DenialViolationError
 */

/**
 * Mutator result shape:
 *   { id, name, mutation, actualStatus, expectedStatus, passed, errorMessage }
 */

/**
 * Mutator 1: Break Turtle syntax
 * Removes or corrupts critical syntax (periods, brackets, etc.)
 */
export function mutator_BrokenSyntax(turtle) {
  const mutated = turtle
    .split('\n')
    .map((line, i) => {
      // On every other line with a period, remove it
      if (i % 2 === 0 && line.includes('.')) {
        return line.replace(/\.$/, '');
      }
      return line;
    })
    .join('\n');

  return {
    id: 'mutator-syntax-break',
    name: 'Break Turtle Syntax',
    mutation: 'Remove trailing periods from half the triples',
    mutated,
    expectedStatus: 'SyntaxError',
  };
}

/**
 * Mutator 2: Inject OWL RL feature the profile refuses
 * Adds unsupported OWL constructs
 */
export function mutator_UnsupportedOWLRL(turtle) {
  const mutated = turtle + '\n' +
    '# Unsupported OWL RL features:\n' +
    'hook:restricted a owl:Class ;\n' +
    '  rdfs:subClassOf [ owl:intersectionOf (hook:Type1 hook:Type2) ] .\n' +
    'hook:disjoint owl:disjointWith hook:other .\n';

  return {
    id: 'mutator-unsupported-owl-rl',
    name: 'Inject Unsupported OWL RL',
    mutation: 'Add owl:intersectionOf and owl:disjointWith (not in profile)',
    mutated,
    expectedStatus: 'UnsupportedFeatureError',
  };
}

/**
 * Mutator 3: Remove required SHACL property
 * Deletes sh:minCount, sh:datatype, or sh:path
 */
export function mutator_RemoveSHACLProperty(turtle) {
  const mutated = turtle
    .replace(/sh:minCount\s+\d+\s*;?\n?/g, '')
    .replace(/sh:datatype\s+xsd:\w+\s*;?\n?/g, '')
    .replace(/sh:path\s+[\w:]+\s*;?\n?/g, '');

  return {
    id: 'mutator-missing-shacl-property',
    name: 'Remove Required SHACL Property',
    mutation: 'Delete sh:minCount, sh:datatype, sh:path',
    mutated,
    expectedStatus: 'ValidationFailed',
  };
}

/**
 * Mutator 4: Change ShEx datatype incompatibly
 * Alters xsd:integer to xsd:string, xsd:boolean, etc.
 */
export function mutator_WrongShExDatatype(turtle) {
  const datatypes = ['xsd:string', 'xsd:boolean', 'xsd:float'];
  let idx = 0;

  const mutated = turtle.replace(/xsd:integer|xsd:string|xsd:boolean/g, () => {
    const replacement = datatypes[idx % datatypes.length];
    idx++;
    return replacement;
  });

  return {
    id: 'mutator-wrong-shex-datatype',
    name: 'Wrong ShEx Datatype',
    mutation: 'Rotate xsd:integer → xsd:string → xsd:boolean → xsd:float',
    mutated,
    expectedStatus: 'TypeError',
  };
}

/**
 * Mutator 5: Overflow to 13+ hooks (max is 12)
 * Adds extra hook definitions
 */
export function mutator_OverflowHooks(turtle) {
  const extraHooks = Array.from({ length: 13 }, (_, i) =>
    `hook:hook_overflow_${i} a hook:Hook ; hook:name "Overflow ${i}" ; hook:order ${i + 100} .`
  ).join('\n');

  const mutated = turtle + '\n# Overflow hooks:\n' + extraHooks;

  return {
    id: 'mutator-overflow-hooks',
    name: 'Overflow Hook Count',
    mutation: 'Add 13 hooks when maximum is 12',
    mutated,
    expectedStatus: 'OverflowError',
  };
}

/**
 * Mutator 6: Use unknown hook: or kh: predicate
 * Injects undefined predicates
 */
export function mutator_UnknownPredicate(turtle) {
  const mutated = turtle + '\n' +
    'hook:test a hook:Hook ;\n' +
    '  hook:unknownPredicate "value" ;\n' +
    '  kh:unknownKHPredicate "another" ;\n' +
    '  hook:name "Test" .\n';

  return {
    id: 'mutator-unknown-predicate',
    name: 'Unknown hook:/kh: Predicate',
    mutation: 'Add hook:unknownPredicate and kh:unknownKHPredicate',
    mutated,
    expectedStatus: 'UnknownPredicateError',
  };
}

/**
 * Mutator 7: Tamper graph after hashing (HASH_MISMATCH)
 * Modifies a triple that invalidates any hash/receipt
 */
export function mutator_TamperAfterHash(turtle) {
  const mutated = turtle + '\n' +
    '# Tampered triple added after hash computation:\n' +
    'hook:receipt_invalid hook:marker "true" .\n' +
    'hook:chain_head hook:invalidated "yes" .\n';

  return {
    id: 'mutator-hash-mismatch',
    name: 'Tamper After Hash',
    mutation: 'Add triples after hash/receipt was computed',
    mutated,
    expectedStatus: 'HashMismatchError',
  };
}

/**
 * Mutator 8: Add triple that trips N3 denial rule
 * Violates a denial rule constraint
 */
export function mutator_N3Denial(turtle) {
  const mutated = turtle + '\n' +
    '# N3 denial violation:\n' +
    '{ ?h a hook:Hook ; hook:order ?o . ?h2 a hook:Hook ; hook:order ?o . ?h log:notEqualTo ?h2 . } => { false } .\n' +
    'hook:h1 a hook:Hook ; hook:order 1 .\n' +
    'hook:h2 a hook:Hook ; hook:order 1 .\n' +
    # Same order on two different hooks triggers the denial
  '';

  return {
    id: 'mutator-n3-denial',
    name: 'N3 Denial Rule Violation',
    mutation: 'Violate a denial rule with duplicate hook:order',
    mutated,
    expectedStatus: 'DenialViolationError',
  };
}

/**
 * Execute all 8 mutators on a given Turtle input and a validation function.
 * @param {string} baseTurtle - Valid Turtle content
 * @param {function} validateFn - async (turtle) => Promise<Status>
 * @returns {Promise<Array<MutatorResult>>}
 */
export async function executeAllMutators(baseTurtle, validateFn) {
  const mutators = [
    mutator_BrokenSyntax,
    mutator_UnsupportedOWLRL,
    mutator_RemoveSHACLProperty,
    mutator_WrongShExDatatype,
    mutator_OverflowHooks,
    mutator_UnknownPredicate,
    mutator_TamperAfterHash,
    mutator_N3Denial,
  ];

  const results = [];

  for (const mutator of mutators) {
    const mutatorDef = mutator(baseTurtle);
    const actualStatus = await validateFn(mutatorDef.mutated).catch((err) => ({
      error: err.message,
      status: 'ExecutionError',
    }));

    const status = actualStatus.status || actualStatus;
    const passed = status === mutatorDef.expectedStatus;

    results.push({
      id: mutatorDef.id,
      name: mutatorDef.name,
      mutation: mutatorDef.mutation,
      actualStatus: status,
      expectedStatus: mutatorDef.expectedStatus,
      passed,
      errorMessage: actualStatus.error || null,
    });
  }

  return results;
}

/**
 * Summary of red-team execution.
 * @param {Array<MutatorResult>} results
 * @returns {object}
 */
export function summarizeMutatorResults(results) {
  const passed = results.filter((r) => r.passed).length;
  const failed = results.filter((r) => !r.passed).length;

  return {
    totalMutators: results.length,
    passed,
    failed,
    passRate: ((passed / results.length) * 100).toFixed(1) + '%',
    failedMutators: results.filter((r) => !r.passed),
    results,
  };
}
