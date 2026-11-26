import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 开发端口和后端地址可用环境变量覆盖，便于同时跑多套本地服务。
const port = Number(process.env.FORUM_WEB_PORT ?? 5185)
const api = process.env.FORUM_API ?? 'http://127.0.0.1:55010'
const ideaPlayers = process.env.FORUM_IDEA_PLAYERS_ORIGIN

export default defineConfig(({ command }) => ({
  // The HTML and APIs stay on the forum; production bundles and public files
  // are served by the resource host. Development keeps its local origin.
  base: command === 'build' ? 'https://res.xiangban.xyz/forum-web/' : '/',
  plugins: [vue()],
  // Preserve the codec module's relative URLs for its WebAssembly binaries in development.
  optimizeDeps: { exclude: ['@jsquash/webp', 'xiaoban-desktop/web-workbench'] },
  // Resolve the linked workbench against the website's Vue and UI dependencies.
  resolve: { preserveSymlinks: true, dedupe: ['vue'], alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) } },
  server: {
    port,
    strictPort: true,
    proxy: {
      '/api': { target: api, changeOrigin: true },
      '/files': { target: api, changeOrigin: true },
      ...(ideaPlayers ? { '/idea-players': { target: ideaPlayers, changeOrigin: true } } : {}),
    },
  },
  build: { chunkSizeWarningLimit: 600 },
}))
