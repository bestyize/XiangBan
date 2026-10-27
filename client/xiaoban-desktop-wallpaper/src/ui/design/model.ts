export interface Board { file:string; title:string; x:number; y:number; w:number; h:number; page?:string; [key:string]:unknown }
export interface Comment {
  id:string;
  file:string;
  node?:string;
  tag?:string;
  path?:string;
  quote?:string;
  text:string;
  revision:number;
  status:'saved'|'sent'|'resolved';
}
export interface DsCard { name:string; path:string; group:string; subtitle:string; viewport:{width:number;height:number} }
export type ProjectStatus = 'draft'|'generating'|'ready'|'failed';
export const statusLabel = (status?:ProjectStatus) => ({draft:'草稿',generating:'生成中',ready:'已完成',failed:'待恢复'}[status??'ready']);
export interface Project { status?:ProjectStatus; id:string; title:string; sessionId:string; sessionIds?:string[]; revision:number; updatedMs:number; canvas:Record<string,any>; files:Record<string,string>; requestId?:string }
export interface Summary { active?:boolean;detached?:boolean; status?:ProjectStatus; issues?:string[]; id:string; title:string; sessionId:string; sessionIds?:string[]; revision:number; updatedMs:number; artboards:number }
export interface Payload { project:Project; origin:string; draft?:Project; draftError?:string }

export const DEVICE_PRESETS: { id:string; label:string; w:number; h:number }[] = [
  { id:'desktop', label:'桌面 1280×800', w:1280, h:800 },
  { id:'tablet', label:'平板 768×1024', w:768, h:1024 },
  { id:'mobile', label:'手机 390×844', w:390, h:844 },
];

const ALLOWED_STYLES = /^(color|background-color|font-size|font-weight|text-align|padding|margin|border-radius|border-color|border-width|gap|width|height|opacity|display|flex-direction|align-items|justify-content)$/;

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
  if(patch.style){let style=target.getAttribute('style')??'';for(const [key,value] of Object.entries(patch.style)){if(!ALLOWED_STYLES.test(key))throw Error('不支持的样式');style=style.replace(new RegExp(`(^|;)\\s*${key}\\s*:[^;]*(;|$)`,'gi'),'$1');if(value!=='')style+=`;${key}:${value};`;}target.setAttribute('style',style);}
  return '<!doctype html>\n'+doc.documentElement.outerHTML;
}
export function propsOf(source:string):Record<string,any>{try{return JSON.parse(sourceDoc(source).querySelector('[data-dc-script]')?.getAttribute('data-props')??'{}');}catch{return {};}}
export function patchProp(source:string,key:string,value:unknown){const doc=sourceDoc(source),script=doc.querySelector('[data-dc-script]');if(!script)throw Error('画板没有可调参数');const config=JSON.parse(script.getAttribute('data-props')??'{}');if(!config[key])throw Error('参数已变化');config[key].default=value;script.setAttribute('data-props',JSON.stringify(config));return '<!doctype html>\n'+doc.documentElement.outerHTML;}
export function tokensOf(canvas:Project['canvas']):Record<string,string>{
  const out:Record<string,string>={};
  const systems=Array.isArray(canvas?.designSystems)?canvas.designSystems:[];
  for(const sys of systems){if(sys?.tokens&&typeof sys.tokens==='object'){for(const [k,v] of Object.entries(sys.tokens))if(v!==undefined&&v!==null)out[k]=String(v);}}
  if(canvas?.tokens&&typeof canvas.tokens==='object'){for(const [k,v] of Object.entries(canvas.tokens))if(v!==undefined&&v!==null)out[k]=String(v);}
  return out;
}
export function setToken(p:Project,key:string,value:string){
  const clean=key.trim().replace(/[^A-Za-z0-9_-]/g,'');
  if(!clean)throw Error('令牌名称无效');
  if(!Array.isArray(p.canvas.designSystems)||!p.canvas.designSystems.length)p.canvas.designSystems=[{name:'Brand',tokens:{}}];
  p.canvas.designSystems[0].tokens??={};
  if(value==='')delete p.canvas.designSystems[0].tokens[clean];
  else p.canvas.designSystems[0].tokens[clean]=value;
}
export function dsCardsOf(p:Project):DsCard[]{
  const out:DsCard[]=[];
  const re=/^\s*<!--\s*@dsCard\b([\s\S]*?)-->/;
  const attrRe=/([A-Za-z0-9_-]+)\s*=\s*["']([^"']*)["']/g;
  for(const [path,source] of Object.entries(p.files)){
    if(!path.endsWith('.dc.html')&&!path.endsWith('.html'))continue;
    const m=re.exec(source);if(!m)continue;
    const attrs:Record<string,string>={};
    for(const a of m[1].matchAll(attrRe))attrs[a[1].toLowerCase()]=a[2];
    const stem=path.split('/').pop()!.replace(/\.dc\.html$|\.html$/,'');
    out.push({
      name:attrs.name||stem,
      path,
      group:attrs.group||'Components',
      subtitle:attrs.subtitle||'',
      viewport:{width:Number(attrs.width)||360,height:Number(attrs.height)||220},
    });
  }
  return out;
}
export function insertComponentImport(source:string,componentPath:string):string{
  const doc=sourceDoc(source);
  const root=doc.querySelector('x-dc main')??doc.querySelector('x-dc')??doc.body;
  const tag=doc.createElement('dc-import');
  tag.setAttribute('src',componentPath);
  root.appendChild(tag);
  return '<!doctype html>\n'+doc.documentElement.outerHTML;
}
export function formatCommentPrompt(project:Project,comments:Comment[]):string{
  if(comments.length===1){
    const c=comments[0];
    const anchor=[
      c.node?`元素 ${c.node}`:'',
      c.path?`路径 ${c.path}`:'',
      c.quote?`选中文字「${c.quote}」`:'',
    ].filter(Boolean).join('，');
    return `请修改设计稿 ${project.id}，画板 ${c.file}${anchor?`，${anchor}`:''}（批注 ${c.id}，基于版本 ${c.revision}）。用户要求：\n${c.text}\n先读取最新版，保留其他改动，完成后在 canvas.xiaoban.comments 中将本批注标记为 resolved。`;
  }
  const lines=comments.map((c,i)=>{
    const loc=[c.file,c.node?`元素 ${c.node}`:'',c.path?`路径 ${c.path}`:'',c.quote?`文字「${c.quote}」`:''].filter(Boolean).join(' · ');
    return `${i+1}. [${c.id}] (${loc})：${c.text}`;
  }).join('\n');
  return `请批量处理设计稿 ${project.id}（基于版本 ${project.revision}）的以下 ${comments.length} 条批注：\n${lines}\n先读取最新版，保留其他改动，完成后在 canvas.xiaoban.comments 中将这些批注全部标记为 resolved。`;
}
export function toMarkdownSpec(project:Project):string{
  const bs=boardsOf(project.canvas),tokens=tokensOf(project.canvas),cards=dsCardsOf(project);
  const comments:Comment[]=project.canvas.xiaoban?.comments??[];
  let md=`# ${project.title}\n\n- 设计稿编号：\`${project.id}\`\n- 版本：\`v${project.revision}\`\n- 画板数：${bs.length}\n\n## 画板清单\n\n| 画板 | 文件 | 尺寸 | 页面 |\n| --- | --- | --- | --- |\n`;
  for(const b of bs)md+=`| ${b.title} | \`${b.file}\` | ${b.w}×${b.h} | ${b.page||'默认'} |\n`;
  const tokenEntries=Object.entries(tokens);
  if(tokenEntries.length){
    md+=`\n## 设计系统令牌\n\n| 令牌 | 变量 | 值 |\n| --- | --- | --- |\n`;
    for(const [k,v] of tokenEntries)md+=`| \`${k}\` | \`--ds-${k}\` | \`${v}\` |\n`;
  }
  if(cards.length){
    md+=`\n## 组件卡片 (@dsCard)\n\n| 分组 | 名称 | 路径 | 说明 |\n| --- | --- | --- | --- |\n`;
    for(const c of cards)md+=`| ${c.group} | ${c.name} | \`${c.path}\` | ${c.subtitle} |\n`;
  }
  if(comments.length){
    md+=`\n## 批注记录\n\n`;
    for(const c of comments)md+=`- **[${c.status}]** \`${c.file}\`${c.node?` (\`${c.node}\`)`:''}${c.path?` \`${c.path}\``:''}: ${c.text}\n`;
  }
  return md;
}
export function targetFile(current:string,href:string,files:string[]):string|null{
  if(!href||/^[a-z]+:|^\/|[?#]/i.test(href))return null;
  const path=current.split('/');path.pop();for(const part of href.split('/')){if(part==='..')path.pop();else if(part!=='.'&&part)path.push(part);}
  const file=path.join('/');return files.includes(file)?file:null;
}
export function previewUrl(origin:string,p:Project,file:string,token:string){return `${origin}/${p.id}/${p.revision}/${file.split('/').map(encodeURIComponent).join('/')}?token=${encodeURIComponent(token)}`;}
