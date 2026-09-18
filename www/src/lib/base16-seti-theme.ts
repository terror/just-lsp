import { EditorView } from '@codemirror/view';

const createBase16SetiTheme = (dark: boolean) =>
  EditorView.theme(
    {
      '&': {
        backgroundColor: 'var(--editor-background)',
        color: 'var(--editor-foreground)',
      },
      '&.cm-focused': {
        outline: 'none',
      },
      '&.cm-focused .cm-cursor': {
        borderLeftColor: 'var(--editor-cursor)',
      },
      '&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, .cm-content::selection, .cm-content ::selection':
        {
          backgroundColor: 'var(--selection-background)',
        },
      '.cm-activeLine': {
        backgroundColor: 'var(--editor-active-line)',
      },
      '.cm-activeLineGutter': {
        backgroundColor: 'var(--editor-active-line)',
        color: 'var(--editor-foreground)',
      },
      '.cm-content': {
        caretColor: 'var(--editor-cursor)',
      },
      '.cm-foldPlaceholder': {
        backgroundColor: 'var(--editor-active-line)',
        borderColor: 'var(--editor-gutter-border)',
        color: 'var(--editor-foreground)',
      },
      '.cm-gutters': {
        backgroundColor: 'var(--editor-background)',
        borderRight: '1px solid var(--editor-gutter-border)',
        color: 'var(--editor-gutter-foreground)',
      },
      '.cm-lineNumbers .cm-gutterElement': {
        color: 'var(--editor-gutter-foreground)',
      },
      '.cm-matchingBracket': {
        backgroundColor: 'var(--editor-highlight-background)',
        color: 'var(--editor-foreground)',
      },
      '.cm-nonmatchingBracket': {
        backgroundColor: 'var(--editor-nonmatching-bracket-background)',
        color: 'var(--just-hl-error)',
      },
      '.cm-scroller': {
        backgroundColor: 'var(--editor-background)',
      },
      '.cm-tooltip': {
        backgroundColor: 'var(--editor-background)',
        borderColor: 'var(--editor-gutter-border)',
        color: 'var(--editor-foreground)',
      },
    },
    { dark }
  );

export const base16SetiLightTheme = createBase16SetiTheme(false);
export const base16SetiDarkTheme = createBase16SetiTheme(true);
