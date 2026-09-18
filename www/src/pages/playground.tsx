import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable';
import { highlightEffect } from '@/lib/extensions/highlight';
import { EditorView } from '@codemirror/view';
import { Loader2 } from 'lucide-react';
import { useCallback, useDeferredValue, useEffect, useState } from 'react';
import { useDefaultLayout } from 'react-resizable-panels';

import defaultJustfile from '../../../justfile?raw';
import { EditorPane } from '../components/editor-pane';
import { Header } from '../components/header';
import { TreePane } from '../components/tree-pane';
import { useEditorExtensions } from '../hooks/use-editor-extensions';
import { useMediaQuery } from '../hooks/use-media-query';
import { usePersistedDoc } from '../hooks/use-persisted-doc';
import {
  type PlaygroundRuntime,
  usePlaygroundRuntime,
} from '../hooks/use-playground-runtime';
import { useTheme } from '../hooks/use-theme';

const EDITOR_STORAGE_KEY = 'just-lsp:editor-code';
const PANEL_LAYOUT_STORAGE_KEY = 'just-lsp:panel-layout';
const STACKED_LAYOUT_QUERY = '(max-width: 767px)';

const PlaygroundEditor = ({ parser, language }: PlaygroundRuntime) => {
  const stackedLayout = useMediaQuery(STACKED_LAYOUT_QUERY);
  const panelDirection = stackedLayout ? 'vertical' : 'horizontal';

  const { defaultLayout, onLayoutChanged } = useDefaultLayout({
    id: `${PANEL_LAYOUT_STORAGE_KEY}:${panelDirection}`,
  });

  const theme = useTheme();

  const [doc, setDoc] = usePersistedDoc(
    EDITOR_STORAGE_KEY,
    defaultJustfile.trim()
  );

  const treeDoc = useDeferredValue(doc);

  const [editor, setEditor] = useState<EditorView>();

  const handleHighlightChange = useCallback(
    (range: { from: number; to: number } | undefined) => {
      if (!editor) return;

      const highlight =
        treeDoc === editor.state.doc.toString() &&
        range &&
        range.from < range.to
          ? range
          : null;

      editor.dispatch({
        effects: [
          highlightEffect.of(highlight),
          ...(highlight
            ? [EditorView.scrollIntoView(highlight.from, { y: 'center' })]
            : []),
        ],
      });
    },
    [editor, treeDoc]
  );

  const extensions = useEditorExtensions({
    language,
    darkMode: theme.darkMode,
  });

  return (
    <div className='flex h-screen max-w-full flex-col'>
      <Header theme={theme} />

      <div className='flex-1 overflow-hidden p-4'>
        <ResizablePanelGroup
          key={panelDirection}
          defaultLayout={defaultLayout}
          onLayoutChanged={onLayoutChanged}
          orientation={panelDirection}
          className='h-full rounded border'
        >
          <ResizablePanel id='editor-panel' defaultSize='50%' minSize='30%'>
            <EditorPane
              value={doc}
              onChange={setDoc}
              onCreateEditor={setEditor}
              extensions={extensions}
            />
          </ResizablePanel>

          <ResizableHandle withHandle />

          <ResizablePanel id='tree-panel' defaultSize='50%' minSize='30%'>
            <TreePane
              parser={parser}
              code={treeDoc}
              onHighlightChange={handleHighlightChange}
            />
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>
    </div>
  );
};

const Playground = () => {
  const state = usePlaygroundRuntime();

  useEffect(() => {
    document.title = 'Playground - just-lsp';
  }, []);

  switch (state.status) {
    case 'loading':
      return (
        <div className='flex h-screen items-center justify-center'>
          <Loader2 className='text-muted-foreground h-8 w-8 animate-spin' />
        </div>
      );
    case 'error':
      return <div className='p-4'>error: {state.error}</div>;
    case 'ready':
      return <PlaygroundEditor {...state.runtime} />;
  }
};

export default Playground;
