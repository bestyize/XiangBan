import JSZip from 'jszip'
import { parse } from 'yaml'
import type { DraftInput } from '@/api/editor'
export interface ImportRow { file: string; selected: boolean; data?: Omit<DraftInput, 'author_uid'>; error?: string; result?: string }
const MAX_ZIP = 100 * 1024 * 1024, MAX_EXPANDED = 80 * 1024 * 1024
function safePath(path: string) {
  path = path.replace(/^\.\//, '')
  if (!path || path.length > 300 || /[\\\0:%?#]/.test(path) || path.startsWith('/') || path.split('/').some(p => !p || p === '..' || p === '.')) throw new Error(`文件路径无效：${path}`)
  return path
}
export function parsePost(source: string) {
  source = source.replace(/^\uFEFF/, '').replace(/\r\n/g, '\n')
  const match = /^---\n([\s\S]*?)\n---(?:\n|$)([\s\S]*)$/.exec(source)
  if (!match) throw new Error('缺少 YAML 元信息，请使用示例格式')
  const meta = parse(match[1]!, { maxAliasCount: 0, uniqueKeys: true })
  if (!meta || typeof meta !== 'object' || Array.isArray(meta)) throw new Error('元信息需要是键值结构')
  for (const key of Object.keys(meta)) if (!['id', 'title', 'category', 'tags'].includes(key)) throw new Error(`不支持的元信息字段：${key}`)
  for (const key of ['id', 'title', 'category']) if (typeof meta[key] !== 'string' || !meta[key].trim()) throw new Error(`缺少 ${key}`)
  if (!/^[A-Za-z0-9_.-]{1,100}$/.test(meta.id)) throw new Error('id 需要是 1–100 位字母、数字、点、下划线或连字符')
  if (meta.tags !== undefined && (!Array.isArray(meta.tags) || meta.tags.some((x: unknown) => typeof x !== 'string'))) throw new Error('tags 需要是文字列表')
  return { external_id: meta.id as string, title: meta.title.trim() as string, category: meta.category.trim() as string, tags: (meta.tags || []) as string[], body: match[2]!.trim() }
}
function readBounded(file: JSZip.JSZipObject, max: number): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const chunks: Uint8Array[] = []; let size = 0
    const stream = (file as JSZip.JSZipObject & { internalStream(type: 'uint8array'): JSZip.JSZipStreamHelper<Uint8Array> }).internalStream('uint8array')
    stream.on('data', chunk => { size += chunk.length; if (size > max) { stream.pause(); reject(new Error(`解压文件过大：${file.name}`)); return } chunks.push(chunk) })
    stream.on('error', reject)
    stream.on('end', () => { const all = new Uint8Array(size); let offset = 0; for (const c of chunks) { all.set(c, offset); offset += c.length } resolve(all) })
    stream.resume()
  })
}
function base64(data: Uint8Array) { let raw = ''; for (let i = 0; i < data.length; i += 32768) raw += String.fromCharCode(...data.subarray(i, i + 32768)); return btoa(raw) }
export async function readBundle(file: File): Promise<ImportRow[]> {
  if (file.size > MAX_ZIP) throw new Error('压缩包不能超过 100 MB')
  if (/\.md$/i.test(file.name)) {
    if (file.size > 120000) throw new Error('Markdown 文件过大')
    try { return [{ file: file.name, selected: true, data: { ...parsePost(await file.text()), images: [] } }] }
    catch (e) { return [{ file: file.name, selected: false, error: String(e instanceof Error ? e.message : e) }] }
  }
  if (!/\.zip$/i.test(file.name)) throw new Error('请选择 ZIP 或 Markdown 文件')
  const zip = await JSZip.loadAsync(await file.arrayBuffer())
  const entries = Object.values(zip.files).filter(x => !x.dir && !x.name.startsWith('__MACOSX/') && !x.name.split('/').some(p => p.startsWith('.')))
  if (entries.length > 500) throw new Error('压缩包最多包含 500 个文件')
  const files = new Map<string, Uint8Array>(); let total = 0
  for (const entry of entries) {
    const original = (entry as JSZip.JSZipObject & { unsafeOriginalName?: string }).unsafeOriginalName || entry.name
    if (safePath(original) !== entry.name) throw new Error(`不支持的文件路径：${original}`)
    if (typeof entry.unixPermissions === 'number' && (entry.unixPermissions & 0o170000) === 0o120000) throw new Error('压缩包不能包含符号链接')
    if (!/\.(md|png|jpe?g|gif|webp)$/i.test(entry.name)) throw new Error(`不支持的文件：${entry.name}`)
    const bytes = await readBounded(entry, /\.md$/i.test(entry.name) ? 120000 : 5 * 1024 * 1024)
    total += bytes.length; if (total > MAX_EXPANDED) throw new Error('解压后总大小不能超过 80 MB')
    files.set(entry.name, bytes)
  }
  const markdowns = [...files.keys()].filter(x => /\.md$/i.test(x)).sort()
  if (!markdowns.length || markdowns.length > 50) throw new Error('每批需要包含 1–50 篇 Markdown 帖子')
  const rows: ImportRow[] = []; const ids = new Set<string>()
  for (const name of markdowns) {
    try {
      const data = parsePost(new TextDecoder('utf-8', { fatal: true }).decode(files.get(name)))
      if (ids.has(data.external_id)) throw new Error(`同一批次稿件 id 重复：${data.external_id}`)
      ids.add(data.external_id)
      const dir = name.includes('/') ? name.slice(0, name.lastIndexOf('/') + 1) : ''
      const images = [...files.entries()].filter(([p]) => p.startsWith(dir) && !/\.md$/i.test(p) && data.body.includes(p.slice(dir.length))).map(([p, bytes]) => ({ path: p.slice(dir.length), data_base64: base64(bytes) }))
      rows.push({ file: name, selected: true, data: { ...data, images } })
    } catch (e) { rows.push({ file: name, selected: false, error: e instanceof Error ? e.message : String(e) }) }
  }
  return rows
}
export const exampleMarkdown = `---\nid: ai-workflow-001\ntitle: 把重复工作整理成可复用的 AI 工作流\ncategory: "请替换为后台显示的板块 slug"\ntags:\n  - AI提效\n  - 工作流\n---\n\n这篇文章介绍一个可以直接照着做的方法。\n\n## 适合哪些场景\n\n在这里写下具体场景和操作步骤。\n\n![工作流示意图](images/workflow.png)\n`
