import { createHash } from 'node:crypto'
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { parseArgs } from 'node:util'
import JSZip from 'jszip'

const { values, positionals } = parseArgs({ options: { out: { type: 'string' } }, allowPositionals: true })
if (!values.out || !positionals.length) {
  throw new Error('用法：node scripts/stage-idea-runtime-packs.mjs --out <packages 目录> <通用 V2 ZIP>...')
}

const packs = []
const ids = new Set()
for (const file of positionals) {
  const bytes = await readFile(file)
  if (bytes.length > 100 * 1024 * 1024) throw new Error(`${file} 超过作品包体积上限`)
  const archive = await JSZip.loadAsync(bytes)
  const manifest = JSON.parse(await archive.file('manifest.json')?.async('string') ?? 'null')
  if (manifest?.runtimeProtocol !== 2 || manifest.schemaVersion !== 3 ||
      !['app', 'game'].includes(manifest.kind) || !/^[a-z0-9][a-z0-9-]{1,39}$/.test(manifest.id ?? '') ||
      !/^[a-zA-Z0-9.-]{1,32}$/.test(manifest.version ?? '') || !archive.file(manifest.entry ?? 'index.html')) {
    throw new Error(`${file} 不是有效的跨端 V2 作品包`)
  }
  if (ids.has(manifest.id)) throw new Error(`重复提供作品 ${manifest.id}`)
  ids.add(manifest.id)
  packs.push({ bytes, manifest })
}

await mkdir(values.out, { recursive: true })
for (const { bytes, manifest: { id, kind, version } } of packs) {
  const sha256 = createHash('sha256').update(bytes).digest('hex')
  const zip = join(values.out, `${id}-${sha256}.zip`)
  const descriptor = join(values.out, `${id}.json`)
  // Publish the immutable download before making its digest visible in the descriptor.
  await writeFile(`${zip}.${process.pid}.tmp`, bytes)
  await rename(`${zip}.${process.pid}.tmp`, zip)
  await writeFile(`${descriptor}.${process.pid}.tmp`, JSON.stringify({ kind, version, sha256, size: bytes.length }) + '\n')
  await rename(`${descriptor}.${process.pid}.tmp`, descriptor)
  console.log(`${id} · ${version} · ${bytes.length} bytes`)
}
