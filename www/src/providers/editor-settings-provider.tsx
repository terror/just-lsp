import { usePersistedState } from '@/hooks/use-persisted-state';
import { ReactNode, createContext, useContext } from 'react';

export interface EditorSettings {
  fontSize: number;
  keybindings: 'default' | 'vim';
  lineNumbers: boolean;
  lineWrapping: boolean;
  tabSize: number;
}

const defaultSettings: EditorSettings = {
  fontSize: 14,
  keybindings: 'default',
  lineNumbers: true,
  lineWrapping: true,
  tabSize: 2,
};

type EditorSettingsContextType = {
  settings: EditorSettings;
  updateSettings: (settings: Partial<EditorSettings>) => void;
};

const EditorSettingsContext = createContext<
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

export const EditorSettingsProvider = ({
  children,
}: {
  children: ReactNode;
}) => {
  const [settings, setSettings] = usePersistedState<EditorSettings>(
    'editor-settings',
    defaultSettings
  );

  const updateSettings = (newSettings: Partial<EditorSettings>) => {
    setSettings((prevSettings) => ({ ...prevSettings, ...newSettings }));
  };

  return (
    <EditorSettingsContext.Provider value={{ settings, updateSettings }}>
      {children}
    </EditorSettingsContext.Provider>
  );
};
