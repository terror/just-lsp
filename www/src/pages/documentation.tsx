import { useEffect } from 'react';
import { useLocation } from 'react-router';

import {
  DocumentationNavigation,
  DocumentationSections,
} from '../components/documentation-sections';
import { Header } from '../components/header';
import { useTheme } from '../hooks/use-theme';
import { loadDocumentation } from '../lib/documentation';

const sections = loadDocumentation(
  import.meta.glob<string>('./**/*.md', {
    base: '../../../docs',
    eager: true,
    import: 'default',
    query: '?raw',
  })
);

const Documentation = () => {
  const theme = useTheme();

  const { hash } = useLocation();

  useEffect(() => {
    document.title = 'Documentation - just-lsp';
  }, []);

  useEffect(() => {
    if (hash) {
      document.getElementById(hash.slice(1))?.scrollIntoView();
    }
  }, [hash]);

  return (
    <div className='min-h-screen'>
      <Header theme={theme} />

      <div className='mx-auto max-w-6xl px-6 py-12 lg:grid lg:grid-cols-[14rem_minmax(0,1fr)] lg:gap-12 lg:py-16'>
        <aside className='hidden lg:block'>
          <div className='sticky top-6 max-h-[calc(100dvh-3rem)] overflow-y-auto pr-4'>
            <DocumentationNavigation sections={sections} />
          </div>
        </aside>

        <main className='min-w-0 leading-[1.75]'>
          <h1 className='font-mono text-3xl font-semibold tracking-tight'>
            Documentation
          </h1>
          <p className='text-muted-foreground mt-4 text-lg'>
            Configuration options and diagnostics for just-lsp.
          </p>

          <details className='my-8 rounded-md border p-4 lg:hidden'>
            <summary className='cursor-pointer font-medium'>
              On this page
            </summary>
            <div className='mt-4 max-h-96 overflow-y-auto'>
              <DocumentationNavigation sections={sections} />
            </div>
          </details>

          <DocumentationSections sections={sections} />
        </main>
      </div>
    </div>
  );
};

export default Documentation;
