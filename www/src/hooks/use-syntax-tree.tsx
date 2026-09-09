import { useCallback, useMemo, useState } from 'react';
import type { Node, Parser } from 'web-tree-sitter';

interface UseSyntaxTreeOptions {
  parser: Parser | undefined;
  code: string;
}

interface UseSyntaxTree {
  root: Node | undefined;
  collapsedNodes: Set<Node>;
  toggleExpand: (node: Node) => void;
}

export function useSyntaxTree({
  parser,
  code,
}: UseSyntaxTreeOptions): UseSyntaxTree {
  const root = useMemo(() => parser?.parse(code)?.rootNode, [parser, code]);

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
