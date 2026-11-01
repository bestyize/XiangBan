import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'node:path'

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  build: { target: 'es2022', sourcemap: false, rollupOptions: { input: { main: resolve(__dirname, 'index.html'), communityChrome: resolve(__dirname, 'community-chrome.html') } } },
})
