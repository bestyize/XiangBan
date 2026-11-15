#!/usr/bin/env node
// Content-addressed, read-through storage for forum images. Public reads are
// re-authorized against the forum so drafts and withdrawn assets stay private.
import { createServer } from 'node:http'
import { createHash, createHmac, randomUUID, timingSafeEqual } from 'node:crypto'
import { createReadStream } from 'node:fs'
import { mkdir, readFile, stat, writeFile, link, unlink, rename } from 'node:fs/promises'
import { join } from 'node:path'
import { pathToFileURL } from 'node:url'

const MAX_IMAGE_BYTES = 10 * 1024 * 1024
const MAX_FILE_BYTES = 32 * 1024 * 1024
const UUID = '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}'
const ROUTE = new RegExp(`^/(forum-content|internal/forum-image)/(images|discover/(?:frontier-assets|article-assets)|open-source/assets)/(${UUID})$`)
const FILE_ROUTE = new RegExp(`^/(forum-files|internal/forum-file)/(${UUID})$`)
const CONTACT_ROUTE = /^\/forum-content\/(contact\/qr)\/(wechat-official|qq-group|wechat-group)$/
const MIME_EXT = { 'image/png':'png', 'image/jpeg':'jpg', 'image/webp':'webp', 'image/gif':'gif' }
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex')
const md5 = bytes => createHash('md5').update(bytes).digest('hex')

function sniff(bytes) {
  if (bytes.subarray(0,8).equals(Buffer.from('89504e470d0a1a0a','hex'))) return 'image/png'
  if (bytes.subarray(0,3).equals(Buffer.from('ffd8ff','hex'))) return 'image/jpeg'
  if (bytes.subarray(0,6).equals(Buffer.from('GIF87a')) || bytes.subarray(0,6).equals(Buffer.from('GIF89a'))) return 'image/gif'
  if (bytes.subarray(0,4).equals(Buffer.from('RIFF')) && bytes.subarray(8,12).equals(Buffer.from('WEBP'))) return 'image/webp'
  return null
}
function fail(res, status) {
  if (!res.headersSent) res.writeHead(status, { 'Cache-Control':'no-store', 'Access-Control-Allow-Origin':'*', 'X-Content-Type-Options':'nosniff' })
  res.end()
}
function statusError(status) { return Object.assign(new Error(`resource error ${status}`), { status }) }
async function boundedBody(stream, limit = MAX_IMAGE_BYTES) {
  let size = 0
  const chunks = []
  for await (const chunk of stream) {
    size += chunk.length
    if (size > limit) throw statusError(413)
    chunks.push(chunk)
  }
  if (!size) throw statusError(400)
  return Buffer.concat(chunks, size)
}

export function createForumImageServer({ root, origin, secret = Buffer.alloc(0), timeoutMs = 20000 }) {
  const inflight = new Map()
  let activeUploads = 0
  const keyPath = (kind,id) => join(root,'index',...kind.split('/'),`${id}.json`)
  const objectPath = object => join(root,'objects',object)
  const originUrl = (kind,id) => new URL(`/_resource-origin/api/${kind}/${id}`,origin)
  const fetchOrigin = (kind,id,method) => fetch(originUrl(kind,id), {
    method, redirect:'error', headers:{'Accept-Encoding':'identity'}, signal:AbortSignal.timeout(timeoutMs),
  })
  function receipt(kind,id,item) {
    const object_key = `forum-content/objects/${item.object}`
    const message = `RECEIPT\n${kind}/${id}\n${item.sha256}\n${item.size}\n${object_key}`
    return { object_key, sha256:item.sha256, size:item.size, signature:createHmac('sha256',secret).update(message).digest('hex') }
  }

  async function record(kind,id) {
    try {
      const item = JSON.parse(await readFile(keyPath(kind,id),'utf8'))
      if (!/^[a-f0-9]{32}\.(?:png|jpg|webp|gif)$/.test(item.object) ||
          !/^[a-f0-9]{64}$/.test(item.sha256) || !Number.isSafeInteger(item.size) || item.size < 1 || item.size > MAX_IMAGE_BYTES ||
          MIME_EXT[item.mime] !== item.object.split('.').at(-1)) return null
      const meta = await stat(objectPath(item.object))
      return meta.isFile() && meta.size === item.size ? item : null
    } catch { return null }
  }

  async function fileRecord(id) {
    try {
      const item = JSON.parse(await readFile(keyPath('attachments',id),'utf8'))
      if (!/^[a-f0-9]{32}\.[a-z0-9]{1,8}$/.test(item.object) || !/^[a-f0-9]{64}$/.test(item.sha256) ||
          !Number.isSafeInteger(item.size) || item.size < 1 || item.size > MAX_FILE_BYTES || item.mime !== 'application/octet-stream') return null
      const meta = await stat(objectPath(item.object))
      return meta.isFile() && meta.size === item.size ? item : null
    } catch { return null }
  }

  async function persist(kind,id,bytes,item,existingRecord,mutable = false) {
    const old = await existingRecord()
    if (!mutable && old && old.sha256 !== item.sha256) throw statusError(409)
    await mkdir(join(root,'objects'),{recursive:true,mode:0o750})
    const target = objectPath(item.object), tmp = join(root,'objects',`.${randomUUID()}.tmp`)
    try {
      await writeFile(tmp,bytes,{flag:'wx',mode:0o640})
      try { await link(tmp,target) } catch (err) {
        if (err.code !== 'EEXIST') throw err
        const existing = await readFile(target)
        if (existing.length !== item.size || sha256(existing) !== item.sha256) throw statusError(409)
      }
    } finally { await unlink(tmp).catch(()=>{}) }
    const index = keyPath(kind,id), parent = join(root,'index',...kind.split('/'))
    await mkdir(parent,{recursive:true,mode:0o750})
    const indexTmp = join(parent,`.${randomUUID()}.tmp`)
    try {
      await writeFile(indexTmp,JSON.stringify(item)+'\n',{flag:'wx',mode:0o640})
      if (mutable) await rename(indexTmp,index)
      else try { await link(indexTmp,index) } catch (err) {
          if (err.code !== 'EEXIST') throw err
          const existing = await existingRecord()
          if (!existing || existing.sha256 !== item.sha256) throw statusError(409)
        }
    } finally { await unlink(indexTmp).catch(()=>{}) }
    return item
  }

  async function store(kind,id,bytes,expectedMime,mutable = false) {
    const mime = sniff(bytes)
    if (!mime || (expectedMime && expectedMime.split(';')[0].trim().toLowerCase() !== mime)) throw statusError(415)
    const item = { object:`${md5(bytes)}.${MIME_EXT[mime]}`, sha256:sha256(bytes), size:bytes.length, mime }
    return persist(kind,id,bytes,item,()=>record(kind,id),mutable)
  }

  async function storeFile(id,bytes,ext) {
    const item = { object:`${md5(bytes)}.${ext}`, sha256:sha256(bytes), size:bytes.length, mime:'application/octet-stream' }
    return persist('attachments',id,bytes,item,()=>fileRecord(id))
  }

  async function fill(kind,id,expectedHash = null) {
    const key = `${kind}/${id}/${expectedHash || ''}`
    if (inflight.has(key)) return inflight.get(key)
    const work = (async () => {
      const existing = await record(kind,id)
      if (existing && (!expectedHash || existing.sha256 === expectedHash)) return existing
      const response = await fetchOrigin(kind,id,'GET')
      if (response.status === 403 || response.status === 404) throw statusError(404)
      if (!response.ok) throw statusError(502)
      const declared = Number(response.headers.get('content-length'))
      if (Number.isFinite(declared) && declared > MAX_IMAGE_BYTES) throw statusError(413)
      const bytes = await boundedBody(response.body)
      if (declared && declared !== bytes.length) throw statusError(502)
      if (expectedHash && sha256(bytes) !== expectedHash) throw statusError(502)
      return store(kind,id,bytes,response.headers.get('content-type'),Boolean(expectedHash))
    })()
    inflight.set(key,work)
    try { return await work } finally { if (inflight.get(key) === work) inflight.delete(key) }
  }

  async function ingest(req,res,kind,id) {
    if (req.method !== 'POST' || secret.length < 32) return fail(res,403)
    const expected = String(req.headers['x-content-sha256'] || '')
    const expires = Number(req.headers['x-expiration'])
    const size = Number(req.headers['content-length'])
    const signature = String(req.headers['x-signature'] || '')
    const now = Math.floor(Date.now()/1000)
    if (!/^[a-f0-9]{64}$/.test(expected) || !/^[a-f0-9]{64}$/.test(signature) ||
        !Number.isSafeInteger(size) || size < 1 || size > MAX_IMAGE_BYTES || !Number.isSafeInteger(expires) || expires < now || expires > now+300) return fail(res,403)
    const message = `POST\n${kind}/${id}\n${expected}\n${size}\n${expires}`
    const mac = createHmac('sha256',secret).update(message).digest()
    const supplied = Buffer.from(signature,'hex')
    if (supplied.length !== mac.length || !timingSafeEqual(supplied,mac)) return fail(res,403)
    if (activeUploads >= 4) return fail(res,503)
    activeUploads++
    try {
      const bytes = await boundedBody(req)
      if (bytes.length !== size || sha256(bytes) !== expected) return fail(res,422)
      const item = await store(kind,id,bytes,req.headers['content-type'])
      res.writeHead(201,{'Content-Type':'application/json','Cache-Control':'no-store','Access-Control-Allow-Origin':'*'})
      res.end(JSON.stringify(receipt(kind,id,item)))
    } finally { activeUploads-- }
  }

  async function ingestFile(req,res,id) {
    if (req.method !== 'POST' || secret.length < 32) return fail(res,403)
    const expected = String(req.headers['x-content-sha256'] || ''), signature = String(req.headers['x-signature'] || '')
    const expires = Number(req.headers['x-expiration']), size = Number(req.headers['content-length'])
    const ext = String(req.headers['x-file-ext'] || ''), now = Math.floor(Date.now()/1000)
    if (!/^[a-f0-9]{64}$/.test(expected) || !/^[a-f0-9]{64}$/.test(signature) || !/^[a-z0-9]{1,8}$/.test(ext) ||
        !Number.isSafeInteger(size) || size < 1 || size > MAX_FILE_BYTES || !Number.isSafeInteger(expires) || expires < now || expires > now+300) return fail(res,403)
    const mac = createHmac('sha256',secret).update(`POST\nattachments/${id}\n${expected}\n${size}\n${expires}\n${ext}`).digest()
    const supplied = Buffer.from(signature,'hex')
    if (supplied.length !== mac.length || !timingSafeEqual(supplied,mac)) return fail(res,403)
    if (activeUploads >= 4) return fail(res,503)
    activeUploads++
    try {
      const bytes = await boundedBody(req,MAX_FILE_BYTES)
      if (bytes.length !== size || sha256(bytes) !== expected) return fail(res,422)
      const item = await storeFile(id,bytes,ext)
      res.writeHead(201,{'Content-Type':'application/json','Cache-Control':'no-store','Access-Control-Allow-Origin':'*'})
      res.end(JSON.stringify(receipt('attachments',id,item)))
    } finally { activeUploads-- }
  }

  async function downloadFile(req,res,id,params) {
    if (!['GET','HEAD'].includes(req.method) || secret.length < 32) return fail(res,403)
    const expires = Number(params.get('exp')), signature = params.get('sig') || ''
    if (!Number.isSafeInteger(expires) || expires < Math.floor(Date.now()/1000) || !/^[a-f0-9]{64}$/.test(signature)) return fail(res,403)
    const mac = createHmac('sha256',secret).update(`GET\nforum-files/${id}\n${expires}`).digest(), supplied = Buffer.from(signature,'hex')
    if (supplied.length !== mac.length || !timingSafeEqual(supplied,mac)) return fail(res,403)
    const item = await fileRecord(id)
    if (!item) return fail(res,404)
    let start=0, end=item.size-1, partial=false
    if (req.headers.range) {
      const m = /^bytes=(\d*)-(\d*)$/.exec(req.headers.range)
      if (!m || (!m[1] && !m[2])) return fail(res,416)
      if (!m[1]) { const suffix=Number(m[2]); if (!Number.isSafeInteger(suffix) || suffix<1) return fail(res,416); start=Math.max(0,item.size-suffix) }
      else { start=Number(m[1]); if (m[2]) end=Math.min(end,Number(m[2])) }
      if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start>end || start>=item.size) return fail(res,416)
      partial=true
    }
    const rawName = (params.get('name') || 'file').slice(0,512).replace(/[\x00-\x1f\x7f/\\]/g,'_')
    const name = encodeURIComponent(rawName).replace(/['()*]/g,c=>`%${c.charCodeAt(0).toString(16).toUpperCase()}`)
    const headers = {'Content-Type':'application/octet-stream','Content-Length':end-start+1,'Cache-Control':'private, no-store',
      'Content-Disposition':`attachment; filename="file"; filename*=UTF-8''${name}`,'Accept-Ranges':'bytes',
      'Access-Control-Allow-Origin':'*','Access-Control-Expose-Headers':'Content-Disposition, Content-Range, Accept-Ranges','X-Content-Type-Options':'nosniff'}
    if (partial) headers['Content-Range'] = `bytes ${start}-${end}/${item.size}`
    res.writeHead(partial?206:200,headers)
    if (req.method==='HEAD') return res.end()
    createReadStream(objectPath(item.object),{start,end}).on('error',()=>res.destroy()).pipe(res)
  }

  return createServer(async (req,res) => {
    try {
      const path = new URL(req.url,'http://localhost')
      const file = FILE_ROUTE.exec(path.pathname)
      if (file) {
        if (file[1] === 'internal/forum-file') {
          if (path.search) return fail(res,404)
          return await ingestFile(req,res,file[2])
        }
        return await downloadFile(req,res,file[2],path.searchParams)
      }
      const contact = CONTACT_ROUTE.exec(path.pathname)
      const m = ROUTE.exec(path.pathname)
      if (contact) {
        if (path.search || !['GET','HEAD'].includes(req.method)) return fail(res,404)
        const [,kind,id] = contact, authorized = await fetchOrigin(kind,id,'HEAD')
        if (authorized.status === 403 || authorized.status === 404) return fail(res,404)
        const expected = authorized.headers.get('x-resource-sha256') || ''
        if (!authorized.ok || !/^[a-f0-9]{64}$/.test(expected)) return fail(res,502)
        const cached = await record(kind,id)
        const item = cached?.sha256 === expected ? cached : await fill(kind,id,expected)
        res.writeHead(200,{'Content-Type':item.mime,'Content-Length':item.size,'Cache-Control':'no-store',
          'Access-Control-Allow-Origin':'*','X-Content-Type-Options':'nosniff','Content-Security-Policy':"default-src 'none'; sandbox"})
        if (req.method === 'HEAD') return res.end()
        createReadStream(objectPath(item.object)).on('error',()=>res.destroy()).pipe(res)
        return
      }
      if (!m || path.search) return fail(res,404)
      const [,route,kind,id] = m
      if (route === 'internal/forum-image') return await ingest(req,res,kind,id)
      if (!['GET','HEAD'].includes(req.method)) return fail(res,405)
      const authorized = await fetchOrigin(kind,id,'HEAD')
      if (authorized.status === 403 || authorized.status === 404) return fail(res,404)
      if (!authorized.ok) return fail(res,502)
      const item = await record(kind,id) || await fill(kind,id)
      res.writeHead(200,{'Content-Type':item.mime,'Content-Length':item.size,'Cache-Control':'no-store',
        'Access-Control-Allow-Origin':'*','X-Content-Type-Options':'nosniff','Content-Security-Policy':"default-src 'none'; sandbox"})
      if (req.method === 'HEAD') return res.end()
      createReadStream(objectPath(item.object)).on('error',()=>res.destroy()).pipe(res)
    } catch (err) {
      if (!err.status) console.error('forum image error:',err.message)
      fail(res,err.status || 502)
    }
  })
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const credential = await readFile(join(process.env.CREDENTIALS_DIRECTORY,'environment'))
  const entry = credential.toString('utf8').split('\0').find(x=>x.startsWith('RESOURCE_SECRET='))
  if (!entry || entry.slice('RESOURCE_SECRET='.length).length < 32) throw new Error('missing resource secret')
  const server = createForumImageServer({root:'/srv/xiangban-resource/data/forum-content',origin:'https://chat.xiangban.xyz',secret:Buffer.from(entry.slice('RESOURCE_SECRET='.length))})
  server.listen(55021,'127.0.0.1',()=>console.log('forum image storage listening on 127.0.0.1:55021'))
}
