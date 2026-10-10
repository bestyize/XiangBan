import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  base: '/admin/',
  plugins: [vue()],
  server: {
    port: 5175,
    proxy: { '/api': { target: 'http://127.0.0.1:55009', changeOrigin: true } },
  },
});
