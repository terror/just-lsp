import { createContext, useContext } from 'react';

export interface EditorSettings {
  fontSize: number;
  keybindings: 'default' | 'vim';
  lineNumbers: boolean;
  lineWrapping: boolean;
  tabSize: number;
}

export const defaultSettings: EditorSettings = {
  fontSize: 14,
  keybindings: 'default',
  lineNumbers: true,
  lineWrapping: true,
  tabSize: 2,
};

export type EditorSettingsContextType = {
  settings: EditorSettings;
  updateSettings: (settings: Partial<EditorSettings>) => void;
};

export const EditorSettingsContext = createContext<
  EditorSettingsContextType | undefined
>(undefined);

export const useEditorSettings = () => {
  const context = useContext(EditorSettingsContext);

  if (context === undefined) {
    throw new Error(
      'useEditorSettings must be used within an EditorSettingsProvider'
    );
  }

  return context;
};
