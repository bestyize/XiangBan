import {describe,it,expect} from 'vitest';
import {unzipSync,strFromU8} from 'fflate';
import {parseFig,parseFigBinary,resolveVectorNodePaths} from 'openfig-core';
import {makePptx} from './pptx-export.ts';
import {makeFigma,zstdRaw} from './figma-export.ts';
import {validateScene,pngBytes,type ExportPage,type ExportScene} from './export-scene.ts';
import {decompress} from 'fzstd';
const black={hex:'112233',alpha:1},red={hex:'ff2244',alpha:.8};
const fixture=():ExportScene=>({version:1,width:960,height:540,warnings:['测试兼容提示'],nodes:[
 {kind:'shape',name:'渐变卡片 & <标题>',x:20,y:30,w:300,h:180,opacity:1,radii:[12,24,12,24],fill:{type:'linear',angle:135,stops:[{offset:0,color:black},{offset:1,color:red}]},shadows:[{x:4,y:8,blur:12,spread:0,inset:false,color:{hex:'000000',alpha:.3}}]},
 {kind:'text',name:'可编辑文字',x:40,y:60,w:240,h:30,opacity:1,text:'中文设计 & Editable <text>',font:'Arial',fontSize:24,bold:true,fill:{type:'solid',color:black}},
 {kind:'path',name:'可编辑曲线',x:0,y:0,w:960,h:540,opacity:1,path:'M400 100 C420 20 500 180 560 100 L560 220 L400 220 Z',fill:{type:'solid',color:red},stroke:{width:2,color:black}},
 {kind:'image',name:'圆角图片',x:650,y:20,w:100,h:100,opacity:.8,radii:[8,8,8,8],image:'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+j0L8AAAAASUVORK5CYII='},
]});
const page=(scene=fixture()):ExportPage=>({title:'首页',file:'Main.dc.html',x:20,y:40,scene});
describe('editable design exports',()=>{
 it('creates native editable text, curves, gradients, shadows and image geometry in PPTX',async()=>{
  const data=await makePptx('设计',[page(),{...page(),title:'第二页',y:600}]),zip=unzipSync(pngBytes(data));
  const slide=strFromU8(zip['ppt/slides/slide1.xml']);
  expect(slide).toContain('中文设计 &amp; Editable &lt;text&gt;');expect(slide).toContain('<a:cubicBezTo>');expect(slide).toContain('<a:gradFill');expect(slide).toContain('<a:outerShdw');expect(slide).toContain('渐变卡片 &amp; &lt;标题&gt;');
  expect(slide.match(/<p:pic>/g)).toHaveLength(1);expect(slide.match(/<p:sp>/g)?.length).toBe(3);expect(slide).toMatch(/<p:pic>[\s\S]*<a:custGeom>/);
  expect(strFromU8(zip['ppt/notesSlides/notesSlide1.xml'])).toContain('测试兼容提示');expect(zip['ppt/slides/slide2.xml']).toBeDefined();
  expect(slide).not.toContain('NaN');expect(slide).not.toContain('Infinity');
 });
 it('writes actual Kiwi .fig with native text, vectors, gradients, image assets and board coordinates',async()=>{
  const bytes=pngBytes(await makeFigma('设计',[page()])),doc=parseFig(bytes);
  const frame=doc.nodes.find(n=>n.type==='FRAME')!;expect(frame.transform?.m02).toBe(20);expect(frame.transform?.m12).toBe(40);
  const text=doc.nodes.find(n=>n.type==='TEXT')!;expect(text.textData?.characters).toContain('中文设计');expect(text.fontSize).toBe(24);
  expect(doc.nodes.find(n=>n.name.startsWith('渐变卡片'))?.fillPaints?.[0].type).toBe('GRADIENT_LINEAR');
  const vector=doc.nodes.find(n=>n.type==='VECTOR')!;expect(resolveVectorNodePaths(doc,vector).fill[0].svgPath).toContain('C');expect(doc.images.size).toBe(1);
  const zip=unzipSync(bytes);expect(strFromU8(zip['canvas.fig'].slice(0,8))).toBe('fig-kiwi');expect(parseFigBinary(zip['canvas.fig']).nodes).toHaveLength(doc.nodes.length);
 });
 it('encodes Zstandard raw blocks across block boundaries',()=>{
  for(const length of [0,1,131071,131072,300000]){const bytes=Uint8Array.from({length},(_,i)=>i%251);expect(decompress(zstdRaw(bytes))).toEqual(bytes);}
 });
 it('rejects unbounded, executable and non-finite sandbox output before export',()=>{
  expect(validateScene(fixture(),960,540).nodes).toHaveLength(4);
  for(const patch of [{x:NaN},{image:'https://example.com/private.png',kind:'image'},{path:'<script/>',kind:'path'},{fontSize:Infinity,kind:'text'}]){
   const f=fixture();Object.assign(f.nodes[0],patch);expect(()=>validateScene(f,960,540)).toThrow();
  }
  expect(()=>validateScene(fixture(),10,20)).toThrow();
 });
});
