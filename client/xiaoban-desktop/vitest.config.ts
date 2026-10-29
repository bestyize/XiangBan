import { configDefaults, defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

// `vite build` reads vite.config.ts; only the test runner reads this file.
// Vue's default asset transform rewrites template URLs, which breaks the public
// /xiaoban.svg reference under vitest on Windows (no drive letter -> fileURLToPath
// rejects file:///xiaoban.svg). Leaving absolute paths untouched avoids that while
// keeping the rest of the build config the same as vite.config.ts.
export default defineConfig({
  plugins: [vue({ template: { transformAssetUrls: { includeAbsolute: false } } })],
  clearScreen: false,
  build: { target: 'es2022', sourcemap: false },
  // Build-script checks under scripts/ use node:test and run through `npm run test:scripts`.
  test: { exclude: [...configDefaults.exclude, 'scripts/**'] },
})
