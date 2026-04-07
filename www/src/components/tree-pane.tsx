import type {
  Position,
  SyntaxNode,
  TreeNode as TreeNodeType,
} from '@/lib/types';
import { getVisibleNodes } from '@/lib/utils';
import { useEffect, useMemo, useState } from 'react';

import { TreeNode } from './tree-node';

interface TreePaneProps {
  formattedTree: TreeNodeType[];
  nodePositionMap: Map<SyntaxNode, Position>;
  expandedNodes: Set<SyntaxNode>;
  expandNode: (node: SyntaxNode) => void;
  onHighlightChange: (range: { from: number; to: number } | undefined) => void;
}

export const TreePane = ({
  formattedTree,
  nodePositionMap,
  expandedNodes,
  expandNode,
  onHighlightChange,
}: TreePaneProps) => {
  const [hoveredNode, setHoveredNode] = useState<SyntaxNode | undefined>(
    undefined
  );

  useEffect(() => {
    if (!hoveredNode) {
      onHighlightChange(undefined);
      return;
    }

    const position = nodePositionMap.get(hoveredNode);

    if (!position) {
      onHighlightChange(undefined);
      return;
    }

    onHighlightChange({ from: position.start, to: position.end });
  }, [hoveredNode, nodePositionMap, onHighlightChange]);

  const visibleTree = useMemo(
    () => getVisibleNodes(formattedTree, expandedNodes),
    [formattedTree, expandedNodes]
  );

  return (
    <div className='h-full overflow-auto'>
      {visibleTree.length > 0 ? (
        <div className='p-2'>
          {visibleTree.map((item, index) => (
            <TreeNode
              key={index}
              item={item}
              hoveredNode={hoveredNode}
              setHoveredNode={setHoveredNode}
              expandedNodes={expandedNodes}
              expandNode={expandNode}
            />
          ))}
        </div>
      ) : (
        <p className='p-4 text-center text-gray-500'>
          No parsed tree available
        </p>
      )}
    </div>
  );
};
