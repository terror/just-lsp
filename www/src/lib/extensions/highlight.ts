import { StateEffect, StateField } from '@codemirror/state';
import { Decoration, DecorationSet, EditorView } from '@codemirror/view';

const highlightMark = Decoration.mark({ class: 'cm-highlighted-node' });

export const highlightEffect = StateEffect.define<{
  from: number;
  to: number;
} | null>();

export const highlightExtension = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(decorations, transaction) {
    if (transaction.docChanged) {
      decorations = Decoration.none;
    }

    for (const effect of transaction.effects) {
      if (effect.is(highlightEffect)) {
        const range = effect.value;

        decorations =
          range &&
          range.from >= 0 &&
          range.from < range.to &&
          range.to <= transaction.newDoc.length
            ? Decoration.set([highlightMark.range(range.from, range.to)])
            : Decoration.none;
      }
    }

    return decorations;
  },
  provide: (field) => EditorView.decorations.from(field),
});
