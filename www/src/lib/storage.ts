export function readStorage(key: string): string | null {
  if (typeof window === 'undefined') return null;

  try {
    return window.localStorage.getItem(key);
  } catch (error) {
    console.warn(`Error reading ${key} from localStorage:`, error);
    return null;
  }
}

export function writeStorage(key: string, value: string): void {
  if (typeof window === 'undefined') return;

  try {
    window.localStorage.setItem(key, value);
  } catch (error) {
    console.warn(`Error saving ${key} to localStorage:`, error);
  }
}
