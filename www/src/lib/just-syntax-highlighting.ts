import { type Extension, RangeSetBuilder } from '@codemirror/state';
import {
  Decoration,
  EditorView,
  ViewPlugin,
  type ViewUpdate,
} from '@codemirror/view';
import type { Language, Parser, Query } from 'web-tree-sitter';
import { Parser as TreeSitterParser } from 'web-tree-sitter';

import highlightsQuerySource from '../../../vendor/tree-sitter-just/queries/just/highlights.scm?raw';

const BASE_CAPTURE_TO_CLASSES: Record<string, string[]> = {
  attribute: ['cm-just-attribute'],
  boolean: ['cm-just-boolean'],
  comment: ['cm-just-comment'],
  error: ['cm-just-error'],
  function: ['cm-just-function'],
  keyword: ['cm-just-keyword'],
  module: ['cm-just-namespace'],
  operator: ['cm-just-operator'],
  punctuation: ['cm-just-punctuation'],
  string: ['cm-just-string'],
  variable: ['cm-just-variable'],
};

const captureNameToClasses = (name: string): string[] => {
  const [base] = name.split('.');
  return BASE_CAPTURE_TO_CLASSES[base] ?? [];
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

    const classes = captureNameToClasses(name);

    if (classes.length === 0) {
      continue;
    }

    const key = `${from}:${to}`;
    const classSet = ranges.get(key) ?? new Set<string>();

    classes.forEach((cls) => classSet.add(cls));
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

export const createJustSyntaxHighlightingExtension = (
  language: Language | undefined
): Extension[] => {
  if (!language) {
    return [];
  }

  let query: Query;

  try {
    query = language.query(highlightsQuerySource);
  } catch (error) {
    console.error('Failed to compile Just highlight query', error);
    return [];
  }
  const lang = language;

  const plugin = ViewPlugin.fromClass(
    class {
      decorations = Decoration.none;
      private parser: Parser;

      constructor(view: EditorView) {
        this.parser = new TreeSitterParser();
        this.parser.setLanguage(lang);
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

  return [plugin];
};
