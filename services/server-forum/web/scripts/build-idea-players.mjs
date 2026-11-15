import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { parseArgs } from 'node:util'
import { aquariumExhibit, rabbitExhibit } from '../src/lib/microGallery.ts'

const { values } = parseArgs({ options: {
  saltline: { type: 'string' },
  out: { type: 'string' },
} })
if (!values.saltline || !values.out) {
  throw new Error('用法：node --experimental-strip-types scripts/build-idea-players.mjs --saltline <原始 HTML> --out <输出目录>')
}

const bridge = await readFile(new URL('../runtime-v2/bridge.js', import.meta.url), 'utf8')
const bootstrap = `<script>
document.currentScript.dataset.parentOrigin = location.origin;
${bridge}
document.addEventListener('DOMContentLoaded', () => {
  requestAnimationFrame(() => window.xb.ready());
}, { once: true });
</script>`

function replaceRequired(html, pattern, replacement) {
  if (!html.includes(pattern)) throw new Error(`作品 HTML 缺少标记 ${pattern}`)
  return html.replace(pattern, () => replacement)
}

function saltlinePlayer(html) {
  const style = '<style>[data-row="fs"] { display: none !important; }</style>'
  const body = '<div id="app"'
  html = html.replace(/<link\b[^>]*>/gi, '')
  html = replaceRequired(html, body, `</head><body>${body}`)
  return `<!doctype html><html lang="en"><head><meta charset="utf-8">${bootstrap}${style}${html}</body></html>`
}

function galleryPlayer(html, id) {
  const controls = `<style>#fullscreen, #preview-fullscreen { display: none !important; }</style>
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
const players = await Promise.all([
  readFile(values.saltline, 'utf8').then(html => ['saltline', saltlinePlayer(html)]),
  ...[aquariumExhibit, rabbitExhibit].map(async ({ id, source }) => {
    const html = await readFile(new URL(source, gallery), 'utf8')
    return [id, galleryPlayer(html, id)]
  }),
])
await mkdir(output, { recursive: true })
for (const [id, html] of players) {
  await writeFile(resolve(output, `${id}.html`), html)
  console.log(`${id}.html  ${Buffer.byteLength(html)} bytes`)
}
