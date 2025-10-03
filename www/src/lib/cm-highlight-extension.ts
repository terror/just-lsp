import { StateEffect } from '@codemirror/state';
import {
  Decoration,
  DecorationSet,
  ViewPlugin,
  ViewUpdate,
} from '@codemirror/view';

const highlightMark = Decoration.mark({ class: 'cm-highlighted-node' });

export const addHighlightEffect = StateEffect.define<{
  from: number;
  to: number;
}>();

export const removeHighlightEffect = StateEffect.define<null>();

export const highlightExtension = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor() {
      this.decorations = Decoration.none;
    }

    update(update: ViewUpdate) {
      const effects = update.transactions
        .flatMap((tr) => tr.effects)
        .filter((e) => e.is(addHighlightEffect) || e.is(removeHighlightEffect));

      if (!effects.length) return;

      for (const effect of effects) {
        if (effect.is(addHighlightEffect)) {
          const { from, to } = effect.value;
          this.decorations = Decoration.set([highlightMark.range(from, to)]);
        } else if (effect.is(removeHighlightEffect)) {
          this.decorations = Decoration.none;
        }
      }
    }
  },
  {
    decorations: (v) => v.decorations,
  }
);
