import { Extension } from '@codemirror/state';

import { Editor } from './editor';
import { EditorSettingsDialog } from './editor-settings-dialog';

interface EditorPaneProps {
  value: string;
  onChange: (value: string) => void;
  extensions: Extension[];
}

export const EditorPane = ({
  value,
  onChange,
  extensions,
}: EditorPaneProps) => {
  return (
    <div className='flex h-full min-h-0 flex-col overflow-hidden'>
      <div className='flex items-center justify-between border-b bg-gray-50 px-2 py-1'>
        <EditorSettingsDialog />
      </div>
      <div className='flex-1 overflow-hidden'>
        <Editor value={value} onChange={onChange} extensions={extensions} />
      </div>
    </div>
  );
};
