import { copyFile, cp, mkdir, readFile, readdir, writeFile } from 'node:fs/promises'
import { join, relative, resolve } from 'node:path'
import { parseArgs } from 'node:util'
import JSZip from 'jszip'

const { values } = parseArgs({ options: {
  dist: { type: 'string' },
  cover: { type: 'string' },
  icon: { type: 'string' },
  out: { type: 'string' },
} })
if (!values.dist || !values.cover || !values.icon || !values.out) {
  throw new Error('用法：node scripts/build-oil-paint-pack.mjs --dist <作者的 dist> --cover <WebP 封面> --icon <WebP 图标> --out <输出目录>')
}

const dist = resolve(values.dist)
const out = resolve(values.out)
for (const asset of ['starter-man.glb', 'starter-man-clips.glb']) {
  const data = await readFile(join(dist, 'figure', asset))
  if (data.subarray(0, 4).toString() !== 'glTF') throw new Error(`缺少 Git LFS 模型内容：${asset}`)
}
const html = await readFile(join(dist, 'index.html'), 'utf8')
if (!html.includes('</head>')) throw new Error('入口 HTML 缺少 head')
const [bridge, touchJs, touchCss] = await Promise.all(['bridge.js', 'oil-paint-touch.js', 'oil-paint-touch.css']
  .map(file => readFile(new URL(`../runtime-v2/${file}`, import.meta.url), 'utf8')))
const bootstrap = `<script>
if (!window.xb) {
  document.currentScript.dataset.parentOrigin = location.hostname === 'res.xiangban.xyz' ? 'https://chat.xiangban.xyz' : location.origin;
  ${bridge}
}
document.addEventListener('DOMContentLoaded', () => {
  requestAnimationFrame(() => window.xb.ready());
}, { once: true });
</script>`

const player = join(out, 'oil-paint')
await mkdir(out, { recursive: true })
await cp(dist, player, { recursive: true, force: false, errorOnExist: true })
const entry = html
  .replace('width=device-width, initial-scale=1', 'width=device-width, initial-scale=1, viewport-fit=cover')
  .replace('</head>', () => `${bootstrap}\n<style>${touchCss}</style>\n<script>${touchJs}</script>\n</head>`)
await writeFile(join(player, 'index.html'), entry)
await copyFile(values.cover, join(player, 'cover.webp'))
await copyFile(values.icon, join(player, 'icon.webp'))
await writeFile(join(player, 'SOURCE.md'), `# Oil Paint\n\nBy Blendi (@BlendiByl), published at https://patch.gg/blendi/oil-paint.\n\nBuilt from https://git.patch.gg/blendi/oil-paint.git at 804c65f (1.1.3), with an added Xiaoban runtime bridge. Original licensing and third-party notices are included.\n`)
const manifest = {
  schemaVersion: 3,
  runtimeProtocol: 2,
  kind: 'game',
  id: 'oil-paint',
  name: 'Oil Paint',
  description: '在实时笔触绘成的 3D 油画里漫步，探索秋林与极光下的雪脊。',
  version: '1.1.3-xb1',
  category: 'casual',
  entry: 'index.html',
  cover: 'cover.webp',
  icon: 'icon.webp',
  orientation: 'landscape',
  input: ['mouse', 'keyboard', 'touch'],
}
await writeFile(join(player, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n')

const archive = new JSZip()
async function addFiles(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name)
    if (entry.isDirectory()) await addFiles(path)
    else if (entry.isFile()) archive.file(relative(player, path), await readFile(path))
    else throw new Error(`作品目录包含不支持的文件：${relative(player, path)}`)
  }
}
await addFiles(player)
const zip = await archive.generateAsync({ type: 'nodebuffer', compression: 'DEFLATE' })
await writeFile(join(out, 'Oil-Paint-v2.zip'), zip, { flag: 'wx' })
console.log(`Oil-Paint-v2.zip  ${zip.byteLength} bytes`)
