import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createServer } from 'node:http'
import { createHash, createHmac } from 'node:crypto'
import { mkdtemp, readFile, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { createForumImageServer } from './forum-image-cache.mjs'

const secret = Buffer.from('0123456789abcdef0123456789abcdef')
const id = '00000000-0000-4000-8000-000000000001'
const another = '00000000-0000-4000-8000-000000000002'
const png = Buffer.concat([Buffer.from('89504e470d0a1a0a','hex'),Buffer.from('test image')])
const sha = createHash('sha256').update(png).digest('hex')
const digest = createHash('md5').update(png).digest('hex')
const listen = server => new Promise(resolve => server.listen(0,'127.0.0.1',()=>resolve(`http://127.0.0.1:${server.address().port}`)))
const close = server => new Promise(resolve => server.close(resolve))

test('published images are stored by MD5, shared and checked on every request', async () => {
  const dir = await mkdtemp(join(tmpdir(),'forum-image-test-'))
  let published = true, gets = 0
  const upstream = createServer((req,res) => {
    if (!published) { res.writeHead(404).end(); return }
    res.setHeader('Content-Type','image/png'); res.setHeader('Content-Length',png.length)
    if (req.method === 'GET') { gets++; res.end(png) } else res.end()
  })
  const origin = await listen(upstream)
  const app = createForumImageServer({root:dir,origin,secret})
  const base = await listen(app)
  try {
    const path = `/forum-content/discover/frontier-assets/${id}`
    const responses = await Promise.all([fetch(base+path),fetch(base+path)])
    assert.deepEqual(Buffer.from(await responses[0].arrayBuffer()),png)
    assert.deepEqual(Buffer.from(await responses[1].arrayBuffer()),png)
    assert.equal(gets,1)
    assert.deepEqual(await readFile(join(dir,'objects',`${digest}.png`)),png)
    const metadata = JSON.parse(await readFile(join(dir,'index','discover','frontier-assets',`${id}.json`),'utf8'))
    assert.equal(metadata.sha256,sha)
    published = false
    assert.equal((await fetch(base+path)).status,404)
    assert.equal((await fetch(base+'/forum-content/images/not-a-uuid')).status,404)
  } finally { await close(app); await close(upstream); await rm(dir,{recursive:true,force:true}) }
})

test('restricted discussion images recheck access even when bytes are cached', async () => {
  const dir = await mkdtemp(join(tmpdir(),'forum-restricted-image-test-'))
  let authorized = true, gets = 0
  const originPath = `/_resource-origin/api/images/${id}/view?key=test-view`
  const upstream = createServer((req,res) => {
    if (!authorized || req.url !== originPath) return res.writeHead(404).end()
    res.setHeader('Content-Type','image/png'); res.setHeader('Content-Length',png.length)
    if (req.method === 'GET') { gets++; res.end(png) } else res.end()
  })
  const origin = await listen(upstream), app = createForumImageServer({root:dir,origin,secret}), base = await listen(app)
  const url = `${base}/forum-content/images/${id}`
  try {
    const allowed = await fetch(url+'?key=test-view')
    assert.equal(allowed.headers.get('Cache-Control'),'no-store')
    assert.deepEqual(Buffer.from(await allowed.arrayBuffer()),png)
    assert.equal(gets,1)
    assert.equal((await fetch(url)).status,404)
    assert.equal((await fetch(url+'?key=test-view',{method:'HEAD'})).status,200)
    authorized = false
    assert.equal((await fetch(url+'?key=test-view')).status,404)
    assert.equal(gets,1)
  } finally { await close(app); await close(upstream); await rm(dir,{recursive:true,force:true}) }
})

test('contact QR images update by content hash and disappear when hidden', async () => {
  const dir = await mkdtemp(join(tmpdir(),'forum-contact-test-'))
  let current = png, visible = true, gets = 0
  const upstream = createServer((req,res) => {
    if (!visible) return res.writeHead(404).end()
    res.setHeader('Content-Type','image/png')
    res.setHeader('Content-Length',current.length)
    res.setHeader('X-Resource-Sha256',createHash('sha256').update(current).digest('hex'))
    if (req.method === 'GET') { gets++; res.end(current) } else res.end()
  })
  const origin = await listen(upstream), app = createForumImageServer({root:dir,origin,secret}), base = await listen(app)
  const url = `${base}/forum-content/contact/qr/qq-group`
  try {
    assert.deepEqual(Buffer.from(await (await fetch(url)).arrayBuffer()),png)
    assert.deepEqual(Buffer.from(await (await fetch(url)).arrayBuffer()),png)
    assert.equal(gets,1)
    current = Buffer.concat([png,Buffer.from('new')])
    assert.deepEqual(Buffer.from(await (await fetch(url)).arrayBuffer()),current)
    assert.equal(gets,2)
    const hash = createHash('md5').update(current).digest('hex')
    assert.deepEqual(await readFile(join(dir,'objects',`${hash}.png`)),current)
    visible = false
    assert.equal((await fetch(url)).status,404)
    assert.equal((await fetch(base+'/forum-content/contact/qr/invalid')).status,404)
  } finally { await close(app); await close(upstream); await rm(dir,{recursive:true,force:true}) }
})

test('only a valid signed upload can store a draft, which remains private until publication', async () => {
  const dir = await mkdtemp(join(tmpdir(),'forum-image-test-'))
  let published = false, gets = 0
  const upstream = createServer((req,res)=>{
    if (!published) return res.writeHead(404).end()
    if (req.method === 'GET') gets++
    res.setHeader('Content-Type','image/png'); res.setHeader('Content-Length',png.length); res.end(req.method==='GET'?png:undefined)
  })
  const origin = await listen(upstream), app = createForumImageServer({root:dir,origin,secret}), base = await listen(app)
  const exp = Math.floor(Date.now()/1000)+60
  const message = `POST\ndiscover/article-assets/${another}\n${sha}\n${png.length}\n${exp}`
  const signature = createHmac('sha256',secret).update(message).digest('hex')
  const url = `${base}/internal/forum-image/discover/article-assets/${another}`
  const headers = {'Content-Type':'image/png','Content-Length':String(png.length),'X-Content-Sha256':sha,'X-Expiration':String(exp),'X-Signature':signature}
  try {
    assert.equal((await fetch(url,{method:'POST',headers:{...headers,'X-Signature':'0'.repeat(64)},body:png})).status,403)
    const stored = await fetch(url,{method:'POST',headers,body:png})
    assert.equal(stored.status,201)
    const receipt = await stored.json()
    const receiptMessage = `RECEIPT\ndiscover/article-assets/${another}\n${sha}\n${png.length}\n${receipt.object_key}`
    assert.equal(receipt.signature, createHmac('sha256',secret).update(receiptMessage).digest('hex'))
    assert.equal((await fetch(`${base}/forum-content/discover/article-assets/${another}`)).status,404)
    published = true
    const response = await fetch(`${base}/forum-content/discover/article-assets/${another}`)
    assert.equal(response.status,200)
    assert.deepEqual(Buffer.from(await response.arrayBuffer()),png)
    assert.equal(gets,0)
  } finally { await close(app); await close(upstream); await rm(dir,{recursive:true,force:true}) }
})

test('attachments require a signed upload and a signed, unexpired download', async () => {
  const dir = await mkdtemp(join(tmpdir(),'forum-file-test-'))
  const app = createForumImageServer({root:dir,origin:'http://127.0.0.1:1',secret})
  const base = await listen(app), bytes = Buffer.from('private attachment'), hash = createHash('sha256').update(bytes).digest('hex')
  const exp = Math.floor(Date.now()/1000)+60, ext = 'zip'
  const signature = createHmac('sha256',secret).update(`POST\nattachments/${id}\n${hash}\n${bytes.length}\n${exp}\n${ext}`).digest('hex')
  const headers = {'Content-Type':'application/octet-stream','Content-Length':String(bytes.length),'X-Content-Sha256':hash,
    'X-Expiration':String(exp),'X-File-Ext':ext,'X-Signature':signature}
  try {
    const upload = await fetch(`${base}/internal/forum-file/${id}`,{method:'POST',headers,body:bytes})
    assert.equal(upload.status,201)
    const receipt = await upload.json()
    assert.match(receipt.object_key,/^forum-content\/objects\/[a-f0-9]{32}\.zip$/)
    assert.equal(receipt.signature,createHmac('sha256',secret).update(`RECEIPT\nattachments/${id}\n${hash}\n${bytes.length}\n${receipt.object_key}`).digest('hex'))
    assert.equal((await fetch(`${base}/forum-files/${id}`)).status,403)
    const sig = createHmac('sha256',secret).update(`GET\nforum-files/${id}\n${exp}`).digest('hex')
    const url = `${base}/forum-files/${id}?exp=${exp}&sig=${sig}&name=${encodeURIComponent('资料.zip')}`
    const download = await fetch(url)
    assert.deepEqual(Buffer.from(await download.arrayBuffer()),bytes)
    assert.match(download.headers.get('content-disposition'),/filename\*=UTF-8''/)
    const part = await fetch(url,{headers:{Range:'bytes=2-5'}})
    assert.equal(part.status,206)
    assert.equal(part.headers.get('content-range'),`bytes 2-5/${bytes.length}`)
    assert.deepEqual(Buffer.from(await part.arrayBuffer()),bytes.subarray(2,6))
    assert.equal((await fetch(url.replace(sig,'0'.repeat(64)))).status,403)
    assert.equal((await fetch(url.replace(`exp=${exp}`,'exp=1'))).status,403)
  } finally { await close(app); await rm(dir,{recursive:true,force:true}) }
})
