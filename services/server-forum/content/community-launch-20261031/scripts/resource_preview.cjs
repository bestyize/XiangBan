const fs=require('fs'),path=require('path');const runtime=require('./pack_runtime.cjs'),options=runtime.args();const ts=runtime.resolveModule('typescript',options.typescript,options.repo);
const out=path.join(options.output,'preview');fs.mkdirSync(out+'/effects',{recursive:true});
for(const f of ['types','parse','sim','draw']){const s=fs.readFileSync(path.join(options.repo,'client/xiaoban-desktop/src/effects',f+'.ts'),'utf8');const result=ts.transpileModule(s,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText.replaceAll('.ts"','.js"');fs.writeFileSync(out+'/effects/'+f+'.js',result)}
const link=path.join(out,'packs');
let previous;try{previous=fs.lstatSync(link)}catch(e){if(e.code!=='ENOENT')throw e}
if(previous) {
 if(!previous.isSymbolicLink()) throw Error('Refusing to replace a non-symlink preview/packs directory');
 if(path.resolve(out,fs.readlinkSync(link))!==options.source) fs.unlinkSync(link);
}
if(!fs.existsSync(link)) fs.symlinkSync(options.source,link,'dir');
fs.writeFileSync(out+'/effect.html','<!doctype html><meta charset="utf-8"><style>html,body{margin:0;height:100%;overflow:hidden;background:#122637}canvas{display:block}.name{position:absolute;bottom:60px;left:70px;font:28px system-ui;color:#e5dfc5;letter-spacing:.2em}.name small{display:block;font-size:13px;letter-spacing:.12em;opacity:.6;margin-top:12px}</style><canvas width="1200" height="750"></canvas><div class="name"></div><script type="module" src="effect-preview.js"></script>');
fs.writeFileSync(out+'/effect-preview.js',`import {parsePack} from './effects/parse.js';import{ParticleSystem}from './effects/sim.js';import{drawEffects}from './effects/draw.js';const slug=new URLSearchParams(location.search).get('slug');const raw=await(await fetch('/packs/'+slug+'/effect.json')).json();const result=parsePack(raw);if(!result.ok)throw Error(result.error);const sys=new ParticleSystem(result.pack);sys.configure(1200,750,{density:1,speed:1,size:1});for(let i=0;i<480;i++)sys.advance(1/60);const canvas=document.querySelector('canvas');document.body.style.background='linear-gradient('+raw.sky.join(',')+')';document.querySelector('.name').textContent=raw.name;const small=document.createElement('small');small.textContent='实际粒子引擎渲染 · 纯数据特效';document.querySelector('.name').append(small);drawEffects(canvas.getContext('2d'),sys);window.preview={ok:true,counts:sys.counts(),visible:sys.layers.reduce((n,l)=>n+l.pool.filter(p=>p.visible).length,0)};`);
console.log('preview ready');
