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

  try {
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

    return Decoration.set(
      Array.from(ranges.values(), ({ from, to, classes }) =>
        Decoration.mark({ class: Array.from(classes).join(' ') }).range(
          from,
          to
        )
      ),
      true
    );
  } finally {
    tree.delete();
  }
};

export const createSyntaxHighlightExtension = (
  language: Language
): Extension => {
  return ViewPlugin.fromClass(
    class {
      decorations = Decoration.none;
      private parser: Parser;
      private query: Query;

      constructor(view: EditorView) {
        this.parser = new Parser();

        try {
          this.parser.setLanguage(language);
          this.query = new Query(language, highlightsQuerySource);

          this.decorations = buildDecorations(
            this.parser,
            this.query,
            view.state.doc.toString()
          );
        } catch (error) {
          this.destroy();
          throw error;
        }
      }

      update(update: ViewUpdate) {
        if (update.docChanged) {
          this.decorations = buildDecorations(
            this.parser,
            this.query,
            update.state.doc.toString()
          );
        }
      }

      destroy() {
        this.query?.delete();
        this.parser.delete();
      }
    },
    {
      decorations: (v) => v.decorations,
    }
  );
};
