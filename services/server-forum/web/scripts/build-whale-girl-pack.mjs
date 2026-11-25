import { createHash } from 'node:crypto'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { join, resolve } from 'node:path'
import { parseArgs } from 'node:util'
import JSZip from 'jszip'

const { values } = parseArgs({ options: {
  source: { type: 'string' }, cover: { type: 'string' }, icon: { type: 'string' }, out: { type: 'string' },
} })
if (!values.source || !values.cover || !values.icon || !values.out) {
  throw new Error('用法：node scripts/build-whale-girl-pack.mjs --source <鲸鱼娘目录> --cover <WebP 封面> --icon <WebP 图标> --out <输出目录>')
}

const upstream = 'https://github.com/bestyize/RealWorldBuilder/tree/037ef10fbe6c368eb931c0c958b6283d25ef0fa2/games/whale-girl'
const runtime = new URL('../runtime-v2/', import.meta.url)
const [source, license, threeLicense, bridge, css, adapter, cover, icon] = await Promise.all([
  readFile(join(values.source, 'preview.html')),
  readFile(join(values.source, 'MODEL-LICENSE.txt')),
  readFile(new URL('three-r187-LICENSE.txt', runtime), 'utf8'),
  readFile(new URL('bridge.js', runtime), 'utf8'),
  readFile(new URL('whale-girl.css', runtime), 'utf8'),
  readFile(new URL('whale-girl.js', runtime), 'utf8'),
  readFile(values.cover),
  readFile(values.icon),
])
const blob = bytes => createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex')
if (blob(source) !== '534771c6f666bdc000105b6894d1c1aa19a22df2' ||
    blob(license) !== 'cbe5ad1670406e4402217edfb82d2c56af7e8631') {
  throw new Error(`作品预览或模型许可与固定版本不一致：${upstream}`)
}

// The Android offline CSP permits blob images but blocks ImageBitmapLoader's fetch(blob:).
const bitmapLoaderChoice = /typeof createImageBitmap>"u"\|\|[^?]{1,60}(?=\?\(this\.textureLoader=new )/
// The published player uses the upstream story topics without the local AI server.
const localChatProbe = 'await fetch("./chat-status",{cache:"no-store",signal:AbortSignal.timeout(5e3)})'
const bootstrap = `<script>
document.currentScript.dataset.parentOrigin = location.hostname === 'res.xiangban.xyz' ? 'https://chat.xiangban.xyz' : location.origin;
if (!window.xb) {
${bridge}
}
${adapter}
</script>`
const html = source.toString('utf8')
  .replace(bitmapLoaderChoice, 'true')
  .replace(localChatProbe, '{ok:false}')
  .replaceAll(/env\(safe-area-inset-(top|right|bottom|left)\)/g, 'var(--study-safe-$1)')
  .replace('</head>', () => `<!--\n${threeLicense}-->\n<style>\n${css}</style>\n${bootstrap}\n</head>`)
  .replace('<h2 id="credit-title">作品署名</h2>', '$&\n    <p>书房由 RealWorldBuilder 创作。小伴版适配了 V2 宿主，围裙上的 DeepSeek 标志归相应权利人所有。</p>')

const manifest = {
  schemaVersion: 3, runtimeProtocol: 2, kind: 'game', id: 'whale-girl', name: '潮汐书房 · 鲸鱼娘',
  description: '在月湾书房陪溟月听潮、翻书，点亮贝壳灯给末班船打招呼。',
  version: '1.1.0', category: 'casual', entry: 'index.html', icon: 'icon.webp', cover: 'cover.webp',
  orientation: 'landscape', input: ['touch', 'mouse', 'keyboard'],
}
const files = {
  'manifest.json': JSON.stringify(manifest, null, 2) + '\n',
  'index.html': html,
  'cover.webp': cover,
  'icon.webp': icon,
  'MODEL-LICENSE.txt': license,
  'SOURCE.md': `# 潮汐书房 · 鲸鱼娘

书房原作由 RealWorldBuilder 创作。${upstream}

角色原型「溟月」由上善无形创作，DeepSeek 女仆版由 ZipZipPipe 设计。PekolandBBQ 制作了 Q 版三维造型、服装和动画。部分动作与外观取自 merzlin 的 shenshen 桌宠及 PC2005-cloud 的 dsh-pet 的参考资料。DeepSeek 标志归相应权利人所有。

本版保留原始角色模型，使用上游绘制的眼睑。角色素材及表情改编沿用 CC BY-NC-SA 4.0（见 MODEL-LICENSE.txt），限非商业用途。模型来源与完整署名见 https://github.com/PekolandBBQ/Deepseek-Dafeiyu-Chibi-3D-Model/blob/v1.0.2/THIRD_PARTY_NOTICES.md 。

小伴版增加 V2 启动、系统安全区和弹窗返回适配，调整了内嵌贴图的加载模式，并从原始封面裁切卡片与图标。Three.js r187 的 MIT 许可位于 index.html 的文件头。
`,
}

const zip = new JSZip()
const tree = createHash('sha256')
for (const name of Object.keys(files).sort()) {
  const bytes = files[name]
  zip.file(name, bytes, { date: new Date('1980-01-01T00:00:00Z') })
  tree.update(name).update('\0').update(createHash('sha256').update(bytes).digest())
}
const bytes = await zip.generateAsync({ type: 'nodebuffer', compression: 'DEFLATE' })
const md5 = createHash('md5').update(bytes).digest('hex')
const distribution = {
  id: 'featured-whale-girl', packId: manifest.id, kind: manifest.kind,
  url: `https://res.xiangban.xyz/files/${md5}.zip`, sha256: createHash('sha256').update(bytes).digest('hex'),
  bytes: bytes.length, sourceSha256: tree.digest('hex'),
}
const out = resolve(values.out)
await mkdir(out, { recursive: true })
await Promise.all([
  writeFile(join(out, 'whale-girl.html'), html),
  writeFile(join(out, 'whale-girl.zip'), bytes),
  writeFile(join(out, 'featured-whale-girl.json'), JSON.stringify(distribution, null, 2) + '\n'),
  writeFile(join(out, 'desktop-manifest.json'), JSON.stringify({ ...manifest, id: distribution.id }, null, 2) + '\n'),
])
