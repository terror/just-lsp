import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { execFileSync } from 'child_process';
import path from 'path';
import { defineConfig } from 'vite';

export default defineConfig({
  css: { devSourcemap: true },
  define: {
    'import.meta.env.VITE_GIT_REVISION': JSON.stringify(
      execFileSync('git', ['rev-parse', 'HEAD'], {
        cwd: import.meta.dirname,
        encoding: 'utf8',
      }).trim()
    ),
  },
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(import.meta.dirname, './src'),
    },
  },
});
