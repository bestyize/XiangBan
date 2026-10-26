export interface Board { file:string; title:string; x:number; y:number; w:number; h:number; page?:string; [key:string]:unknown }
export interface Comment { id:string; file:string; node?:string; text:string; revision:number; status:'saved'|'sent'|'resolved' }
export interface Project { id:string; title:string; sessionId:string; revision:number; updatedMs:number; canvas:Record<string,any>; files:Record<string,string>; requestId?:string }
export interface Summary { id:string; title:string; sessionId:string; revision:number; updatedMs:number; artboards:number }
export interface Payload { project:Project; origin:string; draft?:Project }
export function boardsOf(canvas:Project['canvas']):Board[]{
  const raw=canvas.boards ? (canvas.order as string[]).map(file=>({file,...canvas.boards[file]})) : (canvas.artboards??[]);
  return raw.map((b:any)=>({...b,file:b.file,title:b.title||b.file,x:b.x??0,y:b.y??0,w:b.w,h:b.h}));
}
export const clone=<T>(value:T):T=>JSON.parse(JSON.stringify(value));
export function changeBoard(p:Project,file:string,patch:Partial<Board>){
  if(p.canvas.boards)Object.assign(p.canvas.boards[file],patch);
  else Object.assign(p.canvas.artboards.find((b:Board)=>b.file===file),patch);
}
export function addBoard(p:Project,from?:Board):string{
  let n=1;while(p.files[`Board${n}.dc.html`])n++;
  const file=`Board${n}.dc.html`,bs=boardsOf(p.canvas),last=bs.at(-1);
  const b={x:last?last.x+last.w+60:0,y:last?.y??0,w:from?.w??1280,h:from?.h??800,title:from?`${from.title} 副本`:`画板 ${n}`};
  if(p.canvas.boards){p.canvas.boards[file]=b;p.canvas.order.push(file);}else p.canvas.artboards.push({file,...b});
  p.files[file]=from?p.files[from.file]:'<!doctype html><html><head><meta charset="utf-8"><script src="./support.js"></script></head><body><x-dc><main style="padding:40px;font-family:system-ui"><h1>新画板</h1><p>告诉小伴你想设计什么。</p></main></x-dc></body></html>';
  return file;
}
export function removeBoard(p:Project,file:string){
  if(boardsOf(p.canvas).length<=1)throw Error('至少保留一个画板');
  if(p.canvas.boards){delete p.canvas.boards[file];p.canvas.order=p.canvas.order.filter((f:string)=>f!==file);}else p.canvas.artboards=p.canvas.artboards.filter((b:Board)=>b.file!==file);
  // Retain the source: other components or comments may still refer to it.
}
export function sourceDoc(source:string){return new DOMParser().parseFromString(source,'text/html');}
export function assignNodeIds(doc:Document){
  const root=doc.querySelector('x-dc')??doc.body;const used=new Set([...root.querySelectorAll('[data-ocd-node]')].map(e=>e.getAttribute('data-ocd-node')));
  let n=0;root.querySelectorAll('*').forEach(el=>{if(!el.hasAttribute('data-ocd-node')){while(used.has(`n${n}`))n++;el.setAttribute('data-ocd-node',`n${n}`);used.add(`n${n++}`);}});
}
export function patchNode(source:string,id:string,patch:{text?:string;style?:Record<string,string>}){
  const doc=sourceDoc(source);assignNodeIds(doc);const target=[...doc.querySelectorAll('[data-ocd-node]')].find(e=>e.getAttribute('data-ocd-node')===id);
  if(!target)throw Error('元素已经变化，请重新选择');
  if(patch.text!==undefined){if(target.children.length||target.textContent?.includes('{{'))throw Error('动态文字或组合元素请通过批注让小伴修改');target.textContent=patch.text;}
  if(patch.style){let style=target.getAttribute('style')??'';for(const [key,value] of Object.entries(patch.style)){if(!/^(color|background-color|font-size|padding|border-radius|width|height)$/.test(key))throw Error('不支持的样式');style=style.replace(new RegExp(`(^|;)\\s*${key}\\s*:[^;]*(;|$)`,'gi'),'$1');style+=`;${key}:${value};`;}target.setAttribute('style',style);}
  return '<!doctype html>\n'+doc.documentElement.outerHTML;
}
export function propsOf(source:string):Record<string,any>{try{return JSON.parse(sourceDoc(source).querySelector('[data-dc-script]')?.getAttribute('data-props')??'{}');}catch{return {};}}
export function patchProp(source:string,key:string,value:unknown){const doc=sourceDoc(source),script=doc.querySelector('[data-dc-script]');if(!script)throw Error('画板没有可调参数');const config=JSON.parse(script.getAttribute('data-props')??'{}');if(!config[key])throw Error('参数已变化');config[key].default=value;script.setAttribute('data-props',JSON.stringify(config));return '<!doctype html>\n'+doc.documentElement.outerHTML;}
export function targetFile(current:string,href:string,files:string[]):string|null{
  if(!href||/^[a-z]+:|^\/|[?#]/i.test(href))return null;
  const path=current.split('/');path.pop();for(const part of href.split('/')){if(part==='..')path.pop();else if(part!=='.'&&part)path.push(part);}
  const file=path.join('/');return files.includes(file)?file:null;
}
export function previewUrl(origin:string,p:Project,file:string,token:string){return `${origin}/${p.id}/${p.revision}/${file.split('/').map(encodeURIComponent).join('/')}?token=${encodeURIComponent(token)}`;}
