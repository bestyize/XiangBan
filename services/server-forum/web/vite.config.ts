import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 开发端口和后端地址可用环境变量覆盖，便于同时跑多套本地服务。
const port = Number(process.env.FORUM_WEB_PORT ?? 5185)
const api = process.env.FORUM_API ?? 'http://127.0.0.1:55010'

export default defineConfig({
  plugins: [vue()],
  resolve: { alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) } },
  server: {
    port,
    strictPort: true,
    proxy: { '/api': api },
  },
  build: { chunkSizeWarningLimit: 600 },
})
