import { analyzeSource } from '@/lib/analyzer';
import { useEffect, useState } from 'react';
import { Language, Parser } from 'web-tree-sitter';
import runtime from 'web-tree-sitter/web-tree-sitter.wasm?url';

export interface PlaygroundRuntime {
  language: Language;
  parser: Parser;
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
    let parser: Parser | undefined;

    const initialize = async () => {
      setState({ status: 'loading' });

      try {
        initialization ??= Promise.all([
          Parser.init({ locateFile: () => runtime }).then(() =>
            Language.load(`${import.meta.env.BASE_URL}tree-sitter-just.wasm`)
          ),
          analyzeSource(''),
        ]).then(([language]) => language);

        const language = await initialization;

        if (!active) return;

        parser = new Parser();
        parser.setLanguage(language);

        setState({ status: 'ready', runtime: { language, parser } });
      } catch (error) {
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
      parser?.delete();
    };
  }, []);

  return state;
}
