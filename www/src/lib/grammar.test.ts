import { describe, expect, it } from 'bun:test';
import { readFileSync } from 'fs';
import { Language, Parser, Query } from 'web-tree-sitter';

import highlightsQuerySource from '../../../queries/highlights.scm?raw';

describe('bundled tree-sitter-just wasm', () => {
  it('compiles the highlights query', async () => {
    await Parser.init({
      locateFile: () => 'node_modules/web-tree-sitter/tree-sitter.wasm',
    });

    const wasm = readFileSync('public/tree-sitter-just.wasm');
    const language = await Language.load(wasm);

    expect(() => new Query(language, highlightsQuerySource)).not.toThrow();
  });
});
