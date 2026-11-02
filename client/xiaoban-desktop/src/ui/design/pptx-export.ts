import {unzipSync,zipSync,strFromU8,strToU8} from 'fflate';
import {parseSVGPathData} from 'openfig-core';
import {bytesDataUrl,type ExportNode,type ExportPage,type Fill,type Shadow} from './export-scene.ts';

const xml=(s:string)=>s.replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&apos;'}[c]!));
const emu=(px:number)=>Math.round(px*9525);
const alpha=(a:number)=>`<a:alpha val="${Math.round(Math.max(0,Math.min(1,a))*100000)}"/>`;
function color(hex:string,a:number){return `<a:srgbClr val="${hex}">${alpha(a)}</a:srgbClr>`;}
export function fillXml(fill:Fill|undefined,opacity=1):string{
 if(!fill)return '<a:noFill/>';
 if(fill.type==='solid')return `<a:solidFill>${color(fill.color.hex,fill.color.alpha*opacity)}</a:solidFill>`;
 const stops=fill.stops.map(s=>`<a:gs pos="${Math.round(s.offset*100000)}">${color(s.color.hex,s.color.alpha*opacity)}</a:gs>`).join('');
 const geometry=fill.type==='radial'?'<a:path path="circle"><a:fillToRect l="50000" t="50000" r="50000" b="50000"/></a:path>':`<a:lin ang="${Math.round(((fill.angle-90+360)%360)*60000)}" scaled="0"/>`;
 return `<a:gradFill rotWithShape="1"><a:gsLst>${stops}</a:gsLst>${geometry}<a:tileRect/></a:gradFill>`;
}
function effectsXml(shadows:Shadow[]|undefined,scale:number,opacity:number){
 return `<a:effectLst>${(shadows??[]).map(s=>{const tag=s.inset?'innerShdw':'outerShdw';return `<a:${tag} blurRad="${emu(s.blur*scale/2)}" dist="${emu(Math.hypot(s.x,s.y)*scale)}" dir="${Math.round(((Math.atan2(s.y,s.x)*180/Math.PI+360)%360)*60000)}"${s.inset?'':' algn="ctr" rotWithShape="0"'}>${color(s.color.hex,s.color.alpha*opacity)}</a:${tag}>`;}).join('')}</a:effectLst>`;
}
export function roundedRectPath(w:number,h:number,radii:number[]=[0,0,0,0]){
 const [a,b,c,d]=radii.map(r=>Math.max(0,Math.min(r,w/2,h/2))),k=.5522847498;
 return `M${a} 0 L${w-b} 0 C${w-b+b*k} 0 ${w} ${b-b*k} ${w} ${b} L${w} ${h-c} C${w} ${h-c+c*k} ${w-c+c*k} ${h} ${w-c} ${h} L${d} ${h} C${d-d*k} ${h} 0 ${h-d+d*k} 0 ${h-d} L0 ${a} C0 ${a-a*k} ${a-a*k} 0 ${a} 0 Z`;
}
function points(path:string,scale:number){return parseSVGPathData(path).map(p=>p.type==='Z'?{close:true as const}:p.type==='C'?{x:p.x*scale/96,y:p.y*scale/96,curve:{type:'cubic' as const,x1:p.c1x*scale/96,y1:p.c1y*scale/96,x2:p.c2x*scale/96,y2:p.c2y*scale/96}}:{x:p.x*scale/96,y:p.y*scale/96,...(p.type==='M'?{moveTo:true}:{})});}
function imageGeometry(n:ExportNode){
 const cmds=parseSVGPathData(roundedRectPath(n.w,n.h,n.radii));
 const pt=(x:number,y:number)=>`<a:pt x="${emu(x)}" y="${emu(y)}"/>`;
 return `<a:custGeom><a:avLst/><a:gdLst/><a:ahLst/><a:cxnLst/><a:rect l="0" t="0" r="r" b="b"/><a:pathLst><a:path w="${emu(n.w)}" h="${emu(n.h)}">${cmds.map(p=>p.type==='Z'?'<a:close/>':p.type==='M'?`<a:moveTo>${pt(p.x,p.y)}</a:moveTo>`:p.type==='L'?`<a:lnTo>${pt(p.x,p.y)}</a:lnTo>`:`<a:cubicBezTo>${pt(p.c1x,p.c1y)}${pt(p.c2x,p.c2y)}${pt(p.x,p.y)}</a:cubicBezTo>`).join('')}</a:path></a:pathLst></a:custGeom>`;
}
export async function makePptx(title:string,pages:ExportPage[]):Promise<string>{
 if(!pages.length||pages.length>100)throw Error('请选择 1 到 100 个画板');
 const {default:PptxGenJS}=await import('pptxgenjs');
 const pptx=new PptxGenJS();pptx.author='小伴';pptx.subject='可编辑设计画板';pptx.title=title;
 const width=Math.max(...pages.map(p=>p.scene.width)),height=Math.max(...pages.map(p=>p.scene.height));
 // A presentation has one page size. Preserve each board's pixel scale and center it.
 const scale=Math.min(1,5376/Math.max(width,height));
 pptx.defineLayout({name:'XIAOBAN',width:Math.max(1,width*scale/96),height:Math.max(1,height*scale/96)});pptx.layout='XIAOBAN';
 const patches=new Map<string,{node:ExportNode;scale:number}>();
 let seq=0;
 for(const page of pages){
  const slide=pptx.addSlide(),s=page.scene;
  const dx=(Math.max(96/scale,width)-s.width)/2,dy=(Math.max(96/scale,height)-s.height)/2;
  for(const n of s.nodes){
   const key=`xb-${++seq}`,options={x:(n.x+dx)*scale/96,y:(n.y+dy)*scale/96,w:n.w*scale/96,h:n.h*scale/96,objectName:key,rotate:n.rotation??0};
   patches.set(key,{node:n,scale});
   if(n.kind==='text'){
    const c=n.fill?.type==='solid'?n.fill.color:{hex:'000000',alpha:1};
    slide.addText(n.text!,{...options,w:options.w+scale/96,fontFace:n.font,fontSize:n.fontSize!*scale*.75,color:c.hex,transparency:Math.round((1-c.alpha*n.opacity)*100),bold:n.bold,italic:n.italic,underline:n.underline?{style:'sng'}:undefined,strike:n.strike,rtlMode:n.rtl,charSpacing:(n.letterSpacing??0)*scale*.75,margin:0,breakLine:false,wrap:false,valign:'top',paraSpaceAfter:0,paraSpaceBefore:0,lineSpacing:n.fontSize!*scale*.75,lang:'zh-CN'});
   }else if(n.kind==='image'){
    slide.addImage({...options,data:n.image!,transparency:Math.round((1-n.opacity)*100),altText:n.name});
   }else{
    const line=n.stroke?{color:n.stroke.color.hex,width:n.stroke.width*scale*.75,transparency:Math.round((1-n.stroke.color.alpha*n.opacity)*100),dashType:n.stroke.dash?'dash' as const:'solid' as const}:{transparency:100};
    slide.addShape('custGeom' as Parameters<typeof slide.addShape>[0],{...options,points:points(n.kind==='path'?n.path!:roundedRectPath(n.w,n.h,n.radii),scale),line,fill:{color:'FFFFFF',transparency:100}});
   }
  }
  slide.addNotes([`画板：${page.title} (${s.width} × ${s.height})`,...(s.warnings.length?['导出兼容提示：',...s.warnings]:[]),'文字按浏览器实际换行拆为可编辑文本行；图表与图标按图形导出，不含原数据绑定。','字体未嵌入，接收端缺少对应字体时，显示和换行可能有差异。'].join('\n'));
 }
 const output=await pptx.write({outputType:'uint8array',compression:true}) as Uint8Array;
 const files=unzipSync(output);
 for(const file of Object.keys(files).filter(f=>/^ppt\/slides\/slide\d+\.xml$/.test(f))){
  let content=strFromU8(files[file]);
  content=content.replace(/<p:(sp|pic)\b[\s\S]*?<\/p:\1>/g,block=>{
   const key=/\bname="(xb-\d+)"/.exec(block)?.[1],patch=key?patches.get(key):undefined;if(!patch)return block;
   const {node:n,scale}=patch;
   block=block.replace(/<p:cNvPr\b([^>]*?)name="[^"]*"/,(_whole,attrs:string)=>`<p:cNvPr${attrs}name="${xml(n.name)}"`);
   if(n.kind==='text')return block.replace(/<a:rPr\b[^>]*>[\s\S]*?<\/a:rPr>/g,run=>{
    run=run.replace(/<a:solidFill>[\s\S]*?<\/a:solidFill>/,fillXml(n.fill,n.opacity));
    return n.shadows?.length?run.replace('</a:rPr>',effectsXml(n.shadows,scale,n.opacity)+'</a:rPr>'):run;
   });
   block=block.replace(/<p:spPr>([\s\S]*?)<\/p:spPr>/,(whole,inner:string)=>{
    if(n.kind!=='image')inner=inner.replace(/<a:solidFill>[\s\S]*?<\/a:solidFill>|<a:noFill\s*\/>/,fillXml(n.fill,n.opacity));
    if(n.kind==='image'&&n.radii?.some(r=>r>0))inner=inner.replace(/<a:prstGeom\b[\s\S]*?<\/a:prstGeom>/,imageGeometry(n));
    if(n.shadows?.length){inner=inner.replace(/<a:effectLst\s*\/>|<a:effectLst>[\s\S]*?<\/a:effectLst>/,'');inner+=effectsXml(n.shadows,scale,n.opacity);}
    return `<p:spPr>${inner}</p:spPr>`;
   });return block;
  });files[file]=strToU8(content);
 }
 return bytesDataUrl(zipSync(files,{level:6}),'application/vnd.openxmlformats-officedocument.presentationml.presentation');
}
