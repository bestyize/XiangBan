import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { rm } from 'node:fs/promises'
import { resolve } from 'node:path'

export default defineConfig({
  plugins: [vue(), {
    name: 'external-built-in-models',
    apply: 'build',
    async closeBundle() {
      for (const model of ['arowana.glb', 'blue-guppy.glb']) {
        await rm(resolve('dist/effects/arowana', model), { force: true })
      }
    },
  }],
  clearScreen: false,
  build: { target: 'es2022', sourcemap: false },
})
