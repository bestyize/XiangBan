import {readFile,writeFile} from 'node:fs/promises'
import {createHash} from 'node:crypto'
import assert from 'node:assert/strict'
const directory=new URL('../public/wallpapers/builtin/',import.meta.url)
const catalogUrl=new URL('catalog.json',directory)
const catalog=JSON.parse(await readFile(catalogUrl,'utf8')),write=process.argv.includes('--write')
for(const item of catalog.items){
 for(const resource of [item.resource,item.cover,...Object.values(item.assets??{})].filter(Boolean)){
  assert.ok(resource.url.startsWith('builtin://'))
  const name=resource.url.slice(10);assert.match(name,/^[a-zA-Z0-9_.-]+$/)
  const url=new URL(name,directory)
  let data=await readFile(url)
  if(write&&/\.(js|svg|html|json)$/.test(name)){data=Buffer.from(data.toString('utf8').replace(/\r\n/g,'\n'));await writeFile(url,data)}
  const sha256=createHash('sha256').update(data).digest('hex')
  if(write){resource.bytes=data.length;resource.sha256=sha256}
  else{assert.equal(resource.bytes,data.length,`${name} bytes`);assert.equal(resource.sha256,sha256,`${name} sha256`)}
 }
}
if(write)await writeFile(catalogUrl,JSON.stringify(catalog,null,2)+'\n')
console.log(`PASS: ${catalog.items.length} built-in wallpaper definitions match actual asset bytes and SHA-256.`)
