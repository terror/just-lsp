import { Text } from '@codemirror/state';

import type { AnalyzerRequest, AnalyzerResponse } from './analyzer';
import { analyzeDocument } from './diagnostics';
import init from './just-lsp-wasm/just_lsp_wasm';

let initialization: ReturnType<typeof init> | undefined;

self.addEventListener(
  'message',
  async (event: MessageEvent<AnalyzerRequest>) => {
    const { id, source } = event.data;

    try {
      await (initialization ??= init());

      self.postMessage({
        id,
        diagnostics: analyzeDocument(Text.of(source.split('\n'))),
      } satisfies AnalyzerResponse);
    } catch (error) {
      self.postMessage({
        id,
        error: error instanceof Error ? error.message : String(error),
      } satisfies AnalyzerResponse);
    }
  }
);
