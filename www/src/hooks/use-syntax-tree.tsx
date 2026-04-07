import type { Position, SyntaxNode, TreeNode } from '@/lib/types';
import { parse, processTree } from '@/lib/utils';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { Parser, Language as TSLanguage } from 'web-tree-sitter';

interface UseSyntaxTreeOptions {
  parser: Parser | undefined;
  language: TSLanguage | undefined;
  code: string;
}

interface UseSyntaxTree {
  formattedTree: TreeNode[];
  nodePositionMap: Map<SyntaxNode, Position>;
  expandedNodes: Set<SyntaxNode>;
  expandNode: (node: SyntaxNode) => void;
}

export function useSyntaxTree({
  parser,
  language,
  code,
}: UseSyntaxTreeOptions): UseSyntaxTree {
  const { formattedTree, nodePositionMap, allNodes } = useMemo(() => {
    if (!parser || !language) {
      return {
        formattedTree: [] as TreeNode[],
        nodePositionMap: new Map<SyntaxNode, Position>(),
        allNodes: new Set<SyntaxNode>(),
      };
    }

    const tree = parse({ parser, language, code });

    if (!tree) {
      return {
        formattedTree: [] as TreeNode[],
        nodePositionMap: new Map<SyntaxNode, Position>(),
        allNodes: new Set<SyntaxNode>(),
      };
    }

    return processTree(tree, code);
  }, [parser, language, code]);

  const [expandedNodes, setExpandedNodes] = useState<Set<SyntaxNode>>(
    () => new Set()
  );

  useEffect(() => {
    setExpandedNodes(allNodes);
  }, [allNodes]);

  const expandNode = useCallback((node: SyntaxNode) => {
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

  return { formattedTree, nodePositionMap, expandedNodes, expandNode };
}
