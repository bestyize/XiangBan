import {spawn} from 'node:child_process'
import {mkdir,readFile} from 'node:fs/promises'
import {fileURLToPath} from 'node:url'
import path from 'node:path'
import assert from 'node:assert/strict'
import {startWallpaperFixture} from './wallpaper-fixture.mjs'
const root=fileURLToPath(new URL('../',import.meta.url)),fixture=await startWallpaperFixture()
await mkdir(path.join(root,'artifacts'),{recursive:true})
const report=path.join(root,`artifacts/media-smoke-${Date.now()}.json`)
const binary=process.argv[2]??path.join(root,'src-tauri/target/release/xiaoban-desktop-wallpaper.exe')
const child=spawn(binary,['--smoke-test','--autostart'],{env:{...process.env,XIAOBAN_SMOKE_REPORT:report,XIAOBAN_WALLPAPER_TEST_URL:fixture.catalogUrl,XIAOBAN_LOCAL_WALLPAPER_TEST_DIR:fixture.localDirectory,XIAOBAN_TEST_STARTUP_ID:Date.now().toString()},stdio:['ignore','pipe','pipe']})
let output='';child.stdout.on('data',data=>output=(output+data).slice(-12000));child.stderr.on('data',data=>output=(output+data).slice(-12000))
const timeout=setTimeout(()=>child.kill(),120000)
try{
 await new Promise((resolve,reject)=>{child.on('error',reject);child.on('exit',(code,signal)=>code===0?resolve():reject(new Error(`App failed: ${code??signal}\n${output}`)))})
 const result=JSON.parse(await readFile(report,'utf8'));assert.ok(result.media?.Ok,JSON.stringify(result.media))
 assert.equal(result.media.Ok.cases.length,6);assert.ok(result.media.Ok.cacheOffline);assert.ok(result.media.Ok.hashFailurePreserved);assert.ok(result.media.Ok.updateChecked)
 assert.equal(result.wallpaper.wallpaperGeometry.fillsMonitor,true,'Companion wallpaper must fill the monitor')
 for(const state of result.media.Ok.cases){assert.ok(state.report.width>0&&state.report.height>0);assert.equal(state.report.privilegedIpcBlocked,true);assert.equal(state.geometry.fillsMonitor,true,`${state.id} must fill the monitor without a non-client border`)}
 assert.equal(result.media.Ok.localImports.cases.length,2)
 assert.ok(result.media.Ok.localImports.sourceDeleted&&result.media.Ok.localImports.deduplicated&&result.media.Ok.localImports.restored)
 for(const state of result.media.Ok.localImports.cases){assert.ok(state.report.width>0&&state.report.height>0);assert.equal(state.geometry.fillsMonitor,true)}
 console.log(`PASS: image/video/Canvas, delivery, SHA-256, offline cache, unprivileged IPC and update check. ${report}`)
}finally{clearTimeout(timeout);await fixture.close()}
