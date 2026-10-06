import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

// The renderer imports these exact sources; run their behavioral tests in the
// original project rather than silently testing a copied implementation.
const web = fileURLToPath(new URL('../../../web/', import.meta.url))
const result = spawnSync(process.execPath, ['node_modules/vitest/vitest.mjs', 'run', 'src/robot/robot.test.ts'], {
  cwd: web, stdio: 'inherit',
})
process.exit(result.status ?? 1)
