import { describe, expect, it } from 'bun:test';

import { cn } from './utils';

describe('cn utility', () => {
  it('merges class names correctly', () => {
    expect(cn('foo', 'bar')).toBe('foo bar');
    expect(cn('foo', { bar: true })).toBe('foo bar');
    expect(cn('foo', { bar: false })).toBe('foo');
    expect(cn('foo', ['bar', 'baz'])).toBe('foo bar baz');
  });
});
