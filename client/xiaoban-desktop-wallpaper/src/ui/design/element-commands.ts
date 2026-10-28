import {assignNodeIds,sourceDoc} from './model.ts';
export interface ElementLayer {id:string;parent?:string;depth:number;tag:string;label:string;locked:boolean;hidden:boolean;generated:boolean}
export interface Fragment {kind:'open-claude-design/elements';html:string[]}
export interface Mutation {source:string;ids:string[]}
export type ElementAction =
 | {type:'delete'|'duplicate'|'group'|'ungroup';ids:string[]}
 | {type:'style';patches:{id:string;style:Record<string,string>}[]}
 | {type:'text';id:string;text:string}
 | {type:'reorder';ids:string[];before?:string;direction?:'previous'|'next'|'front'|'back'}
 | {type:'paste';fragment:Fragment;parent?:string}
 | {type:'insert';kind:'text'|'rectangle'|'ellipse'|'note';parent?:string}
 | {type:'lock'|'hide';ids:string[];value:boolean};
const protectedTags=new Set(['script','style','helmet','head','meta','link','base','html','body','x-dc','sc-for','sc-if','sc-else','sc-else-if']);
const allowedStyle=/^(translate|scale|rotate|width|height|min-width|min-height|box-sizing|left|top|position|z-index|color|background-color|font-size|font-weight|text-align|padding|margin|border-radius|border-color|border-width|opacity|display|flex-direction|gap|align-items|justify-content)$/;
const rootOf=(d:Document)=>d.querySelector('x-dc')??d.body;
const uid=()=>`ocd-${crypto.randomUUID()}`;
function read(source:string){const d=sourceDoc(source);assignNodeIds(d);return d;}
function serialize(d:Document){return '<!doctype html>\n'+[...d.childNodes].filter(n=>n.nodeType!==10).map(n=>n.nodeType===8?`<!--${n.textContent}-->`:n instanceof Element?n.outerHTML:n.textContent??'').join('');}
function find(d:Document,id:string){const els=[...rootOf(d).querySelectorAll('[data-ocd-node]')].filter(n=>n.getAttribute('data-ocd-node')===id);if(els.length!==1)throw Error('元素标识已变化或重复，请重新选择');return els[0] as HTMLElement;}
function generated(el:Element){return !!el.closest('sc-for,dc-import');}
function editable(el:Element,structural=true){
 if(protectedTags.has(el.localName))throw Error('不能直接修改设计程序或模板节点');
 if(generated(el))throw Error('循环或组件生成的元素请通过批注修改源模板');
 if(el.closest('[data-ocd-locked="true"]')||(structural&&el.querySelector('[data-ocd-locked="true"]')))throw Error('请先解锁所选元素');
}
function selection(d:Document,ids:string[],check=true){
 if(!Array.isArray(ids)||!ids.length||ids.length>500||ids.some(id=>typeof id!=='string'))throw Error('请选择 1 到 500 个元素');
 const els=[...new Set(ids)].map(id=>find(d,id));const roots=els.filter(el=>!els.some(other=>other!==el&&other.contains(el)));
 roots.sort((a,b)=>a.compareDocumentPosition(b)&Node.DOCUMENT_POSITION_FOLLOWING?-1:1);if(check)roots.forEach(el=>editable(el));return roots;
}
function overrides(el:Element,style:Record<string,string>){
 const parser=el.ownerDocument.createElement('div');parser.setAttribute('style',el.getAttribute('data-ocd-style')??'');
 for(const [key,value] of Object.entries(style)){if(typeof value!=='string'||!allowedStyle.test(key)||value.length>180||/[{};<>]|url\s*\(|expression\s*\(/i.test(value))throw Error('不支持的元素样式');if(value)parser.style.setProperty(key,value);else parser.style.removeProperty(key);}
 el.setAttribute('data-ocd-style',parser.getAttribute('style')??'');
}
function copies(els:Element[]){
 const map=new Map<string,string>();const nodes=els.map(el=>el.cloneNode(true) as HTMLElement);
 const all=nodes.flatMap(n=>[n,...n.querySelectorAll('*')]);
 for(const el of all){el.setAttribute('data-ocd-node',uid());if(el.id&&!el.id.includes('{{')){const next=`${el.id}-copy-${crypto.randomUUID().slice(0,8)}`;map.set(el.id,next);el.id=next;}}
 for(const el of all)for(const a of [...el.attributes]){if(a.name==='id'||a.value.includes('{{'))continue;let v=a.value;
  for(const [old,next] of map){if(['for','aria-labelledby','aria-describedby'].includes(a.name))v=v.split(/\s+/).map(t=>t===old?next:t).join(' ');if(v===`#${old}`)v=`#${next}`;v=v.split(`url(#${old})`).join(`url(#${next})`);}
  if(v!==a.value)el.setAttribute(a.name,v);
 }
 return nodes;
}
export function layersOf(source:string):ElementLayer[]{const d=read(source),root=rootOf(d);return [...root.querySelectorAll('[data-ocd-node]')].filter(el=>!protectedTags.has(el.localName)).map(el=>{
 let depth=0,parent=el.parentElement;while(parent&&parent!==root){if(!protectedTags.has(parent.localName))depth++;parent=parent.parentElement;}
 return {id:el.getAttribute('data-ocd-node')!,parent:el.parentElement?.getAttribute('data-ocd-node')??undefined,depth,tag:el.tagName,label:el.getAttribute('aria-label')||el.id||(el.textContent??'').trim().slice(0,32)||el.localName,locked:!!el.closest('[data-ocd-locked="true"]'),hidden:!!el.closest('[data-ocd-hidden="true"]'),generated:generated(el)};
});}
export function copyElements(source:string,ids:string[]):Fragment{const d=read(source);return {kind:'open-claude-design/elements',html:selection(d,ids).map(el=>el.outerHTML)};}
export function mutateElements(source:string,action:ElementAction):Mutation{
 if(!action||!['delete','duplicate','group','ungroup','style','text','reorder','paste','insert','lock','hide'].includes(action.type))throw Error('不支持的画布指令');
 const d=read(source),root=rootOf(d);let selected:Element[]=[];
 if('ids' in action)selected=selection(d,action.ids,action.type!=='lock'&&action.type!=='hide');
 let ids=selected.map(el=>el.getAttribute('data-ocd-node')!);
 if(action.type==='delete'){selected.forEach(el=>el.remove());ids=[];}
 if(action.type==='duplicate'){ids=[];for(const el of selected){const copy=copies([el])[0];el.after(copy);ids.push(copy.getAttribute('data-ocd-node')!);}}
 if(action.type==='group'){
  const parent=selected[0].parentElement!;if(selected.length<2||selected.some(el=>el.parentElement!==parent))throw Error('请选择同一容器内至少两个元素进行编组');
  const group=d.createElement('div');group.setAttribute('data-ocd-node',uid());group.setAttribute('data-ocd-group','true');group.style.display='contents';selected[0].before(group);group.append(...selected);ids=[group.getAttribute('data-ocd-node')!];
 }
 if(action.type==='ungroup'){ids=[];for(const el of selected){if(el.getAttribute('data-ocd-group')!=='true')throw Error('只能解散通过画布创建的编组');ids.push(...[...el.children].map(n=>n.getAttribute('data-ocd-node')!).filter(Boolean));el.replaceWith(...el.childNodes);}}
 if(action.type==='style'){if(!action.patches.length||action.patches.length>500)throw Error('修改元素过多');ids=[];for(const patch of action.patches){const el=find(d,patch.id);editable(el,false);overrides(el,patch.style);ids.push(patch.id);}}
 if(action.type==='text'){const el=find(d,action.id);editable(el,false);if(el.children.length||el.textContent?.includes('{{'))throw Error('动态文字或组合元素请通过批注修改');if(typeof action.text!=='string'||action.text.length>50000)throw Error('文字过长');el.textContent=action.text;ids=[action.id];}
 if(action.type==='reorder'){
  const parent=selected[0].parentElement!;if(selected.some(el=>el.parentElement!==parent))throw Error('排序需要在同一容器内进行');
  if(action.direction==='front')parent.append(...selected);else if(action.direction==='back')parent.prepend(...selected);
  else if(action.direction==='previous'){const before=selected[0].previousElementSibling;if(before)before.before(...selected);}
  else if(action.direction==='next'){const after=selected.at(-1)!.nextElementSibling;if(after)after.after(...selected);}
  else {const before=action.before?find(d,action.before):null;if(before?.parentElement!==parent&&before)throw Error('不能移入其他容器');if(!before||!selected.includes(before))for(const el of selected)parent.insertBefore(el,before);}
 }
 if(action.type==='lock'||action.type==='hide'){for(const el of selected)el.setAttribute(action.type==='lock'?'data-ocd-locked':'data-ocd-hidden',String(action.value));}
 if(action.type==='paste'){
  if(action.fragment?.kind!=='open-claude-design/elements'||!Array.isArray(action.fragment.html)||action.fragment.html.length>500||action.fragment.html.some(s=>typeof s!=='string')||action.fragment.html.join('').length>500000)throw Error('不是有效的设计元素剪贴板');
  const holder=d.createElement('div');holder.innerHTML=action.fragment.html.join('');if(holder.querySelector('script,iframe,object,embed,base,meta,link'))throw Error('剪贴板包含不支持的程序节点');
  const parent=action.parent?find(d,action.parent):root;if(parent!==root)editable(parent,false);const nodes=copies([...holder.children]);parent.append(...nodes);ids=nodes.map(n=>n.getAttribute('data-ocd-node')!);
 }
 if(action.type==='insert'){
  const parent=action.parent?find(d,action.parent):root;if(parent!==root)editable(parent,false);
  const el=d.createElement(action.kind==='text'?'span':action.kind==='note'?'aside':'div');el.setAttribute('data-ocd-node',uid());
  el.style.cssText='position:absolute;left:32px;top:32px;box-sizing:border-box;';
  if(action.kind==='text'){el.textContent='双击编辑文字';el.style.cssText+='font:24px/1.4 system-ui;color:#30333b;white-space:pre-wrap;';}
  else if(action.kind==='note'){el.textContent='双击编辑便签';el.setAttribute('data-ocd-note','true');el.style.cssText+='width:200px;min-height:120px;padding:16px;background:var(--ds-note,#fef3c7);color:#30333b;white-space:pre-wrap;';}
  else el.style.cssText+=`width:160px;height:100px;background:var(--ds-primary,#d9536b);border-radius:${action.kind==='ellipse'?'50%':'8px'};`;
  parent.append(el);ids=[el.getAttribute('data-ocd-node')!];
 }
 return {source:serialize(d),ids};
}
