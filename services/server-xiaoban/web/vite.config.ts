import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { VitePWA } from 'vite-plugin-pwa'

const apiTarget = process.env.API_TARGET ?? 'http://127.0.0.1:8080'

export default defineConfig({
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  plugins: [
    vue(),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['favicon.ico', 'app-icon.png', 'apple-touch-icon-180x180.png'],
      manifest: {
        name: '小伴壁纸',
        short_name: '小伴壁纸',
        description: '小伴壁纸：4K 高清静态与动态视频壁纸独立站',
        lang: 'zh-CN',
        start_url: '/',
        scope: '/',
        display: 'standalone',
        orientation: 'portrait',
        background_color: '#0f0f11',
        theme_color: '#0f0f11',
        icons: [
          { src: 'pwa-64x64.png', sizes: '64x64', type: 'image/png' },
          { src: 'pwa-192x192.png', sizes: '192x192', type: 'image/png' },
          { src: 'pwa-512x512.png', sizes: '512x512', type: 'image/png' },
          { src: 'maskable-icon-512x512.png', sizes: '512x512', type: 'image/png', purpose: 'maskable' },
        ],
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,svg,png,woff2}'],
        navigateFallback: '/index.html',
        // Keep API, health, admin, and external redirect paths out of the cached SPA shell.
        navigateFallbackDenylist: [
          /^\/api\//,
          /^\/health/,
          /^\/admin/,
          /^\/(desktop|pc|app|mobile|download|features|desktop-experience|plugins)(\/|$)/,
        ],
        runtimeCaching: [
          {
            urlPattern: ({ url }) => url.pathname === '/api/v1/categories' || url.pathname === '/api/v1/providers',
            handler: 'StaleWhileRevalidate',
            options: { cacheName: 'api-meta' },
          },
        ],
      },
    }),
  ],
  server: {
    port: 5173,
    proxy: {
      '/api': apiTarget,
      '/health': apiTarget,
      '/uploads': apiTarget,
    },
  },
})
