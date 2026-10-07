// Rebuilds public/game-packs/dusk-glider/vendor/three.min.js from the pinned `three` dev dependency.
import { readFileSync, writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { build } from 'vite'

const root = fileURLToPath(new URL('..', import.meta.url))
const { version } = JSON.parse(readFileSync(new URL('../node_modules/three/package.json', import.meta.url), 'utf8'))

await build({
  root,
  configFile: false,
  logLevel: 'warn',
  publicDir: false,
  build: {
    outDir: 'public/game-packs/dusk-glider/vendor',
    emptyOutDir: false,
    minify: true,
    lib: { entry: 'scripts/dusk-glider-three.js', name: 'THREE', formats: ['iife'], fileName: () => 'three.min.js' },
  },
})

// The minifier drops license comments, so the MIT notice is prepended afterwards.
const out = new URL('../public/game-packs/dusk-glider/vendor/three.min.js', import.meta.url)
const banner = `/*! three.js r${version.split('.')[1]} subset · MIT License · Copyright 2010-2025 three.js authors · https://threejs.org */\n`
writeFileSync(out, banner + readFileSync(out, 'utf8'))
