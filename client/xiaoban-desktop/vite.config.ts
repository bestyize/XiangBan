import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { readFile, readdir, rm } from 'node:fs/promises'
import { resolve } from 'node:path'
import { checkBuiltinDistribution } from './scripts/check-builtin-distribution.mjs'

export default defineConfig({
  server: { fs: { allow: ['.', '../../assets/pet-packs/mingyue'] } },
  plugins: [vue(), {
    name: 'external-built-in-models',
    apply: 'build',
    async buildStart() { await checkBuiltinDistribution() },
    async closeBundle() {
      for (const model of ['arowana.glb', 'blue-guppy.glb']) {
        await rm(resolve('dist/effects/arowana', model), { force: true })
      }
      const root = resolve('dist/game-packs')
      const catalog = JSON.parse(await readFile(resolve('public/game-packs/catalog.json'), 'utf8'))
      for (const id of catalog.games) {
        const original = JSON.parse(await readFile(resolve('public/game-packs', id, 'manifest.json'), 'utf8'))
        const keep = new Set(['manifest.json', original.icon, original.cover])
        async function prune(dir: string, prefix = '') {
          for (const item of await readdir(dir, { withFileTypes: true })) {
            const path = prefix ? `${prefix}/${item.name}` : item.name
            if (item.isDirectory()) await prune(resolve(dir, item.name), path)
            else if (!keep.has(path)) await rm(resolve(dir, item.name), { force: true })
          }
        }
        await prune(resolve(root, id))
      }
    },
  }],
  clearScreen: false,
  build: { target: 'es2022', sourcemap: false },
})
