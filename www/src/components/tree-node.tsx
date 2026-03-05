import type { SyntaxNode, TreeNode as TreeNodeType } from '@/lib/types';
import { ChevronDown, ChevronRight } from 'lucide-react';

interface TreeNodeProps {
  expandNode: (node: SyntaxNode) => void;
  expandedNodes: Set<SyntaxNode>;
  hoveredNode?: SyntaxNode;
  item: TreeNodeType;
  setHoveredNode: (node?: SyntaxNode) => void;
}

export const TreeNode: React.FC<TreeNodeProps> = ({
  expandNode,
  expandedNodes,
  hoveredNode,
  item,
  setHoveredNode,
}) => {
  const hasChildren = item.node.childCount > 0;
  const isExpanded = expandedNodes.has(item.node);
  const isHovered = item.node === hoveredNode;

  const style = {
    paddingLeft: `${item.level * 16 + 4}px`,
    backgroundColor: isHovered ? 'rgba(59, 130, 246, 0.1)' : 'transparent',
    borderRadius: '2px',
  };

  return (
    <div
      className='tree-node flex cursor-pointer items-center py-1 font-mono text-sm whitespace-nowrap transition-colors hover:bg-blue-50'
      style={style}
      onMouseEnter={() => setHoveredNode(item.node)}
      onMouseLeave={() => setHoveredNode()}
      onClick={() => hasChildren && expandNode(item.node)}
    >
      <span className='mr-1 flex w-4 justify-center'>
        {hasChildren ? (
          isExpanded ? (
            <ChevronDown size={14} />
          ) : (
            <ChevronRight size={14} />
          )
        ) : (
          <span className='w-4'></span>
        )}
      </span>
      <span>{item.node.type}</span>
      <span className='ml-2 text-xs text-gray-500'>
        [{item.node.startPosition.row}: {item.node.startPosition.column}] [
        {item.node.endPosition.row}: {item.node.endPosition.column}]
      </span>
    </div>
  );
};
