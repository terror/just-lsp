import { useEffect, useState } from 'react';

const STACKED_LAYOUT_QUERY = '(max-width: 767px)';

export const useStackedLayout = () => {
  const [stackedLayout, setStackedLayout] = useState(
    () => window.matchMedia(STACKED_LAYOUT_QUERY).matches
  );

  useEffect(() => {
    const mediaQueryList = window.matchMedia(STACKED_LAYOUT_QUERY);

    setStackedLayout(mediaQueryList.matches);

    const handleChange = (event: MediaQueryListEvent) => {
      setStackedLayout(event.matches);
    };

    mediaQueryList.addEventListener('change', handleChange);

    return () => {
      mediaQueryList.removeEventListener('change', handleChange);
    };
  }, []);

  return stackedLayout;
};
