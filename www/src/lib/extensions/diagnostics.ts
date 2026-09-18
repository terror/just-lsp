import { linter } from '@codemirror/lint';

import { analyzeSource } from '../analyzer';

export const diagnosticsExtension = linter(
  (view) => analyzeSource(view.state.doc.toString()),
  {
    delay: 250,
  }
);
