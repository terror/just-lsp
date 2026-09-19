import { memo } from 'react';
import type { Language } from 'web-tree-sitter';

import { useSyntaxTree } from '../hooks/use-syntax-tree';
import { PlaygroundInfoDialog } from './playground-info-dialog';
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
    <div className='flex h-full min-h-0 flex-col overflow-hidden'>
      <div className='bg-muted/50 flex shrink-0 items-center justify-end gap-1 border-b px-2 py-1'>
        <PlaygroundInfoDialog />
      </div>
      <div className='min-h-0 flex-1 overflow-auto'>
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
    </div>
  );
});
