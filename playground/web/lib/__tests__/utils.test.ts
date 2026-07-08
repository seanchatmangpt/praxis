/**
 * Unit tests for lib/utils.ts utility functions.
 * Verifies formatting and parsing functions work correctly and deterministically.
 */

import { describe, it, expect } from 'vitest';
import {
  formatJson,
  parseJson,
  formatBytes,
  formatDuration,
} from '../utils';

describe('utils.ts', () => {
  describe('formatJson', () => {
    it('test_format_json_basic_object', () => {
      const input = { name: 'test', value: 42 };
      const result = formatJson(input);

      expect(typeof result).toBe('string');
      expect(result).toContain('name');
      expect(result).toContain('test');
      expect(result).toContain('42');
    });

    it('test_format_json_deterministic', () => {
      const input = { z: 1, a: 2, m: 3 };
      const result1 = formatJson(input);
      const result2 = formatJson(input);

      expect(result1).toBe(result2);
    });

    it('test_format_json_with_custom_indent', () => {
      const input = { a: 1, b: 2 };
      const resultIndent2 = formatJson(input, 2);
      const resultIndent4 = formatJson(input, 4);

      expect(resultIndent2).not.toBe(resultIndent4);
      expect(resultIndent4.length).toBeGreaterThan(resultIndent2.length);
    });

    it('test_format_json_array', () => {
      const input = [1, 2, 3, 4];
      const result = formatJson(input);

      expect(result).toContain('1');
      expect(result).toContain('2');
      expect(result).toContain('3');
      expect(result).toContain('4');
    });

    it('test_format_json_nested_object', () => {
      const input = {
        user: { name: 'Alice', age: 30 },
        status: 'active',
      };
      const result = formatJson(input);

      expect(result).toContain('Alice');
      expect(result).toContain('30');
      expect(result).toContain('active');
    });

    it('test_format_json_null_value', () => {
      const input = { value: null };
      const result = formatJson(input);

      expect(result).toContain('null');
    });

    it('test_format_json_boolean_values', () => {
      const input = { active: true, deleted: false };
      const result = formatJson(input);

      expect(result).toContain('true');
      expect(result).toContain('false');
    });

    it('test_format_json_empty_object', () => {
      const input = {};
      const result = formatJson(input);

      expect(result).toBe('{}');
    });

    it('test_format_json_roundtrip', () => {
      const original = { a: 1, b: 'text', c: true, d: null };
      const formatted = formatJson(original);
      const parsed = parseJson(formatted);

      expect(parsed).toEqual(original);
    });
  });

  describe('parseJson', () => {
    it('test_parse_json_basic_object', () => {
      const json = '{"name":"test","value":42}';
      const result = parseJson(json);

      expect(result.name).toBe('test');
      expect(result.value).toBe(42);
    });

    it('test_parse_json_array', () => {
      const json = '[1,2,3,4,5]';
      const result = parseJson(json);

      expect(Array.isArray(result)).toBe(true);
      expect(result).toHaveLength(5);
      expect(result[0]).toBe(1);
      expect(result[4]).toBe(5);
    });

    it('test_parse_json_typed_result', () => {
      interface User {
        name: string;
        age: number;
      }
      const json = '{"name":"Bob","age":25}';
      const result = parseJson<User>(json);

      expect(result.name).toBe('Bob');
      expect(result.age).toBe(25);
    });

    it('test_parse_json_nested_structure', () => {
      const json = '{"user":{"id":1,"name":"Charlie"},"active":true}';
      const result = parseJson(json);

      expect(result.user.id).toBe(1);
      expect(result.user.name).toBe('Charlie');
      expect(result.active).toBe(true);
    });

    it('test_parse_json_with_whitespace', () => {
      const json = `{
        "name": "test",
        "value": 42
      }`;
      const result = parseJson(json);

      expect(result.name).toBe('test');
      expect(result.value).toBe(42);
    });
  });

  describe('formatBytes', () => {
    it('test_format_bytes_zero', () => {
      const result = formatBytes(0);
      expect(result).toBe('0 Bytes');
    });

    it('test_format_bytes_single_byte', () => {
      const result = formatBytes(1);
      expect(result).toContain('Bytes');
      expect(result).not.toContain('0 Bytes');
    });

    it('test_format_bytes_kilobytes', () => {
      const result = formatBytes(1024);
      expect(result).toContain('KB');
    });

    it('test_format_bytes_megabytes', () => {
      const result = formatBytes(1024 * 1024);
      expect(result).toContain('MB');
    });

    it('test_format_bytes_gigabytes', () => {
      const result = formatBytes(1024 * 1024 * 1024);
      expect(result).toContain('GB');
    });

    it('test_format_bytes_fractional_units', () => {
      const result = formatBytes(1536); // 1.5 KB
      expect(result).toContain('KB');
      expect(result).toMatch(/1\.\d+/);
    });

    it('test_format_bytes_large_number', () => {
      const result = formatBytes(5242880); // 5 MB
      expect(result).toContain('MB');
      expect(result).toMatch(/5\.*/);
    });

    it('test_format_bytes_returns_string', () => {
      const result = formatBytes(512);
      expect(typeof result).toBe('string');
      expect(result.length).toBeGreaterThan(0);
    });

    it('test_format_bytes_contains_unit', () => {
      const result = formatBytes(1024);
      const validUnits = ['Bytes', 'KB', 'MB', 'GB'];
      expect(validUnits.some((unit) => result.includes(unit))).toBe(true);
    });
  });

  describe('formatDuration', () => {
    it('test_format_duration_milliseconds', () => {
      const result = formatDuration(500);
      expect(result).toContain('ms');
      expect(result).toMatch(/500ms/);
    });

    it('test_format_duration_less_than_second', () => {
      const result = formatDuration(999);
      expect(result).toContain('ms');
    });

    it('test_format_duration_exactly_one_second', () => {
      const result = formatDuration(1000);
      expect(result).toContain('s');
      expect(result).toMatch(/1\.0*s/);
    });

    it('test_format_duration_seconds', () => {
      const result = formatDuration(5000);
      expect(result).toContain('s');
      expect(result).toMatch(/5\.0*s/);
    });

    it('test_format_duration_fractional_seconds', () => {
      const result = formatDuration(1500);
      expect(result).toContain('s');
      expect(result).toMatch(/1\.5s/);
    });

    it('test_format_duration_59_seconds', () => {
      const result = formatDuration(59000);
      expect(result).toContain('s');
      expect(result).not.toContain('m');
    });

    it('test_format_duration_one_minute', () => {
      const result = formatDuration(60000);
      expect(result).toContain('m');
      expect(result).toMatch(/1\.0*m/);
    });

    it('test_format_duration_minutes', () => {
      const result = formatDuration(300000); // 5 minutes
      expect(result).toContain('m');
      expect(result).toMatch(/5\.0*m/);
    });

    it('test_format_duration_returns_non_empty_string', () => {
      const result = formatDuration(2500);
      expect(typeof result).toBe('string');
      expect(result.length).toBeGreaterThan(0);
    });

    it('test_format_duration_boundary_cases', () => {
      expect(formatDuration(0)).toContain('0ms');
      expect(formatDuration(1)).toContain('1ms');
      expect(formatDuration(999)).toContain('ms');
      expect(formatDuration(1000)).toContain('s');
      expect(formatDuration(60000)).toContain('m');
    });
  });

  describe('Integration tests', () => {
    it('test_format_and_parse_json_roundtrip_object', () => {
      const original = {
        user: 'Alice',
        permissions: ['read', 'write'],
        active: true,
        timestamp: 1625097600,
      };

      const formatted = formatJson(original);
      const parsed = parseJson(formatted);

      expect(parsed).toEqual(original);
    });

    it('test_format_and_parse_json_roundtrip_array', () => {
      const original = [
        { id: 1, name: 'Item 1' },
        { id: 2, name: 'Item 2' },
        { id: 3, name: 'Item 3' },
      ];

      const formatted = formatJson(original);
      const parsed = parseJson(formatted);

      expect(parsed).toEqual(original);
    });

    it('test_multiple_utility_functions_work_together', () => {
      // Create a result object using multiple utilities
      const sizeInBytes = 2048576;
      const durationMs = 1250;

      const bytesFormatted = formatBytes(sizeInBytes);
      const durationFormatted = formatDuration(durationMs);

      const result = formatJson({
        bytes: bytesFormatted,
        duration: durationFormatted,
      });

      expect(result).toContain('MB');
      expect(result).toContain('1.25s');
    });
  });

  describe('Edge cases and error handling', () => {
    it('test_format_json_with_undefined_omits_value', () => {
      const input = { a: 1, b: undefined, c: 3 };
      const result = formatJson(input);

      // JSON.stringify omits undefined values
      expect(result).toContain('"a":1');
      expect(result).toContain('"c":3');
      expect(result).not.toContain('undefined');
    });

    it('test_format_bytes_with_very_small_number', () => {
      const result = formatBytes(100);
      expect(result).toContain('Bytes');
      expect(result).toMatch(/100 Bytes/);
    });

    it('test_format_duration_with_zero', () => {
      const result = formatDuration(0);
      expect(result).toBe('0ms');
    });

    it('test_parse_json_with_string_value', () => {
      const json = '"hello world"';
      const result = parseJson(json);
      expect(result).toBe('hello world');
    });

    it('test_parse_json_with_number_value', () => {
      const json = '42';
      const result = parseJson(json);
      expect(result).toBe(42);
    });

    it('test_parse_json_with_boolean_value', () => {
      const jsonTrue = 'true';
      const jsonFalse = 'false';
      expect(parseJson(jsonTrue)).toBe(true);
      expect(parseJson(jsonFalse)).toBe(false);
    });
  });
});
