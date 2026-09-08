import { Suspense, lazy } from 'react';
import { Navigate, Route, Routes } from 'react-router';

import Home from './pages/home';
import { EditorSettingsProvider } from './providers/editor-settings-provider';

const Documentation = lazy(() => import('./pages/documentation'));
const Playground = lazy(() => import('./pages/playground'));

const App = () => (
  <Routes>
    <Route path='/' element={<Home />} />
    <Route
      path='/documentation'
      element={
        <Suspense
          fallback={
            <div
              role='status'
              className='text-muted-foreground flex h-screen items-center justify-center'
            >
              Loading documentation…
            </div>
          }
        >
          <Documentation />
        </Suspense>
      }
    />
    <Route
      path='/playground'
      element={
        <EditorSettingsProvider>
          <Suspense
            fallback={
              <div
                role='status'
                className='text-muted-foreground flex h-screen items-center justify-center'
              >
                Loading playground…
              </div>
            }
          >
            <Playground />
          </Suspense>
        </EditorSettingsProvider>
      }
    />
    <Route path='*' element={<Navigate to='/' replace />} />
  </Routes>
);

export default App;
