import {
  EditorSettings,
  FONT_SIZES,
  TAB_SIZES,
  defaultSettings,
} from '@/contexts/editor-settings-context';
import { readStorage, writeStorage } from '@/lib/storage';
import { useCallback, useEffect, useState } from 'react';
import { z } from 'zod';

const SETTINGS_STORAGE_KEY = 'editor-settings';

const settingsSchema = z
  .object({
    fontSize: z.literal(FONT_SIZES).catch(defaultSettings.fontSize),
    keybindings: z.enum(['default', 'vim']).catch(defaultSettings.keybindings),
    lineNumbers: z.boolean().catch(defaultSettings.lineNumbers),
    lineWrapping: z.boolean().catch(defaultSettings.lineWrapping),
    tabSize: z.literal(TAB_SIZES).catch(defaultSettings.tabSize),
  })
  .catch(defaultSettings);

export function usePersistedSettings() {
  const [settings, setSettings] = useState<EditorSettings>(() => {
    const saved = readStorage(SETTINGS_STORAGE_KEY);

    if (saved === null) return defaultSettings;

    try {
      return settingsSchema.parse(JSON.parse(saved));
    } catch (error) {
      console.warn(
        `Error reading ${SETTINGS_STORAGE_KEY} from localStorage:`,
        error
      );

      return defaultSettings;
    }
  });

  useEffect(() => {
    writeStorage(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
  }, [settings]);

  const updateSettings = useCallback((settings: Partial<EditorSettings>) => {
    setSettings((previous) => ({ ...previous, ...settings }));
  }, []);

  return { settings, updateSettings };
}
