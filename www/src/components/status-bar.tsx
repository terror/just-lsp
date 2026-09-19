import { forEachDiagnostic } from '@codemirror/lint';
import type { EditorState } from '@codemirror/state';
import { CircleCheck, CircleX, TriangleAlert } from 'lucide-react';

interface StatusBarProps {
  state: EditorState | undefined;
}

export const StatusBar = ({ state }: StatusBarProps) => {
  const head = state?.selection.main.head ?? 0;
  const line = state?.doc.lineAt(head);

  const row = line?.number ?? 1;
  const column = head - (line?.from ?? 0) + 1;

  const counts = { error: 0, warning: 0 };

  if (state) {
    forEachDiagnostic(state, ({ severity }) => {
      if (severity === 'error' || severity === 'warning') {
        counts[severity]++;
      }
    });
  }

  return (
    <div className='bg-muted/40 text-muted-foreground flex min-h-7 shrink-0 items-center gap-x-3 overflow-x-auto border-t px-2 font-mono text-xs whitespace-nowrap'>
      <div role='status' className='flex items-center gap-x-3 font-medium'>
        {counts.error === 0 && counts.warning === 0 && (
          <span
            title='No errors or warnings'
            className='text-emerald-700 dark:text-emerald-400'
          >
            <CircleCheck className='h-3.5 w-3.5' aria-hidden='true' />
            <span className='sr-only'>No errors or warnings</span>
          </span>
        )}
        {counts.error > 0 && (
          <span className='flex items-center gap-x-1.5 text-red-700 dark:text-red-400'>
            <CircleX className='h-3.5 w-3.5' aria-hidden='true' />
            <span>
              {counts.error.toLocaleString()}{' '}
              {counts.error === 1 ? 'error' : 'errors'}
            </span>
          </span>
        )}
        {counts.warning > 0 && (
          <span className='flex items-center gap-x-1.5 text-amber-700 dark:text-amber-400'>
            <TriangleAlert className='h-3.5 w-3.5' aria-hidden='true' />
            <span>
              {counts.warning.toLocaleString()}{' '}
              {counts.warning === 1 ? 'warning' : 'warnings'}
            </span>
          </span>
        )}
      </div>
      <span aria-label={`Line ${row}, column ${column}`}>
        {row}:{column}
      </span>
    </div>
  );
};
