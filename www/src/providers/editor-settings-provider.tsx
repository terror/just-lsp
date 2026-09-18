import { EditorSettingsContext } from '@/contexts/editor-settings-context';
import { usePersistedSettings } from '@/hooks/use-persisted-settings';
import { ReactNode, useEffect } from 'react';

export const EditorSettingsProvider = ({
  children,
}: {
  children: ReactNode;
}) => {
  const { settings, updateSettings } = usePersistedSettings();

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
