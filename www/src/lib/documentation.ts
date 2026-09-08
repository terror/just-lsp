import { parse } from 'yaml';
import { z } from 'zod';

const ruleMetadataSchema = z.object({
  category: z.string().trim().min(1),
  order: z.int().positive(),
  severity: z.enum(['error', 'warning']),
  title: z.string().trim().min(1),
});

type DocumentationRule = z.infer<typeof ruleMetadataSchema> & {
  content: string;
  id: string;
};

type RuleGroup = {
  id: string;
  rules: DocumentationRule[];
  title: string;
};

const parseRule = ([path, source]: [string, string]): DocumentationRule => {
  const id = path.match(/\/([a-z][a-z0-9-]*)\.md$/)?.[1];

  const frontmatter = source.match(/^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/);

  if (!id || !frontmatter) {
    throw new Error(`${path}: expected a rule filename and YAML frontmatter`);
  }

  const metadata = ruleMetadataSchema.safeParse(parse(frontmatter[1]));

  const content = source.slice(frontmatter[0].length).trim();

  if (!metadata.success || !content) {
    throw new Error(
      `${path}: expected title, category, severity, order, and Markdown content`
    );
  }

  return { ...metadata.data, content, id };
};

export const loadDocumentation = (
  sources: Record<string, string>
): RuleGroup[] => {
  const rules = Object.entries(sources)
    .map(parseRule)
    .sort(
      (left, right) =>
        left.order - right.order || left.id.localeCompare(right.id)
    );

  const groups = new Map<string, RuleGroup>();
  const ids = new Set<string>();

  for (const rule of rules) {
    if (ids.has(rule.id)) {
      throw new Error(`Duplicate documentation rule: ${rule.id}`);
    }

    ids.add(rule.id);

    const group = groups.get(rule.category) ?? {
      id: rule.category.toLowerCase().replace(/[^a-z0-9]+/g, '-'),
      rules: [],
      title: rule.category,
    };

    group.rules.push(rule);
    groups.set(rule.category, group);
  }

  return [...groups.values()];
};
