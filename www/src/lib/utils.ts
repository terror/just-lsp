import { EditorState } from '@codemirror/state';
import { type ClassValue, clsx } from 'clsx';
import _ from 'lodash';
import { twMerge } from 'tailwind-merge';
import { Language, Parser, Tree } from 'web-tree-sitter';

import { Position, SyntaxNode, TreeNode } from './types';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const createNodePositionMap = (
  formattedTree: TreeNode[],
  doc: EditorState['doc']
): Map<SyntaxNode, Position> => {
  return _.reduce(
    formattedTree,
    (nodeMap, item) => {
      const from = positionToOffset(item.node.startPosition, doc);
      const to = positionToOffset(item.node.endPosition, doc);

      if (from !== null && to !== null) {
        nodeMap.set(item.node, { start: from, end: to });
      }

      return nodeMap;
    },
    new Map<SyntaxNode, Position>()
  );
};

export const createNodeToParentMap = (
  formattedTree: TreeNode[]
): Map<SyntaxNode, SyntaxNode> => {
  const nodeToParentMap = new Map<SyntaxNode, SyntaxNode>();

  for (let i = 1; i < formattedTree.length; i++) {
    const currentNode = formattedTree[i].node;

    const currentLevel = formattedTree[i].level;

    const parentEntry = _.findLast(
      formattedTree.slice(0, i),
      (item) => item.level < currentLevel
    );

    if (parentEntry) {
      nodeToParentMap.set(currentNode, parentEntry.node);
    }
  }

  return nodeToParentMap;
};

export const formatTree = (node: SyntaxNode, indent = 0): TreeNode[] => {
  const nodeInfo = {
    text: `${node.type} [${node.startPosition.row},${node.startPosition.column}]`,
    node,
    level: indent,
    type: node.type,
  };

  if (!node.childCount) {
    return [nodeInfo];
  }

  return [
    nodeInfo,
    ..._.flatMap(node.children, (child: SyntaxNode) =>
      formatTree(child, indent + 1)
    ),
  ];
};

export const getVisibleNodes = (
  formattedTree: TreeNode[],
  expandedNodes: Set<SyntaxNode>
): TreeNode[] => {
  if (_.isEmpty(formattedTree)) return [];

  const nodeToParentMap = createNodeToParentMap(formattedTree);

  return _.filter(formattedTree, (item) => {
    if (item.level === 0) return true;

    return _.every(getAncestors(item.node, nodeToParentMap), (parent) =>
      expandedNodes.has(parent)
    );
  });
};

export const getAncestors = (
  node: SyntaxNode,
  nodeToParentMap: Map<SyntaxNode, SyntaxNode>
): SyntaxNode[] => {
  const ancestors: SyntaxNode[] = [];

  let currentNode = node;

  while (nodeToParentMap.has(currentNode)) {
    const parent = nodeToParentMap.get(currentNode)!;
    ancestors.push(parent);
    currentNode = parent;
  }

  return ancestors;
};

export const parse = ({
  parser,
  language,
  code,
}: {
  parser: Parser;
  language: Language;
  code: string;
}): Tree | null => {
  parser.setLanguage(language);
  return parser.parse(code);
};

export const positionToOffset = (
  position: { row: number; column: number },
  doc: EditorState['doc']
): number | null => {
  const lineNumber = position.row + 1;

  if (lineNumber > doc.lines) {
    return null;
  }

  return (
    doc.line(lineNumber).from +
    Math.min(position.column, doc.line(lineNumber).length)
  );
};

export const processTree = (
  tree: Tree,
  doc: EditorState['doc']
): {
  formattedTree: TreeNode[];
  nodePositionMap: Map<SyntaxNode, Position>;
  allNodes: Set<SyntaxNode>;
} => {
  const formattedTree = formatTree(tree.rootNode as unknown as SyntaxNode);

  const allNodes = new Set<SyntaxNode>(
    _.map(formattedTree, (item) => item.node)
  );

  return {
    formattedTree,
    nodePositionMap: createNodePositionMap(formattedTree, doc),
    allNodes,
  };
};
