import type { Diagnostic } from '@codemirror/lint';

export interface AnalyzerRequest {
  id: number;
  source: string;
}

export type AnalyzerResponse =
  { id: number; diagnostics: Diagnostic[] } | { id: number; error: string };

export class Analyzer {
  private id = 0;
  private error: Error | undefined;
  private pending = new Map<
    number,
    {
      resolve: (diagnostics: Diagnostic[]) => void;
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
          pending?.resolve(event.data.diagnostics);
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
    if (this.error) return Promise.reject(this.error);

    const id = this.id++;

    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.worker.postMessage({ id, source } satisfies AnalyzerRequest);
    });
  }
}

let analyzer: Analyzer | undefined;

export async function analyzeSource(source: string): Promise<Diagnostic[]> {
  analyzer ??= new Analyzer(
    new Worker(new URL('./analyzer-worker.ts', import.meta.url), {
      type: 'module',
    })
  );

  return analyzer.analyze(source);
}
