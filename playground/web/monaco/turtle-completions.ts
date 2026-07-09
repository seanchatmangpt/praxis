/**
 * Monaco completions provider for Turtle language.
 *
 * Provides intelligent code completions for:
 * - Prefix declarations (kh:, hook:, sh:, xsd:, rdf:, fibo:, etc.)
 * - Common predicates for each namespace
 * - Keywords (@prefix, @base, etc.)
 * - Blank nodes and variables
 * - Literals with type hints
 */

// Type-only: never import monaco-editor as a value here. Statically
// bundling it breaks Next.js/webpack (unresolvable AMD loader chunk,
// `vs/nls.messages-loader`). `@monaco-editor/react`'s own loader supplies a
// working, already-instantiated monaco object via `onMount`/`beforeMount`;
// `registerTurtleCompletions` below takes that instance as a parameter.
import type * as Monaco from 'monaco-editor';

/**
 * Helper to create completion items with proper range.
 * Range covers the word being completed or symbol being inserted.
 */
function createCompletionItem(
  label: string,
  kind: Monaco.languages.CompletionItemKind,
  insertText: string,
  documentation?: string,
  filterText?: string,
  sortText?: string,
  range?: Monaco.IRange
): Monaco.languages.CompletionItem {
  // Default range: single character at cursor (will replace existing if selected)
  const defaultRange: Monaco.IRange = range || {
    startLineNumber: 1,
    startColumn: 1,
    endLineNumber: 1,
    endColumn: 1,
  };

  return {
    label,
    kind,
    insertText,
    documentation,
    filterText,
    sortText,
    range: {
      insert: defaultRange,
      replace: defaultRange,
    },
  };
}

/**
 * Well-known Turtle vocabulary predicates by namespace.
 * Used to provide context-aware completions.
 */
export const TurtleVocabulary: Record<string, string[]> = {
  kh: [
    'hook',
    'rule',
    'fact',
    'premise',
    'consequence',
    'validation',
    'schedule',
    'priority',
  ],
  hook: [
    'triggeredWhen',
    'derivesFact',
    'executeAction',
    'onDenialOf',
    'withPriority',
    'afterHook',
    'beforeHook',
  ],
  sh: [
    'NodeShape',
    'PropertyShape',
    'targetNode',
    'targetClass',
    'targetObjectsOf',
    'targetSubjectsOf',
    'path',
    'maxCount',
    'minCount',
    'datatype',
    'hasValue',
    'in',
    'pattern',
    'severity',
  ],
  xsd: [
    'string',
    'integer',
    'decimal',
    'boolean',
    'date',
    'dateTime',
    'time',
    'duration',
    'anyURI',
    'QName',
    'normalizedString',
  ],
  rdf: [
    'type',
    'subject',
    'predicate',
    'object',
    'Property',
    'XMLLiteral',
    'HTML',
    'langString',
  ],
  rdfs: [
    'Resource',
    'Class',
    'Literal',
    'comment',
    'label',
    'seeAlso',
    'isDefinedBy',
    'subClassOf',
    'subPropertyOf',
    'domain',
    'range',
  ],
  fibo: [
    'Identifier',
    'Party',
    'Organisation',
    'Person',
    'PhysicalAddress',
    'FinancialInstrument',
  ],
  agent: [
    'Agent',
    'Role',
    'Capability',
    'Action',
    'Goal',
    'Constraint',
  ],
  prayer: [
    'Kernel',
    'Interface',
    'Implementation',
    'Contract',
    'Effect',
    'Precondition',
  ],
  wf: [
    'Workflow',
    'Step',
    'Condition',
    'Transition',
    'State',
    'Event',
  ],

  // RDF/XML namespace
  rdfxml: [
    'RDF',
    'Description',
    'Property',
  ],

  // OWL 2 vocabulary
  owl: [
    'Ontology',
    'Class',
    'ObjectProperty',
    'DatatypeProperty',
    'Thing',
    'Nothing',
    'equivalentClass',
    'equivalentProperty',
    'disjointWith',
    'inverseOf',
    'transitiveProperty',
    'functionalProperty',
    'inverseFunctionalProperty',
    'symmetricProperty',
  ],

  // SKOS vocabulary (common in semantic systems)
  skos: [
    'Concept',
    'ConceptScheme',
    'prefLabel',
    'altLabel',
    'hiddenLabel',
    'definition',
    'broaderTransitive',
    'narrowerTransitive',
    'related',
  ],
};

/**
 * Builds the completion provider for Turtle language, closed over a live
 * monaco instance (needed for the `CompletionItemKind` enum's runtime
 * values). Triggered when user types after prefixes or at certain
 * positions.
 */
function createTurtleCompletionProvider(
  monaco: typeof Monaco
): Monaco.languages.CompletionItemProvider {
  return {
    provideCompletionItems(model, position) {
      const suggestions: Monaco.languages.CompletionItem[] = [];

      // Get text before cursor
      const lineContent = model.getLineContent(position.lineNumber);
      const textBeforeCursor = lineContent.substring(0, position.column - 1);
      const word = model.getWordUntilPosition(position);

      // Calculate range for word replacement (from start of word to cursor)
      const wordRange: Monaco.IRange = {
        startLineNumber: position.lineNumber,
        startColumn: position.column - word.word.length,
        endLineNumber: position.lineNumber,
        endColumn: position.column,
      };

      // Trigger 1: After @ (for directives)
      if (textBeforeCursor.endsWith('@')) {
        const atRange: Monaco.IRange = {
          startLineNumber: position.lineNumber,
          startColumn: position.column,
          endLineNumber: position.lineNumber,
          endColumn: position.column,
        };
        suggestions.push(
          createCompletionItem(
            '@prefix',
            monaco.languages.CompletionItemKind.Keyword,
            '@prefix ',
            'Define a namespace prefix',
            undefined,
            undefined,
            atRange
          ),
          createCompletionItem(
            '@base',
            monaco.languages.CompletionItemKind.Keyword,
            '@base ',
            'Define base IRI',
            undefined,
            undefined,
            atRange
          )
        );
        return { suggestions };
      }

      // Trigger 2: After ':' (for namespace completions)
      const prefixMatch = textBeforeCursor.match(/(\w+):$/);
      if (prefixMatch) {
        const prefix = prefixMatch[1];
        const predicates = TurtleVocabulary[prefix] || [];

        // Range from end of prefix to cursor
        const colonIndex = textBeforeCursor.lastIndexOf(':');
        const predicateRange: Monaco.IRange = {
          startLineNumber: position.lineNumber,
          startColumn: colonIndex + 2, // After the ':'
          endLineNumber: position.lineNumber,
          endColumn: position.column,
        };

        suggestions.push(
          ...predicates.map((predicate) =>
            createCompletionItem(
              predicate,
              monaco.languages.CompletionItemKind.Property,
              predicate,
              `Property in ${prefix}: namespace`,
              `${prefix}:${predicate}`,
              predicate,
              predicateRange
            )
          )
        );

        // If no specific vocab, still offer generic completions
        if (predicates.length === 0) {
          suggestions.push(
            createCompletionItem(
              'type',
              monaco.languages.CompletionItemKind.Property,
              'type',
              'Generic property',
              undefined,
              undefined,
              predicateRange
            )
          );
        }

        return { suggestions };
      }

      // Trigger 3: At start of line or after whitespace (for prefixes)
      if (
        textBeforeCursor.trim().length === 0 ||
        textBeforeCursor.trim() === '@prefix'
      ) {
        Object.keys(TurtleVocabulary).forEach((prefix) => {
          suggestions.push(
            createCompletionItem(
              `${prefix}:`,
              monaco.languages.CompletionItemKind.Variable,
              `${prefix}:`,
              `Namespace prefix for ${prefix}`,
              undefined,
              `a_${prefix}`, // Sort common prefixes first
              wordRange
            )
          );
        });

        return { suggestions };
      }

      // Trigger 4: After opening angle bracket < (for IRI suggestions)
      if (textBeforeCursor.endsWith('<')) {
        const commonIris = [
          'http://www.w3.org/1999/02/22-rdf-syntax-ns#type',
          'http://www.w3.org/2000/01/rdf-schema#comment',
          'http://www.w3.org/2000/01/rdf-schema#label',
          'http://www.w3.org/1999/02/22-rdf-syntax-ns#Property',
          'http://www.w3.org/2000/01/rdf-schema#Class',
        ];

        const iriRange: Monaco.IRange = {
          startLineNumber: position.lineNumber,
          startColumn: position.column,
          endLineNumber: position.lineNumber,
          endColumn: position.column,
        };

        suggestions.push(
          ...commonIris.map((iri) =>
            createCompletionItem(
              iri,
              monaco.languages.CompletionItemKind.Reference,
              `${iri}>`,
              'Common RDF/RDFS vocabulary IRI',
              undefined,
              undefined,
              iriRange
            )
          )
        );

        return { suggestions };
      }

      // Trigger 5: Keywords and boolean literals
      const keywords = [
        'a',
        'true',
        'false',
      ];
      suggestions.push(
        ...keywords.map((kw) =>
          createCompletionItem(
            kw,
            monaco.languages.CompletionItemKind.Keyword,
            kw,
            undefined,
            undefined,
            undefined,
            wordRange
          )
        )
      );

      return { suggestions };
    },

    /**
     * Resolve a completion item to add additional documentation or details.
     */
    resolveCompletionItem(item) {
      // You can fetch detailed documentation for predicates here if needed
      return item;
    },
  };
}

/**
 * Registers the completion provider with Monaco.
 */
export function registerTurtleCompletions(monaco: typeof Monaco): void {
  monaco.languages.registerCompletionItemProvider(
    'turtle',
    createTurtleCompletionProvider(monaco)
  );
}
