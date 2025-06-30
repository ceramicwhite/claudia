import { defineConfig } from 'vitest/config'
import react from "@vitejs/plugin-react"
import tailwindcss from "@tailwindcss/vite"
import { fileURLToPath, URL } from "node:url"

export default defineConfig({
  plugins: [react(), tailwindcss()],
  
  // Path resolution (same as in vite.config.ts)
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    css: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/test/',
        '**/*.d.ts',
        '**/*.config.*',
        '**/mockdata/**',
        'src-tauri/**',
      ],
    },
    // Mock Tauri API by default
    mockReset: true,
    restoreMocks: true,
    // Exclude src-tauri from test files
    exclude: ['**/node_modules/**', '**/dist/**', '**/src-tauri/**'],
  },
})