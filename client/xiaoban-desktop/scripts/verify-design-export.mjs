// Browser layout regression for editable exports. Requires a running Vite server
// and Playwright (PLAYWRIGHT_MODULE may point to an existing installation).
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,dirname} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {tmpdir} from 'node:os';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const moduleName=process.env.PLAYWRIGHT_MODULE;
const {chromium}=await import(moduleName?.startsWith('/')?pathToFileURL(moduleName).href:moduleName||'playwright');
const origin=process.env.DESIGN_EXPORT_URL||'http://127.0.0.1:1428';
const output=process.env.DESIGN_EXPORT_OUTPUT||resolve(tmpdir(),'xiaoban-design-export-check');
await mkdir(output,{recursive:true});
const browser=await chromium.launch({headless:true,...(process.env.DESIGN_EXPORT_BROWSER?{executablePath:process.env.DESIGN_EXPORT_BROWSER}:{})});
try{
 const page=await browser.newPage({viewport:{width:1100,height:850},deviceScaleFactor:1}),errors=[];
 page.on('pageerror',e=>errors.push(String(e)));
 const html=await readFile(resolve(root,'src/ui/design/fixtures/export-complex.html'),'utf8');
 await page.route('**/export-check.html*',route=>route.fulfill({contentType:'text/html',body:html,headers:{'Content-Security-Policy':"default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self' ws://127.0.0.1:1428"}}));
 await page.goto(`${origin}/export-check.html?token=export-check`);
 await page.addScriptTag({content:await readFile(resolve(root,'../../libs/agent-tools/open-claude-design/runtime/vendor/capture.js'),'utf8')});
 const scene=await page.evaluate(()=>new Promise((resolve,reject)=>{
  const timer=setTimeout(()=>reject(Error('capture timeout')),120000);
  addEventListener('message',function handler(e){if(e.source!==window||e.data?.type!=='capture-result')return;removeEventListener('message',handler);clearTimeout(timer);if(e.data.error)reject(Error(e.data.error));else resolve(e.data.scene);});
  postMessage({channel:'open-claude-design',type:'capture',token:'export-check',requestId:'check',width:960,height:640,scale:1,scene:true},'*');
 }));
 const thumbnail='data:image/png;base64,'+(await page.screenshot({path:resolve(output,'source.png'),clip:{x:0,y:0,width:960,height:640}})).toString('base64');
 await writeFile(resolve(output,'scene.json'),JSON.stringify(scene,null,2));
 const result=await page.evaluate(async ({scene,thumbnail})=>{
  const {makePptx}=await import('/src/ui/design/pptx-export.ts'),{makeFigma}=await import('/src/ui/design/figma-export.ts'),{validateScene}=await import('/src/ui/design/export-scene.ts');
  validateScene(scene,960,640);
  const pages=[{title:'复杂设计测试',file:'Main.dc.html',x:0,y:0,scene}];
  return {pptx:await makePptx('复杂设计测试',pages),fig:await makeFigma('复杂设计测试',pages,thumbnail)};
 },{scene,thumbnail});
 for(const [ext,data] of Object.entries(result))await writeFile(resolve(output,`design.${ext}`),Buffer.from(data.split(',')[1],'base64'));
 const summary={nodes:scene.nodes.length,text:scene.nodes.filter(n=>n.kind==='text').length,paths:scene.nodes.filter(n=>n.kind==='path').length,images:scene.nodes.filter(n=>n.kind==='image').length,warnings:scene.warnings,errors};
 if(!summary.text||summary.paths<2||summary.images||summary.warnings.length||errors.length)throw Error(JSON.stringify(summary));
 if(!scene.nodes.some(n=>n.rotation))throw Error('旋转文字未保留');
 if(!scene.nodes.some(n=>n.kind==='text'&&n.text.includes('小伴')&&n.font!=='Arial'))throw Error('中文字体回退未保留');
 if(await page.locator('[data-xb-pseudo-twin]').count())throw Error('导出没有清理临时元素');
 console.log(JSON.stringify({...summary,output}));
}finally{await browser.close();}
