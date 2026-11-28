// Sync playable games and the artwork used by the mobile plaza.
import { access, cp, mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
const root = fileURLToPath(new URL('../../../../', import.meta.url))
const source = path.join(root, 'client/xiaoban-desktop/public/game-packs')
const mobile = path.join(root, 'client/xiaoban-multiplatform')
const mobilePacks = path.join(mobile, 'androidApp/src/main/assets/game-packs-v2')
const destination = fileURLToPath(new URL('../public/builtin-games', import.meta.url))
const imageAliases = JSON.parse(await readFile(new URL('../static-image-aliases.json', import.meta.url), 'utf8'))
const webImage = publicPath => new URL(`../assets${publicPath}`, import.meta.url)
const mbtiRoot = '/builtin-games/mbti/'

async function syncMbtiImages(dest) {
  const replacements = Object.entries(imageAliases).filter(([old]) => old.startsWith(mbtiRoot))
  for (const [old, target] of replacements) {
    await cp(webImage(target), path.join(dest, target.slice(mbtiRoot.length)))
    await rm(path.join(dest, old.slice(mbtiRoot.length)))
  }
  for (const name of ['share.html', 'share.js', 'report.css', 'share-page.css', 'README.md']) {
    const file = path.join(dest, name)
    let text = await readFile(file, 'utf8')
    for (const [old, target] of replacements) {
      text = text.replaceAll(old.slice(mbtiRoot.length), target.slice(mbtiRoot.length))
    }
    // The host provides static WebP assets; the generated share poster still uses PNG.
    if (name === 'share.js') text = text.replace("d.blob.type!=='image/png'", "d.blob.type!=='image/webp'")
    await writeFile(file, text)
  }
}

const ids = ['dusk-glider','ink-blade','jade-gomoku','pro-cube','snake-arena','block-puzzle','merge-2048','water-sort','mbti']
await rm(destination, {recursive:true, force:true}); await mkdir(destination, {recursive:true})
for (const id of ids) {
  const src = path.join(source,id), dest = path.join(destination,id)
  const manifest = JSON.parse(await readFile(path.join(src,'manifest.json'),'utf8'))
  if (manifest.id !== id || manifest.entry !== 'index.html' || (manifest.runtimeProtocol !== 1 && manifest.runtimeProtocol !== 2)) throw new Error(`Invalid built-in game manifest: ${id}`)
  await access(path.join(src, manifest.icon))
  await cp(src,dest,{recursive:true})
  if (id === 'mbti') await syncMbtiImages(dest)
  const entry = path.join(dest,'index.html'), html = await readFile(entry,'utf8')
  if (!/<head(?:\s[^>]*)?>/i.test(html)) throw new Error(`Missing head: ${id}`)
  // Scripts are official bundled files only. The sandbox supplies an opaque origin.
  const csp = `<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; media-src 'self' data: blob:; font-src 'self' data:; connect-src 'none'; object-src 'none'; frame-src 'none'; form-action 'none'; base-uri 'none'; navigate-to 'none'">`
  await writeFile(entry,html.replace(/<head(?:\s[^>]*)?>/i, match => `${match}${csp}`))
}
for (const id of ['remote-control', 'quick-launch']) {
  const { icon } = JSON.parse(await readFile(path.join(mobilePacks, id, 'manifest.json'), 'utf8'))
  const publicPath = `/builtin-games/${id}/${icon}`
  const converted = imageAliases[publicPath]
  const output = path.join(destination, (converted ?? publicPath).slice('/builtin-games/'.length))
  await mkdir(path.dirname(output), { recursive: true })
  await cp(converted ? webImage(converted) : path.join(mobilePacks, id, icon), output)
}
const wancisou = path.join(destination, 'wancisou')
await mkdir(wancisou)
const wancisouSource = path.join(root, 'assets/app-packs/wancisou')
for (const file of ['app.js', 'styles.css', 'icon.webp', 'cover.svg']) {
  await cp(path.join(wancisouSource, file), path.join(wancisou, file))
}
const { version: wancisouVersion } = JSON.parse(await readFile(path.join(wancisouSource, 'manifest.json'), 'utf8'))
await writeFile(path.join(wancisou, 'version.json'), JSON.stringify({ version: wancisouVersion }))
const wancisouHtml = await readFile(path.join(wancisouSource, 'index.html'), 'utf8')
if (!wancisouHtml.includes('<head>')) throw new Error('Missing head: wancisou')
const wancisouPolicy = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; connect-src https://search.xiangban.xyz; base-uri 'none'; form-action 'none'"
await writeFile(path.join(wancisou, 'index.html'), wancisouHtml
  .replace('<head>', `<head><meta http-equiv="Content-Security-Policy" content="${wancisouPolicy}">`)
  .replace('</head>', '<link rel="stylesheet" href="embedded.css"></head>'))
await writeFile(path.join(wancisou, 'embedded.css'), '.masthead { display: none; }\n')
console.log(`Synced ${ids.length} built-in games, mobile plaza icons and the 万磁搜 web app`)
