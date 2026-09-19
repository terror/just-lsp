import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useEditorSettings } from '@/hooks/use-editor-settings';
import type { AnalysisClient } from '@/lib/analysis/client';
import {
  base16SetiDarkTheme,
  base16SetiLightTheme,
} from '@/lib/base16-seti-theme';
import { createDiagnosticsExtension } from '@/lib/extensions/diagnostics';
import { highlightExtension } from '@/lib/extensions/highlight';
import { createHoverExtension } from '@/lib/extensions/hover';
import { createSyntaxHighlightExtension } from '@/lib/extensions/syntax-highlight';
import { samples } from '@/lib/samples';
import { EditorState, Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { vim } from '@replit/codemirror-vim';
import CodeMirror from '@uiw/react-codemirror';
import { useMemo } from 'react';
import { Language } from 'web-tree-sitter';

import { EditorResetDialog } from './editor-reset-dialog';
import { EditorSettingsDialog } from './editor-settings-dialog';

interface EditorPaneProps {
  analysis: AnalysisClient;
  value: string;
  onChange: (value: string) => void;
  onCreateEditor: (view: EditorView) => void;
  onReset: () => void;
  sample: string;
  onSampleChange: (name: string) => void;
  language: Language;
  darkMode: boolean;
}

export const EditorPane = ({
  analysis,
  value,
  onChange,
  onCreateEditor,
  onReset,
  sample,
  onSampleChange,
  language,
  darkMode,
}: EditorPaneProps) => {
  const { settings, updateSettings } = useEditorSettings();

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

  const analysisExtensions = useMemo(
    () => [
      createDiagnosticsExtension(analysis),
      createHoverExtension(analysis),
    ],
    [analysis]
  );

  const extensions = useMemo(() => {
    const extensions: Extension[] = [
      EditorState.tabSize.of(settings.tabSize),
      ...analysisExtensions,
      syntaxHighlighting,
      highlightExtension,
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
    analysisExtensions,
  ]);

  return (
    <div className='flex h-full min-h-0 flex-col overflow-hidden'>
      <div className='bg-muted/50 flex shrink-0 items-center justify-between gap-1 border-b px-2 py-1'>
        <div className='flex min-w-0 items-center gap-1'>
          <Select value={sample} onValueChange={onSampleChange}>
            <SelectTrigger
              aria-label='Sample justfile'
              className='bg-background h-7 w-36 cursor-pointer text-sm'
            >
              <SelectValue placeholder='Select sample' />
            </SelectTrigger>
            <SelectContent>
              {samples.map((sample) => (
                <SelectItem key={sample.name} value={sample.name}>
                  {sample.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <EditorResetDialog sample={sample} onReset={onReset} />
        </div>
        <EditorSettingsDialog
          settings={settings}
          updateSettings={updateSettings}
        />
      </div>
      <div className='flex-1 overflow-hidden'>
        <div
          className='editor-host h-full w-full overflow-hidden'
          style={{ fontSize: settings.fontSize }}
        >
          <CodeMirror
            value={value}
            theme={darkMode ? base16SetiDarkTheme : base16SetiLightTheme}
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
