import { describe, expect, it } from 'bun:test';
import { renderToStaticMarkup } from 'react-dom/server';

import { DocumentationMarkdown } from './documentation-markdown';

describe('DocumentationMarkdown', () => {
  it('renders tables with links and commands', () => {
    const html = renderToStaticMarkup(
      <DocumentationMarkdown>
        {'| foo | bar |\n| --- | --- |\n| [foo](https://foo.example) | `bar` |'}
      </DocumentationMarkdown>
    );

    expect(html).toContain('overflow-x-auto');
    expect(html).toContain('<thead><tr><th>foo</th><th>bar</th></tr></thead>');
    expect(html).toContain('<a href="https://foo.example">foo</a>');
    expect(html).toContain('<td><code>bar</code></td>');
  });

  it('preserves code block labels', () => {
    for (const [label, caption] of [
      ['reported', 'Reported'],
      ['corrected', 'Corrected'],
    ]) {
      const html = renderToStaticMarkup(
        <DocumentationMarkdown>
          {`\`\`\`just ${label}\nfoo\n\`\`\``}
        </DocumentationMarkdown>
      );

      expect(html).toContain(`${caption}</figcaption>`);
      expect(html).toContain('<code class="language-just">foo\n</code>');
    }
  });
});
