import init, { analyze, hover } from '../just-lsp-wasm/just_lsp_wasm';
import type { Diagnostic, Hover } from '../types';
import type { AnalysisRequest, AnalysisResponse } from './protocol';

let initialization: ReturnType<typeof init> | undefined;

function execute(request: AnalysisRequest): AnalysisResponse {
  const { id } = request;

  switch (request.method) {
    case 'initialize':
      return { id, method: request.method, result: undefined };
    case 'analyze':
      return {
        id,
        method: request.method,
        result: analyze(request.source) as Diagnostic[],
      };
    case 'hover':
      return {
        id,
        method: request.method,
        result: hover(
          request.source,
          request.position.line,
          request.position.character
        ) as Hover | undefined,
      };
  }
}

self.addEventListener(
  'message',
  async (event: MessageEvent<AnalysisRequest>) => {
    try {
      await (initialization ??= init().catch((error) => {
        initialization = undefined;
        throw error;
      }));

      self.postMessage(execute(event.data));
    } catch (error) {
      self.postMessage({
        id: event.data.id,
        error: error instanceof Error ? error.message : String(error),
      } satisfies AnalysisResponse);
    }
  }
);
