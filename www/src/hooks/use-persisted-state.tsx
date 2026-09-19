import { readStorage, writeStorage } from '@/lib/storage';
import { useEffect, useState } from 'react';

export function usePersistedState(
  key: string,
  fallback: string
): [string, (value: string) => void] {
  const [value, setValue] = useState(() => readStorage(key) ?? fallback);

  useEffect(() => {
    writeStorage(key, value);
  }, [key, value]);

  return [value, setValue];
}
