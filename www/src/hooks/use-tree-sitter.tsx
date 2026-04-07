import { useEffect, useState } from 'react';
import { Parser, Language as TSLanguage } from 'web-tree-sitter';

interface UseTreeSitter {
  parser: Parser | undefined;
  language: TSLanguage | undefined;
  loading: boolean;
  error: string | undefined;
}

export function useTreeSitter(): UseTreeSitter {
  const [error, setError] = useState<string | undefined>(undefined);
  const [language, setLanguage] = useState<TSLanguage | undefined>(undefined);
  const [loading, setLoading] = useState<boolean>(true);
  const [parser, setParser] = useState<Parser | undefined>(undefined);

  useEffect(() => {
    let parserInstance: Parser | undefined;

    const initialize = async () => {
      try {
        setLoading(true);

        await Parser.init({
          locateFile(scriptName: string, _scriptDirectory: string) {
            return scriptName;
          },
        });

        parserInstance = new Parser();

        const loadedLanguage = await TSLanguage.load('tree-sitter-just.wasm');

        setParser(parserInstance);
        setLanguage(loadedLanguage);
      } catch (err) {
        setError(
          `Failed to initialize parser: ${err instanceof Error ? err.message : String(err)}`
        );
      } finally {
        setLoading(false);
      }
    };

    initialize();

    return () => {
      if (parserInstance) {
        parserInstance.delete();
      }
    };
  }, []);

  return { parser, language, loading, error };
}
