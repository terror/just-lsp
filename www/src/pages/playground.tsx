import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable';
import { highlightEffect } from '@/lib/extensions/highlight';
import { getSample, samples } from '@/lib/samples';
import { EditorView } from '@codemirror/view';
import { Loader2 } from 'lucide-react';
import {
  useCallback,
  useDeferredValue,
  useEffect,
  useEffectEvent,
  useLayoutEffect,
  useRef,
} from 'react';
import { useDefaultLayout, useGroupRef } from 'react-resizable-panels';

import { EditorPane } from '../components/editor-pane';
import { Header } from '../components/header';
import { TreePane } from '../components/tree-pane';
import { useMediaQuery } from '../hooks/use-media-query';
import { usePersistedState } from '../hooks/use-persisted-state';
import {
  type PlaygroundRuntime,
  usePlaygroundRuntime,
} from '../hooks/use-playground-runtime';
import { useTheme } from '../hooks/use-theme';

const EDITOR_STORAGE_KEY = 'just-lsp:editor-code';
const PANEL_LAYOUT_STORAGE_KEY = 'just-lsp:panel-layout';
const SAMPLE_STORAGE_KEY = 'just-lsp:sample';
const STACKED_LAYOUT_QUERY = '(max-width: 767px)';

interface PlaygroundEditorProps extends PlaygroundRuntime {
  onSampleChange: (name: string) => void;
  sample: ReturnType<typeof getSample>;
}

const PlaygroundEditor = ({
  analysis,
  language,
  onSampleChange,
  sample,
}: PlaygroundEditorProps) => {
  const stackedLayout = useMediaQuery(STACKED_LAYOUT_QUERY);
  const panelDirection = stackedLayout ? 'vertical' : 'horizontal';

  const groupRef = useGroupRef();

  const { defaultLayout, onLayoutChanged } = useDefaultLayout({
    id: `${PANEL_LAYOUT_STORAGE_KEY}:${panelDirection}`,
  });

  const restoreLayout = useEffectEvent(() => {
    groupRef.current?.setLayout(
      defaultLayout ?? { 'editor-panel': 50, 'tree-panel': 50 }
    );
  });

  useLayoutEffect(() => {
    restoreLayout();
  }, [panelDirection]);

  const theme = useTheme();

  const [code, setCode] = usePersistedState(
    `${EDITOR_STORAGE_KEY}:${sample.name}`,
    sample.code
  );

  const treeDoc = useDeferredValue(code);

  const editorRef = useRef<EditorView | null>(null);

  const handleHighlightChange = useCallback(
    (range: { from: number; to: number } | undefined) => {
      const editor = editorRef.current;

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
    [treeDoc]
  );

  return (
    <div className='flex h-screen max-w-full flex-col'>
      <Header theme={theme} />

      <div className='flex-1 overflow-hidden p-4'>
        <ResizablePanelGroup
          defaultLayout={defaultLayout}
          groupRef={groupRef}
          onLayoutChanged={onLayoutChanged}
          orientation={panelDirection}
          className='h-full rounded border'
        >
          <ResizablePanel id='editor-panel' defaultSize='50%' minSize='30%'>
            <EditorPane
              analysis={analysis}
              value={code}
              onChange={setCode}
              onCreateEditor={(editor) => {
                editorRef.current = editor;
              }}
              onReset={() => setCode(sample.code)}
              sample={sample.name}
              onSampleChange={onSampleChange}
              language={language}
              darkMode={theme.darkMode}
            />
          </ResizablePanel>

          <ResizableHandle />

          <ResizablePanel
            id='tree-panel'
            defaultSize='50%'
            minSize='30%'
            collapsible
          >
            <TreePane
              language={language}
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

  const [selectedSample, setSelectedSample] = usePersistedState(
    SAMPLE_STORAGE_KEY,
    samples[0].name
  );

  const sample = getSample(selectedSample);

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
      return (
        <PlaygroundEditor
          key={sample.name}
          {...state.runtime}
          sample={sample}
          onSampleChange={setSelectedSample}
        />
      );
  }
};

export default Playground;
