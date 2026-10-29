import {spawn} from 'node:child_process'
import {mkdir,readFile} from 'node:fs/promises'
import {fileURLToPath} from 'node:url'
import path from 'node:path'
import assert from 'node:assert/strict'
import {startWallpaperFixture} from './wallpaper-fixture.mjs'
const root=fileURLToPath(new URL('../',import.meta.url)),fixture=await startWallpaperFixture()
await mkdir(path.join(root,'artifacts'),{recursive:true})
const report=path.join(root,`artifacts/media-smoke-${Date.now()}.json`)
const binary=process.argv[2]??path.join(root,'src-tauri/target/release/xiaoban-desktop.exe')
const effectsSmoke=process.env.XIAOBAN_EFFECTS_SMOKE==='1'
const child=spawn(binary,['--smoke-test','--autostart'],{env:{...process.env,XIAOBAN_SMOKE_REPORT:report,XIAOBAN_WALLPAPER_TEST_URL:fixture.catalogUrl,XIAOBAN_LOCAL_WALLPAPER_TEST_DIR:fixture.localDirectory,XIAOBAN_TEST_STARTUP_ID:Date.now().toString()},stdio:['ignore','pipe','pipe']})
let output='';child.stdout.on('data',data=>output=(output+data).slice(-12000));child.stderr.on('data',data=>output=(output+data).slice(-12000))
const timeout=setTimeout(()=>child.kill(),120000)
try{
 await new Promise((resolve,reject)=>{child.on('error',reject);child.on('exit',(code,signal)=>code===0?resolve():reject(new Error(`App failed: ${code??signal}\n${output}`)))})
 const result=JSON.parse(await readFile(report,'utf8'));assert.ok(result.media?.Ok,JSON.stringify(result.media))
 assert.equal(result.media.Ok.cases.length,6);assert.ok(result.media.Ok.cacheOffline);assert.ok(result.media.Ok.hashFailurePreserved);assert.ok(result.media.Ok.updateChecked)
 assert.equal(result.wallpaper.wallpaper,null,'No special companion wallpaper should be created')
 assert.ok(result.wallpaper.floating?.visible,'Desktop companion must remain visible')
 if(result.wallpaper.platform==='windows'){
  assert.equal(result.wallpaper.floating.isChild,true,'Wallpaper companion must be hosted below desktop icons')
  assert.ok(['WorkerW','Progman'].includes(result.wallpaper.floating.parentClass))
  assert.equal(result.wallpaper.floating.topmost,false)
  assert.equal(result.floatingAfter.floating.isChild,false,'Floating companion must leave the desktop host')
  assert.equal(result.floatingAfter.floating.topmost,true)
  for(const coordinate of ['x','y','width','height'])assert.equal(result.floatingAfter.floating[coordinate],result.floatingBefore.floating[coordinate],`Mode round trip must preserve ${coordinate}`)
 }
 for(const state of result.media.Ok.cases){assert.ok(state.report.width>0&&state.report.height>0);assert.equal(state.report.privilegedIpcBlocked,true);assert.equal(state.geometry.fillsMonitor,true,`${state.id} must fill the monitor without a non-client border`)}
 assert.equal(result.media.Ok.localImports.cases.length,2)
 assert.ok(result.media.Ok.localImports.sourceDeleted&&result.media.Ok.localImports.deduplicated&&result.media.Ok.localImports.restored)
 for(const state of result.media.Ok.localImports.cases){assert.ok(state.report.width>0&&state.report.height>0);assert.equal(state.geometry.fillsMonitor,true)}
 if(effectsSmoke){
  const effects=result.media.Ok.effects
  assert.ok(effects,JSON.stringify(result.media.Ok))
  assert.equal(effects.cases?.length,12,'effects.cases')
  for(const [index,state] of effects.cases.entries()){
   assert.equal(state.persisted,true,`effects.cases[${index}].persisted`)
   assert.equal(state.report.effects.canvasPresent,true,`effects.cases[${index}] overlay canvas`)
   assert.ok(state.report.effects.particles.length>0&&state.report.effects.particles.every(n=>n>0),`effects.cases[${index}] every layer has particles`)
   assert.equal(state.report.privilegedIpcBlocked,true,`effects.cases[${index}] overlay must not reach desktop commands`)
  }
  assert.deepEqual(effects.cases.map(item=>item.effect),['rain','snow','fireflies','petals','maple','meteors','stars','butterfly-dream','lanterns','fireworks','aurora','galaxy'])
  assert.equal(effects.independent?.length,4,'effects.independent')
  assert.ok(effects.independent.every(item=>item.overlayPresent===true),'The overlay must survive every wallpaper, including the plain system desktop')
  assert.equal(effects.turnedOff,true,'effects.turnedOff')
  assert.equal(effects.persistence?.diskConfigMatch,true,'effects.persistence.diskConfigMatch')
  if(result.wallpaper.platform==='windows')for(const native of [...effects.cases.map(item=>item.report.native),...effects.independent.map(item=>item.native)]){
   assert.equal(native.effects.isChild,true,'Effects must be hosted below desktop icons')
   assert.ok(['WorkerW','Progman'].includes(native.effects.parentClass))
   assert.equal(native.effects.topmost,false)
   if(native.effects.visible){
    assert.equal(native.effects.parent,native.floating.parent,'Effects and wallpaper companion must share the desktop host')
    assert.deepEqual(native.desktopOrder.slice(0,2),['pet','effects'],'Companion must be above effects')
   }
   for(const media of [native.mediaA,native.mediaB].filter(Boolean))assert.equal(media.parent,native.effects.parent,'Wallpaper must share the desktop host below effects')
  }
  if(result.wallpaper.platform==='windows')for(const item of effects.independent){
   if(item.id==='companion')assert.equal(item.native.effects.visible,true,'Native wallpaper must keep standalone effects visible')
   else{
    assert.equal(item.native.effects.visible,false,'A transparent sibling must not cover the custom wallpaper')
    assert.equal(item.native.floating.visible,false,'Wallpaper companion must be composed inside the wallpaper')
    assert.equal(item.composition.effects.effect,'meteors')
    assert.equal(item.composition.effects.canvasPresent,true)
    assert.equal(item.composition.companionVisible,true)
   }
  }
 }
 console.log(`PASS: image/video/Canvas, delivery, SHA-256, offline cache, unprivileged IPC and update check${effectsSmoke?'; effects: 12 effects, kept across 4 wallpapers, off and persistence':''}. ${report}`)
}finally{clearTimeout(timeout);await fixture.close()}
