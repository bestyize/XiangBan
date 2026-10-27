<script setup lang="ts">
import {computed,onBeforeUnmount,onMounted,ref,watch} from 'vue';
import {Check,Copy,Download,Hand,Maximize2,MessageSquare,Minus,MousePointer2,PanelRight,Play,Plus,RotateCcw,RotateCw,Send,Sun,Moon,Trash2,X,ClipboardCopy,Palette,Layers} from 'lucide-vue-next';
import {invoke} from '../../bridge.ts';
import BoardPreview from './BoardPreview.vue';
import {
  statusLabel,addBoard,boardsOf,changeBoard,clone,patchNode,patchProp,previewUrl,propsOf,removeBoard,targetFile,
  tokensOf,setToken,dsCardsOf,insertComponentImport,formatCommentPrompt,toMarkdownSpec,DEVICE_PRESETS,
  type Board,type Comment,type Project,type Summary,type DsCard
} from './model.ts';
import {createDesignStore} from './design-store.ts';
const props=defineProps<{projectId:string;notify:(message:unknown,type?:'success'|'error'|'info')=>void}>();
const emit=defineEmits<{close:[];request:[message:string,session:string]}>();
const ds=createDesignStore(props.notify),selected=ref(''),page=ref(''),mode=ref<'select'|'play'|'hand'>('select'),scale=ref(.4),properties=ref(false),full=ref(false),viewport=ref<HTMLElement|null>(null);
const theme=ref<'light'|'dark'>('light'),hideResolved=ref(false);
const activeTab=ref<'properties'|'design-system'>('properties');
const node=ref<{id:string;text:string;tag:string;path?:string;quote?:string;styles?:Record<string,string>}|null>(null);
const nodeText=ref(''),commentText=ref(''),sendComment=ref(true),commentOpen=ref(false),sourceOpen=ref(false),source=ref(''),renderErrors=ref<Record<string,string>>({});
const history=ref<Project[]>([]),future=ref<Project[]>([]),versions=ref<Summary[]>([]),versionsOpen=ref(false);
const newTokenKey=ref(''),newTokenValue=ref('');
let sourceRevision=0,remoteEpoch=0;
const token=ref(crypto.randomUUID());const project=computed(()=>ds.project);
const allBoards=computed(()=>project.value?boardsOf(project.value.canvas):[]),pages=computed(()=>project.value?.canvas.pages??[]);
const boards=computed(()=>allBoards.value.filter(b=>!page.value||b.page===page.value));
const current=computed(()=>allBoards.value.find(b=>b.file===selected.value)??allBoards.value[0]);
const config=computed(()=>project.value&&current.value?Object.entries(propsOf(project.value.files[current.value.file]??'')).filter(([key,c])=>key!=='$preview'&&c&&typeof c==='object'):[]);
const allComments=computed<Comment[]>(()=>project.value?.canvas.xiaoban?.comments??[]);
const comments=computed<Comment[]>(()=>hideResolved.value?allComments.value.filter(c=>c.status!=='resolved'):allComments.value);
const pendingComments=computed(()=>allComments.value.filter(c=>c.status!=='resolved'));
const tokens=computed(()=>project.value?tokensOf(project.value.canvas):{});
const dsCards=computed<DsCard[]>(()=>project.value?dsCardsOf(project.value):[]);
const minX=computed(()=>Math.min(0,...boards.value.map(b=>b.x))),minY=computed(()=>Math.min(0,...boards.value.map(b=>b.y)));
const extent=computed(()=>({w:Math.max(600,...boards.value.map(b=>b.x+b.w))-minX.value+96,h:Math.max(500,...boards.value.map(b=>b.y+b.h))-minY.value+140}));
const canEdit=computed(()=>!!project.value&&!ds.busy&&!ds.conflict&&!ds.dirty);

function boardPins(file:string){
  return allComments.value
    .filter(c=>c.file===file&&c.status!=='resolved')
    .map((c,idx)=>({num:idx+1,node:c.node,text:c.text}));
}

function close(){if(ds.busy){props.notify('正在保存，请稍候关闭','info');return;}emit('close');}
function report(e:unknown){props.notify(e,'error');}
async function open(id:string){
  remoteEpoch++;
  try{
    await ds.open(id);
    selected.value=allBoards.value[0]?.file??'';
    page.value='';node.value=null;sourceOpen.value=false;commentOpen.value=false;versionsOpen.value=false;
    history.value=[];future.value=[];renderErrors.value={};token.value=crypto.randomUUID();
  }catch(e){ds.error=String(e);report(e);}
}
watch(()=>props.projectId,id=>void open(id),{immediate:true});
onMounted(()=>void ds.init(summary=>{if(summary.id===project.value?.id&&summary.revision>project.value.revision&&!ds.busy){remoteEpoch++;history.value=[];future.value=[];}}).catch(report));
onBeforeUnmount(()=>ds.dispose());
watch(()=>project.value?.revision,()=>{if(!allBoards.value.some(b=>b.file===selected.value))selected.value=allBoards.value[0]?.file??'';renderErrors.value={};node.value=null;});

async function edit(fn:(p:Project)=>void,record=true){
  if(!project.value||!canEdit.value)return false;
  const before=clone(project.value),epoch=remoteEpoch;
  const ok=await ds.commit(fn);
  if(ok&&record&&epoch===remoteEpoch){history.value.push(before);if(history.value.length>30)history.value.shift();future.value=[];}
  return ok;
}
async function undo(){if(!project.value||!canEdit.value||!history.value.length)return;const before=clone(project.value),old=clone(history.value.at(-1)!),epoch=remoteEpoch;old.revision=before.revision;if(await ds.commit(p=>Object.assign(p,old))&&epoch===remoteEpoch){history.value.pop();future.value.push(before);}}
async function redo(){if(!project.value||!canEdit.value||!future.value.length)return;const before=clone(project.value),next=clone(future.value.at(-1)!),epoch=remoteEpoch;next.revision=before.revision;if(await ds.commit(p=>Object.assign(p,next))&&epoch===remoteEpoch){future.value.pop();history.value.push(before);}}

function select(b:Board){selected.value=b.file;node.value=null;}
function add(copy=false){void edit(p=>{selected.value=addBoard(p,copy?current.value:undefined);if(page.value)changeBoard(p,selected.value,{page:page.value});});}
async function remove(){if(!current.value)return;const file=current.value.file;try{await edit(p=>removeBoard(p,file));}catch(e){report(e);}}
function patchBoard(patch:Partial<Board>){if(current.value)void edit(p=>changeBoard(p,current.value!.file,patch));}
function applyPreset(preset:{w:number;h:number}){patchBoard({w:preset.w,h:preset.h});}

function onPreview(b:Board,data:Record<string,any>){
  if(data.type==='select'&&mode.value==='select'){
    select(b);
    node.value={
      id:String(data.node),
      tag:String(data.tag),
      text:String(data.text??''),
      path:data.path?String(data.path):undefined,
      quote:data.quote?String(data.quote):undefined,
      styles:data.styles&&typeof data.styles==='object'?data.styles:{},
    };
    nodeText.value=node.value.text;
    properties.value=true;
  }
  if(data.type==='navigate'&&mode.value==='play'){
    const file=targetFile(b.file,String(data.file),allBoards.value.map(x=>x.file));
    if(file){const dest=allBoards.value.find(b=>b.file===file)!;page.value=dest.page??'';selected.value=file;focus(dest);}
    else props.notify('该链接不属于当前设计稿','info');
  }
}
function focus(b:Board){const el=viewport.value;if(!el)return;el.scrollTo({left:(b.x-minX.value)*scale.value,top:(b.y-minY.value)*scale.value,behavior:'smooth'});}
function fit(){if(!viewport.value)return;scale.value=Math.max(.1,Math.min(1,(viewport.value.clientWidth-32)/extent.value.w));viewport.value.scrollTo({left:0,top:0});}

function patchText(){if(!current.value||!node.value)return;void edit(p=>{p.files[current.value!.file]=patchNode(p.files[current.value!.file],node.value!.id,{text:nodeText.value});});}
function patchStyle(key:string,value:string){
  if(!current.value||!node.value)return;
  void edit(p=>{p.files[current.value!.file]=patchNode(p.files[current.value!.file],node.value!.id,{style:{[key]:value}});});
  if(node.value.styles)node.value.styles[key]=value;
}
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

let drag:{type:'pan'|'board';x:number;y:number;left:number;top:number;board?:Board;dx:number;dy:number}|null=null;
const dragging=ref<{file:string;x:number;y:number}|null>(null);
function down(event:PointerEvent,b?:Board){if(event.button!==0||(!b&&mode.value!=='hand')||(b&&!canEdit.value))return;if(b){select(b);if(mode.value!=='select')return;}const el=viewport.value!;drag={type:b?'board':'pan',x:event.clientX,y:event.clientY,left:el.scrollLeft,top:el.scrollTop,board:b?clone(b):undefined,dx:0,dy:0};(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);event.preventDefault();}
function move(e:PointerEvent){if(!drag)return;drag.dx=(e.clientX-drag.x)/scale.value;drag.dy=(e.clientY-drag.y)/scale.value;if(drag.type==='pan'){viewport.value!.scrollLeft=drag.left+drag.x-e.clientX;viewport.value!.scrollTop=drag.top+drag.y-e.clientY;}else dragging.value={file:drag.board!.file,x:drag.board!.x+drag.dx,y:drag.board!.y+drag.dy};}
function cancelDrag(){drag=null;dragging.value=null;}
function up(){if(!drag)return;const d=drag;drag=null;dragging.value=null;if(d.type==='board'&&(Math.abs(d.dx)>1||Math.abs(d.dy)>1))void edit(p=>changeBoard(p,d.board!.file,{x:Math.round(d.board!.x+d.dx),y:Math.round(d.board!.y+d.dy)}));}
function position(b:Board){const moving=dragging.value?.file===b.file?dragging.value:null;return {left:((moving?.x??b.x)-minX.value+32)+'px',top:((moving?.y??b.y)-minY.value+40)+'px',width:b.w+'px',height:(b.h+28)+'px'};}
function key(e:KeyboardEvent){if((e.target as HTMLElement).matches('input,textarea,select,[contenteditable]'))return;if(e.key==='Escape'){node.value=null;sourceOpen.value=false;commentOpen.value=false;versionsOpen.value=false;full.value=false;}if((e.metaKey||e.ctrlKey)&&e.key==='z'){e.preventDefault();void(e.shiftKey?redo():undo());}}
</script>
<template>
<section class="design-workbench" :class="{full}" tabindex="0" aria-label="设计工作台" data-testid="design-workbench" @keydown="key">
<header>
  <span class="brand">✦</span><b>{{project?.title??'读取设计稿…'}}</b>
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
  <button title="放大画布" @click="full=!full"><Maximize2 :size="14"/></button>
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
  <button :disabled="!canEdit||!current" title="添加批注" @click="commentOpen=true"><MessageSquare :size="14"/></button>
  <span class="spacer"/>
  <button :disabled="!canEdit||!history.length" title="撤销" @click="undo"><RotateCcw :size="13"/></button>
  <button :disabled="!canEdit||!future.length" title="重做" @click="redo"><RotateCw :size="13"/></button>
  <i/>
  <button @click="fit">适合</button>
  <button title="缩小" @click="scale=Math.max(.1,scale-.1)"><Minus :size="12"/></button>
  <span class="zoom">{{Math.round(scale*100)}}%</span>
  <button title="放大" @click="scale=Math.min(2,scale+.1)"><Plus :size="12"/></button>
</div>
<div v-if="ds.error" class="errorbar" role="alert">
  {{ds.error}}<small v-if="ds.dirty"> · 草稿未写入预览，可先导出保留</small>
  <button :disabled="ds.busy||ds.conflict" @click="ds.save()">重试保存</button>
  <button @click="ds.export('zip')">导出草稿</button>
  <button @click="ds.discard()">放弃草稿并读取新版</button>
</div>
<div class="editor">
  <nav class="rail" aria-label="画板列表">
    <div class="rail-head">画板 <small>{{allBoards.length}}</small></div>
    <button v-for="b in allBoards" :key="b.file" :class="{active:selected===b.file}" @click="page=b.page??'';select(b);focus(b)" @dblclick="properties=true">
      <span>▣</span><span>{{b.title}}</span>
    </button>
    <div class="rail-actions">
      <button :disabled="!canEdit||!current" title="复制画板" @click="add(true)"><Copy :size="13"/></button>
      <button :disabled="!canEdit||allBoards.length<2" title="删除画板" @click="remove"><Trash2 :size="13"/></button>
      <button :disabled="!current" title="画板源代码" @click="openSource">&lt;/&gt;</button>
    </div>
  </nav>
  <div ref="viewport" class="viewport" :class="{hand:mode==='hand'}" @pointerdown.self="down($event)" @pointermove="move" @pointerup="up" @pointercancel="cancelDrag">
    <div class="scroll-space" :style="{width:extent.w*scale+'px',height:extent.h*scale+'px'}" @pointerdown.self="down($event)">
      <div class="canvas" :style="{width:extent.w+'px',height:extent.h+'px',transform:`scale(${scale})`}" @pointerdown.self="down($event)">
        <section v-for="b in boards" :key="b.file" class="artboard" :class="{selected:selected===b.file}" :style="position(b)">
          <div class="board-title" @pointerdown.stop="down($event,b)">
            <b>{{b.title}} <small>{{b.w}}×{{b.h}}</small></b>
            <button @pointerdown.stop @click="select(b);mode='play';focus(b)"><Play :size="13"/> Play</button>
          </div>
          <div class="surface" :style="{height:b.h+'px'}">
            <div v-if="!project?.files[b.file]?.trim()" class="pending-board">画板尚未生成，可让小伴继续完成</div>
            <BoardPreview v-else :src="project?previewUrl(ds.origin,project,b.file,token):''" :title="b.title" :editing="mode!=='play'" :commenting="mode==='select'" :theme="theme" :pins="boardPins(b.file)" :token="token" @message="onPreview(b,$event)" @error="renderErrors[b.file]=$event"/>
            <div v-if="mode==='hand'" class="pan-shield" @pointerdown="down($event)"/>
            <div v-if="renderErrors[b.file]" class="render-error" role="alert">预览错误：{{renderErrors[b.file]}}<button @click="select(b);openSource()">查看源代码</button></div>
          </div>
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
.design-workbench{position:relative;container-type:inline-size;flex:1;display:flex;flex-direction:column;min-width:0;height:100%;background:var(--ui-surface,#fff);color:var(--ui-text-primary,#30333b);font-size:11px;outline:none}.design-workbench.full{position:fixed;inset:0;z-index:1000}.design-workbench button{display:inline-flex;gap:4px;align-items:center;justify-content:center;border:0;background:transparent;color:inherit;font:inherit;border-radius:5px;cursor:pointer;min-height:25px;padding:3px 6px}.design-workbench button:hover:not(:disabled){background:var(--ui-control,#f0f0f4)}.design-workbench button:disabled{opacity:.35;cursor:default}.spacer{flex:1}header{display:flex;gap:7px;align-items:center;height:39px;flex:none;padding:0 10px;border-bottom:1px solid var(--ui-line,#e8e9ed)}header b{font-size:11px;max-width:240px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.brand{color:var(--ui-btn,#d94e60)}.saved{color:#8c9b91;font-size:9px}.saved.error{color:#d05b65}.tools{height:36px;display:flex;align-items:center;gap:2px;flex:none;padding:0 7px;border-bottom:1px solid var(--ui-line,#e8e9ed)}.tools select{max-width:110px;font:inherit;color:inherit;background:transparent;border:0}.tools i{width:1px;height:14px;background:var(--ui-line,#e8e9ed);margin:0 4px}.tools button.on{background:var(--ui-soft,#fceff1);color:var(--ui-ink,#c14558)}.zoom{font-size:9px;min-width:30px;text-align:center}.export{position:relative}.export summary{display:flex;align-items:center;gap:4px;cursor:pointer;padding:4px;font-size:10px}.export>div{position:absolute;right:0;top:100%;z-index:5;background:var(--ui-surface,#fff);border:1px solid var(--ui-line,#eee);border-radius:6px;padding:5px;min-width:130px;box-shadow:0 5px 16px #0001}.export>div button{width:100%;justify-content:start}.editor{display:flex;flex:1;min-height:0;position:relative}.rail{width:132px;flex:none;padding:10px 6px;border-right:1px solid var(--ui-line,#eee);overflow:auto;background:var(--ui-surface,#fff)}.rail-head{padding:0 6px 9px;color:var(--ui-text-tertiary,#999);font-size:10px}.rail-head small{margin-left:5px}.rail>button{width:100%;justify-content:start;text-align:left;font-size:10px;padding:6px;gap:7px}.rail>button span:last-child{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.rail>button.active{background:var(--ui-soft,#fceff1);color:var(--ui-ink,#be4054)}.rail-actions{border-top:1px solid var(--ui-line,#eee);margin-top:10px;padding-top:8px;display:flex;justify-content:space-around}.viewport{flex:1;min-width:0;overflow:auto;background:var(--ui-surface-sunken,#f5f5f7);background-image:radial-gradient(#b5b7c233 .7px,transparent .7px);background-size:14px 14px;position:relative}.viewport.hand{cursor:grab}.viewport.hand:active{cursor:grabbing}.scroll-space{position:relative;min-width:100%;min-height:100%}.canvas{position:absolute;left:0;top:0;transform-origin:0 0}.artboard{position:absolute}.board-title{height:28px;display:flex;align-items:center;justify-content:space-between;color:#9799a5;font-size:13px;touch-action:none;user-select:none}.board-title b{font-weight:500;overflow:hidden;white-space:nowrap;text-overflow:ellipsis}.board-title b small{font-weight:400;font-size:10px;margin-left:4px;color:#aaa}.board-title button{font-size:12px}.surface{background:white;position:relative;outline:1px solid #dedfe6}.artboard.selected .surface{outline:3px solid var(--ui-btn,#d94e60)}.pan-shield{position:absolute;inset:0;touch-action:none}.render-error{position:absolute;bottom:0;left:0;right:0;background:#fff0f0;color:#b54454;padding:12px;font-size:14px}.render-error button{border:1px solid #efb5c0;margin-left:12px}.properties{width:220px;flex:none;padding:13px 12px;overflow:auto;border-left:1px solid var(--ui-line,#eee);background:var(--ui-surface,#fff)}.properties h3{font-size:11px;margin:0 0 14px}.properties h4{font-size:10px;border-top:1px solid var(--ui-line,#eee);padding-top:12px;margin:12px 0}.properties label{display:flex;justify-content:space-between;align-items:center;gap:9px;margin:9px 0;font-size:10px;color:var(--ui-text-secondary,#777)}.properties input,.properties select{font:inherit;min-width:0;width:115px;background:var(--ui-control,#f8f8fa);color:var(--ui-text-primary,#333);border:1px solid var(--ui-line,#e8e9ed);border-radius:5px;padding:4px}.properties input[type=color]{height:25px;width:35px;padding:2px}.properties input[type=checkbox]{width:15px}.dimensions{display:flex;gap:8px}.dimensions label{display:block;width:50%}.dimensions input{width:100%;margin-top:4px}.presets{display:flex;gap:4px;margin-bottom:12px}.presets button{flex:1;font-size:9px;padding:2px 4px;border:1px solid var(--ui-line,#e8e9ed)}.prop-tabs{display:flex;gap:4px;margin-bottom:12px;border-bottom:1px solid var(--ui-line,#eee);padding-bottom:6px}.prop-tabs button{flex:1;font-size:10px;padding:4px 6px}.prop-tabs button.active{background:var(--ui-soft,#fceff1);color:var(--ui-ink,#c14558);font-weight:600}.node-path{font-size:9px;color:#999;font-family:monospace;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;margin:-4px 0 8px}.comment-head{display:flex;align-items:center;justify-content:space-between}.mini-btn{font-size:9px;padding:2px 4px;color:var(--ui-text-tertiary,#999)}.comment{padding:8px;border:1px solid var(--ui-line,#eee);border-radius:6px;margin-bottom:8px;font-size:10px}.comment-node{display:block;color:#aaa;font-size:9px}.comment p{white-space:pre-wrap;overflow-wrap:anywhere;color:var(--ui-text-secondary,#777);margin:5px 0}.comment-foot{display:flex;align-items:center;justify-content:space-between}.comment span{font-size:9px;color:#a5a6b0}.comment button{font-size:9px}.text-btn{width:100%;border:1px solid var(--ui-line,#eee)!important;margin-top:6px}.text-btn.primary-soft{background:var(--ui-soft,#fceff1);color:var(--ui-ink,#c14558);border-color:#f5c9cf!important}.text-btn.mini{padding:2px 6px;min-height:20px;font-size:9px}.token-row{display:flex;align-items:center;justify-content:space-between;gap:6px;margin-bottom:6px}.token-row code{font-size:9px;color:#c14558}.token-row input{width:90px;font-size:10px;padding:2px 4px}.add-token{display:flex;flex-direction:column;gap:4px;margin-top:10px}.add-token input{width:100%;box-sizing:border-box}.card-item{padding:8px;border:1px solid var(--ui-line,#eee);border-radius:6px;margin-bottom:8px}.card-meta{display:flex;justify-content:space-between;align-items:center}.card-meta small{color:#999;font-size:9px}.card-file{font-size:9px;color:#888;font-family:monospace;margin:2px 0 4px}.empty-note{color:#aaa;font-size:9px;line-height:1.5}.quote{font-style:italic;color:#666;border-left:2px solid var(--ui-btn,#d94e60);padding-left:6px;margin:4px 0}.empty{padding:30px;color:#999;line-height:1.8}.errorbar{background:#fff0f0;color:#aa4250;font-size:10px;padding:7px 10px}.errorbar button{text-decoration:underline}footer{height:23px;flex:none;display:flex;align-items:center;gap:12px;padding:0 11px;border-top:1px solid var(--ui-line,#eee);font-size:9px;color:var(--ui-text-tertiary,#999)}.overlay{position:absolute;inset:0;z-index:20;background:#25273333;display:flex;align-items:center;justify-content:center}.dialog{width:350px;max-width:90%;max-height:85%;overflow:auto;background:var(--ui-surface,#fff);border:1px solid var(--ui-line,#eee);border-radius:10px;padding:18px;box-shadow:0 12px 40px #0002}.dialog h3{font-size:13px;margin:0 0 10px}.dialog p{font-size:11px;color:#999}.dialog textarea{width:100%;height:90px;resize:vertical;box-sizing:border-box;font:inherit;font-size:12px;line-height:1.6;color:inherit;background:var(--ui-control,#f8f8fa);border:1px solid var(--ui-line,#e8e9ed);border-radius:6px;padding:8px}.check{display:flex;align-items:center;gap:6px;font-size:11px;margin-top:9px}.actions{display:flex;gap:8px;justify-content:end;margin-top:15px}.actions .primary{background:var(--ui-btn,#d94e60);color:white}.source-dialog{width:750px}.source-dialog textarea{height:50vh;font:11px/1.7 monospace}.version-row{display:flex;justify-content:space-between;align-items:center;margin:8px 0;font-size:11px}@container(max-width:760px){.properties{position:absolute;right:0;top:0;bottom:0;z-index:4;box-shadow:-5px 0 20px #0001}.rail{width:115px}.saved{display:none}}
</style>
