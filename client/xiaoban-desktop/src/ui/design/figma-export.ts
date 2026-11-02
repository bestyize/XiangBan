import {assembleCanvasFig,createFigZip,appendVectorPayloadToDocument,encodeVectorNetworkBlob,parseSVGPathData,type FigDocument,type FigNode,type FigPaint} from 'openfig-core';
import {encodeMessage,schemaBase64,version} from './vendor/figma-codec.js';
import {bytesDataUrl,pngBytes,type Color,type Fill,type ExportPage} from './export-scene.ts';

const rgba=(c:Color)=>({r:parseInt(c.hex.slice(0,2),16)/255,g:parseInt(c.hex.slice(2,4),16)/255,b:parseInt(c.hex.slice(4,6),16)/255,a:c.alpha});
const transform=(x=0,y=0)=>({m00:1,m01:0,m02:x,m10:0,m11:1,m12:y});
function paint(f:Fill):FigPaint{
 if(f.type==='solid')return {type:'SOLID',color:{...rgba(f.color),a:1},opacity:f.color.alpha,visible:true,blendMode:'NORMAL'};
 const angle=(f.angle-90)*Math.PI/180,c=Math.cos(angle),s=Math.sin(angle);
 return {type:f.type==='linear'?'GRADIENT_LINEAR':'GRADIENT_RADIAL',opacity:1,visible:true,blendMode:'NORMAL',stops:f.stops.map(t=>({position:t.offset,color:rgba(t.color)})),transform:{m00:c,m01:s,m02:.5-.5*c-.5*s,m10:-s,m11:c,m12:.5+.5*s-.5*c}};
}
// RFC 8878 Zstandard frame containing bounded raw blocks. No WASM, network or eval.
// The Kiwi schema is deflated; modern Figma expects a Zstd message chunk.
export function zstdRaw(data:Uint8Array):Uint8Array{
 const blocks=Math.max(1,Math.ceil(data.length/131071)),out=new Uint8Array(9+blocks*3+data.length),view=new DataView(out.buffer);
 out.set([0x28,0xb5,0x2f,0xfd,0xa0]);view.setUint32(5,data.length,true);let dst=9;
 for(let i=0;i<blocks;i++){const chunk=data.subarray(i*131071,Math.min((i+1)*131071,data.length)),header=(chunk.length<<3)|(i===blocks-1?1:0);out[dst++]=header&255;out[dst++]=(header>>>8)&255;out[dst++]=(header>>>16)&255;out.set(chunk,dst);dst+=chunk.length;}
 return out;
}
export async function makeFigma(title:string,pages:ExportPage[],thumbnail?:string):Promise<string>{
 if(!pages.length||pages.length>100)throw Error('请选择 1 到 100 个画板');
 const nodes:FigNode[]=[],message={type:'NODE_CHANGES',nodeChanges:nodes,blobs:[] as {bytes:Uint8Array}[]},images=new Map<string,Uint8Array>();
 const doc={message} as unknown as FigDocument;let id=0;
 const add=(type:string,name:string,parent?:FigNode):FigNode=>{const n:FigNode={guid:{sessionID:0,localID:id++},type,name,phase:'CREATED',visible:true,opacity:1,transform:transform(),...(parent?{parentIndex:{guid:parent.guid,position:`a${String(id).padStart(8,'0')}`}}:{})};nodes.push(n);return n;};
 const root=add('DOCUMENT',title),canvas=add('CANVAS','设计画板',root);canvas.backgroundEnabled=true;canvas.backgroundColor={r:.94,g:.94,b:.95,a:1};
 for(const page of pages){
  const frame=add('FRAME',page.title,canvas);frame.transform=transform(page.x,page.y);frame.size={x:page.scene.width,y:page.scene.height};frame.frameMaskDisabled=false;frame.fillPaints=[];
  frame.description=page.scene.warnings.join('\n');
  for(const n of page.scene.nodes){
   const node=add(n.kind==='text'?'TEXT':n.kind==='path'?'VECTOR':'RECTANGLE',n.name,frame);
   node.transform=transform(n.x,n.y);node.size={x:n.w,y:n.h};node.opacity=n.opacity;node.fillPaints=n.fill?[paint(n.fill)]:[];node.strokePaints=n.stroke?[paint({type:'solid',color:n.stroke.color})]:[];node.strokeWeight=n.stroke?.width??0;node.strokeAlign='INSIDE';node.strokeJoin='ROUND';
   if(n.rotation){const a=n.rotation*Math.PI/180,c=Math.cos(a),s=Math.sin(a);node.transform={m00:c,m01:-s,m02:n.x+n.w/2-c*n.w/2+s*n.h/2,m10:s,m11:c,m12:n.y+n.h/2-s*n.w/2-c*n.h/2};}
   if(n.radii){const [tl,tr,br,bl]=n.radii;node.cornerRadius=tl;node.rectangleCornerRadiiIndependent=true;node.rectangleTopLeftCornerRadius=tl;node.rectangleTopRightCornerRadius=tr;node.rectangleBottomRightCornerRadius=br;node.rectangleBottomLeftCornerRadius=bl;}
   if(n.shadows?.length)node.effects=n.shadows.map(s=>({type:s.inset?'INNER_SHADOW':'DROP_SHADOW',color:rgba(s.color),offset:{x:s.x,y:s.y},radius:s.blur,spread:s.spread,visible:true,blendMode:'NORMAL',showShadowBehindNode:true}));
   if(n.kind==='text'){
    node.textData={characters:n.text!,lines:[{lineType:'PLAIN',indentationLevel:0,styleId:0}]};node.fontSize=n.fontSize;node.fontName={family:n.font,style:n.bold?(n.italic?'Bold Italic':'Bold'):(n.italic?'Italic':'Regular'),...{postscript:''}};node.textAutoResize='NONE';node.textAlignHorizontal=n.rtl?'RIGHT':'LEFT';node.textAlignVertical='TOP';node.letterSpacing={value:n.letterSpacing??0,units:'PIXELS'};
    if(n.underline)node.textDecoration='UNDERLINE';else if(n.strike)node.textDecoration='STRIKETHROUGH';
   }
   if(n.kind==='path'){
    Object.assign(node,appendVectorPayloadToDocument(doc,{width:n.w,height:n.h,fillPaths:[{svgPath:n.path!}]}));
    if(!n.fill){message.blobs[node.vectorData.vectorNetworkBlob]={bytes:encodeVectorNetworkBlob([parseSVGPathData(n.path!)],{emitRegions:false})};node.fillGeometry=[];}
   }
   if(n.kind==='image'){
    const bytes=pngBytes(n.image!),digest=new Uint8Array(await crypto.subtle.digest('SHA-1',bytes as BufferSource)),hash=Array.from(digest,b=>b.toString(16).padStart(2,'0')).join('');images.set(hash,bytes);
    node.fillPaints=[{type:'IMAGE',opacity:1,visible:true,blendMode:'NORMAL',image:{hash:digest},imageScaleMode:'STRETCH',transform:transform()} as FigPaint];
   }
  }
 }
 const schemaCompressed=Uint8Array.from(atob(schemaBase64),c=>c.charCodeAt(0)),raw=encodeMessage(message);
 if(raw.length>48*1024*1024)throw Error('Figma 图层数据超过 48MiB');
 const binary=assembleCanvasFig({prelude:'fig-kiwi',version,schemaCompressed,messageCompressed:zstdRaw(raw)});
 const bytes=createFigZip({canvasFig:binary,images,thumbnail:thumbnail?pngBytes(thumbnail):undefined,meta:{file_name:title,version:0,exported_at:new Date().toISOString(),client_meta:{backgroundColor:{r:.94,g:.94,b:.95,a:1}},xiaoban:{warnings:pages.flatMap(p=>p.scene.warnings.map(w=>`${p.title}：${w}`))}}});
 return bytesDataUrl(bytes,'application/x-figma');
}
