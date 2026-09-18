import { linter } from '@codemirror/lint';

import type { AnalysisClient } from '../analysis/client';
import { toEditorDiagnostics } from '../diagnostics';

export const createDiagnosticsExtension = (client: AnalysisClient) =>
  linter(
    async (view) => {
      const { doc } = view.state;

      return toEditorDiagnostics(doc, await client.analyze(doc.toString()));
    },
    { delay: 250 }
  );
