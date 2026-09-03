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
    // by git. `rekall-app` copies this folder into the jar under `static/`, which is how one
    // jar serves the UI, the API and MCP. It used to be written straight into
    // `rekall-app/src/main/resources/static` and committed, which meant every UI change was a
    // 205-file diff of hashed filenames and a stale bundle could ship without a word.
    outDir: fileURLToPath(new URL('./dist', import.meta.url)),
    emptyOutDir: true
  },
  server: {
    port: 5173,
    proxy: {
      '/api': 'http://localhost:47355',
      '/mcp': 'http://localhost:47355'
    }
  },
  test: {
    environment: 'happy-dom',
    include: ['tests/**/*.spec.ts']
  }
})
