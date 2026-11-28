import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  base: '/',
  plugins: [vue()],
  server: {
    host: '127.0.0.1',
    port: 5179,
    proxy: {
      '/api/v1/desktop': {
        target: 'https://wallpaper.xiangban.xyz',
        changeOrigin: true,
      },
    },
  },
});
