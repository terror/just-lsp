import { parse } from 'yaml';

type RuleMetadata = {
  category: string;
  order: number;
  severity: 'error' | 'warning';
  title: string;
};

type DocumentationRule = RuleMetadata & {
  content: string;
  id: string;
};

type RuleGroup = {
  id: string;
  rules: DocumentationRule[];
  title: string;
};

const isMetadata = (value: unknown): value is RuleMetadata =>
  typeof value === 'object' &&
  value !== null &&
  'title' in value &&
  typeof value.title === 'string' &&
  value.title.trim().length > 0 &&
  'category' in value &&
  typeof value.category === 'string' &&
  value.category.trim().length > 0 &&
  'severity' in value &&
  (value.severity === 'error' || value.severity === 'warning') &&
  'order' in value &&
  typeof value.order === 'number' &&
  Number.isSafeInteger(value.order) &&
  value.order > 0;

const parseRule = ([path, source]: [string, string]): DocumentationRule => {
  const id = path.match(/\/([a-z][a-z0-9-]*)\.md$/)?.[1];
  const frontmatter = source.match(/^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/);

  if (!id || !frontmatter) {
    throw new Error(`${path}: expected a rule filename and YAML frontmatter`);
  }

  const metadata: unknown = parse(frontmatter[1]);
  const content = source.slice(frontmatter[0].length).trim();

  if (!isMetadata(metadata) || !content) {
    throw new Error(
      `${path}: expected title, category, severity, order, and Markdown content`
    );
  }

  return { ...metadata, content, id };
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
