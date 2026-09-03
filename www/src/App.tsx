import { Button } from '@/components/ui/button';
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable';
import { Bot, Loader2, Moon, Sun } from 'lucide-react';
import { useCallback, useLayoutEffect, useState } from 'react';

import defaultJustfile from '../../justfile?raw';
import { AboutDialog } from './components/about-dialog';
import { EditorPane } from './components/editor-pane';
import { TreePane } from './components/tree-pane';
import { useEditorExtensions } from './hooks/use-editor-extensions';
import { useMediaQuery } from './hooks/use-media-query';
import { usePersistedDoc } from './hooks/use-persisted-doc';
import { useSyntaxTree } from './hooks/use-syntax-tree';
import { useTreeSitter } from './hooks/use-tree-sitter';

const EDITOR_STORAGE_KEY = 'just-lsp:editor-code';
const PANEL_LAYOUT_STORAGE_KEY = 'just-lsp:panel-layout';
const THEME_STORAGE_KEY = 'just-lsp:theme';
const STACKED_LAYOUT_QUERY = '(max-width: 767px)';

const App = () => {
  const { parser, language: justLanguage, loading, error } = useTreeSitter();
  const stackedLayout = useMediaQuery(STACKED_LAYOUT_QUERY);
  const panelDirection = stackedLayout ? 'vertical' : 'horizontal';
  const [darkMode, setDarkMode] = useState(() => {
    const savedTheme = localStorage.getItem(THEME_STORAGE_KEY);

    if (savedTheme === 'dark' || savedTheme === 'light') {
      return savedTheme === 'dark';
    }

    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  });

  useLayoutEffect(() => {
    document.documentElement.classList.toggle('dark', darkMode);
    localStorage.setItem(THEME_STORAGE_KEY, darkMode ? 'dark' : 'light');
  }, [darkMode]);

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
    darkMode,
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
        <div className='ml-auto flex items-center gap-1'>
          <Button
            variant='ghost'
            size='icon'
            className='h-8 w-8 cursor-pointer'
            onClick={() => setDarkMode((enabled) => !enabled)}
            aria-label={
              darkMode ? 'Switch to light mode' : 'Switch to dark mode'
            }
            aria-pressed={darkMode}
            title={darkMode ? 'Switch to light mode' : 'Switch to dark mode'}
          >
            {darkMode ? <Sun /> : <Moon />}
          </Button>
          <AboutDialog />
        </div>
      </div>

      <div className='flex-1 overflow-hidden p-4'>
        <ResizablePanelGroup
          key={panelDirection}
          autoSaveId={`${PANEL_LAYOUT_STORAGE_KEY}:${panelDirection}`}
          direction={panelDirection}
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
