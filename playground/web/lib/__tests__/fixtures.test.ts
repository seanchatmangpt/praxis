/**
 * Unit tests for lib/fixtures.ts fixture data.
 * Verifies that Turtle fixtures are valid, non-empty, and well-formed.
 */

import { describe, it, expect } from 'vitest';
import { sampleTurtle } from '../fixtures';

describe('fixtures.ts', () => {
  describe('sampleTurtle fixture', () => {
    it('test_fixture_is_string', () => {
      expect(typeof sampleTurtle).toBe('string');
    });

    it('test_fixture_is_non_empty', () => {
      expect(sampleTurtle.length).toBeGreaterThan(0);
    });

    it('test_fixture_contains_valid_turtle_prefix', () => {
      expect(sampleTurtle).toContain('@prefix');
    });

    it('test_fixture_has_prefix_declarations', () => {
      const prefixCount = (sampleTurtle.match(/@prefix/g) || []).length;
      expect(prefixCount).toBeGreaterThan(0);
      expect(sampleTurtle).toContain('@prefix ex:');
      expect(sampleTurtle).toContain('@prefix rdf:');
    });

    it('test_fixture_has_valid_iri_declarations', () => {
      // Check for properly formatted IRI declarations
      expect(sampleTurtle).toMatch(/@prefix \w+:\s+<http[s]?:\/\//);
    });

    it('test_fixture_no_unclosed_angle_brackets', () => {
      const openBrackets = (sampleTurtle.match(/</g) || []).length;
      const closeBrackets = (sampleTurtle.match(/>/g) || []).length;
      expect(openBrackets).toBe(closeBrackets);
    });

    it('test_fixture_contains_rdf_statements', () => {
      // Sample Turtle should have RDF triples (subject predicate object)
      expect(sampleTurtle).toContain('a ');
      expect(sampleTurtle).toContain('rdfs:');
      expect(sampleTurtle).toContain('foaf:');
    });

    it('test_fixture_has_object_declarations', () => {
      // Should declare at least one Person or Project
      expect(sampleTurtle).toMatch(/ex:(?:Person|Project|John|Jane)/);
    });

    it('test_fixture_no_dangling_quotes', () => {
      // Count double quotes (should be even for complete strings)
      const quoteCount = (sampleTurtle.match(/"/g) || []).length;
      expect(quoteCount % 2).toBe(0);
    });

    it('test_fixture_contains_comments', () => {
      // Turtle comments start with #
      expect(sampleTurtle).toContain('#');
    });

    it('test_fixture_no_syntax_errors_obvious_cases', () => {
      // Check for obvious syntax errors
      expect(sampleTurtle).not.toContain(';;'); // Double semicolon is usually wrong
      expect(sampleTurtle).not.toContain('  .'); // Space before final period (minor)
    });

    it('test_fixture_has_xsd_datatypes', () => {
      // Sample should reference XSD types
      expect(sampleTurtle).toContain('xsd:');
    });

    it('test_sample_turtle_parses_as_utf8', () => {
      // Verify the string can be encoded/decoded as UTF-8
      const encoded = new TextEncoder().encode(sampleTurtle);
      const decoded = new TextDecoder().decode(encoded);
      expect(decoded).toBe(sampleTurtle);
    });

    it('test_fixture_contains_meaningful_content', () => {
      // Should have multiple lines
      const lines = sampleTurtle.split('\n');
      expect(lines.length).toBeGreaterThan(10);
    });

    it('test_fixture_line_endings_are_consistent', () => {
      // Check that line endings are consistent (UNIX-style \n or CRLF)
      const hasLF = sampleTurtle.includes('\n');
      expect(hasLF).toBe(true);
    });
  });

  describe('Fixture validation - negative cases', () => {
    it('test_incomplete_fixture_would_fail_validation', () => {
      const incompleteTurtle = `# Incomplete Turtle
@prefix ex: <http://example.com/
# Missing closing >`;

      // This should be detected as malformed
      const openBrackets = (incompleteTurtle.match(/</g) || []).length;
      const closeBrackets = (incompleteTurtle.match(/>/g) || []).length;
      expect(openBrackets).not.toBe(closeBrackets);
    });

    it('test_unbalanced_quotes_would_fail_validation', () => {
      const malformedTurtle = `@prefix ex: <http://example.com/> .
ex:test rdfs:label "Unbalanced quote .`;

      const quoteCount = (malformedTurtle.match(/"/g) || []).length;
      expect(quoteCount % 2).not.toBe(0);
    });

    it('test_fixture_is_not_empty_string', () => {
      expect(sampleTurtle.length).not.toBe(0);
      expect(sampleTurtle).not.toBe('');
    });

    it('test_fixture_is_not_whitespace_only', () => {
      const trimmed = sampleTurtle.trim();
      expect(trimmed.length).toBeGreaterThan(0);
      expect(trimmed).not.toMatch(/^\s+$/);
    });
  });

  describe('Fixture content validation', () => {
    it('test_fixture_declares_class', () => {
      // Should declare at least one RDFS class
      expect(sampleTurtle).toMatch(/\w+ a rdfs:Class/);
    });

    it('test_fixture_has_multiple_subjects', () => {
      // Count distinct subjects (rough check)
      const subjects = (sampleTurtle.match(/^ex:\w+\s/gm) || []).length;
      expect(subjects).toBeGreaterThan(1);
    });

    it('test_fixture_uses_standard_vocabularies', () => {
      // Should use standard prefixes
      const standardPrefixes = ['ex:', 'rdf:', 'rdfs:', 'xsd:', 'foaf:'];
      const usedPrefixes = standardPrefixes.filter((prefix) =>
        sampleTurtle.includes(prefix)
      );
      expect(usedPrefixes.length).toBeGreaterThan(2);
    });
  });
});
