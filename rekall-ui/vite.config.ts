import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) }
  },
  build: {
    // Build output, and it lives where build output belongs: outside the source tree, ignored
    // by git. `rekall-app` embeds this folder into `rekall-server` at compile time, which is how
    // one binary serves the UI, the API and MCP.
    outDir: fileURLToPath(new URL('./dist', import.meta.url)),
    emptyOutDir: true
  },
  server: {
    port: 5173,
    proxy: {
      // ws:true so the terminal pane's WebSocket (/api/terminal/{id}/io) proxies through too.
      '/api': { target: 'http://localhost:47355', ws: true },
      '/mcp': 'http://localhost:47355'
    }
  },
  test: {
    environment: 'happy-dom',
    include: ['tests/**/*.spec.ts'],
    // md-editor-v3 has no `exports` map, so Node would load its CommonJS `main` and with it a
    // second @codemirror/state beside the ESM one the live preview imports. The app's bundler
    // takes the ES build; the tests take the same one.
    alias: [{ find: /^md-editor-v3$/, replacement: 'md-editor-v3/lib/es/index.mjs' }]
  }
})
