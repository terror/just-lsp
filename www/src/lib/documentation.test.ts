import { describe, expect, it } from 'bun:test';
import { stringify } from 'yaml';

import { loadDocumentation } from './documentation';

const source = (metadata: Record<string, unknown> = {}, content = 'bar') =>
  `---\n${stringify({ title: 'foo', order: 1, ...metadata })}---\n${content}`;

describe('loadDocumentation', () => {
  it('parses frontmatter and defaults missing titles to document IDs', () => {
    for (const [newline, title, expected] of [
      ['\n', ' baz ', 'baz'],
      ['\r\n', undefined, 'foo'],
    ] as const) {
      expect(
        loadDocumentation({
          './foo.md': source({ severity: 'warning', title }, '\nbar\n').replace(
            /\n/g,
            newline
          ),
        })
      ).toEqual([
        {
          children: [],
          content: 'bar',
          id: 'foo',
          order: 1,
          severity: 'warning',
          title: expected,
        },
      ]);
    }
  });

  it('orders siblings and attaches children even when parents sort later', () => {
    const sections = loadDocumentation({
      'foo/qux.md': source({ order: 2 }),
      'bar.md': source({ order: 4 }),
      'foo/baz/quux.md': source(),
      'foo.md': source({ order: 3 }, ''),
      'foo/baz.md': source({ order: 2 }),
    });

    expect(sections.map((section) => section.id)).toEqual(['foo', 'bar']);
    expect(sections[0].content).toBe('');
    expect(sections[0].children.map((section) => section.id)).toEqual([
      'baz',
      'qux',
    ]);
    expect(sections[0].children[0].children[0].id).toBe('quux');
  });

  it('includes the source path when rejecting invalid documents', () => {
    for (const [path, content] of [
      ['Foo.md', source()],
      ['foo_bar/baz.md', source()],
      ['foo.md', 'bar'],
      ['foo.md', '---\ntitle: [\n---\nbar'],
      ['foo.md', source({ title: ' ' })],
      ['foo.md', source({ severity: 'foo' })],
      ['foo.md', source({ order: 0 })],
      ['foo.md', source({ order: 1.5 })],
    ]) {
      expect(() => loadDocumentation({ [path]: content })).toThrow(`${path}:`);
    }
  });

  it('rejects missing parents', () => {
    expect(() => loadDocumentation({ 'foo/bar.md': source() })).toThrow(
      'foo/bar.md: missing parent document foo.md'
    );
  });

  it('rejects duplicate anchors', () => {
    expect(() =>
      loadDocumentation({ 'foo.md': source(), 'foo/foo.md': source() })
    ).toThrow('Duplicate documentation ID: foo');
  });

  it('allows an empty collection', () => {
    expect(loadDocumentation({})).toEqual([]);
  });
});
