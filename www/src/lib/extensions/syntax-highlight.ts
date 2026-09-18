import { type Extension, RangeSetBuilder } from '@codemirror/state';
import {
  Decoration,
  EditorView,
  ViewPlugin,
  type ViewUpdate,
} from '@codemirror/view';
import { type Language, Parser, Query } from 'web-tree-sitter';

import highlightsQuerySource from '../../../../queries/highlights.scm?raw';

const BASE_CAPTURE_TO_CLASS: Record<string, string> = {
  attribute: 'cm-just-attribute',
  boolean: 'cm-just-boolean',
  comment: 'cm-just-comment',
  error: 'cm-just-error',
  function: 'cm-just-function',
  keyword: 'cm-just-keyword',
  module: 'cm-just-namespace',
  operator: 'cm-just-operator',
  punctuation: 'cm-just-punctuation',
  string: 'cm-just-string',
  variable: 'cm-just-variable',
};

const captureNameToClass = (name: string): string | undefined => {
  const [base] = name.split('.');
  return BASE_CAPTURE_TO_CLASS[base];
};

const buildDecorations = (parser: Parser, query: Query, content: string) => {
  const tree = parser.parse(content);

  if (!tree) {
    return Decoration.none;
  }

  const captures = query.captures(tree.rootNode);
  const ranges = new Map<string, Set<string>>();

  for (const { name, node } of captures) {
    const from = node.startIndex;
    const to = node.endIndex;

    if (from === to) {
      continue;
    }

    const className = captureNameToClass(name);

    if (className === undefined) {
      continue;
    }

    const key = `${from}:${to}`;
    const classSet = ranges.get(key) ?? new Set<string>();

    classSet.add(className);
    ranges.set(key, classSet);
  }

  const builder = new RangeSetBuilder<Decoration>();

  Array.from(ranges.entries())
    .map(([key, classSet]) => {
      const [from, to] = key.split(':').map(Number);
      return { from, to, className: Array.from(classSet).join(' ') };
    })
    .sort((a, b) => a.from - b.from || a.to - b.to)
    .forEach(({ from, to, className }) => {
      builder.add(from, to, Decoration.mark({ class: className }));
    });

  tree.delete();

  return builder.finish();
};

export const createSyntaxHighlightExtension = (
  language: Language
): Extension => {
  let query: Query;

  try {
    query = new Query(language, highlightsQuerySource);
  } catch (error) {
    console.error('Failed to compile Just highlight query', error);
    return [];
  }

  return ViewPlugin.fromClass(
    class {
      decorations = Decoration.none;
      private parser: Parser;

      constructor(view: EditorView) {
        this.parser = new Parser();

        this.parser.setLanguage(language);

        this.decorations = buildDecorations(
          this.parser,
          query,
          view.state.doc.toString()
        );
      }

      update(update: ViewUpdate) {
        if (update.docChanged) {
          this.decorations = buildDecorations(
            this.parser,
            query,
            update.state.doc.toString()
          );
        }
      }

      destroy() {
        this.parser.delete();
      }
    },
    {
      decorations: (v) => v.decorations,
    }
  );
};
