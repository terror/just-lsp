import { describe, expect, it, mock } from 'bun:test';
import { type Language, type Parser, type Tree } from 'web-tree-sitter';

import { cn, parse } from './utils';

describe('cn utility', () => {
  it('merges class names correctly', () => {
    expect(cn('foo', 'bar')).toBe('foo bar');
    expect(cn('foo', { bar: true })).toBe('foo bar');
    expect(cn('foo', { bar: false })).toBe('foo');
    expect(cn('foo', ['bar', 'baz'])).toBe('foo bar baz');
  });
});

describe('parse', () => {
  it('sets language and calls parse', () => {
    const mockParser = {
      setLanguage: mock(() => undefined),
      parse: mock(() => ({ rootNode: {} }) as unknown as Tree),
    };

    const mockLanguage = { name: 'javascript' };
    const code = 'const x = 1;';

    const result = parse({
      parser: mockParser as unknown as Parser,
      language: mockLanguage as unknown as Language,
      code,
    });

    expect(mockParser.setLanguage).toHaveBeenCalledWith(mockLanguage);
    expect(mockParser.parse).toHaveBeenCalledWith(code);

    expect(result).toBeDefined();
  });
});
