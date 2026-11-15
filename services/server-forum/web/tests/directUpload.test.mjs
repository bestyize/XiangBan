import { test } from 'node:test'
import assert from 'node:assert/strict'
import { directForumUpload } from '../src/api/directUpload.ts'

const id = '00112233-4455-6677-8899-aabbccddeeff'
const fail = (status, message) => Object.assign(new Error(message), { status })

for (const kind of ['image','attachment']) test(`${kind} uploads bytes directly and completes with the receipt`, async () => {
  let ticketBody, xhr, finalBody, progress
  const previous = globalThis.XMLHttpRequest
  class FakeXHR {
    upload = {}
    headers = {}
    status = 201
    responseText = JSON.stringify({object_key:'forum-content/objects/hash.png',sha256:'hash',size:3,signature:'receipt'})
    constructor() { xhr=this }
    open(method,url) { this.method=method; this.url=url }
    setRequestHeader(key,value) { this.headers[key]=value }
    send(file) { this.file=file; this.upload.onprogress?.({lengthComputable:true,loaded:2,total:3}); queueMicrotask(()=>this.onload()) }
  }
  globalThis.XMLHttpRequest = FakeXHR
  const file = new File(['abc'],kind==='image'?'a.png':'a.zip',{type:kind==='image'?'image/png':'application/zip'})
  const request = async (_method,path,body) => {
    if (path==='/resource-uploads/ticket') {
      ticketBody=body
      return {id,upload_url:`https://res.xiangban.xyz/${kind==='image'?'internal/forum-image/images':'internal/forum-file'}/${id}`,
        headers:{'Content-Type':kind==='image'?'image/png':'application/octet-stream','X-Content-Sha256':body.sha256,
          'X-Expiration':'2000000000','X-Signature':'a'.repeat(64),...(kind==='attachment'?{'X-File-Ext':'zip'}:{})}}
    }
    assert.equal(path,`/resource-uploads/${id}/complete`); finalBody=body; return {id}
  }
  try {
    assert.deepEqual(await directForumUpload(kind,file,request,fail,value=>{progress=value}),{id})
    assert.equal(ticketBody.size,3); assert.equal(xhr.file,file); assert.equal(xhr.method,'POST')
    assert.match(xhr.url,/^https:\/\/res\.xiangban\.xyz\//)
    assert.equal(xhr.headers.Authorization,undefined); assert.equal(progress,2/3)
    assert.equal(finalBody.signature,'receipt')
  } finally { globalThis.XMLHttpRequest=previous }
})

test('rejects a foreign upload host and never sends the file', async () => {
  let opened=false
  const file = new File(['abc'],'a.png',{type:'image/png'})
  const request = async (_method,_path,body) => ({id,upload_url:`https://evil.example/internal/forum-image/images/${id}`,
    headers:{'Content-Type':'image/png','X-Content-Sha256':body.sha256,'X-Expiration':'2000000000','X-Signature':'a'.repeat(64)}})
  const previous=globalThis.XMLHttpRequest
  globalThis.XMLHttpRequest=class { constructor(){ opened=true } }
  try { await assert.rejects(directForumUpload('image',file,request,fail),/上传地址无效/); assert.equal(opened,false) }
  finally { globalThis.XMLHttpRequest=previous }
})
