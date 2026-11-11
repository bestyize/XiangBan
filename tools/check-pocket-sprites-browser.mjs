// Requires Playwright and a local HTTP server serving the repository on 1473.
import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { pathToFileURL } from 'node:url';
const modulePath = process.env.PLAYWRIGHT_MODULE || join(tmpdir(), 'xiaoban-playwright/node_modules/playwright/index.mjs');
const { chromium } = await import(pathToFileURL(modulePath).href);
const browser = await chromium.launch({headless:true, executablePath:process.env.PLAYWRIGHT_BROWSER || 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe'});
const output = process.env.POCKET_SCREENSHOTS || join(tmpdir(), 'pocket-sprites-check');
await mkdir(output,{recursive:true});
const url = (process.env.POCKET_ORIGIN || 'http://127.0.0.1:1473') + '/assets/game-packs/pocket-sprites/index.html';
const errors=[];
try {
 const page=await browser.newPage({viewport:{width:375,height:800},deviceScaleFactor:2,hasTouch:true,reducedMotion:'reduce'});
 await page.addInitScript(()=>{
  const Native=window.AudioContext;
  if(!Native)return;
  window.__musicSources=0;
  window.__rewardSources=0;
  window.__musicTargets=[];
  window.AudioContext=class extends Native {
   constructor(...args){super(...args);window.__testAudio=this;}
   createBufferSource(){
    const source=super.createBufferSource(),connect=source.connect.bind(source);
    source.connect=(node,...args)=>{
     if(source.loop) window.__musicSources++;
     else window.__rewardSources++;
     if(source.loop&&node.gain){const target=node.gain.setTargetAtTime.bind(node.gain);node.gain.setTargetAtTime=(value,...rest)=>{window.__musicTargets.push(value);return target(value,...rest)}}
     return connect(node,...args);
    };
    return source;
   }
  };
 });
 page.on('pageerror',e=>{errors.push(e.message);console.error(e.message)});
 page.on('console',msg=>{if(msg.type()==='error')console.error(msg.text())});
 page.on('response',r=>{if(r.status()>=400&&!r.url().endsWith('/favicon.ico')) errors.push(`${r.status()} ${r.url()}`)});
 await page.goto(url);
 await page.locator('[data-action="start"]').click();
 await page.waitForFunction(()=>window.__musicSources===1);
 await page.screenshot({path:join(output,'level-01.png')});
 assert.equal(await page.locator('#board .cell').count(),36);
 const remainingAfterSwap=String(Number(await page.locator('#moves').textContent())-1);
 await page.locator('#cell-19').tap(); await page.locator('#cell-27').tap();
 await page.waitForFunction(value=>document.querySelector('#moves').textContent===value,remainingAfterSwap);
 await page.waitForFunction(()=>!document.querySelector('[data-action="hint"]').disabled);
 assert.ok(await page.evaluate(()=>window.__musicTargets.includes(.1)&&window.__musicTargets.includes(.022)),'music ducks beneath clear SFX');
 await page.reload(); await page.waitForFunction(value=>document.body.dataset.screen==='game'&&document.querySelector('#moves').textContent===value,remainingAfterSwap);
 assert.equal(await page.locator('#moves').textContent(),remainingAfterSwap,'round restored');
 await page.locator('[data-action="pause"]').click();
 assert.equal(await page.locator('#dialog-title').textContent(),'游戏暂停');
 await page.waitForFunction(()=>window.__testAudio?.state==='suspended');
 await page.locator('[data-action="music"]').click();
 assert.equal(await page.locator('[data-action="music"]').textContent(),'开启背景音乐');
 await page.locator('[data-action="music"]').click();
 await page.locator('[data-action="resume"]').click();
 await page.waitForFunction(()=>window.__testAudio?.state==='running');
 await page.waitForFunction(()=>window.__musicSources===1);
 assert.equal(await page.evaluate(()=>window.__musicSources),1,'one music source after pause and mute');
 await page.evaluate(async()=>{const {LEVELS}=await import('./levels.mjs');localStorage.setItem('pocket-sprites.v1',JSON.stringify({version:1,stars:Object.fromEntries(LEVELS.map(l=>[l.id,3])),muted:true,round:null}))});
 for (const id of [3,6,9,12,15,18,20]) {
  await page.evaluate(()=>history.replaceState({screen:'home'},''));
  await page.reload(); await page.locator('#home [data-action="levels"]').click();
  const target=page.locator(`[data-level="${id}"]`); if(!await target.count()) continue;
  await target.click();
  await page.screenshot({path:join(output,`level-${String(id).padStart(2,'0')}.png`)});
  const bounds=await page.locator('#board').boundingBox();
  assert.ok(bounds.x>=0&&bounds.x+bounds.width<=375&&bounds.y+bounds.height<=800,`level ${id} bounds`);
  const goalsFit=await page.locator('.goals-card').evaluate(card=>{
   const b=card.getBoundingClientRect();
   return [...card.querySelectorAll('.goal')].every(goal=>{const g=goal.getBoundingClientRect();return g.left>=b.left&&g.right<=b.right&&g.bottom<=b.bottom});
  });
  assert.ok(goalsFit,`level ${id}: collection goals must fit the HUD`);
  if(id===6){
   assert.ok(Number(await page.locator('#board').getAttribute('aria-rowcount'))>7);
   assert.ok(Number(await page.locator('#board').getAttribute('aria-colcount'))>6);
   const turn=await page.evaluate(async()=>{
    const {LEVELS}=await import('./levels.mjs'),{legalMoves}=await import('./engine.mjs');
    const saved=JSON.parse(localStorage.getItem('pocket-sprites.v1')).round;
    const game={level:LEVELS.find(level=>level.id===saved.id),...saved.state};
    return {pair:legalMoves(game)[0],remaining:String(game.moves-1)};
   });
   await page.locator(`#cell-${turn.pair[0]}`).tap();await page.locator(`#cell-${turn.pair[1]}`).tap();
   await page.waitForFunction(value=>document.querySelector('#moves').textContent===value&&!document.querySelector('[data-action="hint"]').disabled,turn.remaining);
   await page.reload();
   await page.waitForFunction(value=>document.querySelector('#moves').textContent===value,turn.remaining);
   assert.equal(await page.evaluate(()=>JSON.parse(localStorage.getItem('pocket-sprites.v1')).round.state.board.length),81,'restore a 9×9 round');
  }
 }
 await page.emulateMedia({reducedMotion:'no-preference'});
 await page.evaluate(()=>history.replaceState({screen:'home'},''));
 await page.evaluate(()=>{const saved=JSON.parse(localStorage.getItem('pocket-sprites.v1'));saved.muted=false;localStorage.setItem('pocket-sprites.v1',JSON.stringify(saved))});
 await page.reload(); await page.locator('#home [data-action="levels"]').click();
 await page.locator('[data-level="3"]').click();
 await page.locator('#cell-20').tap(); await page.locator('#cell-28').tap();
 await page.locator('.power-label').first().waitFor({state:'visible'});
 await page.screenshot({path:join(output,'power-created.png')});
 await page.waitForFunction(()=>!document.querySelector('[data-action="hint"]').disabled);
 assert.equal(await page.locator('.power-layer').count(),0,'power celebration cleans up');
 assert.equal(await page.evaluate(()=>window.__rewardSources),1,'one decoded reward voice per achievement turn');
 await page.emulateMedia({reducedMotion:'reduce'});
 for(const width of [320,375]) {
  await page.setViewportSize({width,height:width===320?568:800});
  await page.evaluate(()=>{const r=document.documentElement.style;r.setProperty('--xb-safe-top','28px');r.setProperty('--xb-capsule-top','36px');r.setProperty('--xb-capsule-height','48px');r.setProperty('--xb-capsule-width','48px');r.setProperty('--xb-capsule-right','6px');r.setProperty('--xb-safe-bottom','24px')});
  const header=await page.locator('.game-header h1').boundingBox();
  assert.ok(header.y<84&&header.x+header.width<=width-54,'use the free space beside capsule');
  const boardBounds=await page.locator('#board').boundingBox();
  assert.ok(boardBounds.y>=84&&boardBounds.y+boardBounds.height<=(width===320?568:800)-24);
  const squares=await page.locator('.cell').evaluateAll(nodes=>nodes.map(n=>{const b=n.getBoundingClientRect();return Math.abs(b.width-b.height)}));
  assert.ok(squares.every(delta=>delta<1),'square cells across viewport sizes');
  await page.screenshot({path:join(output,`capsule-game-${width}.png`)});
  await page.locator('[data-action="pause"]').click();
  const positions=await page.locator('button:visible').evaluateAll(nodes=>nodes.map(n=>{const r=n.getBoundingClientRect();return {text:n.textContent,top:r.top,right:r.right,bottom:r.bottom,left:r.left}}));
  assert.ok(positions.every(p=>p.top>=28&&p.left>=0&&p.right<=width&&!(p.right>width-54&&p.left<width-6&&p.bottom>36&&p.top<84)),JSON.stringify(positions));
  await page.screenshot({path:join(output,`pause-${width}.png`)});
  await page.locator('[data-action="resume"]').click();
 }
 assert.deepEqual(errors,[]);
 console.log(`PASS mobile UI: tutorial swap, save restore, pause, level layouts, capsule avoidance. Screenshots: ${output}`);
} finally {await browser.close()}
