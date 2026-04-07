import { EditorState } from '@codemirror/state';
import { describe, expect, it, mock } from 'bun:test';
import { type Tree } from 'web-tree-sitter';

import { cn, parse, positionToOffset } from './utils';

describe('cn utility', () => {
  it('merges class names correctly', () => {
    expect(cn('foo', 'bar')).toBe('foo bar');
    expect(cn('foo', { bar: true })).toBe('foo bar');
    expect(cn('foo', { bar: false })).toBe('foo');
    expect(cn('foo', ['bar', 'baz'])).toBe('foo bar baz');
  });
});

describe('positionToOffset', () => {
  it('returns null when line number exceeds document lines', () => {
    const doc = EditorState.create({ doc: 'line1\nline2' }).doc;
    expect(positionToOffset({ row: 2, column: 0 }, doc)).toBeNull();
  });

  it('converts position to offset correctly', () => {
    const doc = EditorState.create({ doc: 'line1\nline2\nline3' }).doc;

    expect(positionToOffset({ row: 0, column: 0 }, doc)).toBe(0);
    expect(positionToOffset({ row: 0, column: 3 }, doc)).toBe(3);
    expect(positionToOffset({ row: 1, column: 0 }, doc)).toBe(6);
    expect(positionToOffset({ row: 1, column: 2 }, doc)).toBe(8);
    expect(positionToOffset({ row: 0, column: 100 }, doc)).toBe(5);
  });
});

describe('parse', () => {
  it('sets language and calls parse', () => {
    const mockParser = {
      setLanguage: mock(() => {}),
      parse: mock(() => ({ rootNode: {} }) as unknown as Tree),
    };

    const mockLanguage = { name: 'javascript' };
    const code = 'const x = 1;';

    const result = parse({
      parser: mockParser as any,
      language: mockLanguage as any,
      code,
    });

    expect(mockParser.setLanguage).toHaveBeenCalledWith(mockLanguage);
    expect(mockParser.parse).toHaveBeenCalledWith(code);

    expect(result).toBeDefined();
  });
});
