import { EditorState } from '@codemirror/state';
import { Decoration, EditorView } from '@codemirror/view';
import { describe, expect, it } from 'bun:test';

import { highlightEffect, highlightExtension } from './highlight';

describe('highlightExtension', () => {
  it('sets and clears highlights', () => {
    const state = EditorState.create({
      doc: 'foo',
      extensions: highlightExtension,
    }).update({ effects: highlightEffect.of({ from: 0, to: 3 }) }).state;

    expect(state.facet(EditorView.decorations)).toEqual([
      Decoration.set([
        Decoration.mark({ class: 'cm-highlighted-node' }).range(0, 3),
      ]),
    ]);

    for (const transaction of [
      { effects: highlightEffect.of(null) },
      { changes: { from: 0, to: 3 } },
    ]) {
      expect(state.update(transaction).state.field(highlightExtension)).toBe(
        Decoration.none
      );
    }
  });

  it('ignores an empty document’s root', () => {
    const state = EditorState.create({
      extensions: highlightExtension,
    }).update({ effects: highlightEffect.of({ from: 0, to: 0 }) }).state;

    expect(state.field(highlightExtension)).toBe(Decoration.none);
  });
});
