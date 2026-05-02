import {
  EditorSettings,
  EditorSettingsContext,
  defaultSettings,
} from '@/contexts/editor-settings-context';
import { usePersistedState } from '@/hooks/use-persisted-state';
import { ReactNode, useEffect } from 'react';

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

  useEffect(() => {
    document.documentElement.style.setProperty(
      '--editor-font-size',
      `${settings.fontSize}px`
    );
  }, [settings.fontSize]);

  return (
    <EditorSettingsContext.Provider value={{ settings, updateSettings }}>
      {children}
    </EditorSettingsContext.Provider>
  );
};
