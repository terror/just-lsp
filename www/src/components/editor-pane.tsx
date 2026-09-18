import { useEditorSettings } from '@/contexts/editor-settings-context';
import {
  base16SetiDarkTheme,
  base16SetiLightTheme,
} from '@/lib/base16-seti-theme';
import { diagnosticsExtension } from '@/lib/extensions/diagnostics';
import { highlightExtension } from '@/lib/extensions/highlight';
import { hoverExtension } from '@/lib/extensions/hover';
import { createSyntaxHighlightExtension } from '@/lib/extensions/syntax-highlight';
import { EditorState, Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { vim } from '@replit/codemirror-vim';
import CodeMirror from '@uiw/react-codemirror';
import { useMemo } from 'react';
import { Language } from 'web-tree-sitter';

import { EditorSettingsDialog } from './editor-settings-dialog';

interface EditorPaneProps {
  value: string;
  onChange: (value: string) => void;
  onCreateEditor: (view: EditorView) => void;
  language: Language;
  darkMode: boolean;
}

export const EditorPane = ({
  value,
  onChange,
  onCreateEditor,
  language,
  darkMode,
}: EditorPaneProps) => {
  const { settings } = useEditorSettings();

  const basicSetup = useMemo(
    () => ({
      lineNumbers: settings.lineNumbers,
      tabSize: settings.tabSize,
      foldGutter: false,
      closeBrackets: false,
      autocompletion: false,
      highlightSelectionMatches: false,
      lintKeymap: false,
    }),
    [settings.lineNumbers, settings.tabSize]
  );

  const syntaxHighlighting = useMemo(
    () => createSyntaxHighlightExtension(language),
    [language]
  );

  const extensions = useMemo(() => {
    const extensions: Extension[] = [
      darkMode ? base16SetiDarkTheme : base16SetiLightTheme,
      EditorState.tabSize.of(settings.tabSize),
      diagnosticsExtension,
      syntaxHighlighting,
      highlightExtension,
      hoverExtension,
    ];

    if (settings.keybindings === 'vim') {
      extensions.push(vim());
    }

    if (settings.lineWrapping) {
      extensions.push(EditorView.lineWrapping);
    }

    return extensions;
  }, [
    settings.tabSize,
    settings.keybindings,
    settings.lineWrapping,
    syntaxHighlighting,
    darkMode,
  ]);

  return (
    <div className='flex h-full min-h-0 flex-col overflow-hidden'>
      <div className='bg-muted/50 flex items-center justify-between border-b px-2 py-1'>
        <EditorSettingsDialog />
      </div>
      <div className='flex-1 overflow-hidden'>
        <div className='editor-host h-full w-full overflow-hidden'>
          <CodeMirror
            value={value}
            extensions={extensions}
            basicSetup={basicSetup}
            onChange={onChange}
            onCreateEditor={onCreateEditor}
            height='100%'
            style={{ height: '100%' }}
          />
        </div>
      </div>
    </div>
  );
};
