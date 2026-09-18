import { useEditorSettings } from '@/contexts/editor-settings-context';
import {
  base16SetiDarkTheme,
  base16SetiLightTheme,
} from '@/lib/base16-seti-theme';
import { diagnosticsExtension } from '@/lib/extensions/diagnostics';
import { highlightExtension } from '@/lib/extensions/highlight';
import { createJustSyntaxHighlightingExtension } from '@/lib/just-syntax-highlighting';
import { EditorState, Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { vim } from '@replit/codemirror-vim';
import { useMemo } from 'react';
import { Language as TSLanguage } from 'web-tree-sitter';

interface UseEditorExtensionsOptions {
  language: TSLanguage;
  highlight: { from: number; to: number } | undefined;
  darkMode: boolean;
}

export function useEditorExtensions({
  language,
  highlight,
  darkMode,
}: UseEditorExtensionsOptions): Extension[] {
  const { settings } = useEditorSettings();

  const syntaxHighlighting = useMemo(
    () => createJustSyntaxHighlightingExtension(language),
    [language]
  );

  return useMemo(() => {
    const extensions: Extension[] = [
      darkMode ? base16SetiDarkTheme : base16SetiLightTheme,
      EditorState.tabSize.of(settings.tabSize),
      diagnosticsExtension,
      ...syntaxHighlighting,
      highlightExtension(highlight),
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
    highlight,
    darkMode,
  ]);
}
