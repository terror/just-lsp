import { type ComponentProps } from 'react';
import Markdown, { type ExtraProps } from 'react-markdown';
import remarkGfm from 'remark-gfm';

import { remarkExampleDiffs } from '../lib/remark-example-diffs';
import { cn } from '../lib/utils';

const DiffBlock = ({ children }: { children: string }) => (
  <figure className='min-w-0 overflow-hidden rounded-md border'>
    <figcaption className='bg-muted flex flex-wrap gap-x-4 gap-y-1 border-b px-4 py-2 text-sm font-medium'>
      <span className='text-red-700 dark:text-red-300'>− Reported</span>
      <span className='text-green-700 dark:text-green-300'>+ Corrected</span>
    </figcaption>
    <pre
      aria-label='Changes from reported to corrected'
      className='text-foreground overflow-x-auto py-3 text-sm leading-6 [tab-size:4]'
    >
      <code className='block w-max min-w-full'>
        {children
          .replace(/\n$/, '')
          .split('\n')
          .map((line, index) => (
            <span
              key={index}
              className={cn(
                'flex min-h-6 px-4',
                line.startsWith('-') &&
                  'bg-red-500/10 text-red-700 dark:bg-red-500/15 dark:text-red-300',
                line.startsWith('+') &&
                  'bg-green-500/10 text-green-700 dark:bg-green-500/15 dark:text-green-300'
              )}
            >
              <span className='w-6 shrink-0 select-none'>{line[0]}</span>
              <span>
                {line.slice(1)}
                {'\n'}
              </span>
            </span>
          ))}
      </code>
    </pre>
  </figure>
);

const CodeBlock = ({ children, node }: ComponentProps<'pre'> & ExtraProps) => {
  const code = node?.children[0];

  const label = code?.type === 'element' ? code.data?.meta : undefined;

  if (
    label === 'reported-corrected' &&
    code?.type === 'element' &&
    code.children[0]?.type === 'text'
  ) {
    return <DiffBlock>{code.children[0].value}</DiffBlock>;
  }

  const block = (
    <pre className='bg-muted text-foreground overflow-x-auto rounded-md border px-4 py-3 text-sm leading-6 [tab-size:4]'>
      {children}
    </pre>
  );

  return label === 'reported' || label === 'corrected' ? (
    <figure className='min-w-0'>
      <figcaption className='text-muted-foreground mb-2 text-sm font-medium'>
        {label === 'reported' ? 'Reported' : 'Corrected'}
      </figcaption>
      {block}
    </figure>
  ) : (
    block
  );
};

const Table = ({ children }: ComponentProps<'table'>) => (
  <div className='min-w-0 overflow-x-auto rounded-md border'>
    <table className='[&_th]:bg-muted [&_th]:text-foreground w-full border-collapse text-left text-sm [&_td]:border-t [&_td]:px-4 [&_td]:py-3 [&_td_code]:whitespace-nowrap [&_th]:px-4 [&_th]:py-3 [&_th]:font-semibold'>
      {children}
    </table>
  </div>
);

export const DocumentationMarkdown = ({ children }: { children: string }) => (
  <div className='text-muted-foreground [&>p:first-child]:text-foreground [&_a]:text-foreground [&_code]:text-foreground [&_h5]:text-foreground [&_h6]:text-foreground mt-4 grid min-w-0 grid-cols-1 gap-4 [&_a]:underline [&_a]:underline-offset-4 [&_blockquote]:border-l-2 [&_blockquote]:pl-4 [&_code]:font-mono [&_code]:text-[0.875em] [&_h5]:mt-2 [&_h5]:font-semibold [&_h6]:font-medium [&_li+li]:mt-2 [&_ol]:list-decimal [&_ol]:pl-5 [&_pre_code]:text-sm [&_ul]:list-disc [&_ul]:pl-5 [&>*]:col-span-full'>
    <Markdown
      components={{ h2: 'h5', h3: 'h6', pre: CodeBlock, table: Table }}
      remarkPlugins={[remarkGfm, remarkExampleDiffs]}
    >
      {children}
    </Markdown>
  </div>
);
