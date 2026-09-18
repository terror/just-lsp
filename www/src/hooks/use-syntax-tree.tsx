import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from 'react';
import { type Language, type Node, Parser, type Tree } from 'web-tree-sitter';

interface UseSyntaxTreeOptions {
  language: Language;
  code: string;
}

interface ParsedTree {
  tree: Tree | null;
}

interface UseSyntaxTree {
  root: Node | undefined;
  collapsedNodes: Set<Node>;
  toggleExpand: (node: Node) => void;
}

export function useSyntaxTree({
  language,
  code,
}: UseSyntaxTreeOptions): UseSyntaxTree {
  const parser = useRef<Parser | null>(null);

  const trees = useRef(new Set<ParsedTree>());

  const [state, setState] = useState<{
    parsed: ParsedTree;
    root: Node | undefined;
    collapsedNodes: Set<Node>;
  }>();

  const parsed = state?.parsed;

  useEffect(() => {
    const allocated = trees.current;

    return () => {
      for (const { tree } of allocated) {
        tree?.delete();
      }

      allocated.clear();
    };
  }, []);

  useLayoutEffect(() => {
    const instance = new Parser();

    try {
      instance.setLanguage(language);
    } catch (error) {
      instance.delete();
      throw error;
    }

    parser.current = instance;

    return () => {
      parser.current = null;
      instance.delete();
    };
  }, [language]);

  useLayoutEffect(() => {
    const parsed = { tree: parser.current?.parse(code) ?? null };

    trees.current.add(parsed);

    setState({
      parsed,
      root: parsed.tree?.rootNode,
      collapsedNodes: new Set<Node>(),
    });
  }, [language, code]);

  useEffect(() => {
    if (!parsed || !trees.current.has(parsed)) return;

    for (const previous of trees.current) {
      if (previous === parsed) break;

      previous.tree?.delete();
      trees.current.delete(previous);
    }
  }, [parsed]);

  const toggleExpand = useCallback(
    (node: Node) => {
      setState((prev) => {
        if (!prev || prev.parsed !== parsed) return prev;

        const nodes = new Set(prev.collapsedNodes);

        if (nodes.has(node)) {
          nodes.delete(node);
        } else {
          nodes.add(node);
        }

        return { ...prev, collapsedNodes: nodes };
      });
    },
    [parsed]
  );

  return {
    root: state?.root,
    collapsedNodes: state?.collapsedNodes ?? new Set<Node>(),
    toggleExpand,
  };
}
