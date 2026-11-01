/** Strict offline v1 archive reader. Limits are checked while inflating, not only from ZIP metadata. */
const ZIP_MAX = 5 * 1024 * 1024, FILE_MAX = 1024 * 1024, TOTAL_MAX = 4 * 1024 * 1024
const allowed = /\.(html|js|css|json|svg|md|txt)$/i
const builtin = new Set(['falling-blocks','merge-2048','block-puzzle','water-sort','snake-arena','ink-blade','dusk-glider','pro-cube','jade-gomoku'])
export function sandboxUrl(value: string, forumOrigin: string): string {
  const url = new URL(value)
  if (!['https:', 'http:'].includes(url.protocol) || url.username || url.password || url.origin === forumOrigin || (url.protocol !== 'https:' && !['localhost','127.0.0.1','[::1]'].includes(url.hostname))) throw new Error('试玩宿主必须使用独立的 HTTPS 域名（本地可用不同端口）')
  return url.href
}
export function validateDownload(pack: { id:number; kind:string; status:string; trust_level:number; versions:{version:number;sha256:string;size:number}[] }, info: {pack_id:number;kind:string;version:number;sha256:string;size:number;trust_level:number}, version:number) {
  const v = pack.versions.find(v => v.version === version)
  if (pack.id !== info.pack_id || pack.kind !== 'game' || info.kind !== 'game' || pack.status !== 'published' || ![0,1].includes(pack.trust_level) || ![0,1].includes(info.trust_level) || version <= 0 || version !== info.version || !v || !/^[a-f0-9]{64}$/i.test(info.sha256) || info.sha256.toLowerCase() !== v.sha256.toLowerCase() || info.size !== v.size || info.size < 1 || info.size > ZIP_MAX) throw new Error('游戏已下架、未审核或版本校验信息不匹配')
}
function pathOk(path:string) { const parts = path.split('/'); return parts.length <= 4 && parts.every(s => /^[A-Za-z0-9_][A-Za-z0-9._-]*$/.test(s)) && parts[0] !== '__MACOSX' && allowed.test(path) && !(parts[0]?.toLowerCase() === 'manifest.json' && parts.length > 1) }
async function inflate(data:Uint8Array, limit:number): Promise<Uint8Array> {
  const reader = new Blob([data as BlobPart]).stream().pipeThrough(new DecompressionStream('deflate-raw')).getReader()
  const chunks:Uint8Array[] = []; let total = 0
  try { while (true) { const {done,value} = await reader.read(); if (done) break; total += value.length; if (total > limit) throw new Error('游戏包展开后过大'); chunks.push(value) } } finally { await reader.cancel().catch(() => {}) }
  const result = new Uint8Array(total); let offset = 0; for (const chunk of chunks) { result.set(chunk,offset); offset += chunk.length } return result
}
export async function readGameZip(bytes:Uint8Array, size:number, sha256:string): Promise<Record<string,string>> {
  if (bytes.length !== size || size < 1 || size > ZIP_MAX) throw new Error('游戏包大小不匹配')
  const digest = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes as BufferSource)), b => b.toString(16).padStart(2,'0')).join('')
  if (digest !== sha256.toLowerCase()) throw new Error('游戏包 SHA-256 校验失败')
  const d = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength); const u16 = (n:number) => d.getUint16(n,true), u32 = (n:number) => d.getUint32(n,true)
  let end = -1; for (let i = bytes.length - 22; i >= Math.max(0,bytes.length - 65557); i--) if (u32(i) === 0x06054b50 && i + 22 + u16(i+20) === bytes.length) { end=i; break }
  if (end < 0 || u16(end+4) || u16(end+6) || u16(end+8) !== u16(end+10) || u16(end+10) > 161 || u32(end+16) + u32(end+12) !== end) throw new Error('不支持的 ZIP 格式')
  let offset = u32(end+16), total=0; const count=u16(end+10), entries:{name:string;method:number;compressed:number;size:number;local:number}[] = [], seen = new Set<string>(), decoder = new TextDecoder('utf-8',{fatal:true})
  for (let i=0;i<count;i++) {
    if (offset+46 > end || u32(offset)!==0x02014b50) throw new Error('ZIP 目录损坏')
    const flags=u16(offset+8), method=u16(offset+10), compressed=u32(offset+20), length=u32(offset+24), nl=u16(offset+28), extra=u16(offset+30), comment=u16(offset+32), local=u32(offset+42), next=offset+46+nl+extra+comment
    if (next>end || flags & 1 || ![0,8].includes(method)) throw new Error('ZIP 条目不受支持')
    const name=decoder.decode(bytes.subarray(offset+46,offset+46+nl)); offset=next
    if (name.endsWith('/')) { if (!pathOk(name.slice(0,-1)+'/check.txt')) throw new Error('游戏路径无效'); continue }
    if (!pathOk(name) || seen.has(name.toLowerCase()) || entries.length>=41 || length > (name==='manifest.json'?65536:FILE_MAX)) throw new Error('游戏路径、文件数或大小不受支持')
    seen.add(name.toLowerCase()); total+=length; if (total>TOTAL_MAX+65536) throw new Error('游戏包展开后过大')
    entries.push({name,method,compressed,size:length,local})
  }
  if (offset!==end || !seen.has('manifest.json') || !seen.has('index.html') || [...seen].some(p => [...seen].some(q => q.startsWith(p+'/')))) throw new Error('游戏包缺少入口或路径冲突')
  if (entries.filter(e => e.name !== 'manifest.json').reduce((sum,e) => sum + e.size,0) > TOTAL_MAX) throw new Error('游戏文件合计超过 4 MB')
  const files:Record<string,string> = Object.create(null)
  for (const e of entries) {
    if (e.local+30>end || u32(e.local)!==0x04034b50 || u16(e.local+8)!==e.method) throw new Error('ZIP 文件头损坏')
    const start=e.local+30+u16(e.local+26)+u16(e.local+28)
    if (start+e.compressed>u32(end+16)) throw new Error('ZIP 文件范围无效')
    const raw=bytes.subarray(start,start+e.compressed), data=e.method===0?raw:await inflate(raw,e.size)
    if (data.length!==e.size) throw new Error('ZIP 文件大小不匹配')
    const text=decoder.decode(data); if (text.includes('\0')) throw new Error('游戏文件必须是 UTF-8 文本'); files[e.name]=text
  }
  let manifest:Record<string,unknown>; try { manifest=JSON.parse(files['manifest.json']!) } catch { throw new Error('游戏清单无效') }
  if (!manifest || manifest.schemaVersion!==1 || manifest.kind!=='game' || typeof manifest.id!=='string' || !/^[a-z0-9][a-z0-9-]{1,39}$/.test(manifest.id) || builtin.has(manifest.id) || typeof manifest.name!=='string' || !manifest.name.trim() || [...manifest.name.trim()].length>24 || typeof manifest.description!=='string' || !manifest.description.trim() || [...manifest.description.trim()].length>80 || !['space',undefined].includes(manifest.layer as string) || !['portrait','landscape',undefined].includes(manifest.orientation as string) || (manifest.save!==undefined && typeof manifest.save!=='boolean') || (manifest.capabilities!==undefined && (!Array.isArray(manifest.capabilities) || manifest.capabilities.length))) throw new Error('仅支持无系统权限的 v1 文本游戏')
  delete files['manifest.json']; return files
}
