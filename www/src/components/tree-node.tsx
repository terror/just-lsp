import { ChevronDown, ChevronRight } from 'lucide-react';
import type { Node } from 'web-tree-sitter';

interface TreeNodeProps {
  node: Node;
  level: number;
  collapsedNodes: Set<Node>;
  toggleExpand: (node: Node) => void;
  onHighlightChange: (range?: { from: number; to: number }) => void;
}

export const TreeNode: React.FC<TreeNodeProps> = ({
  node,
  level,
  collapsedNodes,
  toggleExpand,
  onHighlightChange,
}) => {
  const hasChildren = node.childCount > 0;
  const isExpanded = !collapsedNodes.has(node);

  return (
    <>
      <div
        className='tree-node hover:bg-accent flex cursor-pointer items-center py-1 font-mono text-sm whitespace-nowrap'
        style={{ paddingLeft: `${level * 16 + 4}px` }}
        onMouseEnter={() =>
          onHighlightChange({ from: node.startIndex, to: node.endIndex })
        }
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
        <span className='text-muted-foreground ml-2 text-xs'>
          [{node.startPosition.row}: {node.startPosition.column}] [
          {node.endPosition.row}: {node.endPosition.column}]
        </span>
      </div>
      {isExpanded &&
        hasChildren &&
        node.children.map((child) => (
          <TreeNode
            key={child.id}
            node={child}
            level={level + 1}
            collapsedNodes={collapsedNodes}
            toggleExpand={toggleExpand}
            onHighlightChange={onHighlightChange}
          />
        ))}
    </>
  );
};
