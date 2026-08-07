import type { SyntaxNode } from '@/lib/syntax-node';
import { parse } from '@/lib/utils';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { Parser, Language as TSLanguage, Tree } from 'web-tree-sitter';

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
  const [tree, setTree] = useState<Tree | undefined>(undefined);

  useEffect(() => {
    if (!parser || !language) {
      setTree(undefined);
      return;
    }

    const tree = parse({ parser, language, code });

    setTree(tree ?? undefined);

    return () => tree?.delete();
  }, [parser, language, code]);

  const root = useMemo(
    () => (tree?.rootNode as unknown as SyntaxNode) ?? undefined,
    [tree]
  );

  const [expandedNodes, setExpandedNodes] = useState<Set<SyntaxNode>>(
    () => new Set()
  );

  useEffect(() => {
    if (!root) {
      setExpandedNodes(new Set());
      return;
    }

    const all = new Set<SyntaxNode>();

    const walk = (node: SyntaxNode) => {
      all.add(node);
      node.children.forEach(walk);
    };

    walk(root);

    setExpandedNodes(all);
  }, [root]);

  const toggleExpand = useCallback((node: SyntaxNode) => {
    setExpandedNodes((prev) => {
      const next = new Set(prev);

      if (next.has(node)) {
        next.delete(node);
      } else {
        next.add(node);
      }

      return next;
    });
  }, []);

  return { root, expandedNodes, toggleExpand };
}
