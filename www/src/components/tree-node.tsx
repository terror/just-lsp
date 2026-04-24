import type { SyntaxNode } from '@/lib/syntax-node';
import { positionToOffset } from '@/lib/utils';
import { Text } from '@codemirror/state';
import { ChevronDown, ChevronRight } from 'lucide-react';

interface TreeNodeProps {
  node: SyntaxNode;
  level: number;
  doc: Text;
  expandedNodes: Set<SyntaxNode>;
  toggleExpand: (node: SyntaxNode) => void;
  onHighlightChange: (range?: { from: number; to: number }) => void;
}

export const TreeNode: React.FC<TreeNodeProps> = ({
  node,
  level,
  doc,
  expandedNodes,
  toggleExpand,
  onHighlightChange,
}) => {
  const hasChildren = node.childCount > 0;
  const isExpanded = expandedNodes.has(node);

  const handleMouseEnter = () => {
    const from = positionToOffset(node.startPosition, doc);
    const to = positionToOffset(node.endPosition, doc);

    if (from !== null && to !== null) {
      onHighlightChange({ from, to });
    }
  };

  return (
    <>
      <div
        className='tree-node flex cursor-pointer items-center py-1 font-mono text-sm whitespace-nowrap hover:bg-blue-50'
        style={{ paddingLeft: `${level * 16 + 4}px` }}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={() => onHighlightChange(undefined)}
        onClick={() => hasChildren && toggleExpand(node)}
      >
        <span className='mr-1 flex w-4 justify-center'>
          {hasChildren ? (
            isExpanded ? (
              <ChevronDown size={14} />
            ) : (
              <ChevronRight size={14} />
            )
          ) : null}
        </span>
        <span>{node.type}</span>
        <span className='ml-2 text-xs text-gray-500'>
          [{node.startPosition.row}: {node.startPosition.column}] [
          {node.endPosition.row}: {node.endPosition.column}]
        </span>
      </div>
      {isExpanded &&
        hasChildren &&
        node.children.map((child, index) => (
          <TreeNode
            key={child.id ?? index}
            node={child}
            level={level + 1}
            doc={doc}
            expandedNodes={expandedNodes}
            toggleExpand={toggleExpand}
            onHighlightChange={onHighlightChange}
          />
        ))}
    </>
  );
};
