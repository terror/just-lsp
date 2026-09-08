import { parse } from 'yaml';
import { z } from 'zod';

const metadataSchema = z.object({
  order: z.int().positive(),
  severity: z.enum(['error', 'warning']).optional(),
  title: z.string().trim().min(1),
});

export type DocumentationSection = z.infer<typeof metadataSchema> & {
  children: DocumentationSection[];
  content: string;
  id: string;
};

const parseDocument = ([path, source]: [string, string]): [
  string,
  DocumentationSection,
] => {
  const filename = path.match(
    /^(?:\.\/)?((?:[a-z][a-z0-9-]*\/)*)([a-z][a-z0-9-]*)\.md$/
  );

  const frontmatter = source.match(/^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/);

  if (!filename || !frontmatter) {
    throw new Error(
      `${path}: expected a documentation filename and YAML frontmatter`
    );
  }

  try {
    const metadata = metadataSchema.parse(parse(frontmatter[1]));

    const content = source.slice(frontmatter[0].length).trim();

    return [
      `${filename[1]}${filename[2]}`,
      { ...metadata, children: [], content, id: filename[2] },
    ];
  } catch (error) {
    throw new Error(
      `${path}: ${error instanceof Error ? error.message : error}`,
      { cause: error }
    );
  }
};

export const loadDocumentation = (
  sources: Record<string, string>
): DocumentationSection[] => {
  const documents = Object.entries(sources)
    .map(parseDocument)
    .sort(
      ([, left], [, right]) =>
        left.order - right.order || left.id.localeCompare(right.id)
    );

  const paths = new Map(documents);
  const sections: DocumentationSection[] = [];
  const ids = new Set<string>();

  for (const [path, section] of documents) {
    if (ids.has(section.id)) {
      throw new Error(`Duplicate documentation ID: ${section.id}`);
    }

    ids.add(section.id);

    const parentPath = path.match(/^(.*)\//)?.[1];
    const parent = parentPath ? paths.get(parentPath) : undefined;

    if (parentPath && !parent) {
      throw new Error(`${path}.md: missing parent document ${parentPath}.md`);
    }

    (parent?.children ?? sections).push(section);
  }

  return sections;
};
