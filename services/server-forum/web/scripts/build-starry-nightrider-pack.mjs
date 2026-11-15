import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { join, resolve } from 'node:path'
import { parseArgs } from 'node:util'
import JSZip from 'jszip'

const { values } = parseArgs({ options: {
  site: { type: 'string' }, cover: { type: 'string' }, icon: { type: 'string' }, out: { type: 'string' },
} })
if (!values.site || !values.cover || !values.icon || !values.out) {
  throw new Error('用法：node scripts/build-starry-nightrider-pack.mjs --site <复刻版 site> --cover <WebP 封面> --icon <WebP 图标> --out <输出目录>')
}

function replaceRequired(text, match, replacement) {
  if (!text.includes(match)) throw new Error(`复刻版缺少标记：${match}`)
  return text.replace(match, () => replacement)
}

const site = resolve(values.site)
const html = await readFile(join(site, 'index.html'), 'utf8')
const script = html.match(/<script type="module" crossorigin src="\/(assets\/index-[\w-]+\.js)"><\/script>/)
const stylesheet = html.match(/<link rel="stylesheet" crossorigin href="\/(assets\/index-[\w-]+\.css)">/)
if (!script || !stylesheet) throw new Error('复刻版缺少主脚本或样式表')

const [main, css, bridge, offline, cover, icon] = await Promise.all([
  readFile(join(site, script[1]), 'utf8'),
  readFile(join(site, stylesheet[1]), 'utf8'),
  readFile(new URL('../runtime-v2/bridge.js', import.meta.url), 'utf8'),
  readFile(new URL('../runtime-v2/starry-offline.js', import.meta.url), 'utf8'),
  readFile(values.cover),
  readFile(values.icon),
])
const textures = await Promise.all([
  'painting.webp', 'painting-s.webp', 'sky.webp', 'sky-s.webp', 'flow.png', 'cypress.png', 'fill.png',
].map(async name => {
  const type = name.endsWith('.png') ? 'png' : 'webp'
  const bytes = await readFile(join(site, name))
  return [name, `data:image/${type};base64,${bytes.toString('base64')}`]
}))

// These anchors belong to the pinned translation build. Keep the game and the
// local scoring interface, dropping the network heartbeat and analytics boot.
const presenceStart = main.indexOf('function yh(e){let t=`/api/presence`')
const presenceEnd = main.indexOf('var bh=', presenceStart)
if (presenceStart < 0 || presenceEnd < 0 || (main.match(/fetch\(zh,/g) ?? []).length !== 2) {
  throw new Error('复刻版接口与固定版本不一致')
}
let game = main.slice(0, presenceStart) + 'function yh(){return{report(){}}}' + main.slice(presenceEnd)
game = replaceRequired(game, 'cg.load(Qh+e,', 'cg.load(starryTextures[e],')
game = replaceRequired(game, 'function Lh(){if(!Ih){Ih=!0;try{Ph()}catch{}}}', 'function Lh(){}')
game = replaceRequired(game, 'Bh(`board-form`).addEventListener(`submit`,e=>{e.preventDefault(),m()})',
  'Bh(`board-post`).addEventListener(`click`,m),Bh(`board-handle`).addEventListener(`keydown`,e=>{if(e.key===`Enter`){e.preventDefault();m()}})')
game = game.replaceAll('fetch(zh,', 'starryLocalBoard(zh,')
  .replaceAll('`排行榜`', '`本机排行榜`')
  .replaceAll('发布中…', '正在保存…')
  .replaceAll('已发布', '已保存')
  .replaceAll('分数发布失败,请稍后再试。', '成绩保存失败，请重试。')
  .replaceAll('无法连接排行榜,请检查网络后重试。', '无法读取本机成绩，请重试。')
  .replaceAll('画作资源加载失败,请检查网络后刷新。', '画作资源加载失败，请重新打开。')

const bootstrap = `<script>\n` + "document.currentScript.dataset.parentOrigin = location.hostname === 'res.xiangban.xyz' ? 'https://chat.xiangban.xyz' : location.origin;\n" + `if (!window.xb) {\n${bridge}\n}
document.addEventListener('DOMContentLoaded', () => requestAnimationFrame(() => window.xb.ready()), { once: true });
</script>`
const module = [`const starryTextures = ${JSON.stringify(Object.fromEntries(textures))};`, offline, game]
  .join('\n').replace(/<\/script/gi, '<\\/script')
let page = replaceRequired(html, script[0], `${bootstrap}\n<script type="module">\n${module}\n</script>`)
page = replaceRequired(page, stylesheet[0], `<style>\n${css}\n</style>`)
page = replaceRequired(page, '<script type="module" src="/ghosts.mjs"></script>', '')
const extrasStart = page.indexOf('<!-- 交流浮层:')
const extrasEnd = page.indexOf('<!-- 昵称浮层:')
if (extrasStart < 0 || extrasEnd <= extrasStart) throw new Error('复刻版浮层布局与固定版本不一致')
page = page.slice(0, extrasStart) + page.slice(extrasEnd)
page = page.replace(/<button id="(?:chat|games)-open"[^>]*>[^<]*<\/button>/g, '')
  .replace('id="board-post" type="submit"', 'id="board-post" type="button"')
  .replaceAll('>排行榜<', '>本机排行榜<')
  .replaceAll('发布分数', '保存成绩')
  .replaceAll('还没有人上榜,来做第一个吧。', '这里还没有本机成绩。')
  .replaceAll('名号会留在排行榜与同行骑手眼中', '名号仅显示在本机排行榜')

const manifest = {
  schemaVersion: 3, runtimeProtocol: 2, kind: 'game', id: 'starry-nightrider', name: '星夜骑士',
  description: '骑进梵高《星夜》，收集光球并把成绩保存到本机。', version: '1.0.0', category: 'casual',
  entry: 'index.html', icon: 'icon.webp', cover: 'cover.webp', orientation: 'landscape',
  input: ['mouse', 'keyboard', 'touch'],
}
const zip = new JSZip()
zip.file('manifest.json', JSON.stringify(manifest, null, 2) + '\n')
zip.file('index.html', page)
zip.file('icon.webp', icon)
zip.file('cover.webp', cover)
zip.file('SOURCE.md', '# Starry Nightrider\n\n原作 @ebbyamir · https://starrynightrider.com/\n中文复刻 ByteRax · https://github.com/ByteRax/Starrynightrider/tree/0c5714aa67059d1ae804db2d5cfc868cf27c1b6a\n\n本版本将复刻版前端内嵌为单文件，并将排行榜改为本机存储。游戏与美术归原作者。\n')
const out = resolve(values.out)
await mkdir(out, { recursive: true })
await writeFile(join(out, 'starry-nightrider.html'), page)
await writeFile(join(out, 'Starry-Nightrider-v2.zip'), await zip.generateAsync({ type: 'nodebuffer', compression: 'DEFLATE' }))
