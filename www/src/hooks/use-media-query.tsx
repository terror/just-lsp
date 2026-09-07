import { useCallback, useMemo, useSyncExternalStore } from 'react';

export const useMediaQuery = (query: string) => {
  const mediaQueryList = useMemo(() => window.matchMedia(query), [query]);
  const subscribe = useCallback(
    (onChange: () => void) => {
      mediaQueryList.addEventListener('change', onChange);

      return () => mediaQueryList.removeEventListener('change', onChange);
    },
    [mediaQueryList]
  );

  return useSyncExternalStore(subscribe, () => mediaQueryList.matches);
};
