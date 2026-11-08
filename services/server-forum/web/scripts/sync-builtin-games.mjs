// Sync trusted, versioned desktop game assets before the forum dev/build step.
import { cp, mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
const root = fileURLToPath(new URL('../../../../', import.meta.url))
const source = path.join(root, 'client/xiaoban-desktop/public/game-packs')
const destination = fileURLToPath(new URL('../public/builtin-games', import.meta.url))
const ids = ['dusk-glider','ink-blade','jade-gomoku','pro-cube','snake-arena','block-puzzle','merge-2048','water-sort','falling-blocks']
await rm(destination, {recursive:true, force:true}); await mkdir(destination, {recursive:true})
for (const id of ids) {
  const src = path.join(source,id), dest = path.join(destination,id)
  const manifest = JSON.parse(await readFile(path.join(src,'manifest.json'),'utf8'))
  if (manifest.id !== id || manifest.entry !== 'index.html' || (manifest.runtimeProtocol !== 1 && manifest.runtimeProtocol !== 2)) throw new Error(`Invalid built-in game manifest: ${id}`)
  await cp(src,dest,{recursive:true})
  const entry = path.join(dest,'index.html'), html = await readFile(entry,'utf8')
  if (!/<head(?:\s[^>]*)?>/i.test(html)) throw new Error(`Missing head: ${id}`)
  // Scripts are official bundled files only. The sandbox supplies an opaque origin.
  const csp = `<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; media-src 'self' data: blob:; font-src 'self' data:; connect-src 'none'; object-src 'none'; frame-src 'none'; form-action 'none'; base-uri 'none'; navigate-to 'none'">`
  await writeFile(entry,html.replace(/<head(?:\s[^>]*)?>/i, match => `${match}${csp}`))
}
console.log(`Synced ${ids.length} built-in games`)
