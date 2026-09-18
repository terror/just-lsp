import type { Diagnostic } from '@codemirror/lint';

import type { Hover } from './types';

export interface AnalyzerRequest {
  id: number;
  position?: { line: number; character: number };
  source: string;
}

export type AnalyzerResponse =
  | { id: number; result: Diagnostic[] | Hover | undefined }
  | { id: number; error: string };

export class Analyzer {
  private id = 0;
  private error: Error | undefined;
  private pending = new Map<
    number,
    {
      resolve: (result: Diagnostic[] | Hover | undefined) => void;
      reject: (error: Error) => void;
    }
  >();

  constructor(private worker: Worker) {
    worker.addEventListener(
      'message',
      (event: MessageEvent<AnalyzerResponse>) => {
        const pending = this.pending.get(event.data.id);
        this.pending.delete(event.data.id);

        if ('error' in event.data) {
          pending?.reject(new Error(event.data.error));
        } else {
          pending?.resolve(event.data.result);
        }
      }
    );

    worker.addEventListener('error', (event) => {
      this.error = new Error(event.message);

      for (const pending of this.pending.values()) {
        pending.reject(this.error);
      }

      this.pending.clear();
    });
  }

  analyze(source: string): Promise<Diagnostic[]> {
    return this.request({ source });
  }

  hover(
    source: string,
    line: number,
    character: number
  ): Promise<Hover | undefined> {
    return this.request({ source, position: { line, character } });
  }

  private request<T extends Diagnostic[] | Hover | undefined>(
    request: Omit<AnalyzerRequest, 'id'>
  ): Promise<T> {
    if (this.error) return Promise.reject(this.error);

    const id = this.id++;

    return new Promise((resolve, reject) => {
      this.pending.set(id, {
        resolve: (result) => resolve(result as T),
        reject,
      });
      this.worker.postMessage({ id, ...request } satisfies AnalyzerRequest);
    });
  }
}

let analyzer: Analyzer | undefined;

function getAnalyzer(): Analyzer {
  analyzer ??= new Analyzer(
    new Worker(new URL('./analyzer-worker.ts', import.meta.url), {
      type: 'module',
    })
  );

  return analyzer;
}

export async function analyzeSource(source: string): Promise<Diagnostic[]> {
  return getAnalyzer().analyze(source);
}

export async function hoverSource(
  source: string,
  line: number,
  character: number
): Promise<Hover | undefined> {
  return getAnalyzer().hover(source, line, character);
}
