import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable';
import { Bot, Loader2 } from 'lucide-react';
import { useCallback, useState } from 'react';

import defaultJustfile from '../../justfile?raw';
import { EditorPane } from './components/editor-pane';
import { TreePane } from './components/tree-pane';
import { useEditorExtensions } from './hooks/use-editor-extensions';
import { usePersistedDoc } from './hooks/use-persisted-doc';
import { useSyntaxTree } from './hooks/use-syntax-tree';
import { useTreeSitter } from './hooks/use-tree-sitter';

const EDITOR_STORAGE_KEY = 'just-lsp:editor-code';
const PANEL_LAYOUT_STORAGE_KEY = 'just-lsp:panel-layout';

const App = () => {
  const { parser, language: justLanguage, loading, error } = useTreeSitter();

  const [doc, setDoc] = usePersistedDoc(
    EDITOR_STORAGE_KEY,
    defaultJustfile.trim()
  );

  const { root, expandedNodes, toggleExpand } = useSyntaxTree({
    parser,
    language: justLanguage,
    code: doc,
  });

  const [highlight, setHighlight] = useState<
    { from: number; to: number } | undefined
  >(undefined);

  const handleHighlightChange = useCallback(
    (range: { from: number; to: number } | undefined) => {
      setHighlight(range);
    },
    []
  );

  const extensions = useEditorExtensions({
    language: justLanguage,
    highlight,
  });

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
            <TreePane
              root={root}
              code={doc}
              expandedNodes={expandedNodes}
              toggleExpand={toggleExpand}
              onHighlightChange={handleHighlightChange}
            />
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>
    </div>
  );
};

export default App;
