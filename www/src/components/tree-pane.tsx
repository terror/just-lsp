import type { Node } from 'web-tree-sitter';

import { TreeNode } from './tree-node';

interface TreePaneProps {
  root: Node | undefined;
  collapsedNodes: Set<Node>;
  toggleExpand: (node: Node) => void;
  onHighlightChange: (range: { from: number; to: number } | undefined) => void;
}

export const TreePane = ({
  root,
  collapsedNodes,
  toggleExpand,
  onHighlightChange,
}: TreePaneProps) => {
  return (
    <div className='h-full overflow-auto'>
      {root ? (
        <div className='p-2'>
          <TreeNode
            node={root}
            level={0}
            collapsedNodes={collapsedNodes}
            toggleExpand={toggleExpand}
            onHighlightChange={onHighlightChange}
          />
        </div>
      ) : (
        <p className='text-muted-foreground p-4 text-center'>
          No parsed tree available
        </p>
      )}
    </div>
  );
};
