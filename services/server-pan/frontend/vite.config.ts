import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

// 后端端口：可用环境变量 PANDOWN_BACKEND_PORT / PANDOWN_BACKEND_HOST 覆盖，默认 55005
const backendPort = process.env.PANDOWN_BACKEND_PORT || '55005';
const backendHost = process.env.PANDOWN_BACKEND_HOST || '127.0.0.1';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
  ],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: `http://${backendHost}:${backendPort}`,
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
});
