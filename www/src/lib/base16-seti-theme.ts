import { EditorView } from '@codemirror/view';

const base16Seti = {
  base00: '#151718',
  base01: '#282a2b',
  base02: '#3b758c',
  base03: '#41535b',
  base04: '#43a5d5',
  base05: '#d6d6d6',
  base06: '#eeeeee',
  base07: '#ffffff',
  base08: '#cd3f45',
  base09: '#db7b55',
  base0A: '#e6cd69',
  base0B: '#9fca56',
  base0C: '#55dbbe',
  base0D: '#55b5db',
  base0E: '#a074c4',
  base0F: '#8a553f',
};

const lightEditor = {
  activeLine: '#f2f7f9',
  background: '#ffffff',
  boolean: '#a84f24',
  comment: '#667984',
  cursor: '#151718',
  foreground: '#151718',
  function: '#147fa8',
  gutterBackground: '#ffffff',
  gutterBorder: '#e8eef1',
  gutterForeground: '#7d8e96',
  highlightBackground: '#e1f3f8',
  highlightOutline: '#78bdd6',
  keyword: '#8055a5',
  namespace: '#168a78',
  punctuation: '#70828b',
  selectionBackground: '#cfe9f2',
  string: '#5d861e',
};

const darkEditor = {
  activeLine: base16Seti.base01,
  background: base16Seti.base00,
  boolean: base16Seti.base09,
  comment: '#78909c',
  cursor: base16Seti.base06,
  foreground: base16Seti.base05,
  function: base16Seti.base0D,
  gutterBackground: base16Seti.base00,
  gutterBorder: base16Seti.base01,
  gutterForeground: '#70838c',
  highlightBackground: base16Seti.base02,
  highlightOutline: base16Seti.base0D,
  keyword: base16Seti.base0E,
  namespace: base16Seti.base0C,
  punctuation: '#91a7b0',
  selectionBackground: '#315f70',
  string: base16Seti.base0B,
};

const createBase16SetiTheme = (editor: typeof lightEditor, dark: boolean) =>
  EditorView.theme(
    {
      '&': {
        backgroundColor: editor.background,
        color: editor.foreground,
        '--editor-active-line': editor.activeLine,
        '--editor-cursor-secondary': `${base16Seti.base0D}4d`,
        '--editor-fat-cursor': `${base16Seti.base0D}80`,
        '--editor-gutter-background': editor.gutterBackground,
        '--editor-gutter-border': `1px solid ${editor.gutterBorder}`,
        '--editor-gutter-foreground': editor.gutterForeground,
        '--editor-highlight-background': editor.highlightBackground,
        '--editor-highlight-outline': editor.highlightOutline,
        '--just-hl-attribute': editor.boolean,
        '--just-hl-boolean': editor.boolean,
        '--just-hl-comment': editor.comment,
        '--just-hl-error': base16Seti.base08,
        '--just-hl-function': editor.function,
        '--just-hl-keyword': editor.keyword,
        '--just-hl-namespace': editor.namespace,
        '--just-hl-operator': editor.namespace,
        '--just-hl-punctuation': editor.punctuation,
        '--just-hl-string': editor.string,
        '--just-hl-variable': editor.foreground,
      },
      '&.cm-focused': {
        outline: 'none',
      },
      '&.cm-focused .cm-cursor': {
        borderLeftColor: editor.cursor,
      },
      '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection':
        {
          backgroundColor: editor.selectionBackground,
        },
      '.cm-activeLine': {
        backgroundColor: editor.activeLine,
      },
      '.cm-activeLineGutter': {
        backgroundColor: editor.activeLine,
        color: editor.foreground,
      },
      '.cm-content': {
        caretColor: editor.cursor,
      },
      '.cm-foldPlaceholder': {
        backgroundColor: editor.activeLine,
        borderColor: editor.gutterBorder,
        color: editor.foreground,
      },
      '.cm-gutters': {
        backgroundColor: editor.gutterBackground,
        borderRight: `1px solid ${editor.gutterBorder}`,
        color: editor.gutterForeground,
      },
      '.cm-lineNumbers .cm-gutterElement': {
        color: editor.gutterForeground,
      },
      '.cm-matchingBracket': {
        backgroundColor: editor.highlightBackground,
        color: editor.foreground,
      },
      '.cm-nonmatchingBracket': {
        backgroundColor: dark ? '#4a2428' : '#f9dcde',
        color: base16Seti.base08,
      },
      '.cm-scroller': {
        backgroundColor: editor.background,
      },
      '.cm-tooltip': {
        backgroundColor: editor.background,
        borderColor: editor.gutterBorder,
        color: editor.foreground,
      },
    },
    { dark }
  );

export const base16SetiLightTheme = createBase16SetiTheme(lightEditor, false);
export const base16SetiDarkTheme = createBase16SetiTheme(darkEditor, true);
