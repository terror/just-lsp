import { afterEach, describe, expect, it } from 'bun:test';
import { renderToStaticMarkup } from 'react-dom/server';

import { usePersistedState } from './use-persisted-state';

const windowDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'window');

afterEach(() => {
  if (windowDescriptor) {
    Object.defineProperty(globalThis, 'window', windowDescriptor);
  } else {
    Reflect.deleteProperty(globalThis, 'window');
  }
});

function Value() {
  const [value] = usePersistedState('foo', 'bar');
  return value;
}

describe('usePersistedState', () => {
  it('uses the fallback without a window', () => {
    Reflect.deleteProperty(globalThis, 'window');

    expect(renderToStaticMarkup(<Value />)).toBe('bar');
  });

  it('uses the fallback only when no value is saved', () => {
    for (const [stored, expected] of [
      [null, 'bar'],
      ['', ''],
      ['foo', 'foo'],
    ] as const) {
      Object.defineProperty(globalThis, 'window', {
        configurable: true,
        value: { localStorage: { getItem: () => stored } },
      });

      expect(renderToStaticMarkup(<Value />)).toBe(expected);
    }
  });
});
