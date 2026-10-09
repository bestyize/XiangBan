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
      includeAssets: ['favicon.svg', 'apple-touch-icon-180x180.png'],
      manifest: {
        name: 'Wallpaper Collect',
        short_name: 'Wallpapers',
        description: '多平台壁纸聚合：图片与动态壁纸',
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
        // The admin console needs the server anyway, so it is never answered from the cached app shell.
        // Otherwise an out-of-date shell would send /admin to the home page instead of showing the real error.
        navigateFallbackDenylist: [/^\/api\//, /^\/health/, /^\/admin/],
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
