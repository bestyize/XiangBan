import http from 'node:http'
import {readFile} from 'node:fs/promises'
import {createHash} from 'node:crypto'
import {deflateSync} from 'node:zlib'
import {fileURLToPath} from 'node:url'
import path from 'node:path'

// Original procedural fixtures; no live service, account or user files are involved.
function crc32(bytes){let crc=0xffffffff;for(const b of bytes){crc^=b;for(let i=0;i<8;i++)crc=(crc>>>1)^((crc&1)?0xedb88320:0)}return (crc^0xffffffff)>>>0}
function chunk(type,bytes){const name=Buffer.from(type),size=Buffer.alloc(4),crc=Buffer.alloc(4);size.writeUInt32BE(bytes.length);crc.writeUInt32BE(crc32(Buffer.concat([name,bytes])));return Buffer.concat([size,name,bytes,crc])}
function png(){const width=320,height=200,raw=Buffer.alloc((width*3+1)*height);for(let y=0;y<height;y++){let p=y*(width*3+1);raw[p++]=0;for(let x=0;x<width;x++){raw[p++]=170+Math.round(x/width*40);raw[p++]=215+Math.round(y/height*20);raw[p++]=190+Math.round(x/width*25)}}const ihdr=Buffer.alloc(13);ihdr.writeUInt32BE(width);ihdr.writeUInt32BE(height,4);ihdr[8]=8;ihdr[9]=2;return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',ihdr),chunk('IDAT',deflateSync(raw)),chunk('IEND',Buffer.alloc(0))])}
export async function startWallpaperFixture(){
 const root=fileURLToPath(new URL('../public/wallpapers/builtin/',import.meta.url))
 const files=new Map([['fixture.png',{data:png(),mime:'image/png'}],['mint-orbits.js',{data:await readFile(path.join(root,'mint-orbits.js')),mime:'application/javascript'}],['sea-breeze.mp4',{data:await readFile(path.join(root,'sea-breeze.mp4')),mime:'video/mp4'}],['update.zip',{data:Buffer.from('test-only-update-not-an-installer'),mime:'application/zip'}]])
 let base='',offline=false
 const resource=name=>{const file=files.get(name);return {url:`${base}/${name}`,sha256:createHash('sha256').update(file.data).digest('hex'),bytes:file.data.length,mime:file.mime}}
 const server=http.createServer((request,response)=>{
  const url=new URL(request.url,base)
  if(url.pathname==='/offline'){offline=true;response.writeHead(204);response.end();return}
  if(offline){response.writeHead(503);response.end('fixture offline');return}
  const send=value=>{const data=Buffer.from(JSON.stringify(value));response.writeHead(200,{'Content-Type':'application/json','Content-Length':data.length});response.end(data)}
  if(url.pathname==='/catalog.json'){send({schemaVersion:1,revision:'test-1',items:[['image','fixture.png'],['video','sea-breeze.mp4'],['canvas','mint-orbits.js']].map(([kind,name])=>({id:`test-${kind}`,version:'1.0.0',title:`测试 ${kind}`,kind,resource:resource(name)})).concat([{id:'test-bad-hash',version:'1.0.0',title:'完整性失败测试',kind:'image',resource:{...resource('fixture.png'),sha256:'0'.repeat(64)}}])});return}
  if(url.pathname==='/updates.json'){send({schemaVersion:1,version:'99.0.0',notes:'本地接口检查，没有实际安装。',publishedAt:'2026-10-08T00:00:00Z',platforms:Object.fromEntries(['windows-x86_64','darwin-aarch64','darwin-x86_64'].map(target=>[target,resource('update.zip')]))});return}
  const file=files.get(url.pathname.slice(1));if(!file){response.writeHead(404);response.end();return}
  response.writeHead(200,{'Content-Type':file.mime,'Content-Length':file.data.length});response.end(file.data)
 })
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve))
 base=`http://127.0.0.1:${server.address().port}`
 return {base,catalogUrl:`${base}/catalog.json`,close:()=>new Promise(resolve=>server.close(resolve))}
}
if(process.argv.includes('--server')){
 const fixture=await startWallpaperFixture();console.log(fixture.catalogUrl)
 process.on('SIGINT',()=>fixture.close().then(()=>process.exit(0)))
}
