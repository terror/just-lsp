import { type ComponentProps } from 'react';
import Markdown, { type ExtraProps } from 'react-markdown';

const CodeBlock = ({ children, node }: ComponentProps<'pre'> & ExtraProps) => {
  const code = node?.children[0];

  const label = code?.type === 'element' ? code.data?.meta : undefined;

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

export const DocumentationMarkdown = ({ children }: { children: string }) => (
  <div className='text-muted-foreground [&>p:first-child]:text-foreground [&_a]:text-foreground [&_code]:text-foreground [&_h5]:text-foreground [&_h6]:text-foreground mt-4 grid min-w-0 grid-cols-1 gap-4 xl:grid-cols-2 [&_a]:underline [&_a]:underline-offset-4 [&_blockquote]:border-l-2 [&_blockquote]:pl-4 [&_code]:font-mono [&_code]:text-[0.875em] [&_h5]:mt-2 [&_h5]:font-semibold [&_h6]:font-medium [&_li+li]:mt-2 [&_ol]:list-decimal [&_ol]:pl-5 [&_pre_code]:text-sm [&_ul]:list-disc [&_ul]:pl-5 [&>*]:col-span-full [&>figure]:col-span-1'>
    <Markdown components={{ h2: 'h5', h3: 'h6', pre: CodeBlock }}>
      {children}
    </Markdown>
  </div>
);
