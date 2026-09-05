import type { SyntaxNode } from '@/lib/syntax-node';
import { parse } from '@/lib/utils';
import { useCallback, useMemo, useState } from 'react';
import { Parser, Language as TSLanguage } from 'web-tree-sitter';

interface UseSyntaxTreeOptions {
  parser: Parser | undefined;
  language: TSLanguage | undefined;
  code: string;
}

interface UseSyntaxTree {
  root: SyntaxNode | undefined;
  expandedNodes: Set<SyntaxNode>;
  toggleExpand: (node: SyntaxNode) => void;
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

    return (tree?.rootNode as unknown as SyntaxNode) ?? undefined;
  }, [parser, language, code]);

  const [collapsed, setCollapsed] = useState<{
    root: SyntaxNode | undefined;
    nodes: Set<SyntaxNode>;
  }>();

  const expandedNodes = useMemo(() => {
    const all = new Set<SyntaxNode>();

    const walk = (node: SyntaxNode) => {
      if (collapsed?.root !== root || !collapsed?.nodes.has(node)) {
        all.add(node);
      }
      node.children.forEach(walk);
    };

    if (root) {
      walk(root);
    }

    return all;
  }, [root, collapsed]);

  const toggleExpand = useCallback(
    (node: SyntaxNode) => {
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

  return { root, expandedNodes, toggleExpand };
}
