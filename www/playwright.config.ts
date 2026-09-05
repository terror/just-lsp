import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  use: {
    baseURL: 'http://127.0.0.1:4173/just-lsp/',
    viewport: { width: 1280, height: 800 },
  },
  webServer: {
    command:
      'bun run build && bun run preview --host 127.0.0.1 --port 4173 --strictPort',
    url: 'http://127.0.0.1:4173/just-lsp/',
  },
});
