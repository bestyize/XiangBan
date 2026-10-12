<script setup lang="ts">
import { ref, watch } from 'vue';
import { emptyNote, validateNote } from './content.ts';
import type { NoteData, WidgetContent } from './model.ts';
const props=defineProps<{note?:NoteData;preview?:boolean;editing?:boolean;save?:(content:WidgetContent)=>Promise<void>}>();
const draft=ref<NoteData>(JSON.parse(JSON.stringify(props.note??emptyNote()))),dirty=ref(false),busy=ref(false),error=ref(''),task=ref('');
watch(()=>props.note,value=>{if(!dirty.value&&!busy.value)draft.value=JSON.parse(JSON.stringify(value??emptyNote()));},{deep:true});
async function commit(){
  if(props.preview||props.editing||busy.value||!props.save)return;
  error.value='';
  try{validateNote(draft.value);busy.value=true;await props.save({note:JSON.parse(JSON.stringify(draft.value))});dirty.value=false;}
  catch(e){dirty.value=true;error.value=String(e);}finally{busy.value=false;}
}
async function mode(value:NoteData['mode']){draft.value.mode=value;dirty.value=true;await commit();}
async function add(){if(!task.value.trim()||draft.value.items.length>=12)return;draft.value.items.push({id:crypto.randomUUID().replaceAll('-','').slice(0,16),text:task.value.trim(),done:false});task.value='';dirty.value=true;await commit();}
async function toggle(index:number){draft.value.items[index].done=!draft.value.items[index].done;dirty.value=true;await commit();}
async function remove(index:number){draft.value.items.splice(index,1);dirty.value=true;await commit();}
</script>
<template>
  <div class="notes-content" :class="{preview}" @pointerdown.stop>
    <input v-model="draft.title" class="note-title" aria-label="便签标题" maxlength="40" :disabled="preview||editing||busy" @input="dirty=true"/>
    <nav class="note-modes" aria-label="便签视图"><button :class="{on:draft.mode==='note'}" :disabled="preview||editing||busy" @click="mode('note')">便签</button><button :class="{on:draft.mode==='todo'}" :disabled="preview||editing||busy" @click="mode('todo')">待办 <small>{{ draft.items.filter(item=>!item.done).length }}</small></button></nav>
    <textarea v-if="draft.mode==='note'" v-model="draft.text" aria-label="便签正文" placeholder="记下此刻的灵感，或留一句话给自己…" maxlength="1200" :disabled="preview||editing||busy" @input="dirty=true" @keydown.ctrl.enter.prevent="commit" @keydown.meta.enter.prevent="commit"/>
    <template v-else><div class="todo-list"><p v-if="!draft.items.length" class="note-empty">今天想完成什么？</p><div v-for="(item,index) in draft.items" :key="item.id" class="todo-item"><input type="checkbox" :checked="item.done" :aria-label="`完成：${item.text}`" :disabled="preview||editing||busy" @change="toggle(index)"/><span :class="{done:item.done}">{{ item.text }}</span><button :aria-label="`删除：${item.text}`" :disabled="preview||editing||busy" @click="remove(index)">×</button></div></div><form class="todo-add" @submit.prevent="add"><input v-model="task" aria-label="新增待办" maxlength="120" :placeholder="draft.items.length>=12?'最多 12 项':'添加一件小事…'" :disabled="preview||editing||busy||draft.items.length>=12"/><button :disabled="preview||editing||busy||!task.trim()||draft.items.length>=12" aria-label="添加待办">＋</button></form></template>
    <footer><span>{{ dirty?'有未保存的修改':'仅保存在这台设备' }}</span><button :disabled="preview||editing||busy||!dirty" @click="commit">{{ busy?'保存中…':'保存' }}</button></footer>
    <p v-if="error" class="content-error" role="alert">{{ error }}，请重试保存</p>
  </div>
</template>
<style scoped>
.notes-content{display:flex;flex-direction:column;gap:13px;height:285px;padding-top:12px;cursor:default;user-select:text;touch-action:auto}.note-title{width:100%;font-size:20px;font-weight:500}.notes-content input,.notes-content textarea{box-sizing:border-box;border:0;background:none;color:inherit;font-family:inherit;outline:none}.notes-content input:focus-visible,.notes-content textarea:focus-visible{outline:1px solid #bba26d;outline-offset:3px;border-radius:4px}.note-modes{display:flex;gap:15px;font-size:12px;border-bottom:1px solid #9e937733;padding-bottom:8px}.notes-content button{color:inherit;border:0;background:none;cursor:pointer;font:inherit}.note-modes button{opacity:.5;padding:0}.note-modes button.on{opacity:1;font-weight:600}.note-modes small{font-size:10px;opacity:.6}.notes-content textarea{resize:none;flex:1;width:100%;font-size:13px;line-height:1.9;padding:0;min-height:0}.notes-content textarea::placeholder,.todo-add input::placeholder{color:inherit;opacity:.45}.notes-content footer{display:flex;align-items:center;justify-content:space-between;font-size:10px;opacity:.7}.notes-content footer button{border:1px solid #a9966b66;border-radius:5px;padding:4px 12px}.todo-list{flex:1;overflow:auto;min-height:0}.todo-item{display:flex;align-items:start;gap:8px;font-size:12px;line-height:1.7;padding:7px 0;border-bottom:1px solid #9e93771a}.todo-item input{flex:none;accent-color:#a58b5f;margin-top:5px}.todo-item span{flex:1;overflow-wrap:anywhere}.todo-item .done{text-decoration:line-through;opacity:.4}.todo-item button{opacity:.5;font-size:17px}.todo-add{display:flex;border-bottom:1px solid #9e937755;gap:6px;padding-bottom:5px}.todo-add input{min-width:0;flex:1;font-size:12px}.todo-add button{font-size:18px}.note-empty{font-size:12px;opacity:.5}.content-error{margin:0;font-size:10px;color:#c26051;max-height:36px;overflow:auto}.notes-content button:disabled{opacity:.35;cursor:default}.preview{pointer-events:none}
</style>
