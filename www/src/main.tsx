import React from 'react';
import ReactDOM from 'react-dom/client';

import App from './App.tsx';
import './index.css';
import { EditorSettingsProvider } from './providers/editor-settings-provider.tsx';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <EditorSettingsProvider>
      <App />
    </EditorSettingsProvider>
  </React.StrictMode>
);
