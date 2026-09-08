import { parse } from '@/lib/utils';
import { useCallback, useMemo, useState } from 'react';
import type { Language, Node, Parser } from 'web-tree-sitter';

interface UseSyntaxTreeOptions {
  parser: Parser | undefined;
  language: Language | undefined;
  code: string;
}

interface UseSyntaxTree {
  root: Node | undefined;
  collapsedNodes: Set<Node>;
  toggleExpand: (node: Node) => void;
}

export function useSyntaxTree({
  parser,
  language,
  code,
}: UseSyntaxTreeOptions): UseSyntaxTree {
  const root = useMemo(() => {
    if (!parser || !language) {
      return undefined;
    }

    const tree = parse({ parser, language, code });

    return tree?.rootNode ?? undefined;
  }, [parser, language, code]);

  const [collapsed, setCollapsed] = useState<{
    root: Node | undefined;
    nodes: Set<Node>;
  }>();

  const collapsedNodes =
    collapsed && collapsed.root === root ? collapsed.nodes : new Set<Node>();

  const toggleExpand = useCallback(
    (node: Node) => {
      setCollapsed((prev) => {
        const nodes = new Set(prev?.root === root ? prev?.nodes : []);

        if (nodes.has(node)) {
          nodes.delete(node);
        } else {
          nodes.add(node);
        }

        return { root, nodes };
      });
    },
    [root]
  );

  return { root, collapsedNodes, toggleExpand };
}
