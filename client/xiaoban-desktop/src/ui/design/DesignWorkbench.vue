<script setup lang="ts">
import {computed,onBeforeUnmount,onMounted,nextTick,ref,watch} from 'vue';
import {Check,Copy,Download,Hand,ExternalLink,Pencil,Maximize2,MessageSquare,Minus,MousePointer2,PanelRight,Play,Plus,RotateCcw,RotateCw,Send,Sun,Moon,Trash2,X,ClipboardCopy,Palette,Layers} from 'lucide-vue-next';
import {invoke} from '../../bridge.ts';
import BoardPreview from './BoardPreview.vue';
import ElementToolbar from './ElementToolbar.vue';
import {mutateElements,copyElements,layersOf,textFragment,fragmentNeedsContext,type ElementAction,type Fragment} from './element-commands.ts';
import {
  statusLabel,addBoard,boardsOf,changeBoard,clone,patchNode,patchProp,previewUrl,propsOf,removeBoard,targetFile,
  tokensOf,setToken,dsCardsOf,insertComponentImport,formatCommentPrompt,toMarkdownSpec,DEVICE_PRESETS,
  type Board,type Comment,type Project,type Summary,type DsCard
} from './model.ts';
import {clampScale,zoomAt,wheelPixels,type CanvasView} from './viewport.ts';
import {createDesignStore} from './design-store.ts';
const props=defineProps<{projectId:string;detached?:boolean;notify:(message:unknown,type?:'success'|'error'|'info')=>void}>();
const emit=defineEmits<{close:[];detach:[];dock:[];copied:[id:string];request:[message:string,session:string]}>();
const ds=createDesignStore(props.notify),selected=ref(''),page=ref(''),mode=ref<'select'|'play'|'hand'>('select'),scale=ref(.4),properties=ref(false),spaceHeld=ref(false),viewport=ref<HTMLElement|null>(null);
const theme=ref<'light'|'dark'>('light'),hideResolved=ref(false),renameOpen=ref(false),projectTitle=ref('');let offDeleted:(()=>void)|undefined,unmounted=false;
const activeTab=ref<'properties'|'design-system'>('properties');
const selectedIds=ref<string[]>([]),selectionFile=ref(''),commentTool=ref(false),guides=ref(true),editorCommand=ref<{seq:number;command:string}>(),previewEpoch=ref(0),context=ref<{x:number;y:number}|null>(null);
let commandSeq=0,preserveSelection=false,clipboard:Fragment|undefined;
const node=ref<{readonly?:boolean;group?:boolean;id:string;text:string;tag:string;path?:string;quote?:string;styles?:Record<string,string>}|null>(null);
const nodeText=ref(''),commentText=ref(''),sendComment=ref(true),commentOpen=ref(false),sourceOpen=ref(false),source=ref(''),renderErrors=ref<Record<string,string>>({});
const history=ref<Project[]>([]),future=ref<Project[]>([]),versions=ref<Summary[]>([]),versionsOpen=ref(false);
const newTokenKey=ref(''),newTokenValue=ref('');
let sourceRevision=0,remoteEpoch=0,viewReady=false;let viewTimer:ReturnType<typeof setTimeout>|undefined;
const token=ref(crypto.randomUUID());const project=computed(()=>ds.project);
const allBoards=computed(()=>project.value?boardsOf(project.value.canvas):[]),pages=computed(()=>project.value?.canvas.pages??[]);
const boards=computed(()=>allBoards.value.filter(b=>!page.value||b.page===page.value));
const current=computed(()=>allBoards.value.find(b=>b.file===selected.value)??allBoards.value[0]);
const collapsedLayers=ref(new Set<string>());
const layers=computed(()=>project.value&&current.value?layersOf(project.value.files[current.value.file]??''):[]);
const layerParents=computed(()=>new Map(layers.value.map(n=>[n.id,n.parent])));
const layerContainers=computed(()=>new Set(layers.value.map(n=>n.parent).filter(Boolean)));
const visibleLayers=computed(()=>layers.value.filter(n=>{let p=n.parent;while(p){if(collapsedLayers.value.has(p))return false;p=layerParents.value.get(p);}return true;}));
function toggleLayer(id:string){if(collapsedLayers.value.has(id))collapsedLayers.value.delete(id);else collapsedLayers.value.add(id);}
const config=computed(()=>project.value&&current.value?Object.entries(propsOf(project.value.files[current.value.file]??'')).filter(([key,c])=>key!=='$preview'&&c&&typeof c==='object'):[]);
const allComments=computed<Comment[]>(()=>project.value?.canvas.xiaoban?.comments??[]);
const comments=computed<Comment[]>(()=>hideResolved.value?allComments.value.filter(c=>c.status!=='resolved'):allComments.value);
const pendingComments=computed(()=>allComments.value.filter(c=>c.status!=='resolved'));
const tokens=computed(()=>project.value?tokensOf(project.value.canvas):{});
const dsCards=computed<DsCard[]>(()=>project.value?dsCardsOf(project.value):[]);
const minX=computed(()=>Math.min(0,...boards.value.map(b=>b.x))),minY=computed(()=>Math.min(0,...boards.value.map(b=>b.y)));
const extent=computed(()=>({w:Math.max(600,...boards.value.map(b=>b.x+b.w))-minX.value+96,h:Math.max(500,...boards.value.map(b=>b.y+b.h))-minY.value+140}));
const canEdit=computed(()=>!!project.value&&!ds.busy&&!ds.conflict);

function boardPins(file:string){
  return allComments.value
    .filter(c=>c.file===file&&c.status!=='resolved')
    .map((c,idx)=>({num:idx+1,node:c.node,text:c.text}));
}

function close(){if(ds.busy){props.notify('正在保存，请稍候关闭','info');return;}emit('close');}
function report(e:unknown){props.notify(e,'error');}
async function open(id:string){
  remoteEpoch++;viewReady=false;
  try{
    await ds.open(id);
    if(project.value?.id!==id)return;
    selected.value=allBoards.value[0]?.file??'';
    page.value='';clearElements();sourceOpen.value=false;commentOpen.value=false;versionsOpen.value=false;
    history.value=[];future.value=[];renderErrors.value={};token.value=crypto.randomUUID();
    const view=await invoke<CanvasView|null>('design_view_read',{id}).catch(()=>null);
    if(project.value?.id!==id)return;
    if(view){
      if(allBoards.value.some(b=>b.file===view.selected))selected.value=view.selected!;
      if(pages.value.some((p:any)=>p.id===view.page))page.value=view.page!;
      if(Number.isFinite(view.scale))scale.value=clampScale(view.scale!);
      if(['select','play','hand'].includes(view.mode??''))mode.value=view.mode!;
      if(view.theme==='dark'||view.theme==='light')theme.value=view.theme;
      await nextTick();if(viewport.value){viewport.value.scrollLeft=Math.max(0,Number(view.left)||0);viewport.value.scrollTop=Math.max(0,Number(view.top)||0);}
    }
    viewReady=true;
  }catch(e){ds.error=String(e);report(e);}
}
watch(()=>props.projectId,id=>void open(id),{immediate:true});
onMounted(()=>void ds.init(summary=>{if(summary.id===project.value?.id&&summary.revision>project.value.revision&&!ds.busy){remoteEpoch++;history.value=[];future.value=[];clearElements();}}).catch(report));
function scheduleView(){if(!viewReady)return;clearTimeout(viewTimer);viewTimer=setTimeout(()=>void rememberView().catch(()=>{}),200);}
watch([selected,page,scale,mode,theme],scheduleView);
function rememberView(){if(!project.value)return Promise.resolve();return invoke('design_view_save',{id:project.value.id,view:{selected:selected.value,page:page.value,scale:scale.value,left:viewport.value?.scrollLeft??0,top:viewport.value?.scrollTop??0,mode:mode.value,theme:theme.value}});}
async function handoff(){if(ds.busy||ds.dirty){props.notify('请先保存当前修改再切换窗口','info');return;}try{await rememberView();if(props.detached)emit('dock');else emit('detach');}catch(e){report(e);}}
function releaseSpace(){spaceHeld.value=false;cancelDrag();}
onMounted(async()=>{offDeleted=await (await import('../../bridge.ts')).listen<{id:string}>('design-deleted',row=>{if(row.id===project.value?.id){ds.close();props.notify('设计稿已被删除','info');}});if(unmounted){offDeleted();return;}window.addEventListener('keyup',keyup);window.addEventListener('blur',releaseSpace);});
onBeforeUnmount(()=>{unmounted=true;offDeleted?.();clearTimeout(viewTimer);void rememberView().catch(()=>{});ds.dispose();window.removeEventListener('keyup',keyup);window.removeEventListener('blur',releaseSpace);});
watch(()=>project.value?.revision,()=>{if(!allBoards.value.some(b=>b.file===selected.value))selected.value=allBoards.value[0]?.file??'';renderErrors.value={};node.value=null;if(!preserveSelection)clearElements();});

async function copyProject(){
 if(!project.value||!canEdit.value)return;
 try{const result=await invoke<{project:Project}>('design_copy',{id:project.value.id,session:project.value.sessionIds?.at(-1)??project.value.sessionId});
  if(!props.detached)emit('copied',result.project.id);props.notify('副本已保存','success');
 }catch(e){report(e);}
}
function renameProject(){projectTitle.value=project.value?.title??'';renameOpen.value=true;}
async function saveTitle(){if(await edit(p=>{p.title=projectTitle.value.trim();}))renameOpen.value=false;}
async function edit(fn:(p:Project)=>void,record=true){
  if(!project.value||!canEdit.value)return false;
  const before=clone(project.value),epoch=remoteEpoch;
  const ok=await ds.commit(fn);
  if(ok&&record&&epoch===remoteEpoch){history.value.push(before);if(history.value.length>30)history.value.shift();future.value=[];}
  return ok;
}
async function undo(){if(!project.value||!canEdit.value||!history.value.length)return;const before=clone(project.value),old=clone(history.value.at(-1)!),epoch=remoteEpoch;old.revision=before.revision;if(await ds.commit(p=>Object.assign(p,old))&&epoch===remoteEpoch){history.value.pop();future.value.push(before);}}
async function redo(){if(!project.value||!canEdit.value||!future.value.length)return;const before=clone(project.value),next=clone(future.value.at(-1)!),epoch=remoteEpoch;next.revision=before.revision;if(await ds.commit(p=>Object.assign(p,next))&&epoch===remoteEpoch){future.value.pop();history.value.push(before);}}

function select(b:Board){selected.value=b.file;clearElements();}
function add(copy=false){void edit(p=>{selected.value=addBoard(p,copy?current.value:undefined);if(page.value)changeBoard(p,selected.value,{page:page.value});});}
async function remove(){if(!current.value)return;const file=current.value.file;try{await edit(p=>removeBoard(p,file));}catch(e){report(e);}}
function patchBoard(patch:Partial<Board>){if(current.value)void edit(p=>changeBoard(p,current.value!.file,patch));}
function applyPreset(preset:{w:number;h:number}){patchBoard({w:preset.w,h:preset.h});}

function clearElements(){node.value=null;selectedIds.value=[];selectionFile.value='';context.value=null;}
function sendEditor(command:string){editorCommand.value={seq:++commandSeq,command};}
function chooseElements(ids:string[]){mode.value='select';selectionFile.value=current.value?.file??'';selectedIds.value=ids;context.value=null;}
async function elementEdit(action:ElementAction,file=current.value?.file){
 if(!file||mode.value!=='select'||commentTool.value||!canEdit.value)return false;
 const epoch=remoteEpoch;let result:ReturnType<typeof mutateElements>|undefined;preserveSelection=true;
 try{const ok=await edit(p=>{result=mutateElements(p.files[file],action);p.files[file]=result.source;});
  if(ok&&epoch===remoteEpoch&&result){if(current.value?.file===file){selectionFile.value=file;selectedIds.value=result.ids;node.value=null;}else clearElements();}else previewEpoch.value++;
  return ok;
 }finally{preserveSelection=false;context.value=null;}
}
function focusArea(reverse=false){const root=viewport.value?.closest('.design-workbench');if(!root)return;const areas=[root.querySelector('header'),viewport.value,root.querySelector('.properties')??root.querySelector('.element-toolbar')].filter(Boolean) as HTMLElement[];const current=areas.findIndex(el=>el.contains(document.activeElement));const next=areas[(current+(reverse?-1:1)+areas.length)%areas.length];(next.querySelector<HTMLElement>('button:not(:disabled),input:not(:disabled)')??next).focus();}
function layerKey(e:KeyboardEvent){const row=(e.target as HTMLElement).closest<HTMLElement>('[data-layer-id]');if(!row)return;const list=visibleLayers.value,index=list.findIndex(n=>n.id===row.dataset.layerId);if(index<0)return;
 if(e.key==='Enter'||e.key===' '){e.preventDefault();e.stopPropagation();if(selectedIds.value.length===1&&selectedIds.value[0]===list[index].id&&e.key==='Enter')sendEditor('editText');else chooseElements([list[index].id]);return;}
 let target=-1;if(e.key==='ArrowUp')target=Math.max(0,index-1);else if(e.key==='ArrowDown')target=Math.min(list.length-1,index+1);else if(e.key==='ArrowRight'){if(collapsedLayers.value.has(list[index].id)){collapsedLayers.value.delete(list[index].id);e.preventDefault();e.stopPropagation();return;}target=list.findIndex(n=>n.parent===list[index].id);}else if(e.key==='ArrowLeft'){if(layerContainers.value.has(list[index].id)&&!collapsedLayers.value.has(list[index].id)){collapsedLayers.value.add(list[index].id);e.preventDefault();e.stopPropagation();return;}target=list.findIndex(n=>n.id===list[index].parent);}else return;e.preventDefault();e.stopPropagation();if(target>=0)row.parentElement?.querySelector<HTMLElement>(`[data-layer-id="${CSS.escape(list[target].id)}"] .layer-select`)?.focus();
}
async function elementCommand(command:string){
 context.value=null;
 if(command==='properties'){properties.value=!properties.value;await nextTick();if(properties.value)viewport.value?.closest('.design-workbench')?.querySelector<HTMLElement>('.properties input,.properties button')?.focus();return;}
 if(command==='focusNext'||command==='focusPrevious'){focusArea(command==='focusPrevious');return;}
 if(command==='guides'){guides.value=!guides.value;return;}
 if(command==='undo'){await undo();return;}if(command==='redo'){await redo();return;}
 if(command==='leaveArtboard'){clearElements();viewport.value?.querySelector<HTMLElement>(`[data-board="${CSS.escape(current.value?.file??'')}"] .board-title`)?.focus();return;}
 if(command==='clear'){clearElements();return;}
 if(command==='editText'||command==='selectAll'||command.startsWith('align:')){sendEditor(command);return;}
 if(!canEdit.value||mode.value!=='select')return;
 const ids=[...selectedIds.value],file=current.value?.file;if(!file||!project.value)return;const identity=project.value.id,revision=project.value.revision;const stillCurrent=()=>project.value?.id===identity&&project.value.revision===revision;
 if(command==='copy'||command==='cut'){
  try{clipboard=copyElements(project.value.files[file],ids);clipboard.origin={project:identity,file};try{await navigator.clipboard.writeText(JSON.stringify(clipboard));}catch{}if(command==='cut'){if(stillCurrent())await elementEdit({type:'delete',ids},file);else props.notify('设计稿已变化，未执行剪切，请重新选择','info');}}catch(e){report(e);}return;
 }
 if(command==='paste'){
  let fragment=clipboard,clipboardText:string|undefined;try{clipboardText=await navigator.clipboard.readText();}catch{}
  if(clipboardText!==undefined){
   if(clipboardText.length>500000){props.notify('剪贴板内容过大','error');return;}
   let candidate;try{candidate=JSON.parse(clipboardText);}catch{}
   if(candidate?.kind==='open-claude-design/elements'){
    if(!Array.isArray(candidate.html)||candidate.html.length>500||candidate.html.some((s:unknown)=>typeof s!=='string')){props.notify('剪贴板中的设计元素无效','error');return;}fragment=candidate;
   }else{try{fragment=clipboardText?textFragment(clipboardText):undefined;}catch(e){report(e);return;}}
  }
  if(fragment&&fragmentNeedsContext(fragment)&&(fragment.origin?.project!==identity||fragment.origin?.file!==file)){props.notify('动态元素依赖原画板的程序，请通过批注跨画板复制','info');return;}
  if(!stillCurrent()){props.notify('设计稿已变化，未执行粘贴，请重试','info');return;}if(fragment)await elementEdit({type:'paste',fragment},file);else props.notify('请先复制画布中的元素','info');return;
 }
 if(!ids.length){if(command==='delete')await remove();else if(command==='duplicate')add(true);return;}
 if(['delete','duplicate','group','ungroup'].includes(command))await elementEdit({type:command as 'delete',ids});
 else if(command==='lock'||command==='unlock')await elementEdit({type:'lock',ids,value:command==='lock'});
 else if(command==='front'||command==='back')await elementEdit({type:'reorder',ids,direction:command});
}
function onPreview(b:Board,data:Record<string,any>){
 if(data.revision!==undefined&&data.revision!==project.value?.revision)return;
 if(data.type==='element-edit'){if(mode.value==='select'&&!commentTool.value)void elementEdit(data.action,b.file);return;}
 if(data.type==='editor-command'){if(mode.value==='select'){selected.value=b.file;if(data.command==='showProperties'){properties.value=true;activeTab.value='properties';context.value=null;}else void elementCommand(String(data.command));}return;}
 if(data.type==='editor-notice'){props.notify(String(data.message),'info');return;}
 if(data.type==='editor-context'){if(mode.value==='select'){const editor=viewport.value?.closest('.editor')?.getBoundingClientRect(),surface=viewport.value?.querySelector(`[data-board="${CSS.escape(b.file)}"] .surface`)?.getBoundingClientRect();if(editor&&surface)context.value={x:Math.max(0,Math.min(editor.width-112,surface.left-editor.left+Number(data.x)*scale.value)),y:Math.max(0,Math.min(editor.height-370,surface.top-editor.top+Number(data.y)*scale.value))};}return;}

  if(data.type==='canvas-wheel'){
    const rect=viewport.value?.querySelector(`[data-board="${CSS.escape(b.file)}"] .surface`)?.getBoundingClientRect();
    if(rect)wheel({deltaX:Number(data.dx)||0,deltaY:Number(data.dy)||0,deltaMode:Number(data.deltaMode)||0,ctrlKey:!!data.ctrl,metaKey:!!data.meta,shiftKey:!!data.shift,clientX:rect.left+(Number(data.x)||0)*scale.value,clientY:rect.top+(Number(data.y)||0)*scale.value});return;
  }
  if(data.type==='canvas-key'){if(data.key===' ')spaceHeld.value=!!data.down;else if(data.down)canvasKey(String(data.key));return;}
  if(data.type==='canvas-gesture'){const rect=viewport.value?.querySelector(`[data-board="${CSS.escape(b.file)}"] .surface`)?.getBoundingClientRect();if(data.start)gestureStart();else if(rect)gestureChange({scale:Number(data.scale),clientX:rect.left+(Number(data.x)||0)*scale.value,clientY:rect.top+(Number(data.y)||0)*scale.value} as any);return;}
  if(data.type==='canvas-pan'){if(viewport.value){viewport.value.scrollLeft-=(Number(data.dx)||0)*scale.value;viewport.value.scrollTop-=(Number(data.dy)||0)*scale.value;}return;}

  if(data.type==='select'&&mode.value==='select'){
    const ids=Array.isArray(data.nodes)?data.nodes.map((n:any)=>String(n.node)):data.node?[String(data.node)]:[];
    if(!ids.length){if(selected.value===b.file)clearElements();return;}
    context.value=null;selected.value=b.file;selectionFile.value=b.file;selectedIds.value=ids;
    node.value={id:String(data.node),tag:String(data.tag),text:String(data.text??''),readonly:Boolean(data.readonly)||(Array.isArray(data.nodes)&&data.nodes.some((n:any)=>n.readonly)),group:Boolean(data.group),path:data.path?String(data.path):undefined,quote:data.quote?String(data.quote):undefined,styles:data.styles&&typeof data.styles==='object'?data.styles:{}};
    nodeText.value=node.value.text;
    if(commentTool.value){properties.value=true;commentOpen.value=true;commentTool.value=false;}
  }
  if(data.type==='navigate'&&mode.value==='play'){
    const file=targetFile(b.file,String(data.file),allBoards.value.map(x=>x.file));
    if(file){const dest=allBoards.value.find(b=>b.file===file)!;page.value=dest.page??'';selected.value=file;focus(dest);}
    else props.notify('该链接不属于当前设计稿','info');
  }
}
function focus(b:Board){const el=viewport.value;if(!el)return;el.scrollTo({left:(b.x-minX.value)*scale.value,top:(b.y-minY.value)*scale.value,behavior:'smooth'});}
function zoom(next:number,x?:number,y?:number){const el=viewport.value;if(!el)return;const r=el.getBoundingClientRect();const z=zoomAt(scale.value,next,el.scrollLeft,el.scrollTop,x===undefined?el.clientWidth/2:x-r.left,y===undefined?el.clientHeight/2:y-r.top);scale.value=z.scale;void nextTick(()=>{el.scrollLeft=z.left;el.scrollTop=z.top;});}
function wheel(e:Pick<WheelEvent,'deltaX'|'deltaY'|'deltaMode'|'ctrlKey'|'metaKey'|'shiftKey'|'clientX'|'clientY'>){
 const el=viewport.value;if(!el)return;const dx=wheelPixels(e.deltaX,e.deltaMode,el.clientWidth),dy=wheelPixels(e.deltaY,e.deltaMode,el.clientHeight);
 if(e.ctrlKey||e.metaKey)zoom(scale.value*Math.exp(-dy*.002),e.clientX,e.clientY);
 else if(e.shiftKey)el.scrollLeft+=dy||dx;else{el.scrollLeft+=dx;el.scrollTop+=dy;}
}
let gestureScale=1;
function gestureStart(){gestureScale=scale.value;}
function gestureChange(event:Event){const e=event as Event&{scale:number;clientX:number;clientY:number};if(Number.isFinite(e.scale))zoom(gestureScale*e.scale,e.clientX,e.clientY);}
function fit(){const el=viewport.value;if(!el)return;scale.value=clampScale(Math.min((el.clientWidth-32)/extent.value.w,(el.clientHeight-32)/extent.value.h));void nextTick(()=>el.scrollTo({left:0,top:0}));}
function fitCurrent(){const el=viewport.value,b=current.value;if(!el||!b)return;scale.value=clampScale(Math.min((el.clientWidth-48)/b.w,(el.clientHeight-64)/(b.h+28)));void nextTick(()=>el.scrollTo({left:Math.max(0,(b.x-minX.value+32)*scale.value-24),top:Math.max(0,(b.y-minY.value+40)*scale.value-24)}));}
function canvasKey(key:string){if(key==='+'||key==='=')zoom(scale.value*1.2);if(key==='-')zoom(scale.value/1.2);if(key==='0')fit();if(key==='1')zoom(1);}
function keyup(e:KeyboardEvent){if(e.key===' ')releaseSpace();}

function patchText(){if(!node.value)return;void elementEdit({type:'text',id:node.value.id,text:nodeText.value});}
function patchStyle(key:string,value:string){if(node.value)void elementEdit({type:'style',patches:(selectedIds.value.length?selectedIds.value:[node.value.id]).map(id=>({id,style:{[key]:value}}))});}
function styleVal(key:string,fallback=''):string{return node.value?.styles?.[key]??fallback;}
function numStyleVal(key:string,fallback=0):number{
  const v=node.value?.styles?.[key];
  if(!v)return fallback;
  const n=parseFloat(v);
  return Number.isNaN(n)?fallback:n;
}

function propChange(key:string,c:any,event:Event){const input=event.target as HTMLInputElement;const value=c.editor==='boolean'?input.checked:['number','range'].includes(c.editor)?Number(input.value):input.value;void edit(p=>{p.files[current.value!.file]=patchProp(p.files[current.value!.file],key,value);});}

async function postComment(){
  if(!current.value||!commentText.value.trim()||!project.value)return;
  const c:Comment={
    id:crypto.randomUUID(),
    file:current.value.file,
    node:node.value?.id,
    tag:node.value?.tag,
    path:node.value?.path,
    quote:node.value?.quote,
    text:commentText.value.trim(),
    revision:project.value.revision,
    status:sendComment.value?'sent':'saved'
  };
  const ok=await edit(p=>{
    if(c.node)try{p.files[c.file]=patchNode(p.files[c.file],c.node,{});}catch{}
    p.canvas.xiaoban??={};p.canvas.xiaoban.comments??=[];p.canvas.xiaoban.comments.push(c);
  });
  if(!ok)return;
  commentOpen.value=false;commentText.value='';
  if(sendComment.value){
    const msg=formatCommentPrompt(project.value,[c]);
    emit('request',msg,project.value.sessionIds?.at(-1)??project.value.sessionId);
  }
}

async function sendAllComments(){
  if(!project.value||!pendingComments.value.length)return;
  const toSend=pendingComments.value;
  const msg=formatCommentPrompt(project.value,toSend);
  await edit(p=>{
    for(const c of p.canvas.xiaoban?.comments??[]){
      if(toSend.some(t=>t.id===c.id))c.status='sent';
    }
  });
  emit('request',msg,project.value.sessionIds?.at(-1)??project.value.sessionId);
  props.notify(`已将 ${toSend.length} 条批注发送给小伴`,'success');
}

function resolve(c:Comment){void edit(p=>{const comment=p.canvas.xiaoban?.comments?.find((v:Comment)=>v.id===c.id);if(comment)comment.status='resolved';});}

function saveToken(key:string,val:string){void edit(p=>{setToken(p,key,val);});}
function addToken(){
  if(!newTokenKey.value.trim())return;
  saveToken(newTokenKey.value.trim(),newTokenValue.value.trim());
  newTokenKey.value='';newTokenValue.value='';
}
function insertCardImport(card:DsCard){
  if(!current.value)return;
  void edit(p=>{
    p.files[current.value!.file]=insertComponentImport(p.files[current.value!.file],card.path);
  });
  props.notify(`已插入组件 <dc-import src="${card.path}" />`,'success');
}

async function copyMarkdown(){
  if(!project.value)return;
  const md=toMarkdownSpec(project.value);
  try{await navigator.clipboard.writeText(md);props.notify('已复制设计规范 Markdown','success');}
  catch{props.notify('复制失败，请重试','error');}
}

function openSource(){if(!current.value||!project.value)return;source.value=project.value.files[current.value.file]??'';sourceRevision=project.value.revision;sourceOpen.value=true;}
async function saveSource(){if(!current.value)return;if(project.value?.revision!==sourceRevision){props.notify('源文件已有新版本，请先复制当前草稿，再重新打开源代码','error');return;}if(await edit(p=>{p.files[current.value!.file]=source.value;}))sourceOpen.value=false;}
async function showVersions(){if(!project.value)return;try{versions.value=await invoke('design_history',{id:project.value.id});versionsOpen.value=true;}catch(e){report(e);}}
async function restore(v:Summary){if(!project.value||!canEdit.value)return;try{const result=await invoke<{project:Project;origin:string}>('design_restore',{id:project.value.id,revision:v.revision,expectedRevision:project.value.revision});ds.project=result.project;ds.origin=result.origin;versionsOpen.value=false;history.value=[];future.value=[];}catch(e){report(e);}}

let drag:{type:'pan'|'board'|'resize-board';x:number;y:number;left:number;top:number;board?:Board;dx:number;dy:number}|null=null;
const dragging=ref<{file:string;x:number;y:number;w?:number;h?:number}|null>(null);
function down(event:PointerEvent,b?:Board,resize=false){const panning=event.button===1||spaceHeld.value||mode.value==='hand';if(!panning&&(event.button!==0||(!b&&mode.value!=='hand')||(b&&!canEdit.value)))return;if(panning)b=undefined;if(b){select(b);if(mode.value!=='select')return;}const el=viewport.value!;drag={type:b?(resize?'resize-board':'board'):'pan',x:event.clientX,y:event.clientY,left:el.scrollLeft,top:el.scrollTop,board:b?clone(b):undefined,dx:0,dy:0};(event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);event.preventDefault();}
function move(e:PointerEvent){if(!drag)return;drag.dx=(e.clientX-drag.x)/scale.value;drag.dy=(e.clientY-drag.y)/scale.value;if(drag.type==='pan'){viewport.value!.scrollLeft=drag.left+drag.x-e.clientX;viewport.value!.scrollTop=drag.top+drag.y-e.clientY;}else if(drag.type==='resize-board'){let w=Math.max(1,Math.min(16000,Math.round(drag.board!.w+drag.dx))),h=Math.max(1,Math.min(16000,Math.round(drag.board!.h+drag.dy)));if(e.shiftKey)h=Math.max(1,Math.min(16000,Math.round(w*drag.board!.h/drag.board!.w)));dragging.value={file:drag.board!.file,x:drag.board!.x,y:drag.board!.y,w,h};}else dragging.value={file:drag.board!.file,x:drag.board!.x+drag.dx,y:drag.board!.y+drag.dy};}
function cancelDrag(){drag=null;dragging.value=null;}
function up(){if(!drag)return;const d=drag,size=dragging.value;drag=null;dragging.value=null;if(d.type==='resize-board'&&size?.w&&size?.h)void edit(p=>changeBoard(p,d.board!.file,{w:size.w,h:size.h}));else if(d.type==='board'&&(Math.abs(d.dx)>1||Math.abs(d.dy)>1))void edit(p=>changeBoard(p,d.board!.file,{x:Math.round(d.board!.x+d.dx),y:Math.round(d.board!.y+d.dy)}));}
function position(b:Board){const moving=dragging.value?.file===b.file?dragging.value:null;return {left:((moving?.x??b.x)-minX.value+32)+'px',top:((moving?.y??b.y)-minY.value+40)+'px',width:(moving?.w??b.w)+'px',height:((moving?.h??b.h)+28)+'px'};}
function key(e:KeyboardEvent){if(e.isComposing||e.keyCode===229)return;if((e.metaKey||e.ctrlKey)&&e.key==='\\'){e.preventDefault();void elementCommand('properties');return;}if(e.key==='F6'){e.preventDefault();focusArea(e.shiftKey);return;}if(props.detached&&e.key==='F11'){e.preventDefault();void invoke('design_window_fullscreen',{id:props.projectId}).catch(report);return;}if((e.target as HTMLElement).matches('input,textarea,select,[contenteditable]'))return;if(e.key==='Escape'){clearElements();sourceOpen.value=false;commentOpen.value=false;versionsOpen.value=false;releaseSpace();}if(e.key===' '){e.preventDefault();spaceHeld.value=true;}if(['+','=','-','0','1'].includes(e.key)){e.preventDefault();canvasKey(e.key);}if((e.metaKey||e.ctrlKey)&&['z','y','d','c','x','v','g','a','\\'].includes(e.key.toLowerCase())){e.preventDefault();const k=e.key.toLowerCase();void elementCommand(k==='z'?(e.shiftKey?'redo':'undo'):k==='y'?'redo':k==='d'?'duplicate':k==='c'?'copy':k==='x'?'cut':k==='v'?'paste':k==='g'?(e.shiftKey?'ungroup':'group'):k==='a'?'selectAll':'properties');}else if(e.key==='Delete'||e.key==='Backspace'){e.preventDefault();void elementCommand('delete');}else if(e.key==='Enter'&&selectedIds.value.length){e.preventDefault();sendEditor('editText');}else if(!selectedIds.value.length&&e.key.startsWith('Arrow')&&current.value){e.preventDefault();const d=e.shiftKey?10:1;patchBoard({x:current.value.x+(e.key==='ArrowLeft'?-d:e.key==='ArrowRight'?d:0),y:current.value.y+(e.key==='ArrowUp'?-d:e.key==='ArrowDown'?d:0)});}}
</script>
<template>
<section class="design-workbench" tabindex="0" aria-label="设计工作台" data-testid="design-workbench" @keydown="key">
<header>
  <span class="brand">✦</span><b>{{project?.title??'读取设计稿…'}}</b>
  <button :disabled="!canEdit" title="重命名设计稿" @click="renameProject"><Pencil :size="13"/></button>
  <button :disabled="!canEdit" title="创建副本" @click="copyProject"><Copy :size="13"/></button>
  <span v-if="project" class="saved">{{statusLabel(project.status)}}</span>
  <span class="saved" :class="{error:ds.error}">{{ds.busy?'保存中…':ds.error?'未保存':ds.dirty?'有未保存修改':'已保存'}}</span>
  <span class="spacer"/>
  <button :title="theme==='light'?'切换暗色预览':'切换亮色预览'" @click="theme=theme==='light'?'dark':'light'">
    <Moon v-if="theme==='light'" :size="14"/><Sun v-else :size="14"/>
  </button>
  <button title="历史版本" @click="showVersions"><RotateCcw :size="14"/></button>
  <button :aria-expanded="properties" title="属性面板" @click="properties=!properties"><PanelRight :size="14"/></button>
  <details class="export">
    <summary><Download :size="14"/>导出</summary>
    <div>
      <button @click="ds.export('zip')">ZIP 源文件</button>
      <button @click="ds.export('html')">独立 HTML</button>
      <button @click="ds.export('md')">Markdown 规范 (.md)</button>
      <button @click="copyMarkdown"><ClipboardCopy :size="12"/> 复制 Markdown</button>
    </div>
  </details>
  <button v-if="detached" title="窗口全屏 (F11)" @click="invoke('design_window_fullscreen',{id:projectId}).catch(report)"><Maximize2 :size="14"/></button>
  <button :title="detached?'回到分屏':'独立窗口'" @click="handoff"><ExternalLink :size="14"/></button>
  <button aria-label="关闭设计画布" @click="close"><X :size="15"/></button>
</header>
<div class="tools">
  <select v-model="page" aria-label="画布页面"><option value="">全部页面</option><option v-for="p in pages" :key="p.id" :value="p.id">{{p.name??p.title??p.id}}</option></select>
  <i/>
  <button :class="{on:mode==='select'}" title="选择" @click="mode='select'"><MousePointer2 :size="14"/></button>
  <button :class="{on:mode==='hand'}" title="抓手" @click="mode='hand'"><Hand :size="14"/></button>
  <button :class="{on:mode==='play'}" title="演示原型" @click="mode='play'"><Play :size="13"/></button>
  <i/>
  <button :disabled="!canEdit" title="新增画板" @click="add()"><Plus :size="14"/></button>
  <button :disabled="!canEdit||!current" title="添加批注" :class="{on:commentTool}" @click="selectedIds.length?commentOpen=true:commentTool=!commentTool"><MessageSquare :size="14"/></button>
  <select :disabled="!canEdit" aria-label="插入元素" value="" @change="elementEdit({type:'insert',kind:($event.target as HTMLSelectElement).value as 'text'});($event.target as HTMLSelectElement).value=''">
    <option value="" disabled>插入</option><option value="text">文字</option><option value="rectangle">矩形</option><option value="ellipse">椭圆</option><option value="note">便签</option>
  </select>
  <button :class="{on:guides}" title="布局参考线 (Shift+G)" @click="guides=!guides">参考线</button>
  <span class="spacer"/>
  <button :disabled="!canEdit||!history.length" title="撤销" @click="undo"><RotateCcw :size="13"/></button>
  <button :disabled="!canEdit||!future.length" title="重做" @click="redo"><RotateCw :size="13"/></button>
  <i/>
  <button title="适应全部画板 (0)" @click="fit">适合</button><button title="适应当前画板" @click="fitCurrent">当前</button>
  <button title="缩小" @click="zoom(scale/1.2)"><Minus :size="12"/></button>
  <span class="zoom" title="双击恢复 100% (1)" @dblclick="zoom(1)">{{Math.round(scale*100)}}%</span>
  <button title="放大" @click="zoom(scale*1.2)"><Plus :size="12"/></button>
</div>
<div v-if="ds.error" class="errorbar" role="alert">
  {{ds.error}}<small v-if="ds.dirty"> · 草稿未写入预览，可先导出保留</small>
  <button :disabled="ds.busy||ds.conflict" @click="ds.save()">重试保存</button>
  <button @click="ds.export('zip')">导出草稿</button>
  <button @click="ds.discard()">放弃草稿并读取新版</button>
</div>
<div class="editor" @pointerdown.self="context=null">
  <ElementToolbar v-if="selectedIds.length&&mode==='select'" :count="selectedIds.length" :disabled="!canEdit" :readonly="node?.readonly" @command="elementCommand"/>
  <div v-if="context" class="xb-menu element-context" :style="{left:context.x+'px',top:context.y+'px'}" role="menu" aria-label="元素菜单">
    <button v-for="[label,command] in [['复制','copy'],['剪切','cut'],['粘贴','paste'],['创建副本','duplicate'],['删除','delete'],['编组','group'],['解组','ungroup'],['移到最前','front'],['移到最后','back'],['锁定','lock'],['解锁','unlock']]" :key="command" class="xb-menu-item" role="menuitem" :disabled="!canEdit" @click="elementCommand(command)">{{label}}</button>
  </div>
  <nav class="rail" aria-label="画板列表">
    <div class="rail-head">画板 <small>{{allBoards.length}}</small></div>
    <button v-for="b in allBoards" :key="b.file" :class="{active:selected===b.file}" @click="page=b.page??'';select(b);focus(b)" @dblclick="properties=true">
      <span>▣</span><span>{{b.title}}</span>
    </button>
    <div class="layer-head">图层 <small>{{layers.length}}</small></div>
    <div class="layers" role="tree" aria-label="图层列表" @keydown="layerKey"><div v-for="layer in visibleLayers" :key="layer.id" :data-layer-id="layer.id" role="treeitem" :aria-selected="selectedIds.includes(layer.id)" :aria-level="layer.depth+1" :aria-expanded="layerContainers.has(layer.id)?!collapsedLayers.has(layer.id):undefined" class="layer-row" :class="{active:selectedIds.includes(layer.id)}" :style="{paddingLeft:Math.min(32,layer.depth*8)+'px'}">
      <button v-if="layerContainers.has(layer.id)" :aria-label="(collapsedLayers.has(layer.id)?'展开':'收起')+layer.label" @click="toggleLayer(layer.id)">{{collapsedLayers.has(layer.id)?'›':'⌄'}}</button>
      <button class="layer-select" :title="layer.label" :aria-label="'选择图层 '+layer.label" @click="chooseElements([layer.id])">{{layer.tag.toLowerCase()}} · {{layer.label}}</button>
      <button :disabled="!canEdit||layer.generated" :title="layer.hidden?'显示图层':'隐藏图层'" :aria-label="(layer.hidden?'显示':'隐藏')+layer.label" @click="elementEdit({type:'hide',ids:[layer.id],value:!layer.hidden})">{{layer.hidden?'○':'◉'}}</button>
      <button :disabled="!canEdit||layer.generated" :title="layer.locked?'解锁图层':'锁定图层'" :aria-label="(layer.locked?'解锁':'锁定')+layer.label" @click="elementEdit({type:'lock',ids:[layer.id],value:!layer.locked})">{{layer.locked?'◆':'◇'}}</button>
    </div></div>
    <div class="rail-actions">
      <button :disabled="!canEdit||!current" title="复制画板" @click="add(true)"><Copy :size="13"/></button>
      <button :disabled="!canEdit||allBoards.length<2" title="删除画板" @click="remove"><Trash2 :size="13"/></button>
      <button :disabled="!current" title="画板源代码" @click="openSource">&lt;/&gt;</button>
    </div>
  </nav>
  <div ref="viewport" class="viewport" tabindex="0" :class="{hand:mode==='hand'||spaceHeld}" @wheel.prevent="wheel" @scroll.passive="scheduleView" @gesturestart.prevent="gestureStart" @gesturechange.prevent="gestureChange" @pointerdown="down($event)" @pointermove="move" @pointerup="up" @pointercancel="cancelDrag">
    <div class="scroll-space" :style="{width:extent.w*scale+'px',height:extent.h*scale+'px'}" @pointerdown.self.stop="down($event)">
      <div class="canvas" :style="{width:extent.w+'px',height:extent.h+'px',transform:`scale(${scale})`}" @pointerdown.self.stop="down($event)">
        <section v-for="b in boards" :key="b.file" class="artboard" :data-board="b.file" :class="{selected:selected===b.file}" :style="position(b)">
          <div class="board-title" tabindex="0" role="button" :aria-label="'画板 '+b.title" @keydown.enter.stop.prevent="select(b);sendEditor('enter')" @keydown.space.stop.prevent="select(b)" @pointerdown.stop="down($event,b)">
            <b>{{b.title}} <small>{{b.w}}×{{b.h}}</small></b>
            <button @pointerdown.stop @click="select(b);mode='play';focus(b)"><Play :size="13"/> Play</button>
          </div>
          <div class="surface" :style="{height:(dragging?.file===b.file&&dragging.h?dragging.h:b.h)+'px'}">
            <div v-if="!project?.files[b.file]?.trim()" class="pending-board">画板尚未生成，可让小伴继续完成</div>
            <BoardPreview v-else :src="project?previewUrl(ds.origin,project,b.file,token):''" :title="b.title" :editing="mode!=='play'" :commenting="commentTool" :writable="canEdit" :zoom="scale" :guides="guides" :selection="selectionFile===b.file?selectedIds:[]" :editor-command="selected===b.file?editorCommand:undefined" :key="b.file+':'+project?.revision+':'+previewEpoch" :theme="theme" :pins="boardPins(b.file)" :token="token" @message="onPreview(b,$event)" @error="renderErrors[b.file]=$event"/>
            <div v-if="mode==='hand'||spaceHeld" class="pan-shield" @pointerdown.stop="down($event)"/>
            <div v-if="renderErrors[b.file]" class="render-error" role="alert">预览错误：{{renderErrors[b.file]}}<button @click="select(b);openSource()">查看源代码</button></div>
          </div>
          <button v-if="selected===b.file&&!selectedIds.length&&mode==='select'" class="board-size-handle" title="拖动调整画板大小（Shift 保持比例）" :style="{width:12/scale+'px',height:12/scale+'px'}" :disabled="!canEdit" @pointerdown.stop="down($event,b,true)" @pointermove.stop="move" @pointerup.stop="up" @pointercancel="cancelDrag"/>
        </section>
      </div>
    </div>
    <p v-if="project&&!boards.length" class="empty">暂无画板，点击 ＋ 创建，或继续告诉小伴设计需求。</p>
  </div>
  <aside v-if="properties&&current&&project" class="properties">
    <div class="prop-tabs">
      <button :class="{active:activeTab==='properties'}" @click="activeTab='properties'">属性 & 批注</button>
      <button :class="{active:activeTab==='design-system'}" @click="activeTab='design-system'">设计系统</button>
    </div>
    <template v-if="activeTab==='properties'">
      <h3>{{node?node.tag.toLowerCase():'画板属性'}}</h3>
      <label>名称<input :value="current.title" :disabled="!canEdit" @change="patchBoard({title:($event.target as HTMLInputElement).value})"/></label>
      <div class="dimensions">
        <label>宽<input type="number" min="1" max="16000" :value="current.w" :disabled="!canEdit" @change="patchBoard({w:+($event.target as HTMLInputElement).value})"/></label>
        <label>高<input type="number" min="1" max="16000" :value="current.h" :disabled="!canEdit" @change="patchBoard({h:+($event.target as HTMLInputElement).value})"/></label>
      </div>
      <div class="presets">
        <button v-for="p in DEVICE_PRESETS" :key="p.id" :disabled="!canEdit" @click="applyPreset(p)">{{p.label.split(' ')[0]}}</button>
      </div>
      <template v-if="node">
        <h4>选中元素</h4>
        <div v-if="node.path" class="node-path" :title="node.path">{{node.path}}</div>
        <label>文字<input v-model="nodeText" :disabled="!canEdit" @change="patchText"/></label>
        <label>文字颜色<input type="color" :value="styleVal('color','#000000')" :disabled="!canEdit" @change="patchStyle('color',($event.target as HTMLInputElement).value)"/></label>
        <label>背景颜色<input type="color" :value="styleVal('background-color','#ffffff')" :disabled="!canEdit" @change="patchStyle('background-color',($event.target as HTMLInputElement).value)"/></label>
        <label>字号<input type="number" min="1" max="300" :value="numStyleVal('font-size',14)" :disabled="!canEdit" @change="patchStyle('font-size',($event.target as HTMLInputElement).value+'px')"/></label>
        <label>圆角<input type="number" min="0" max="1000" :value="numStyleVal('border-radius',0)" :disabled="!canEdit" @change="patchStyle('border-radius',($event.target as HTMLInputElement).value+'px')"/></label>
        <label>间距<input type="number" min="0" max="1000" :value="numStyleVal('padding',0)" :disabled="!canEdit" @change="patchStyle('padding',($event.target as HTMLInputElement).value+'px')"/></label>
        <label>对齐
          <select :value="styleVal('text-align','left')" :disabled="!canEdit" @change="patchStyle('text-align',($event.target as HTMLSelectElement).value)">
            <option value="left">左对齐</option>
            <option value="center">居中</option>
            <option value="right">右对齐</option>
          </select>
        </label>
      </template>
      <template v-if="config.length">
        <h4>可调参数</h4>
        <label v-for="[key,c] in config" :key="key">{{c.label??key}}
          <select v-if="c.editor==='enum'" :disabled="!canEdit" :value="c.default" @change="propChange(key,c,$event)"><option v-for="v in c.options" :key="String(v)" :value="v">{{v}}</option></select>
          <input v-else-if="c.editor==='boolean'" type="checkbox" :disabled="!canEdit" :checked="c.default" @change="propChange(key,c,$event)"/>
          <input v-else-if="['color','number','range','string','text'].includes(c.editor)" :type="c.editor==='string'||c.editor==='text'?'text':c.editor" :min="c.min" :max="c.max" :step="c.step" :disabled="!canEdit" :value="c.default" @change="propChange(key,c,$event)"/>
          <small v-else>此参数只读保留</small>
        </label>
      </template>
      <div class="comment-head">
        <h4>批注 <small>{{comments.length}}</small></h4>
        <button class="mini-btn" @click="hideResolved=!hideResolved">{{hideResolved?'显示全部':'隐藏已解决'}}</button>
      </div>
      <button v-if="pendingComments.length>1" :disabled="!canEdit" class="text-btn primary-soft" @click="sendAllComments">
        <Send :size="12"/> 全部发给小伴 ({{pendingComments.length}})
      </button>
      <article v-for="c in comments" :key="c.id" class="comment">
        <b>{{allBoards.find(b=>b.file===c.file)?.title??'原画板已删除'}}</b>
        <small v-if="c.node" class="comment-node">{{c.tag?c.tag.toLowerCase():'节点'}} #{{c.node}}</small>
        <p>{{c.text}}</p>
        <div class="comment-foot">
          <span>{{c.status==='resolved'?'已解决':c.status==='sent'?'修改请求已发送':'已保存在画布'}}</span>
          <button v-if="c.status!=='resolved'" :disabled="!canEdit" @click="resolve(c)"><Check :size="12"/>解决</button>
        </div>
      </article>
      <button :disabled="!canEdit" class="text-btn" @click="commentOpen=true">＋ 添加批注</button>
    </template>
    <template v-else>
      <h3>设计系统 (Design System)</h3>
      <h4>设计令牌 (Tokens)</h4>
      <div v-for="(v,k) in tokens" :key="k" class="token-row">
        <span><code>--ds-{{k}}</code></span>
        <input :value="v" :disabled="!canEdit" @change="saveToken(String(k),($event.target as HTMLInputElement).value)"/>
      </div>
      <form class="add-token" @submit.prevent="addToken">
        <input v-model="newTokenKey" placeholder="新变量名 (如 primary)" :disabled="!canEdit"/>
        <input v-model="newTokenValue" placeholder="值 (如 #d94e60)" :disabled="!canEdit"/>
        <button :disabled="!canEdit||!newTokenKey.trim()" class="text-btn">＋ 新增令牌</button>
      </form>
      <h4>组件卡片 (@dsCard) <small>{{dsCards.length}}</small></h4>
      <p v-if="!dsCards.length" class="empty-note">在任意 .dc.html 顶部添加 <code>&lt;!-- @dsCard group="..." name="..." --&gt;</code> 声明组件卡片。</p>
      <article v-for="card in dsCards" :key="card.path" class="card-item">
        <div class="card-meta"><b>{{card.name}}</b><small>{{card.group}}</small></div>
        <div class="card-file">{{card.path}}</div>
        <p v-if="card.subtitle">{{card.subtitle}}</p>
        <button :disabled="!canEdit" class="text-btn mini" @click="insertCardImport(card)">插入到当前画板</button>
      </article>
    </template>
  </aside>
</div>
<footer>
  <span>open-claude-design</span><span>版本 {{project?.revision??'—'}}</span>
  <span class="spacer"/>
  <span>我的文档 · {{allBoards.length}} 个画板 · {{dsCards.length}} 个组件卡片</span>
</footer>
<div v-if="renameOpen" class="overlay"><form class="dialog" @submit.prevent="saveTitle"><h3>重命名设计稿</h3><input v-model="projectTitle" required maxlength="120" aria-label="设计稿名称"/><div class="actions"><button type="button" @click="renameOpen=false">取消</button><button class="primary" :disabled="!canEdit||!projectTitle.trim()">保存</button></div></form></div>
<div v-if="commentOpen" class="overlay">
  <form class="dialog" @submit.prevent="postComment">
    <h3>添加批注</h3>
    <p>{{current?.title}}<template v-if="node"> / {{node.tag.toLowerCase()}} {{node.id?`(#${node.id})`:''}}</template></p>
    <p v-if="node?.quote" class="quote">“{{node.quote}}”</p>
    <textarea v-model="commentText" required maxlength="4000" placeholder="描述想修改的内容…"/>
    <label class="check"><input v-model="sendComment" type="checkbox"/>交给小伴修改</label>
    <div class="actions">
      <button type="button" @click="commentOpen=false">取消</button>
      <button class="primary" :disabled="!canEdit||!commentText.trim()">保存批注</button>
    </div>
  </form>
</div>
<div v-if="sourceOpen" class="overlay"><div class="dialog source-dialog"><h3>{{current?.file}}</h3><textarea v-model="source" spellcheck="false" aria-label="画板源代码"/><div class="actions"><button @click="sourceOpen=false">关闭</button><button class="primary" :disabled="!canEdit" @click="saveSource">保存源文件</button></div></div></div>
<div v-if="versionsOpen" class="overlay"><div class="dialog"><h3>历史版本</h3><p v-if="!versions.length">还没有历史版本。</p><div v-for="v in versions" :key="v.revision" class="version-row"><span>版本 {{v.revision}} · {{v.artboards}} 个画板</span><button :disabled="!canEdit" @click="restore(v)">恢复为新版本</button></div><div class="actions"><button @click="versionsOpen=false">关闭</button></div></div></div>
</section>
</template>
<style scoped>
.board-size-handle{position:absolute;right:0;bottom:0;transform:translate(50%,50%);background:var(--ui-btn)!important;border:1px solid var(--ui-surface)!important;min-height:0!important;min-width:0!important;padding:0!important;border-radius:2px!important;cursor:nwse-resize!important;touch-action:none;z-index:3}
.element-context{position:absolute;z-index:30;max-height:calc(100% - 16px);overflow:auto;box-sizing:border-box;width:104px;min-width:0;max-width:calc(100% - 16px)}.design-workbench .element-context .xb-menu-item{display:flex;justify-content:flex-start;min-height:32px;padding:0 10px;font-size:12px;white-space:nowrap}.layer-head{padding:12px 6px 5px;color:var(--ui-text-tertiary);font-size:10px}.layers{border-top:1px solid var(--ui-line);margin-top:6px;padding-top:4px}.layer-row{display:flex;align-items:center;gap:0;min-width:0;font-size:9px}.layer-row.active{background:var(--ui-soft);color:var(--ui-ink)}.layer-row .layer-select{flex:1;min-width:0;display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;text-align:left;padding:3px}.layer-row>button:not(.layer-select){padding:2px;min-width:15px;opacity:.45}.layer-row:hover>button,.layer-row.active>button{opacity:1}

.design-workbench{position:relative;container-type:inline-size;flex:1;display:flex;flex-direction:column;min-width:0;height:100%;background:var(--ui-surface,#fff);color:var(--ui-text-primary,#30333b);font-size:11px;outline:none}.design-workbench button{display:inline-flex;gap:4px;align-items:center;justify-content:center;border:0;background:transparent;color:inherit;font:inherit;border-radius:5px;cursor:pointer;min-height:25px;padding:3px 6px}.design-workbench button:hover:not(:disabled){background:var(--ui-control,#f0f0f4)}.design-workbench button:disabled{opacity:.35;cursor:default}.spacer{flex:1}header{display:flex;gap:7px;align-items:center;height:39px;flex:none;padding:0 10px;border-bottom:1px solid var(--ui-line,#e8e9ed)}header b{font-size:11px;max-width:240px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.brand{color:var(--ui-btn,#d94e60)}.saved{color:var(--ui-success-text);font-size:9px}.saved.error{color:var(--ui-danger)}.tools{height:36px;display:flex;align-items:center;gap:2px;flex:none;padding:0 7px;border-bottom:1px solid var(--ui-line,#e8e9ed)}.tools select{max-width:110px;font:inherit;color:inherit;background:transparent;border:0}.tools i{width:1px;height:14px;background:var(--ui-line,#e8e9ed);margin:0 4px}.tools button.on{background:var(--ui-soft,#fceff1);color:var(--ui-ink,#c14558)}.zoom{font-size:9px;min-width:30px;text-align:center}.export{position:relative}.export summary{display:flex;align-items:center;gap:4px;cursor:pointer;padding:4px;font-size:10px}.export>div{position:absolute;right:0;top:100%;z-index:5;background:var(--ui-surface,#fff);border:1px solid var(--ui-line,#eee);border-radius:6px;padding:5px;min-width:130px;box-shadow:0 5px 16px #0001}.export>div button{width:100%;justify-content:start}.editor{display:flex;flex:1;min-height:0;position:relative}.rail{width:132px;flex:none;padding:10px 6px;border-right:1px solid var(--ui-line,#eee);overflow:auto;background:var(--ui-surface,#fff)}.rail-head{padding:0 6px 9px;color:var(--ui-text-tertiary,#999);font-size:10px}.rail-head small{margin-left:5px}.rail>button{width:100%;justify-content:start;text-align:left;font-size:10px;padding:6px;gap:7px}.rail>button span:last-child{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.rail>button.active{background:var(--ui-soft,#fceff1);color:var(--ui-ink,#be4054)}.rail-actions{border-top:1px solid var(--ui-line,#eee);margin-top:10px;padding-top:8px;display:flex;justify-content:space-around}.viewport{flex:1;min-width:0;overflow:auto;background:var(--ui-surface-sunken,#f5f5f7);background-image:radial-gradient(var(--ui-border-strong) .7px,transparent .7px);background-size:14px 14px;position:relative}.viewport.hand{cursor:grab}.viewport.hand:active{cursor:grabbing}.scroll-space{position:relative;min-width:100%;min-height:100%}.canvas{position:absolute;left:0;top:0;transform-origin:0 0}.artboard{position:absolute}.board-title{height:28px;display:flex;align-items:center;justify-content:space-between;color:var(--ui-text-secondary);font-size:13px;touch-action:none;user-select:none}.board-title b{font-weight:500;overflow:hidden;white-space:nowrap;text-overflow:ellipsis}.board-title b small{font-weight:400;font-size:10px;margin-left:4px;color:var(--ui-text-tertiary)}.board-title button{font-size:12px}.surface{background:var(--ui-surface);position:relative;outline:1px solid var(--ui-border-strong)}.artboard.selected .surface{outline:3px solid var(--ui-btn,#d94e60)}.pending-board{display:grid;place-items:center;height:100%;font-size:18px;color:var(--ui-text-tertiary);background:var(--ui-surface-alt)}.pan-shield{position:absolute;inset:0;touch-action:none}.render-error{position:absolute;bottom:0;left:0;right:0;background:var(--ui-soft);color:var(--ui-danger);padding:12px;font-size:14px}.render-error button{border:1px solid var(--ui-danger);margin-left:12px}.properties{width:220px;flex:none;padding:13px 12px;overflow:auto;border-left:1px solid var(--ui-line,#eee);background:var(--ui-surface,#fff)}.properties h3{font-size:11px;margin:0 0 14px}.properties h4{font-size:10px;border-top:1px solid var(--ui-line,#eee);padding-top:12px;margin:12px 0}.properties label{display:flex;justify-content:space-between;align-items:center;gap:9px;margin:9px 0;font-size:10px;color:var(--ui-text-secondary,#777)}.properties input,.properties select{font:inherit;min-width:0;width:115px;background:var(--ui-control,#f8f8fa);color:var(--ui-text-primary,#333);border:1px solid var(--ui-line,#e8e9ed);border-radius:5px;padding:4px}.properties input[type=color]{height:25px;width:35px;padding:2px}.properties input[type=checkbox]{width:15px}.dimensions{display:flex;gap:8px}.dimensions label{display:block;width:50%}.dimensions input{width:100%;margin-top:4px}.presets{display:flex;gap:4px;margin-bottom:12px}.presets button{flex:1;font-size:9px;padding:2px 4px;border:1px solid var(--ui-line,#e8e9ed)}.prop-tabs{display:flex;gap:4px;margin-bottom:12px;border-bottom:1px solid var(--ui-line,#eee);padding-bottom:6px}.prop-tabs button{flex:1;font-size:10px;padding:4px 6px}.prop-tabs button.active{background:var(--ui-soft,#fceff1);color:var(--ui-ink,#c14558);font-weight:600}.node-path{font-size:9px;color:var(--ui-text-tertiary);font-family:monospace;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;margin:-4px 0 8px}.comment-head{display:flex;align-items:center;justify-content:space-between}.mini-btn{font-size:9px;padding:2px 4px;color:var(--ui-text-tertiary,#999)}.comment{padding:8px;border:1px solid var(--ui-line,#eee);border-radius:6px;margin-bottom:8px;font-size:10px}.comment-node{display:block;color:var(--ui-text-tertiary);font-size:9px}.comment p{white-space:pre-wrap;overflow-wrap:anywhere;color:var(--ui-text-secondary,#777);margin:5px 0}.comment-foot{display:flex;align-items:center;justify-content:space-between}.comment span{font-size:9px;color:var(--ui-text-tertiary)}.comment button{font-size:9px}.text-btn{width:100%;border:1px solid var(--ui-line,#eee)!important;margin-top:6px}.text-btn.primary-soft{background:var(--ui-soft,#fceff1);color:var(--ui-ink,#c14558);border-color:var(--ui-border-strong)!important}.text-btn.mini{padding:2px 6px;min-height:20px;font-size:9px}.token-row{display:flex;align-items:center;justify-content:space-between;gap:6px;margin-bottom:6px}.token-row code{font-size:9px;color:var(--ui-ink)}.token-row input{width:90px;font-size:10px;padding:2px 4px}.add-token{display:flex;flex-direction:column;gap:4px;margin-top:10px}.add-token input{width:100%;box-sizing:border-box}.card-item{padding:8px;border:1px solid var(--ui-line,#eee);border-radius:6px;margin-bottom:8px}.card-meta{display:flex;justify-content:space-between;align-items:center}.card-meta small{color:var(--ui-text-tertiary);font-size:9px}.card-file{font-size:9px;color:var(--ui-text-tertiary);font-family:monospace;margin:2px 0 4px}.empty-note{color:var(--ui-text-tertiary);font-size:9px;line-height:1.5}.quote{font-style:italic;color:var(--ui-text-secondary);border-left:2px solid var(--ui-btn,#d94e60);padding-left:6px;margin:4px 0}.empty{padding:30px;color:var(--ui-text-tertiary);line-height:1.8}.errorbar{background:var(--ui-soft);color:var(--ui-danger);font-size:10px;padding:7px 10px}.errorbar button{text-decoration:underline}footer{height:23px;flex:none;display:flex;align-items:center;gap:12px;padding:0 11px;border-top:1px solid var(--ui-line,#eee);font-size:9px;color:var(--ui-text-tertiary,#999)}.overlay{position:absolute;inset:0;z-index:20;background:var(--ui-overlay);display:flex;align-items:center;justify-content:center}.dialog{width:350px;max-width:90%;max-height:85%;overflow:auto;background:var(--ui-surface,#fff);border:1px solid var(--ui-line,#eee);border-radius:10px;padding:18px;box-shadow:0 12px 40px #0002}.dialog h3{font-size:13px;margin:0 0 10px}.dialog p{font-size:11px;color:var(--ui-text-tertiary)}.dialog textarea{width:100%;height:90px;resize:vertical;box-sizing:border-box;font:inherit;font-size:12px;line-height:1.6;color:inherit;background:var(--ui-control,#f8f8fa);border:1px solid var(--ui-line,#e8e9ed);border-radius:6px;padding:8px}.check{display:flex;align-items:center;gap:6px;font-size:11px;margin-top:9px}.actions{display:flex;gap:8px;justify-content:end;margin-top:15px}.actions .primary{background:var(--ui-btn,#d94e60);color:white}.source-dialog{width:750px}.source-dialog textarea{height:50vh;font:11px/1.7 monospace}.version-row{display:flex;justify-content:space-between;align-items:center;margin:8px 0;font-size:11px}@container(max-width:760px){.properties{position:absolute;right:0;top:0;bottom:0;z-index:4;box-shadow:-5px 0 20px #0001}.rail{width:115px}.saved{display:none}}
</style>
