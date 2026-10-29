// Run against a disposable website database/schema, never a production website.
import {spawn} from 'node:child_process'
import {readFile,mkdir} from 'node:fs/promises'
import {fileURLToPath} from 'node:url'
import path from 'node:path'
import assert from 'node:assert/strict'
import {startWallpaperFixture} from './wallpaper-fixture.mjs'
const base=process.env.XIAOBAN_WEBSITE_TEST_URL
if(!base || !['localhost','127.0.0.1'].includes(new URL(base).hostname))throw new Error('Set XIAOBAN_WEBSITE_TEST_URL to an isolated localhost test website.')
const root=fileURLToPath(new URL('../',import.meta.url))
const binary=process.argv[2]??path.join(root,'src-tauri/target/release/xiaoban-desktop.exe')
const fixture=await startWallpaperFixture()
let cookie=''
async function request(route,body,method=body?'POST':'GET') {
 const response=await fetch(`${base}/api/v1${route}`,{method,headers:{...(cookie?{Cookie:cookie}:{}),...(body?{'Content-Type':'application/json'}:{})},body:body?JSON.stringify(body):undefined})
 assert.equal(response.status,200,await response.clone().text())
 if(response.headers.get('set-cookie'))cookie=response.headers.get('set-cookie').split(';')[0]
 return response.json()
}
try {
 await request('/admin/auth/login',{username:'admin',password:'123456'})
 const config=await request('/admin/desktop')
 // Deliberately retain the production origin: the client must use its configured local
 // website for that website's immutable uploads, including the generated video poster.
 config.siteUrl='https://wallpaper.xiangban.xyz'
 config.releases=[{version:'99.0.0',notes:'测试更新，不执行安装。',published:true,publishedAt:'2026-01-01T00:00:00Z',platforms:{'windows-x86_64':{url:'https://example.com/test-only.zip'},'darwin-aarch64':{url:'https://example.com/test-only.zip'}}}]
 config.announcements=[{id:'website-smoke',title:'网站联调公告',body:'验证公告接入与已读状态。',published:true,audience:['desktop','website'],startsAt:null,endsAt:null}]
 await request('/admin/desktop',config,'PUT')
 for(const [kind,name] of [['image','fixture.png'],['video','sea-breeze.mp4']]) {
  const media=await fetch(`${fixture.base}/${name}`).then(r=>r.blob())
  const form=new FormData();form.append('file',media,name)
  const uploaded=await fetch(`${base}/api/v1/admin/wallpapers/upload`,{method:'POST',headers:{Cookie:cookie},body:form})
  assert.equal(uploaded.status,200,await uploaded.clone().text())
  const {file}=await uploaded.json()
  await request('/admin/wallpapers/custom',{title:`网站联调 ${kind}`,media_type:kind,preview_url:file.url,regular_url:file.url,video_url:file.url,status:'published',category:'nature',width:320,height:200})
 }
 await mkdir(path.join(root,'artifacts'),{recursive:true})
 const report=path.join(root,`artifacts/website-smoke-${Date.now()}.json`)
 const child=spawn(binary,['--smoke-test','--autostart'],{windowsHide:true,env:{...process.env,XIAOBAN_SMOKE_REPORT:report,XIAOBAN_TEST_STARTUP_ID:Date.now().toString()},stdio:['ignore','pipe','pipe']})
 let output='';child.stdout.on('data',d=>output=(output+d).slice(-12000));child.stderr.on('data',d=>output=(output+d).slice(-12000))
 const timeout=setTimeout(()=>child.kill(),120000)
 try {
  await new Promise((resolve,reject)=>{child.on('error',reject);child.on('exit',(code,signal)=>code===0?resolve():reject(new Error(`${code??signal}: ${output}`)))})
  const result=JSON.parse(await readFile(report,'utf8'));assert.ok(result.website?.Ok,JSON.stringify(result.website))
  assert.equal(result.website.Ok.cases.length,2);assert.ok(result.website.Ok.updateChecked&&result.website.Ok.announcementsChecked&&result.website.Ok.dismissalPersisted)
  for(const item of result.website.Ok.cases){assert.equal(item.report.privilegedIpcBlocked,true);assert.ok(item.report.width>0&&item.report.height>0);if(item.geometry)assert.equal(item.geometry.fillsMonitor,true)}
  console.log(`PASS: actual website image/video search, resource download/render, updates, announcements and persistent dismissals. ${report}`)
 } finally { clearTimeout(timeout) }
} finally { await fixture.close() }
