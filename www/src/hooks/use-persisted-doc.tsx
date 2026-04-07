import { useCallback, useEffect, useState } from 'react';

export function usePersistedDoc(
  key: string,
  fallback: string
): [string, (value: string) => void] {
  const [value, setValue] = useState<string>(() => {
    if (typeof window === 'undefined') return fallback;

    const stored = window.localStorage.getItem(key);

    return stored && stored.length > 0 ? stored : fallback;
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;

    window.localStorage.setItem(key, value);
  }, [key, value]);

  return [value, useCallback((next: string) => setValue(next), [])];
}
