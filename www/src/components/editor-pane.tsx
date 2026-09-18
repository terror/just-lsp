import { Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';

import { Editor } from './editor';
import { EditorSettingsDialog } from './editor-settings-dialog';

interface EditorPaneProps {
  value: string;
  onChange: (value: string) => void;
  onCreateEditor: (view: EditorView) => void;
  extensions: Extension[];
}

export const EditorPane = ({
  value,
  onChange,
  onCreateEditor,
  extensions,
}: EditorPaneProps) => {
  return (
    <div className='flex h-full min-h-0 flex-col overflow-hidden'>
      <div className='bg-muted/50 flex items-center justify-between border-b px-2 py-1'>
        <EditorSettingsDialog />
      </div>
      <div className='flex-1 overflow-hidden'>
        <Editor
          value={value}
          onChange={onChange}
          onCreateEditor={onCreateEditor}
          extensions={extensions}
        />
      </div>
    </div>
  );
};
