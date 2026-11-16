import { createHash } from 'node:crypto'
import { readdir, readFile } from 'node:fs/promises'
import { extname, join, relative, resolve, sep } from 'node:path'

const allowed = new Set(['.html', '.htm', '.js', '.mjs', '.css', '.json', '.svg', '.png', '.jpg', '.jpeg', '.webp', '.gif', '.mp3', '.ogg', '.wav', '.woff2', '.md', '.txt'])
const order = (a, b) => a < b ? -1 : a > b ? 1 : 0
const hash = bytes => createHash('sha256').update(bytes).digest()

export async function checkBuiltinDistribution() {
  const source = resolve('public/game-packs')
  const distribution = JSON.parse(await readFile(resolve('../../assets/game-packs/distribution.json'), 'utf8'))
  const catalog = JSON.parse(await readFile(join(source, 'catalog.json'), 'utf8'))
  if (distribution.version !== 1 || JSON.stringify(distribution.games.map(x => x.id)) !== JSON.stringify(catalog.games)) throw new Error('内置游戏分发清单与目录不一致')
  for (const game of distribution.games) {
    const root = join(source, game.id), files = []
    async function collect(dir) {
      for (const item of await readdir(dir, { withFileTypes:true })) {
        const path = join(dir, item.name)
        if (item.isSymbolicLink()) throw new Error(`内置游戏不能包含符号链接：${path}`)
        if (item.isDirectory()) await collect(path)
        else if (item.isFile() && !item.name.includes('.test.')) {
          if (!allowed.has(extname(item.name).toLowerCase())) throw new Error(`内置游戏文件类型不支持：${path}`)
          files.push(relative(root, path).split(sep).join('/'))
        }
      }
    }
    await collect(root)
    const digest = createHash('sha256')
    for (const name of files.sort(order)) digest.update(name).update('\0').update(hash(await readFile(join(root, name))))
    if (digest.digest('hex') !== game.sourceSha256) throw new Error(`内置游戏 ${game.id} 已更改，请重新生成并部署资源包`)
  }
}
