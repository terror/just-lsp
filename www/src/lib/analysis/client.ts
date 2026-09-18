import type { Diagnostic, Hover } from '../types';
import type {
  AnalysisOperation,
  AnalysisRequest,
  AnalysisResponse,
  AnalysisResult,
} from './protocol';

export class AnalysisClient {
  private id = 0;
  private error: Error | undefined;

  private pending = new Map<
    number,
    {
      resolve: (response: AnalysisResult) => void;
      reject: (error: Error) => void;
    }
  >();

  private worker = new Worker(new URL('./worker.ts', import.meta.url), {
    type: 'module',
  });

  constructor() {
    this.worker.addEventListener(
      'message',
      (event: MessageEvent<AnalysisResponse>) => {
        const pending = this.pending.get(event.data.id);

        this.pending.delete(event.data.id);

        if ('error' in event.data) {
          pending?.reject(new Error(event.data.error));
        } else {
          pending?.resolve(event.data);
        }
      }
    );

    this.worker.addEventListener('error', (event) => {
      this.stop(new Error(event.message));
    });

    this.worker.addEventListener('messageerror', () => {
      this.stop(new Error('Failed to deserialize analysis response'));
    });
  }

  analyze(source: string): Promise<Diagnostic[]> {
    return this.request({ method: 'analyze', source });
  }

  dispose(): void {
    this.stop(new Error('Analysis client disposed'));
  }

  hover(
    source: string,
    line: number,
    character: number
  ): Promise<Hover | undefined> {
    return this.request({
      method: 'hover',
      source,
      position: { line, character },
    });
  }

  initialize(): Promise<void> {
    return this.request({ method: 'initialize' });
  }

  private request(
    request: Extract<AnalysisOperation, { method: 'initialize' }>
  ): Promise<undefined>;

  private request(
    request: Extract<AnalysisOperation, { method: 'analyze' }>
  ): Promise<Diagnostic[]>;

  private request(
    request: Extract<AnalysisOperation, { method: 'hover' }>
  ): Promise<Hover | undefined>;

  private request(
    request: AnalysisOperation
  ): Promise<AnalysisResult['result']> {
    if (this.error) return Promise.reject(this.error);

    const id = this.id++;

    return new Promise((resolve, reject) => {
      this.pending.set(id, {
        resolve: (response) => {
          if (response.method === request.method) {
            resolve(response.result);
          } else {
            reject(
              new Error(
                `Unexpected analysis response: expected ${request.method}, received ${response.method}`
              )
            );
          }
        },
        reject,
      });

      try {
        this.worker.postMessage({ id, ...request } satisfies AnalysisRequest);
      } catch (error) {
        this.pending.delete(id);
        reject(error);
      }
    });
  }

  private stop(error: Error): void {
    this.error ??= error;

    this.worker.terminate();

    for (const pending of this.pending.values()) {
      pending.reject(this.error);
    }

    this.pending.clear();
  }
}
