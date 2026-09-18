import type * as lint from '@codemirror/lint';
import type { Text } from '@codemirror/state';

import type { Diagnostic } from './types';

export function toEditorDiagnostics(
  doc: Text,
  diagnostics: Diagnostic[]
): lint.Diagnostic[] {
  const offset = (line: number, character: number) => {
    if (line >= doc.lines) return doc.length;

    const { from, length } = doc.line(line + 1);

    return from + Math.min(character, length);
  };

  return diagnostics.map((diagnostic) => ({
    from: offset(diagnostic.start_line, diagnostic.start_character),
    to: offset(diagnostic.end_line, diagnostic.end_character),
    severity:
      diagnostic.severity === 'error' ||
      diagnostic.severity === 'warning' ||
      diagnostic.severity === 'hint'
        ? diagnostic.severity
        : 'info',
    message: diagnostic.message,
    source: diagnostic.id,
  }));
}
