/**
 * Monaco Turtle language definition using Monarch grammar.
 *
 * Provides syntax highlighting and tokenization for RDF Turtle format:
 * - IRIs (<...>)
 * - Prefixed names (kh:hook, ex:foo)
 * - Literals ("string", "string"@en, "string"^^xsd:string)
 * - Blank nodes (_:bn)
 * - Keywords (@prefix, @base, @import, a)
 * - Comments (# comment to end-of-line)
 * - Bracket pairs for structure navigation
 *
 * This module never imports `monaco-editor` as a value — doing so statically
 * pulls the full editor bundle into the webpack/Next.js graph, which fails
 * to resolve monaco's internal AMD loader chunks (`vs/nls.messages-loader`)
 * outside a dedicated monaco webpack plugin. `@monaco-editor/react`'s own
 * loader already provides a working, already-instantiated `monaco` object
 * via its `onMount`/`beforeMount` callbacks — every function here takes
 * that instance as a parameter instead of importing the package itself.
 */
import type * as Monaco from 'monaco-editor';

function getTurtleLanguageDefinition(): Monaco.languages.IMonarchLanguage {
  return {
  defaultToken: '',

  keywords: [
    '@prefix',
    '@base',
    '@import',
    '@export',
    'a',
    'true',
    'false',
  ],

  tokenizer: {
    root: [
      // Directives: @prefix, @base, @import
      [
        /(@prefix|@base|@import|@export)\b/,
        'keyword.directive',
        '@directive',
      ],

      // Comments: # ... (to end of line)
      [/#.*$/, 'comment'],

      // IRIs: <...> (full IRI in angle brackets)
      [/<[^>]*>/, 'string.iri'],

      // Prefixed names with common namespaces
      [
        /\b(kh|hook|sh|xsd|rdf|rdfs|ex|foaf|dcat|fibo|agent|prayer|wf):[a-zA-Z_][a-zA-Z0-9_]*/,
        { token: 'variable.prefixed-name', bracket: '@close' },
      ],

      // Prefixed names (generic: prefix:localName)
      [/[a-zA-Z_][a-zA-Z0-9_-]*:[a-zA-Z_][a-zA-Z0-9_-]*/, 'variable.prefixed-name'],

      // Blank nodes: _:bn or []
      [/_:[a-zA-Z0-9_]+/, 'variable.blank-node'],
      [/\[\]/, 'variable.blank-node'],

      // Typed literals: "value"^^<type> or "value"^^prefix:type
      [
        /"(?:\\.|[^"\\])*"\^\^(?:<[^>]*>|[a-zA-Z_][a-zA-Z0-9_-]*:[a-zA-Z_][a-zA-Z0-9_-]*)/,
        'string.typed-literal',
      ],

      // Language-tagged literals: "value"@en
      [/"(?:\\.|[^"\\])*"@[a-zA-Z0-9-]+/, 'string.language-literal'],

      // String literals with common escape sequences
      [/"(?:\\.|[^"\\])*"/, 'string.literal'],
      [/'(?:\\.|[^'\\])*'/, 'string.literal'],
      [/"""(?:\\.|(?!""")[\s\S])*"""/, 'string.triple-quoted'],
      [/'''(?:\\.|(?!''')[\s\S])*'''/, 'string.triple-quoted'],

      // Numbers (integer and decimal)
      [/[-+]?[0-9]+(?:\.[0-9]+)?(?:[eE][-+]?[0-9]+)?/, 'number'],

      // Whitespace
      [/\s+/, 'white'],

      // Operators and punctuation
      [/[{}[\]();,.]/, '@brackets'],
      [/[;|,]/, 'delimiter'],
      [/[=<>!+\-*/%&^|?]/, 'operator'],

      // Keywords (like 'a' for rdf:type)
      [/\ba\b/, 'keyword'],

      // Variables: ?var or $var
      [/[\?$][a-zA-Z_][a-zA-Z0-9_]*/, 'variable'],

      // Identifiers and unknown tokens
      [/[a-zA-Z_][a-zA-Z0-9_-]*/, 'identifier'],
    ],

    directive: [
      // After @prefix/@base, consume the rest of the line
      [/[^.]*\./, 'keyword.directive', '@pop'],
      [/[^.]*$/, 'keyword.directive', '@pop'],
    ],
  },

    brackets: [
      { open: '{', close: '}', token: 'delimiter.curly' },
      { open: '[', close: ']', token: 'delimiter.square' },
      { open: '(', close: ')', token: 'delimiter.paren' },
    ],
  };
}

/**
 * Registers the Turtle language in Monaco.
 * Call this once at initialization to make Turtle available as a language option.
 */
export function registerTurtleLanguage(monaco: typeof Monaco): void {
  monaco.languages.register({ id: 'turtle' });
  monaco.languages.setMonarchTokensProvider('turtle', getTurtleLanguageDefinition());

  monaco.editor.defineTheme('turtle-light', {
    base: 'vs',
    inherit: true,
    rules: [
      { token: 'keyword.directive', foreground: '0000FF' },
      { token: 'string.iri', foreground: '008000' },
      { token: 'variable.prefixed-name', foreground: '0070C0' },
      { token: 'variable.blank-node', foreground: '70AD47' },
      { token: 'string.typed-literal', foreground: 'C00000' },
      { token: 'string.language-literal', foreground: 'C00000' },
      { token: 'string.literal', foreground: 'A31515' },
      { token: 'string.triple-quoted', foreground: 'A31515' },
      { token: 'number', foreground: '098658' },
      { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
      { token: 'keyword', foreground: '0000FF', fontStyle: 'bold' },
      { token: 'variable', foreground: '00B0E8' },
      { token: 'operator', foreground: '666666' },
      { token: 'delimiter', foreground: '666666' },
    ],
    colors: {},
  });

  monaco.editor.defineTheme('turtle-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'keyword.directive', foreground: '569CD6' },
      { token: 'string.iri', foreground: '6A9955' },
      { token: 'variable.prefixed-name', foreground: '9CDCFE' },
      { token: 'variable.blank-node', foreground: 'A8D69F' },
      { token: 'string.typed-literal', foreground: 'CE9178' },
      { token: 'string.language-literal', foreground: 'CE9178' },
      { token: 'string.literal', foreground: 'CE9178' },
      { token: 'string.triple-quoted', foreground: 'CE9178' },
      { token: 'number', foreground: 'B5CEA8' },
      { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
      { token: 'keyword', foreground: '569CD6', fontStyle: 'bold' },
      { token: 'variable', foreground: '4EC9B0' },
      { token: 'operator', foreground: 'D4D4D4' },
      { token: 'delimiter', foreground: 'D4D4D4' },
    ],
    colors: {},
  });
}

/**
 * Language configuration: bracket pairs, comments, etc.
 *
 * `indentAction` is monaco's `languages.IndentAction.Indent`, a stable
 * numeric enum member (value `1`) documented at
 * https://microsoft.github.io/monaco-editor/typedoc/enums/languages.IndentAction.html
 * -- hardcoded here (rather than threading a monaco instance through this
 * static config object) since it's a plain constant, not a call into the
 * live editor API.
 */
const TurtleLanguageConfig = {
  comments: {
    lineComment: '#',
  },
  brackets: [
    ['{', '}'],
    ['[', ']'],
    ['(', ')'],
    ['<', '>'],
  ] as [string, string][],
  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '<', close: '>' },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  surroundingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '<', close: '>' },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  indentationRules: {
    // Increase indent after { [ (
    increaseIndentPattern: /^.*[\{\[\(].*$/,
    // Decrease indent before } ] )
    decreaseIndentPattern: /^\s*[\}\]\)]/,
  },
  onEnterRules: [
    // Auto-indent continuation lines
    {
      beforeText: /.*[;,]$/,
      afterText: /^\s*/,
      action: {
        indentAction: 1, // Monaco.languages.IndentAction.Indent
      },
    },
  ],
  folding: {
    offSide: true,
    markers: {
      start: /^\s*#\s*region/,
      end: /^\s*#\s*endregion/,
    },
  },
};

/**
 * Installs language configuration for Turtle.
 */
export function configureTurtleLanguage(monaco: typeof Monaco): void {
  monaco.languages.setLanguageConfiguration(
    'turtle',
    TurtleLanguageConfig as Monaco.languages.LanguageConfiguration
  );
}
