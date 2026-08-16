import { useEditorSettings } from '@/contexts/editor-settings-context';
import { Extension } from '@codemirror/state';
import CodeMirror from '@uiw/react-codemirror';

interface EditorProps {
  value: string;
  onChange: (value: string) => void;
  extensions: Extension[];
}

export const Editor = ({ value, onChange, extensions }: EditorProps) => {
  const { settings } = useEditorSettings();

  return (
    <div className='editor-host h-full w-full overflow-hidden'>
      <CodeMirror
        value={value}
        extensions={extensions}
        basicSetup={{
          lineNumbers: settings.lineNumbers,
          highlightActiveLineGutter: true,
          highlightActiveLine: true,
          bracketMatching: true,
          history: true,
          indentOnInput: true,
          syntaxHighlighting: true,
          foldGutter: false,
          closeBrackets: false,
          autocompletion: false,
          highlightSelectionMatches: false,
        }}
        onChange={onChange}
        height='100%'
        style={{ height: '100%' }}
      />
    </div>
  );
};
