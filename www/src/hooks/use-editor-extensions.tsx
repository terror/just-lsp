import { useEditorSettings } from '@/contexts/editor-settings-context';
import {
  base16SetiDarkTheme,
  base16SetiLightTheme,
} from '@/lib/base16-seti-theme';
import { highlightExtension } from '@/lib/cm-highlight-extension';
import { createJustSyntaxHighlightingExtension } from '@/lib/just-syntax-highlighting';
import { EditorState, Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { vim } from '@replit/codemirror-vim';
import { useMemo } from 'react';
import { Language as TSLanguage } from 'web-tree-sitter';

interface UseEditorExtensionsOptions {
  language: TSLanguage | undefined;
  highlight: { from: number; to: number } | undefined;
  darkMode: boolean;
}

export function useEditorExtensions({
  language,
  highlight,
  darkMode,
}: UseEditorExtensionsOptions): Extension[] {
  const { settings } = useEditorSettings();

  return useMemo(() => {
    const extensions: Extension[] = [
      darkMode ? base16SetiDarkTheme : base16SetiLightTheme,
      EditorState.tabSize.of(settings.tabSize),
      ...createJustSyntaxHighlightingExtension(language),
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
    language,
    highlight,
    darkMode,
  ]);
}
