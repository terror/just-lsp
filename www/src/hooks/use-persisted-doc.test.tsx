import { afterEach, describe, expect, it } from 'bun:test';
import { renderToStaticMarkup } from 'react-dom/server';

import { usePersistedDoc } from './use-persisted-doc';

const windowDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'window');

afterEach(() => {
  if (windowDescriptor) {
    Object.defineProperty(globalThis, 'window', windowDescriptor);
  } else {
    Reflect.deleteProperty(globalThis, 'window');
  }
});

function Document() {
  const [doc] = usePersistedDoc('foo', 'bar');

  return doc;
}

describe('usePersistedDoc', () => {
  it('uses the fallback without a window', () => {
    Reflect.deleteProperty(globalThis, 'window');

    expect(renderToStaticMarkup(<Document />)).toBe('bar');
  });

  it('uses the fallback only when no document is saved', () => {
    for (const [stored, expected] of [
      [null, 'bar'],
      ['', ''],
      ['foo', 'foo'],
    ] as const) {
      Object.defineProperty(globalThis, 'window', {
        configurable: true,
        value: { localStorage: { getItem: () => stored } },
      });

      expect(renderToStaticMarkup(<Document />)).toBe(expected);
    }
  });
});
