import { expect, it } from 'bun:test';
import { renderToStaticMarkup } from 'react-dom/server';
import { MemoryRouter } from 'react-router';
import { resolveConfig } from 'vite';

import { Header } from '../components/header';

it('serves parser assets from the site root', async () => {
  async function check(command: 'serve' | 'build') {
    const config = await resolveConfig({ logLevel: 'silent' }, command);

    expect(`${config.env.BASE_URL}tree-sitter-just.wasm`).toBe(
      '/tree-sitter-just.wasm'
    );
  }

  await check('serve');
  await check('build');
});

it('resolves navigation links and the active page', () => {
  function check(path: string, activeHref: string) {
    const html = renderToStaticMarkup(
      <MemoryRouter initialEntries={[path]}>
        <Header theme={{ darkMode: false, toggleTheme: () => undefined }} />
      </MemoryRouter>
    );
    const links = [...html.matchAll(/<a\b[^>]*>/g)].map(([tag]) => tag);
    const activeLinks = links.filter((tag) =>
      tag.includes('aria-current="page"')
    );

    expect(html).toContain('href="/"');
    expect(html).toContain('href="/playground"');
    expect(activeLinks).toHaveLength(1);
    expect(activeLinks[0]).toContain(`href="${activeHref}"`);
  }

  check('/', '/');
  check('/playground', '/playground');
  check('/playground/', '/playground');
});
