import { memo } from 'react';
import type { Language } from 'web-tree-sitter';

import { useSyntaxTree } from '../hooks/use-syntax-tree';
import { TreeNode } from './tree-node';

interface TreePaneProps {
  language: Language;
  code: string;
  onHighlightChange: (range: { from: number; to: number } | undefined) => void;
}

export const TreePane = memo(function TreePane({
  language,
  code,
  onHighlightChange,
}: TreePaneProps) {
  const { root, collapsedNodes, toggleExpand } = useSyntaxTree({
    language,
    code,
  });

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
});
