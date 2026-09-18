import Markdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

export const HoverContent = ({
  content,
  markdown,
}: {
  content: string;
  markdown: boolean;
}) =>
  markdown ? (
    <div className='[&_pre]:bg-muted grid gap-3 font-sans text-sm leading-relaxed break-words [&_a]:underline [&_a]:underline-offset-4 [&_blockquote]:border-l-2 [&_blockquote]:pl-3 [&_code]:font-mono [&_h1]:font-semibold [&_h2]:font-semibold [&_h3]:font-semibold [&_ol]:list-decimal [&_ol]:pl-5 [&_pre]:overflow-x-auto [&_pre]:rounded [&_pre]:p-3 [&_table]:border-collapse [&_td]:border [&_td]:px-2 [&_td]:py-1 [&_th]:border [&_th]:px-2 [&_th]:py-1 [&_ul]:list-disc [&_ul]:pl-5'>
      <Markdown remarkPlugins={[remarkGfm]}>{content}</Markdown>
    </div>
  ) : (
    <pre className='font-mono text-sm whitespace-pre-wrap'>{content}</pre>
  );
