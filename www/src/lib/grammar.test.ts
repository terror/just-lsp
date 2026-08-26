import { describe, expect, it } from 'bun:test';
import { readFileSync } from 'fs';
import { Language, Parser, Query } from 'web-tree-sitter';

import highlightsQuerySource from '../../../queries/highlights.scm?raw';

describe('bundled tree-sitter-just wasm', () => {
  it('compiles the highlights query', async () => {
    const runtime = 'public/tree-sitter.wasm';

    expect(readFileSync(runtime)).toEqual(
      readFileSync('node_modules/web-tree-sitter/tree-sitter.wasm')
    );

    await Parser.init({
      locateFile: () => runtime,
    });

    const wasm = readFileSync('public/tree-sitter-just.wasm');
    const language = await Language.load(wasm);

    expect(() => new Query(language, highlightsQuerySource)).not.toThrow();
  });
});
