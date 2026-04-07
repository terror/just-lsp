import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable';
import type {
  Position,
  SyntaxNode,
  TreeNode as TreeNodeType,
} from '@/lib/types';
import { getVisibleNodes, parse, processTree } from '@/lib/utils';
import { Bot, Loader2 } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';

import defaultJustfile from '../../justfile?raw';
import { EditorPane } from './components/editor-pane';
import { TreeNode } from './components/tree-node';
import { useEditorExtensions } from './hooks/use-editor-extensions';
import { usePersistedDoc } from './hooks/use-persisted-doc';
import { useTreeSitter } from './hooks/use-tree-sitter';

const EDITOR_STORAGE_KEY = 'just-lsp:editor-code';
const PANEL_LAYOUT_STORAGE_KEY = 'just-lsp:panel-layout';

const App = () => {
  const { parser, language: justLanguage, loading, error } = useTreeSitter();

  const [doc, setDoc] = usePersistedDoc(
    EDITOR_STORAGE_KEY,
    defaultJustfile.trim()
  );

  const [hoveredNode, setHoveredNode] = useState<SyntaxNode | undefined>(
    undefined
  );

  const [expandedNodes, setExpandedNodes] = useState<Set<SyntaxNode>>(
    new Set()
  );

  const { formattedTree, nodePositionMap } = useMemo<{
    formattedTree: TreeNodeType[];
    nodePositionMap: Map<SyntaxNode, Position>;
  }>(() => {
    if (!parser || !justLanguage) {
      return { formattedTree: [], nodePositionMap: new Map() };
    }

    const tree = parse({ parser, language: justLanguage, code: doc });

    if (!tree) {
      return { formattedTree: [], nodePositionMap: new Map() };
    }

    const result = processTree(tree, doc);

    setExpandedNodes(result.allNodes);

    return {
      formattedTree: result.formattedTree,
      nodePositionMap: result.nodePositionMap,
    };
  }, [parser, justLanguage, doc]);

  const highlight = useMemo(() => {
    if (!hoveredNode) return undefined;

    const position = nodePositionMap.get(hoveredNode);

    if (!position) return undefined;

    return { from: position.start, to: position.end };
  }, [hoveredNode, nodePositionMap]);

  const extensions = useEditorExtensions({
    language: justLanguage,
    highlight,
  });

  const expandNode = useCallback((node: SyntaxNode) => {
    setExpandedNodes((prevExpandedNodes) => {
      const newExpandedNodes = new Set(prevExpandedNodes);

      if (newExpandedNodes.has(node)) {
        newExpandedNodes.delete(node);
      } else {
        newExpandedNodes.add(node);
      }

      return newExpandedNodes;
    });
  }, []);

  const visibleTree = useMemo(
    () => getVisibleNodes(formattedTree, expandedNodes),
    [formattedTree, expandedNodes]
  );

  if (error) {
    return <div className='p-4'>error: {error}</div>;
  }

  if (loading || !parser || !justLanguage) {
    return (
      <div className='flex h-screen items-center justify-center'>
        <Loader2 className='text-muted-foreground h-8 w-8 animate-spin' />
      </div>
    );
  }

  return (
    <div className='flex h-screen max-w-full flex-col'>
      <div className='flex items-center gap-x-2 px-4 py-4'>
        <Bot className='h-4 w-4' />
        <a href='/' className='font-semibold'>
          just-lsp
        </a>
      </div>

      <div className='flex-1 overflow-hidden p-4'>
        <ResizablePanelGroup
          autoSaveId={PANEL_LAYOUT_STORAGE_KEY}
          direction='horizontal'
          className='h-full rounded border'
        >
          <ResizablePanel id='editor-panel' defaultSize={50} minSize={30}>
            <EditorPane value={doc} onChange={setDoc} extensions={extensions} />
          </ResizablePanel>

          <ResizableHandle withHandle />

          <ResizablePanel id='tree-panel' defaultSize={50} minSize={30}>
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
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>
    </div>
  );
};

export default App;
