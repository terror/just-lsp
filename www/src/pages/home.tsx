import { useEffect } from 'react';
import { Link } from 'react-router';

import screenshot from '../../../screenshot.png';
import { Header } from '../components/header';
import { useTheme } from '../hooks/use-theme';

const Home = () => {
  const theme = useTheme();

  useEffect(() => {
    document.title = 'just-lsp';
  }, []);

  return (
    <div className='min-h-screen'>
      <Header theme={theme} />

      <main className='mx-auto max-w-3xl px-6 py-12 leading-[1.75] sm:py-16'>
        <h1 className='font-mono text-3xl font-semibold tracking-tight'>
          just-lsp
        </h1>
        <p className='text-muted-foreground mt-4 text-lg'>
          A language server for{' '}
          <a
            href='https://github.com/casey/just'
            className='text-foreground underline decoration-current/30 underline-offset-4 hover:decoration-current'
          >
            just
          </a>
          , the command runner.
        </p>
        <img
          src={screenshot}
          alt='just-lsp in Neovim, showing hover documentation and inline diagnostics in a justfile'
          width={3334}
          height={2166}
          className='mt-6 h-auto w-full rounded-md'
        />
        <p className='text-muted-foreground mt-4'>
          Get completions, hover documentation, diagnostics, and navigation in
          your justfiles. Rename symbols, find references, format files, and run
          recipes directly from your editor.
        </p>

        <section aria-labelledby='installation' className='mt-12'>
          <h2
            id='installation'
            className='mb-4 text-xl font-semibold tracking-tight'
          >
            Installation
          </h2>
          <p className='text-muted-foreground'>Install with Cargo:</p>
          <pre className='bg-muted my-4 overflow-x-auto rounded-md border px-4 py-3'>
            <code className='text-foreground font-mono text-sm'>
              cargo install just-lsp
            </code>
          </pre>
          <p className='text-muted-foreground'>Or with Homebrew:</p>
          <pre className='bg-muted my-4 overflow-x-auto rounded-md border px-4 py-3'>
            <code className='text-foreground font-mono text-sm'>
              brew install just-lsp
            </code>
          </pre>
          <p className='text-muted-foreground'>
            <a
              href='https://github.com/terror/just-lsp/releases'
              className='text-foreground underline decoration-current/30 underline-offset-4 hover:decoration-current'
            >
              Pre-built binaries
            </a>{' '}
            and{' '}
            <a
              href='https://github.com/terror/just-lsp#installation'
              className='text-foreground underline decoration-current/30 underline-offset-4 hover:decoration-current'
            >
              other packages
            </a>{' '}
            are also available.
          </p>
        </section>

        <section aria-labelledby='editor-setup' className='mt-12'>
          <h2
            id='editor-setup'
            className='mb-4 text-xl font-semibold tracking-tight'
          >
            Usage
          </h2>
          <p className='text-muted-foreground'>
            just-lsp works with any editor that supports the Language Server
            Protocol. Make sure{' '}
            <code className='text-foreground font-mono text-sm'>just-lsp</code>{' '}
            is on your{' '}
            <code className='text-foreground font-mono text-sm'>PATH</code>,
            then enable it in your editor.
          </p>
          <ul className='mt-4 list-disc space-y-2 pl-5'>
            <li className='text-muted-foreground'>
              <a
                href='https://github.com/terror/just-lsp#neovim'
                className='text-foreground underline decoration-current/30 underline-offset-4 hover:decoration-current'
              >
                Neovim
              </a>
              : with Neovim 0.11.3+ and nvim-lspconfig installed, add the
              following to your configuration.
              <pre className='bg-muted my-4 overflow-x-auto rounded-md border px-4 py-3'>
                <code className='text-foreground font-mono text-sm'>
                  vim.lsp.enable('just')
                </code>
              </pre>
            </li>
            <li className='text-muted-foreground'>
              <a
                href='https://github.com/nefrob/vscode-just'
                className='text-foreground underline decoration-current/30 underline-offset-4 hover:decoration-current'
              >
                Visual Studio Code
              </a>
              : follow the Just extension’s setup instructions.
            </li>
            <li className='text-muted-foreground'>
              <a
                href='https://github.com/jackTabsCode/zed-just'
                className='text-foreground underline decoration-current/30 underline-offset-4 hover:decoration-current'
              >
                Zed
              </a>
              : install the Justfile extension.
            </li>
          </ul>
        </section>

        <section aria-labelledby='reference' className='mt-12'>
          <h2
            id='reference'
            className='mb-4 text-xl font-semibold tracking-tight'
          >
            Reference
          </h2>
          <ul className='list-disc space-y-2 pl-5'>
            <li className='text-muted-foreground'>
              <a
                href='https://github.com/terror/just-lsp#configuration'
                className='text-foreground underline decoration-current/30 underline-offset-4 hover:decoration-current'
              >
                Configuration
              </a>{' '}
              — formatting options and diagnostic severity levels.
            </li>
            <li className='text-muted-foreground'>
              <a
                href='https://github.com/terror/just-lsp/blob/master/docs/diagnostics.md'
                className='text-foreground underline decoration-current/30 underline-offset-4 hover:decoration-current'
              >
                Diagnostics
              </a>{' '}
              — the full list of rules and examples.
            </li>
            <li className='text-muted-foreground'>
              <a
                href='https://github.com/terror/just-lsp#cli'
                className='text-foreground underline decoration-current/30 underline-offset-4 hover:decoration-current'
              >
                Command line
              </a>{' '}
              — check a justfile with{' '}
              <code className='text-foreground font-mono text-sm'>
                just-lsp analyze
              </code>
              .
            </li>
            <li className='text-muted-foreground'>
              <Link
                to='/playground'
                className='text-foreground underline decoration-current/30 underline-offset-4 hover:decoration-current'
              >
                Playground
              </Link>{' '}
              — edit a justfile and explore its syntax tree in your browser.
            </li>
          </ul>
        </section>
      </main>
    </div>
  );
};

export default Home;
