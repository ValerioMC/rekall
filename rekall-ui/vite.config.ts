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
    // Built straight into the Spring Boot module, so one jar serves the UI, the API and MCP.
    outDir: fileURLToPath(new URL('../rekall-app/src/main/resources/static', import.meta.url)),
    emptyOutDir: true
  },
  server: {
    port: 5173,
    proxy: {
      '/api': 'http://localhost:8080',
      '/mcp': 'http://localhost:8080'
    }
  },
  test: {
    environment: 'happy-dom',
    include: ['tests/**/*.spec.ts']
  }
})
