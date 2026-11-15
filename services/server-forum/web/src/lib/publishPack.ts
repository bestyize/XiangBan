import JSZip from 'jszip'
import { fmtSize, KIND_LIMIT } from './content.ts'

export const WORK_TAGS = ['工具', '游戏', '3D模型'] as const
const htmlExtension = /\.html?$/i
const imageExtension = /\.(svg|png|jpe?g|webp|avif|gif|ico)$/i
const workExtensions = /\.(html?|m?js|css|json|svg|md|txt|xml|csv|map|atlas|glsl|png|jpe?g|webp|avif|gif|ico|mp3|ogg|wav|m4a|aac|mp4|webm|woff2?|ttf|otf|wasm|glb|gltf|bin|data|ktx2|basis)$/i
const manifestLimit = 64 * 1024
const htmlLimit = 50 * 1024 * 1024
const readyScript = `<script>
document.addEventListener('DOMContentLoaded', () => {
  requestAnimationFrame(() => window.xb?.ready?.());
}, { once: true });
</script>
`
const icon = `<svg xmlns="http://www.w3.org/2000/svg" width="192" height="192" viewBox="0 0 192 192">
  <rect width="192" height="192" rx="36" fill="#324475"/>
  <path d="M70 71 43 96l27 25m52-50 27 25-27 25m-17-57-18 64" fill="none" stroke="#e4ebff" stroke-width="10" stroke-linecap="round" stroke-linejoin="round"/>
</svg>`

type WorkKind = 'app' | 'game'
const categories = { app: ['tool', 'quiz'], game: ['action', 'puzzle', 'board', 'casual', 'story'] }
const inputs = ['touch', 'mouse', 'keyboard']
const orientations = ['portrait', 'landscape', 'any']
const capabilities = ['haptics', 'share', 'clipboard', 'screen', 'notify', 'profile', 'camera', 'microphone', 'contacts', 'files']

export interface PreviousPack {
  kind: string
  slug: string
  title: string
  version: number
  versions: { manifest: Record<string, unknown> }[]
}

export interface PreparedPack {
  file: File
  kind: string
  name: string
  description: string
  protocol: number
  converted: boolean
}

function isWorkKind(kind: unknown): kind is WorkKind { return kind === 'game' || kind === 'app' }
export function canPublishHtml(previous?: PreviousPack | null): boolean {
  return !previous || (isWorkKind(previous.kind) && /^[a-z0-9_-]{2,64}$/.test(previous.slug))
}

export function isPublishHtml(name: string): boolean { return htmlExtension.test(name) }

function checkSize(file: File, max: number, label: string) {
  if (file.size === 0) throw new Error(`${label}为空，请重新选择`)
  if (file.size > max) throw new Error(`${label}不能超过 ${fmtSize(max)}`)
}

function textField(manifest: Record<string, unknown>, key: string, max: number) {
  const value = manifest[key]
  const cleaned = typeof value === 'string' ? value.trim() : ''
  if (!cleaned || Array.from(cleaned).length > max) throw new Error(`manifest.json 的 ${key} 必填，最多 ${max} 字`)
  return cleaned
}

function checkList(value: unknown, options: string[], field: string, required: boolean) {
  if (value === undefined && !required) return
  if (!Array.isArray(value) || (required && !value.length) ||
    value.some(item => !options.includes(item)) || new Set(value).size !== value.length) {
    throw new Error(`manifest.json 的 ${field} 可选值为 ${options.join('、')}，不能重复`)
  }
}

function checkWorkFiles(zip: JSZip, manifest: Record<string, unknown>) {
  const paths = new Map<string, { path: string; dir: boolean }>()
  const files = Object.values(zip.files)
  if (files.filter(file => !file.dir).length > 3000) throw new Error('V2 作品最多包含 3000 个文件')
  for (const file of files) {
    const path = file.dir ? file.name.replace(/\/$/, '') : file.name
    const original = file.unsafeOriginalName
    const parts = path.split('/')
    if ((original !== undefined && original !== path) || parts.length > 8 ||
      parts.some(part => !/^[a-z0-9_@-][a-z0-9._@-]*$/i.test(part)) ||
      (!file.dir && !workExtensions.test(path))) throw new Error(`V2 作品不支持这个文件路径或格式：${original ?? path}`)
    if (typeof file.unixPermissions === 'number' && (file.unixPermissions & 0o170000) === 0o120000) {
      throw new Error('V2 作品不能包含符号链接')
    }
    for (let i = 1; i <= parts.length; i++) {
      const prefix = parts.slice(0, i).join('/')
      const key = prefix.toLowerCase()
      const dir = i !== parts.length || file.dir
      const seen = paths.get(key)
      if (seen && (seen.path !== prefix || seen.dir !== dir)) throw new Error(`V2 作品路径冲突：${prefix}`)
      paths.set(key, { path: prefix, dir })
    }
  }
  for (const [field, path, extensions] of [
    ['entry', manifest.entry === undefined ? 'index.html' : manifest.entry, htmlExtension],
    ['icon', manifest.icon, imageExtension],
    ['cover', manifest.cover, imageExtension],
  ] as const) {
    if (typeof path !== 'string' || !extensions.test(path) || !zip.file(path)) {
      throw new Error(`ZIP 缺少 manifest.json 指定的 ${field} 文件，或文件格式不正确`)
    }
  }
}

function readManifest(file: JSZip.JSZipObject): Promise<Record<string, unknown>> {
  return new Promise((resolve, reject) => {
    const stream = (file as JSZip.JSZipObject & {
      internalStream(type: 'uint8array'): JSZip.JSZipStreamHelper<Uint8Array>
    }).internalStream('uint8array')
    const decoder = new TextDecoder('utf-8', { fatal: true })
    let bytes = 0
    let source = ''
    const stop = (error: Error) => { stream.pause(); reject(error) }
    stream.on('data', chunk => {
      bytes += chunk.length
      if (bytes > manifestLimit) return stop(new Error('manifest.json 不能超过 64 KB'))
      try { source += decoder.decode(chunk, { stream: true }) } catch { stop(new Error('manifest.json 必须使用 UTF-8 编码')) }
    })
    stream.on('error', () => reject(new Error('无法解压 manifest.json')))
    stream.on('end', () => {
      try {
        const value: unknown = JSON.parse(source + decoder.decode())
        if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error()
        resolve(value as Record<string, unknown>)
      } catch { reject(new Error('manifest.json 必须是有效的 JSON 对象')) }
    })
    stream.resume()
  })
}

export async function inspectPackZip(file: File, previous?: PreviousPack | null): Promise<PreparedPack> {
  checkSize(file, KIND_LIMIT.app, 'ZIP 文件')
  let zip: JSZip
  try { zip = await JSZip.loadAsync(await file.arrayBuffer()) } catch { throw new Error('无法读取这个 ZIP 文件，请检查压缩包是否完整') }
  const entry = zip.file('manifest.json')
  if (entry?.unsafeOriginalName !== 'manifest.json') throw new Error('ZIP 根目录需要有 manifest.json')
  const manifest = await readManifest(entry)
  const { kind, id, schemaVersion, runtimeProtocol = 1 } = manifest
  if (typeof kind !== 'string' || !Object.hasOwn(KIND_LIMIT, kind)) throw new Error('manifest.json 的内容包类型无效')
  if (previous && (kind !== previous.kind || id !== previous.slug)) throw new Error('新版本的类型和作品 ID 必须与原版本一致')
  if (runtimeProtocol !== 1 && runtimeProtocol !== 2) throw new Error('manifest.json 的 runtimeProtocol 无效')
  const latest = previous?.versions[0]?.manifest
  if (runtimeProtocol === 1 && (kind === 'app' || (kind === 'game' && (!previous || latest?.runtimeProtocol === 2)))) {
    throw new Error('请选择符合 V2 规范的作品 ZIP')
  }
  checkSize(file, KIND_LIMIT[kind]!, '内容包')
  const v2 = runtimeProtocol === 2
  if (typeof id !== 'string' || !(v2 ? /^[a-z0-9_-]{2,64}$/ : /^[a-z0-9_-]{1,64}$/).test(id)) throw new Error('manifest.json 的作品 ID 无效')
  if (!/^[a-z0-9.-]+$/i.test(textField(manifest, 'version', 32))) throw new Error('manifest.json 的版本号无效')
  const name = textField(manifest, 'name', v2 ? 24 : 60)
  let description = ''
  if (v2) {
    if (schemaVersion !== 3 || !isWorkKind(kind)) throw new Error('V2 作品需要 schemaVersion 为 3，类型为 app 或 game')
    description = textField(manifest, 'description', 80)
    if (!categories[kind].includes(manifest.category as string)) throw new Error(`manifest.json 的 category 可选值为 ${categories[kind].join('、')}`)
    if (manifest.orientation !== undefined && !orientations.includes(manifest.orientation as string)) throw new Error('manifest.json 的 orientation 无效')
    if (manifest.layer !== undefined && manifest.layer !== 'desktop') throw new Error('manifest.json 的 layer 无效')
    checkList(manifest.input, inputs, 'input', true)
    checkList(manifest.capabilities, manifest.layer === 'desktop' ? [...capabilities, 'windows', 'pointer'] : capabilities, 'capabilities', false)
    checkWorkFiles(zip, manifest)
  } else if (typeof schemaVersion !== 'number' || !Number.isInteger(schemaVersion) || schemaVersion < 1) {
    throw new Error('manifest.json 的 schemaVersion 无效')
  }
  return { file, kind, name, description, protocol: runtimeProtocol, converted: false }
}

// Match the desktop and Android importers: preserve the prolog and register the V2
// ready handshake before the author's scripts. The input is never parsed into the page.
function insertReady(html: string) {
  const prolog = /[\t\n\f\r ]+|<!--[\s\S]*?-->|<(?:!doctype|html|head)(?=[\t\n\f\r />])(?:[^'"<>]|"[^"]*"|'[^']*')*>/iy
  let offset = html.startsWith('\uFEFF') ? 1 : 0
  prolog.lastIndex = offset
  while (prolog.test(html)) offset = prolog.lastIndex
  return html.slice(0, offset) + readyScript + html.slice(offset)
}

export async function convertPublishHtml(file: File, previous?: PreviousPack | null): Promise<PreparedPack> {
  if (!isPublishHtml(file.name)) throw new Error('请选择 .html 或 .htm 文件')
  if (!canPublishHtml(previous)) throw new Error('这个内容包的新版本需要上传 ZIP')
  checkSize(file, htmlLimit - readyScript.length, '单个 HTML 文件')
  let html: string
  try { html = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(await file.arrayBuffer()) } catch {
    throw new Error('请选择 UTF-8 编码的 HTML 文件')
  }
  if (!html.trim() || html.includes('\0')) throw new Error('请选择非空的 HTML 文件')

  const latest = previous?.versions[0]?.manifest
  const kind = previous?.kind === 'game' ? 'game' : 'app'
  const baseName = file.name.replace(htmlExtension, '')
  const cleanName = (typeof latest?.name === 'string' ? latest.name : previous?.title ?? baseName)
    .replace(/[\u0000-\u001f\u007f-\u009f\ufffe\uffff]/g, '').trim()
  const name = Array.from(cleanName).slice(0, 24).join('').trimEnd() || 'HTML 作品'
  const title = name.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  const cover = `<svg xmlns="http://www.w3.org/2000/svg" width="640" height="360" viewBox="0 0 640 360">
  <rect width="640" height="360" fill="#324475"/>
  <path d="M286 106 250 140l36 34m68-68 36 34-36 34m-22-78-24 88" fill="none" stroke="#e4ebff" stroke-width="12" stroke-linecap="round" stroke-linejoin="round"/>
  <text x="320" y="265" fill="#fff" font-family="sans-serif" font-size="23" text-anchor="middle">${title}</text>
</svg>`
  const id = previous?.slug ?? `html-${crypto.randomUUID().replaceAll('-', '')}`
  const description = typeof latest?.description === 'string' && latest.description.trim() && Array.from(latest.description).length <= 80
    ? latest.description : '从 HTML 导入的作品'
  const zip = new JSZip()
  zip.file('manifest.json', JSON.stringify({
    schemaVersion: 3, runtimeProtocol: 2, id, name, kind, description,
    version: `${previous ? previous.version + 1 : 1}.0.0`,
    category: categories[kind].includes(latest?.category as string) ? latest!.category : categories[kind][0],
    entry: 'index.html', icon: 'icon.svg', cover: 'cover.svg',
    orientation: orientations.includes(latest?.orientation as string) ? latest!.orientation : 'any',
    input: inputs, capabilities: [],
  }))
  zip.file('index.html', insertReady(html))
  zip.file('icon.svg', icon)
  zip.file('cover.svg', cover)
  const blob = await zip.generateAsync({ type: 'blob', compression: 'DEFLATE', compressionOptions: { level: 3 } })
  return { file: new File([blob], `${id}.zip`, { type: 'application/zip' }), kind, name, description, protocol: 2, converted: true }
}
