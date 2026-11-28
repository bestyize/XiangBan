import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  base: './',
  plugins: [vue()],
  server: {
    host: '127.0.0.1',
    port: 5178,
    proxy: {
      '/account-api': {
        target: 'https://account.xiangban.xyz',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/account-api/, '/api')
      }
    }
  }
});
