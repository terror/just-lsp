import { ChevronRight } from 'lucide-react';

import { type DocumentationSection } from '../lib/documentation';
import { cn } from '../lib/utils';
import { DocumentationMarkdown } from './documentation-markdown';

const DocumentationLink = ({
  section,
  root = false,
}: {
  section: DocumentationSection;
  root?: boolean;
}) => {
  const link = (
    <a
      href={`#${section.id}`}
      className={cn(
        'min-w-0 break-words',
        root
          ? 'text-base font-semibold hover:underline'
          : section.children.length
            ? 'font-medium hover:underline'
            : 'text-muted-foreground hover:text-foreground block py-1'
      )}
    >
      {section.title}
    </a>
  );

  return section.children.length ? (
    <details open>
      <summary className='focus-visible:outline-ring flex cursor-pointer list-none items-center gap-1 rounded-sm focus-visible:outline-2 focus-visible:outline-offset-4 [&::-webkit-details-marker]:hidden [[open]>&>svg]:rotate-90'>
        <ChevronRight aria-hidden='true' className='size-4 shrink-0' />
        {link}
      </summary>
      <DocumentationLinks sections={section.children} />
    </details>
  ) : (
    link
  );
};

const DocumentationLinks = ({
  sections,
}: {
  sections: DocumentationSection[];
}) => (
  <ul
    className={cn(
      'mt-3 border-l pl-3',
      sections.some((section) => section.children.length)
        ? 'space-y-4'
        : 'space-y-1'
    )}
  >
    {sections.map((section) => (
      <li key={section.id}>
        <DocumentationLink section={section} />
      </li>
    ))}
  </ul>
);

export const DocumentationNavigation = ({
  sections,
}: {
  sections: DocumentationSection[];
}) => (
  <nav aria-label='Documentation navigation' className='space-y-6 text-sm'>
    {sections.map((section) => (
      <div key={section.id}>
        <DocumentationLink section={section} root />
      </div>
    ))}
  </nav>
);

const DocumentationContent = ({
  section,
  depth,
}: {
  section: DocumentationSection;
  depth: number;
}) => {
  const Heading = (['h2', 'h3', 'h4', 'h5', 'h6'] as const)[Math.min(depth, 4)];
  const Container = section.severity ? 'article' : 'section';

  return (
    <Container
      aria-labelledby={section.id}
      className={
        depth === 0
          ? 'mt-16 border-t pt-12 first:mt-12 first:border-t-0 first:pt-0'
          : section.severity
            ? 'border-b py-8 last:border-b-0 last:pb-0'
            : 'mt-10'
      }
    >
      <div className='flex flex-wrap items-center gap-x-3 gap-y-2'>
        <Heading
          id={section.id}
          className={cn(
            'scroll-mt-6 font-semibold tracking-tight',
            depth === 0 ? 'text-2xl' : depth === 1 ? 'text-xl' : 'text-lg',
            !section.content && 'w-full border-b pb-3'
          )}
        >
          <a
            href={`#${section.id}`}
            className='hover:underline hover:underline-offset-4'
          >
            {section.title}
          </a>
        </Heading>
        {section.severity && (
          <span className='text-muted-foreground rounded border px-2 py-0.5 text-sm'>
            Default: {section.severity}
          </span>
        )}
      </div>
      {section.severity && (
        <p className='text-muted-foreground mt-1 font-mono text-sm break-all'>
          {section.id}
        </p>
      )}
      {section.content && (
        <DocumentationMarkdown>{section.content}</DocumentationMarkdown>
      )}
      {section.children.map((section) => (
        <DocumentationContent
          key={section.id}
          section={section}
          depth={depth + 1}
        />
      ))}
    </Container>
  );
};

export const DocumentationSections = ({
  sections,
}: {
  sections: DocumentationSection[];
}) => (
  <div>
    {sections.map((section) => (
      <DocumentationContent key={section.id} section={section} depth={0} />
    ))}
  </div>
);
