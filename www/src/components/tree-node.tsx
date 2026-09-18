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
      <button
        type='button'
        aria-expanded={hasChildren ? isExpanded : undefined}
        className='tree-node hover:bg-accent focus-visible:outline-ring flex w-full cursor-pointer items-center py-1 text-left font-mono text-sm whitespace-nowrap focus-visible:outline-2 focus-visible:-outline-offset-2'
        style={{ paddingLeft: `${level * 16 + 4}px` }}
        onMouseEnter={() =>
          onHighlightChange({ from: node.startIndex, to: node.endIndex })
        }
        onMouseLeave={() => onHighlightChange(undefined)}
        onFocus={() =>
          onHighlightChange({ from: node.startIndex, to: node.endIndex })
        }
        onBlur={() => onHighlightChange(undefined)}
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
      </button>
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
