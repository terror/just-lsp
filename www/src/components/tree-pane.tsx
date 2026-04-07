import type { SyntaxNode } from '@/lib/types';
import { Text } from '@codemirror/state';
import { useMemo } from 'react';

import { TreeNode } from './tree-node';

interface TreePaneProps {
  root: SyntaxNode | undefined;
  code: string;
  expandedNodes: Set<SyntaxNode>;
  toggleExpand: (node: SyntaxNode) => void;
  onHighlightChange: (range: { from: number; to: number } | undefined) => void;
}

export const TreePane = ({
  root,
  code,
  expandedNodes,
  toggleExpand,
  onHighlightChange,
}: TreePaneProps) => {
  const doc = useMemo(() => Text.of(code.split('\n')), [code]);

  return (
    <div className='h-full overflow-auto'>
      {root ? (
        <div className='p-2'>
          <TreeNode
            node={root}
            level={0}
            doc={doc}
            expandedNodes={expandedNodes}
            toggleExpand={toggleExpand}
            onHighlightChange={onHighlightChange}
          />
        </div>
      ) : (
        <p className='p-4 text-center text-gray-500'>
          No parsed tree available
        </p>
      )}
    </div>
  );
};
