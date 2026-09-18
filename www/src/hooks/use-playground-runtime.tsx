import { AnalysisClient } from '@/lib/analysis/client';
import { useEffect, useState } from 'react';
import { Language, Parser } from 'web-tree-sitter';
import runtime from 'web-tree-sitter/web-tree-sitter.wasm?url';

export interface PlaygroundRuntime {
  analysis: AnalysisClient;
  language: Language;
}

type PlaygroundState =
  | { status: 'loading' }
  | { status: 'error'; error: string }
  | { status: 'ready'; runtime: PlaygroundRuntime };

let initialization: Promise<Language> | undefined;

export function usePlaygroundRuntime(): PlaygroundState {
  const [state, setState] = useState<PlaygroundState>({ status: 'loading' });

  useEffect(() => {
    let active = true;
    let analysis: AnalysisClient | undefined;

    const initialize = async () => {
      setState({ status: 'loading' });

      try {
        analysis = new AnalysisClient();

        initialization ??= Parser.init({ locateFile: () => runtime })
          .then(() =>
            Language.load(`${import.meta.env.BASE_URL}tree-sitter-just.wasm`)
          )
          .catch((error) => {
            initialization = undefined;
            throw error;
          });

        const [language] = await Promise.all([
          initialization,
          analysis.initialize(),
        ]);

        if (!active) return;

        setState({ status: 'ready', runtime: { analysis, language } });
      } catch (error) {
        analysis?.dispose();

        if (active) {
          setState({
            status: 'error',
            error: `Failed to initialize playground: ${error instanceof Error ? error.message : String(error)}`,
          });
        }
      }
    };

    initialize();

    return () => {
      active = false;
      analysis?.dispose();
    };
  }, []);

  return state;
}
