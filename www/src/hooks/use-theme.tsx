import { readStorage, writeStorage } from '@/lib/storage';
import { useLayoutEffect, useState } from 'react';

const THEME_STORAGE_KEY = 'just-lsp:theme';

export function useTheme() {
  const [darkMode, setDarkMode] = useState(() => {
    const savedTheme = readStorage(THEME_STORAGE_KEY);

    if (savedTheme === 'dark' || savedTheme === 'light') {
      return savedTheme === 'dark';
    }

    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  });

  useLayoutEffect(() => {
    document.documentElement.classList.toggle('dark', darkMode);
    writeStorage(THEME_STORAGE_KEY, darkMode ? 'dark' : 'light');
  }, [darkMode]);

  return {
    darkMode,
    toggleTheme: () => setDarkMode((enabled) => !enabled),
  };
}
