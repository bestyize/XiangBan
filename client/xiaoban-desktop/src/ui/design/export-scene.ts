// Versioned, bounded interchange between the opaque preview and local exporters.
export type Color={hex:string;alpha:number};
export type Fill={type:'solid';color:Color}|{type:'linear'|'radial';angle:number;stops:{offset:number;color:Color}[]};
export type Shadow={x:number;y:number;blur:number;spread:number;inset:boolean;color:Color};
export type ExportNode={
 kind:'shape'|'text'|'image'|'path';name:string;x:number;y:number;w:number;h:number;
 opacity:number;rotation?:number;fill?:Fill;stroke?:{color:Color;width:number;dash?:boolean};
 radii?:number[];shadows?:Shadow[];path?:string;image?:string;
 text?:string;font?:string;fontSize?:number;bold?:boolean;italic?:boolean;underline?:boolean;
 strike?:boolean;letterSpacing?:number;rtl?:boolean;
};
export type ExportScene={version:1;width:number;height:number;nodes:ExportNode[];warnings:string[]};
export type ExportPage={title:string;file:string;x:number;y:number;scene:ExportScene};
export const MAX_EXPORT_BYTES=48*1024*1024;
export function validateScene(value:unknown,width:number,height:number):ExportScene{
 const s=value as ExportScene;
 if(!s||s.version!==1||s.width!==width||s.height!==height||!Array.isArray(s.nodes)||s.nodes.length>20000||!Array.isArray(s.warnings)||s.warnings.length>200)throw Error('画板元素数据无效或超出限制');
 if(JSON.stringify(s).length>MAX_EXPORT_BYTES)throw Error('画板元素数据超过 48MiB');
 const finite=(n:unknown)=>typeof n==='number'&&Number.isFinite(n)&&Math.abs(n)<=1e7;
 const color=(c:Color)=>c&&/^[0-9a-f]{6}$/i.test(c.hex)&&finite(c.alpha)&&c.alpha>=0&&c.alpha<=1;
 for(const n of s.nodes){
  if(!n||!['shape','text','image','path'].includes(n.kind)||![n.x,n.y,n.w,n.h,n.opacity,n.rotation??0].every(finite)||n.w<=0||n.h<=0||n.opacity<0||n.opacity>1||typeof n.name!=='string'||n.name.length>500)throw Error('画板元素尺寸或类型无效');
  if(n.fill&&(n.fill.type==='solid'?!color(n.fill.color):!['linear','radial'].includes(n.fill.type)||!finite(n.fill.angle)||!Array.isArray(n.fill.stops)||n.fill.stops.length>100||n.fill.stops.length<2||n.fill.stops.some(t=>!finite(t.offset)||t.offset<0||t.offset>1||!color(t.color))))throw Error('画板填充数据无效');
  if(n.stroke&&(!color(n.stroke.color)||!finite(n.stroke.width)||n.stroke.width<0))throw Error('画板描边无效');
  if(n.radii&&(!Array.isArray(n.radii)||n.radii.length!==4||!n.radii.every(r=>finite(r)&&r>=0)))throw Error('画板圆角无效');
  if(n.shadows&&(!Array.isArray(n.shadows)||n.shadows.length>20||n.shadows.some(t=>![t.x,t.y,t.blur,t.spread].every(finite)||!color(t.color))))throw Error('画板阴影无效');
  if(n.kind==='image'&&(typeof n.image!=='string'||!/^data:image\/png;base64,[A-Za-z0-9+/=]+$/.test(n.image)))throw Error('导出仅接受已解码的本地 PNG 图片');
  if(n.kind==='path'&&(typeof n.path!=='string'||n.path.length>2000000||!/^[MLCQZmlcqz0-9eE.,\s+\-]+$/.test(n.path)))throw Error('画板矢量路径无效');
  if(n.kind==='text'&&(typeof n.text!=='string'||n.text.length>100000||typeof n.font!=='string'||n.font.length>300||!finite(n.fontSize)||(n.fontSize??0)<=0||!finite(n.letterSpacing??0)))throw Error('画板文字数据无效');
 }
 if(s.warnings.some(w=>typeof w!=='string'||w.length>500))throw Error('画板导出提示无效');
 return s;
}
export function bytesDataUrl(bytes:Uint8Array,mime:string){
 if(bytes.length>MAX_EXPORT_BYTES)throw Error('导出文件超过 48MiB，请减少画板数量');
 let binary='';for(let i=0;i<bytes.length;i+=32768)binary+=String.fromCharCode(...bytes.subarray(i,i+32768));
 return `data:${mime};base64,${btoa(binary)}`;
}
export function pngBytes(data:string){return Uint8Array.from(atob(data.slice(data.indexOf(',')+1)),c=>c.charCodeAt(0));}
