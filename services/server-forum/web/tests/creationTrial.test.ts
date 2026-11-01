import { test } from 'node:test'
import { strict as assert } from 'node:assert'
import { sandboxUrl, validateDownload, readGameZip } from '../src/lib/creationTrial.ts'
const hash = 'a'.repeat(64)
const pack = { id:1, kind:'game', status:'published', trust_level:0, versions:[{version:1,sha256:hash,size:100}] }
const info = { pack_id:1,kind:'game',version:1,sha256:hash,size:100,trust_level:0 }
test('requires an independent secure sandbox origin', () => {
  assert.throws(() => sandboxUrl('https://forum.test/host', 'https://forum.test'))
  assert.throws(() => sandboxUrl('http://sandbox.test/', 'https://forum.test'))
  assert.equal(sandboxUrl('http://localhost:8101/', 'http://localhost:8100'), 'http://localhost:8101/')
})
test('rejects removed, unreviewed, changed and oversized releases', () => {
  validateDownload(pack,info,1)
  for (const bad of [{...pack,status:'removed'},{...pack,trust_level:2},{...pack,versions:[{...pack.versions[0]!,sha256:'b'.repeat(64)}]}]) assert.throws(() => validateDownload(bad,info,1))
  assert.throws(() => validateDownload(pack,{...info,version:2},1))
  assert.throws(() => validateDownload(pack,{...info,size:6*1024*1024},1))
})
test('rejects corrupt and unverified archives', async () => {
  await assert.rejects(readGameZip(new Uint8Array([1,2,3]),3,hash), /SHA-256/)
  await assert.rejects(readGameZip(new Uint8Array([1,2,3]),4,hash), /大小/)
})

test('accepts a valid v1 text archive and rejects traversal, duplicates and permission manifests', async () => {
  const { default: JSZip } = await import('jszip')
  async function archive(extra: Record<string,string>, manifest = {schemaVersion:1,kind:'game',id:'test-game',name:'游戏',description:'测试'}) {
    const zip = new JSZip(); zip.file('manifest.json',JSON.stringify(manifest)); zip.file('index.html','<h1>游戏</h1>')
    for (const [path,content] of Object.entries(extra)) zip.file(path,content,{createFolders:false})
    const bytes = await zip.generateAsync({type:'uint8array',compression:'DEFLATE'})
    const digest = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('')
    return readGameZip(bytes,bytes.length,digest)
  }
  assert.equal((await archive({'game.js':'console.log(1)'}))['game.js'],'console.log(1)')
  await assert.rejects(archive({'../escape.js':'x'}), /路径/)
  await assert.rejects(archive({'INDEX.HTML':'x'}), /路径/)
  await assert.rejects(archive({}, {schemaVersion:1,kind:'game',id:'test-game',name:'游戏',description:'测试',capabilities:['network']} as never), /权限/)
})
