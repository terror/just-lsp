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

export const base16SetiTheme = EditorView.theme(
  {
    '&': {
      backgroundColor: lightEditor.background,
      color: lightEditor.foreground,
      '--editor-active-line': lightEditor.activeLine,
      '--editor-cursor-secondary': `${base16Seti.base0D}4d`,
      '--editor-fat-cursor': `${base16Seti.base0D}80`,
      '--editor-gutter-background': lightEditor.gutterBackground,
      '--editor-gutter-border': `1px solid ${lightEditor.gutterBorder}`,
      '--editor-gutter-foreground': lightEditor.gutterForeground,
      '--editor-highlight-background': lightEditor.highlightBackground,
      '--editor-highlight-outline': lightEditor.highlightOutline,
      '--just-hl-attribute': lightEditor.boolean,
      '--just-hl-boolean': lightEditor.boolean,
      '--just-hl-comment': lightEditor.comment,
      '--just-hl-error': base16Seti.base08,
      '--just-hl-function': lightEditor.function,
      '--just-hl-keyword': lightEditor.keyword,
      '--just-hl-namespace': lightEditor.namespace,
      '--just-hl-operator': lightEditor.namespace,
      '--just-hl-punctuation': lightEditor.punctuation,
      '--just-hl-string': lightEditor.string,
      '--just-hl-variable': lightEditor.foreground,
    },
    '&.cm-focused': {
      outline: 'none',
    },
    '&.cm-focused .cm-cursor': {
      borderLeftColor: lightEditor.cursor,
    },
    '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection':
      {
        backgroundColor: lightEditor.selectionBackground,
      },
    '.cm-activeLine': {
      backgroundColor: lightEditor.activeLine,
    },
    '.cm-activeLineGutter': {
      backgroundColor: lightEditor.activeLine,
      color: lightEditor.foreground,
    },
    '.cm-content': {
      caretColor: lightEditor.cursor,
    },
    '.cm-foldPlaceholder': {
      backgroundColor: lightEditor.activeLine,
      borderColor: lightEditor.gutterBorder,
      color: lightEditor.foreground,
    },
    '.cm-gutters': {
      backgroundColor: lightEditor.gutterBackground,
      borderRight: `1px solid ${lightEditor.gutterBorder}`,
      color: lightEditor.gutterForeground,
    },
    '.cm-lineNumbers .cm-gutterElement': {
      color: lightEditor.gutterForeground,
    },
    '.cm-matchingBracket': {
      backgroundColor: lightEditor.highlightBackground,
      color: lightEditor.foreground,
    },
    '.cm-nonmatchingBracket': {
      backgroundColor: '#f9dcde',
      color: base16Seti.base08,
    },
    '.cm-scroller': {
      backgroundColor: lightEditor.background,
    },
    '.cm-tooltip': {
      backgroundColor: lightEditor.background,
      borderColor: lightEditor.gutterBorder,
      color: lightEditor.foreground,
    },
  },
  { dark: false }
);
