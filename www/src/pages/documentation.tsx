import { useEffect } from 'react';
import { useLocation } from 'react-router';

import configuration from '../../../docs/configuration.md?raw';
import diagnostics from '../../../docs/configuration/diagnostics.md?raw';
import formatting from '../../../docs/configuration/formatting.md?raw';
import overview from '../../../docs/rules.md?raw';
import { DocumentationMarkdown } from '../components/documentation-markdown';
import { Header } from '../components/header';
import { useTheme } from '../hooks/use-theme';
import { loadDocumentation } from '../lib/documentation';

const configurationSections = [
  { content: formatting, id: 'formatting', title: 'Formatting' },
  {
    content: diagnostics,
    id: 'diagnostics',
    title: 'Diagnostics',
  },
];

const ruleGroups = loadDocumentation(
  import.meta.glob<string>('../../../docs/rules/*.md', {
    eager: true,
    import: 'default',
    query: '?raw',
  })
);

const DocumentationNavigation = () => (
  <nav aria-label='Documentation navigation' className='space-y-6 text-sm'>
    <div>
      <a href='#configuration' className='font-medium hover:underline'>
        Configuration
      </a>
      <ul className='mt-2 space-y-1 border-l pl-3'>
        {configurationSections.map((section) => (
          <li key={section.id}>
            <a
              href={`#${section.id}`}
              className='text-muted-foreground hover:text-foreground block py-1 break-words'
            >
              {section.title}
            </a>
          </li>
        ))}
      </ul>
    </div>
    <a
      href='#rules'
      className='text-muted-foreground hover:text-foreground block'
    >
      Diagnostics
    </a>
    {ruleGroups.map((group) => (
      <div key={group.id}>
        <a href={`#${group.id}`} className='font-medium hover:underline'>
          {group.title}
        </a>
        <ul className='mt-2 space-y-1 border-l pl-3'>
          {group.rules.map((rule) => (
            <li key={rule.id}>
              <a
                href={`#${rule.id}`}
                className='text-muted-foreground hover:text-foreground block py-1 break-words'
              >
                {rule.title}
              </a>
            </li>
          ))}
        </ul>
      </div>
    ))}
  </nav>
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
            <DocumentationNavigation />
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
              <DocumentationNavigation />
            </div>
          </details>

          <section aria-labelledby='configuration' className='mt-12'>
            <h2
              id='configuration'
              className='scroll-mt-6 text-xl font-semibold tracking-tight'
            >
              Configuration
            </h2>
            <DocumentationMarkdown>{configuration}</DocumentationMarkdown>
            {configurationSections.map((section) => (
              <section
                key={section.id}
                aria-labelledby={section.id}
                className='mt-10'
              >
                <h3
                  id={section.id}
                  className='scroll-mt-6 text-lg font-semibold tracking-tight'
                >
                  <a
                    href={`#${section.id}`}
                    className='hover:underline hover:underline-offset-4'
                  >
                    {section.title}
                  </a>
                </h3>
                <DocumentationMarkdown>{section.content}</DocumentationMarkdown>
              </section>
            ))}
          </section>

          <section aria-labelledby='rules' className='mt-16'>
            <h2
              id='rules'
              className='scroll-mt-6 text-xl font-semibold tracking-tight'
            >
              Diagnostics
            </h2>
            <DocumentationMarkdown>{overview}</DocumentationMarkdown>
          </section>

          {ruleGroups.map((group) => (
            <section
              key={group.id}
              aria-labelledby={group.id}
              className='mt-16'
            >
              <h2
                id={group.id}
                className='scroll-mt-6 border-b pb-4 text-2xl font-semibold tracking-tight'
              >
                {group.title}
              </h2>
              {group.rules.map((rule) => (
                <article
                  key={rule.id}
                  id={rule.id}
                  aria-labelledby={`${rule.id}-title`}
                  className='scroll-mt-6 border-b py-10 last:border-b-0 last:pb-0'
                >
                  <div className='flex flex-wrap items-center gap-x-3 gap-y-2'>
                    <h3
                      id={`${rule.id}-title`}
                      className='text-xl font-semibold tracking-tight'
                    >
                      <a
                        href={`#${rule.id}`}
                        className='hover:underline hover:underline-offset-4'
                      >
                        {rule.title}
                      </a>
                    </h3>
                    <span className='text-muted-foreground rounded border px-2 py-0.5 text-sm'>
                      Default: {rule.severity}
                    </span>
                  </div>
                  <p className='text-muted-foreground mt-1 font-mono text-sm break-all'>
                    {rule.id}
                  </p>
                  <DocumentationMarkdown>{rule.content}</DocumentationMarkdown>
                </article>
              ))}
            </section>
          ))}
        </main>
      </div>
    </div>
  );
};

export default Documentation;
