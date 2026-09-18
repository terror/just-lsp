import { Text } from '@codemirror/state';

import type { AnalyzerRequest, AnalyzerResponse } from './analyzer';
import { toEditorDiagnostics } from './diagnostics';
import init, { analyze, hover } from './just-lsp-wasm/just_lsp_wasm';
import type { Diagnostic, Hover } from './types';

let initialization: ReturnType<typeof init> | undefined;

self.addEventListener(
  'message',
  async (event: MessageEvent<AnalyzerRequest>) => {
    const { id, position, source } = event.data;

    try {
      await (initialization ??= init());

      self.postMessage({
        id,
        result: position
          ? (hover(source, position.line, position.character) as
              Hover | undefined)
          : toEditorDiagnostics(
              Text.of(source.split('\n')),
              analyze(source) as Diagnostic[]
            ),
      } satisfies AnalyzerResponse);
    } catch (error) {
      self.postMessage({
        id,
        error: error instanceof Error ? error.message : String(error),
      } satisfies AnalyzerResponse);
    }
  }
);
