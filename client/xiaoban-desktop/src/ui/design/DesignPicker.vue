<script setup lang="ts">
import {computed,onMounted,ref} from 'vue';
import {invoke} from '../../bridge.ts';
import {statusLabel,type Summary,type Payload} from './model.ts';
const props=defineProps<{session:string|null;notify:(message:unknown,type?:'error'|'info'|'success')=>void}>();
const emit=defineEmits<{close:[];open:[id:string]}>();
const rows=ref<Summary[]>([]),query=ref(''),busy=ref(false),error=ref('');
const filtered=computed(()=>rows.value.filter(p=>!query.value.trim()||[p.title,p.id].some(v=>v.toLowerCase().includes(query.value.trim().toLowerCase()))));
const current=computed(()=>filtered.value.filter(p=>p.sessionId===props.session||p.sessionIds?.includes(props.session??'')));
const recent=computed(()=>filtered.value.filter(p=>!current.value.includes(p)));
onMounted(async()=>{try{rows.value=await invoke('design_list',{session:'*'});}catch(e){error.value=String(e);}});
async function openProject(id:string){
 if(busy.value)return;busy.value=true;
 try{await invoke(props.session?'design_link':'design_read',props.session?{id,session:props.session}:{id});emit('open',id);}catch(e){error.value=String(e);}finally{busy.value=false;}
}
async function create(){
 if(busy.value)return;busy.value=true;
 try{const result=await invoke<Payload>('design_create',{session:props.session??'library'});emit('open',result.project.id);}catch(e){error.value=String(e);}finally{busy.value=false;}
}
</script>
<template>
<div class="design-picker" role="dialog" aria-label="打开设计稿">
 <header><b>设计稿</b><button aria-label="关闭设计稿列表" @click="emit('close')">×</button></header>
 <input v-model="query" autofocus aria-label="搜索设计稿" placeholder="搜索名称或设计稿 ID" @keydown.enter="/^design-[0-9a-f]{24}$/i.test(query.trim())&&openProject(query.trim())" />
 <p v-if="error" role="alert">{{error}}</p>
 <div class="list">
  <template v-for="group in [{title:'本会话',items:current},{title:'最近与全部设计稿',items:recent}]" :key="group.title">
   <h4 v-if="group.items.length">{{group.title}}</h4>
   <button v-for="p in group.items" :key="p.id" class="item" :disabled="busy" @click="openProject(p.id)"><b>{{p.title}}</b><small>{{statusLabel(p.status)}} · {{p.artboards}} 个画板</small></button>
  </template>
  <p v-if="!filtered.length">暂无匹配的设计稿</p>
 </div>
 <footer><small>打开同一份设计稿，可在工作台创建副本</small><button :disabled="busy" @click="create">＋ 新建设计稿</button></footer>
</div>
</template>
<style scoped>
.design-picker{position:absolute;right:0;top:36px;z-index:25;width:320px;padding:14px;border:1px solid var(--ui-line);border-radius:12px;background:var(--ui-surface);box-shadow:var(--ui-shadow-menu);color:var(--ui-text-primary)}header,footer{display:flex;justify-content:space-between;align-items:center;gap:8px}button{border:0;background:transparent;color:inherit;cursor:pointer;padding:5px}input{width:100%;box-sizing:border-box;padding:8px 10px;margin:12px 0 4px;border:1px solid var(--ui-line);border-radius:8px;background:var(--ui-control);color:inherit;font:inherit}.list{max-height:330px;overflow:auto}h4{font-size:11px;font-weight:500;color:var(--ui-text-tertiary);margin:12px 0 4px}.item{display:flex;width:100%;text-align:left;flex-direction:column;gap:4px;padding:9px 8px;border-radius:8px}.item:hover{background:var(--ui-control)}.item b{font-size:12px;font-weight:500}.item small,footer small,p{font-size:10px;color:var(--ui-text-tertiary)}footer{border-top:1px solid var(--ui-line);padding-top:10px;margin-top:8px}footer small{max-width:150px}footer button{font-size:11px;color:var(--ui-ink)}
</style>
