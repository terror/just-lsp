import { type Extension } from '@codemirror/state';
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

const buildDecorations = (parser: Parser, query: Query, content: string) => {
  const tree = parser.parse(content);

  if (!tree) {
    return Decoration.none;
  }

  const captures = query.captures(tree.rootNode);

  const ranges = new Map<
    string,
    { from: number; to: number; classes: Set<string> }
  >();

  for (const { name, node } of captures) {
    const from = node.startIndex;
    const to = node.endIndex;

    if (from === to) {
      continue;
    }

    const className = BASE_CAPTURE_TO_CLASS[name.split('.')[0]];

    if (className === undefined) {
      continue;
    }

    const key = `${from}:${to}`;
    const range = ranges.get(key) ?? { from, to, classes: new Set<string>() };

    range.classes.add(className);
    ranges.set(key, range);
  }

  tree.delete();

  return Decoration.set(
    Array.from(ranges.values(), ({ from, to, classes }) =>
      Decoration.mark({ class: Array.from(classes).join(' ') }).range(from, to)
    ),
    true
  );
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
