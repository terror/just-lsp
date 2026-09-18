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
    from: offset(diagnostic.startLine, diagnostic.startCharacter),
    to: offset(diagnostic.endLine, diagnostic.endCharacter),
    severity: diagnostic.severity,
    message: diagnostic.message,
    source: diagnostic.id,
  }));
}
