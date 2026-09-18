import { AnalysisClient } from '@/lib/analysis/client';
import { useEffect, useState } from 'react';
import { Language, Parser } from 'web-tree-sitter';
import runtime from 'web-tree-sitter/web-tree-sitter.wasm?url';

export interface PlaygroundRuntime {
  analysis: AnalysisClient;
  language: Language;
  parser: Parser;
}

type PlaygroundState =
  | { status: 'loading' }
  | { status: 'error'; error: string }
  | { status: 'ready'; runtime: PlaygroundRuntime };

let initialization: Promise<Language> | undefined;

function initializeLanguage(): Promise<Language> {
  initialization ??= Parser.init({ locateFile: () => runtime })
    .then(() =>
      Language.load(`${import.meta.env.BASE_URL}tree-sitter-just.wasm`)
    )
    .catch((error) => {
      initialization = undefined;
      throw error;
    });

  return initialization;
}

export function usePlaygroundRuntime(): PlaygroundState {
  const [state, setState] = useState<PlaygroundState>({ status: 'loading' });

  useEffect(() => {
    let active = true;
    let analysis: AnalysisClient | undefined;
    let parser: Parser | undefined;

    const initialize = async () => {
      setState({ status: 'loading' });

      try {
        analysis = new AnalysisClient();

        const [language] = await Promise.all([
          initializeLanguage(),
          analysis.initialize(),
        ]);

        if (!active) return;

        parser = new Parser();
        parser.setLanguage(language);

        setState({ status: 'ready', runtime: { analysis, language, parser } });
      } catch (error) {
        analysis?.dispose();

        parser?.delete();
        parser = undefined;

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
      parser?.delete();
    };
  }, []);

  return state;
}
