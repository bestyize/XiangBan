import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { parseArgs } from 'node:util'
import JSZip from 'jszip'
import catalog from '../public/ideas-catalog.json' with { type: 'json' }
import { aquariumExhibit, rabbitExhibit } from '../src/lib/microGallery.ts'

const { values } = parseArgs({ options: {
  saltline: { type: 'string' },
  out: { type: 'string' },
} })
if (!values.out) {
  throw new Error('用法：node --experimental-strip-types scripts/build-idea-players.mjs --out <输出目录> [--saltline <原始 HTML>]')
}

const bridge = await readFile(new URL('../runtime-v2/bridge.js', import.meta.url), 'utf8')
const bootstrap = `<script>
if (!window.xb) {
  document.currentScript.dataset.parentOrigin = location.origin;
  ${bridge}
}
function syncHostCapsule() {
  document.documentElement.toggleAttribute('data-host-capsule', Boolean(window.xb.info?.capsule?.width));
}
syncHostCapsule();
window.xb.on?.('safearea', syncHostCapsule);
document.addEventListener('DOMContentLoaded', () => {
  requestAnimationFrame(() => window.xb.ready());
}, { once: true });
</script>`

function replaceRequired(html, pattern, replacement) {
  if (!html.includes(pattern)) throw new Error(`作品 HTML 缺少标记 ${pattern}`)
  return html.replace(pattern, () => replacement)
}

function saltlinePlayer(html) {
  const style = `<style>
    [data-row="fs"] { display: none !important; }
    [data-host-capsule] .t-pause { right: calc(var(--xb-capsule-right) + var(--xb-capsule-width) + 8px); }
  </style>`
  const body = '<div id="app"'
  html = html.replace(/<link\b[^>]*>/gi, '')
  html = replaceRequired(html, body, `${style}</head><body>${body}`)
  return `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">${bootstrap}${html}</body></html>`
}

function galleryPlayer(html, id) {
  // GLTFLoader's TextureLoader branch decodes embedded images through <img>.
  // Android's offline CSP allows blob images but blocks ImageBitmapLoader's fetch(blob:).
  const bitmapLoaderChoice = /typeof createImageBitmap>"u"\|\|[^?]{1,60}(?=\?\(this\.textureLoader=new )/
  if (!bitmapLoaderChoice.test(html)) throw new Error(`${id} 的 glTF 贴图加载器与固定版本不一致`)
  html = html.replace(bitmapLoaderChoice, 'true')

  const controls = `<style>
  #fullscreen, #preview-fullscreen, [data-host-capsule] .gallery-link,
  [data-host-capsule] .masthead button[onclick="window.xb.exit()"] { display: none !important; }
  [data-host-capsule] .masthead { padding-top: var(--xb-safe-top, 0px); }
  [data-host-capsule] .settings-toggle {
    right: calc(var(--xb-capsule-right) + var(--xb-capsule-width) + 8px);
  }
</style>
<script>
document.addEventListener('keydown', event => {
  if (event.key.toLowerCase() === 'f' && !event.ctrlKey && !event.altKey && !event.metaKey) {
    event.stopImmediatePropagation();
  }
}, true);
</script>`
  html = replaceRequired(html, '</head>', `${bootstrap}${controls}</head>`)
  html = replaceRequired(html, "openCommunityExhibit('gallery:back')", 'window.xb.exit()')
  html = replaceRequired(html, '当前无法显示 3D 预览，可以查看展品图片。', '当前无法显示 3D 预览。')
  if (id === 'aquarium') {
    const scene = 'new URLSearchParams(location.search).get("scene")'
    html = replaceRequired(html, scene, `(${scene} || "living-room")`)
  }
  return html
}

const gallery = new URL('../public/gallery-exhibits/', import.meta.url)
const output = resolve(values.out)
const sources = [aquariumExhibit, rabbitExhibit].map(async ({ id, source }) => {
  const html = await readFile(new URL(source, gallery), 'utf8')
  return [id, galleryPlayer(html, id)]
})
if (values.saltline) sources.unshift(readFile(values.saltline, 'utf8').then(html => ['saltline', saltlinePlayer(html)]))
const players = await Promise.all(sources)
await mkdir(output, { recursive: true })
for (const [id, html] of players) {
  await writeFile(resolve(output, `${id}.html`), html)
  const item = catalog.items.find(row => row.player === id) ?? (id === aquariumExhibit.id ? {
    title: aquariumExhibit.name, description: aquariumExhibit.description,
    cover: `/gallery-exhibits/assets/gallery/${aquariumExhibit.cover}.webp`,
  } : undefined)
  if (!item) throw new Error(`精选目录缺少 ${id}`)
  const kind = id === 'saltline' ? 'game' : 'app'
  const manifest = {
    schemaVersion: 3, runtimeProtocol: 2, id, name: item.title, description: item.description,
    version: kind === 'game' ? '1.0.0' : '1.0.1', kind, category: kind === 'game' ? 'action' : 'tool',
    entry: 'index.html', icon: 'cover.webp', cover: 'cover.webp',
    orientation: id === rabbitExhibit.id ? 'any' : 'landscape', input: ['touch', 'mouse', 'keyboard'],
  }
  const archive = new JSZip()
  archive.file('manifest.json', JSON.stringify(manifest, null, 2) + '\n')
  archive.file('index.html', html)
  archive.file('cover.webp', await readFile(new URL(`../public${item.cover}`, import.meta.url)))
  const bytes = await archive.generateAsync({ type: 'nodebuffer', compression: 'DEFLATE' })
  await writeFile(resolve(output, `${id}.zip`), bytes)
  console.log(`${id}.html + ${id}.zip  ${bytes.length} bytes`)
}
