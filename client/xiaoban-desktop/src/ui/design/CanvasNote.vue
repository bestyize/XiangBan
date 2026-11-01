<script setup lang="ts">
import {computed,nextTick,ref,watch} from 'vue';
import type {CanvasNote} from './canvas-notes.ts';
const props=defineProps<{note:CanvasNote;selected:boolean;writable:boolean;scale:number;offsetX:number;offsetY:number}>();
const emit=defineEmits<{select:[];patch:[patch:Partial<CanvasNote>];remove:[];duplicate:[]}>();
const editing=ref(false),text=ref(''),input=ref<HTMLTextAreaElement|null>(null),moving=ref<{x:number;y:number;w:number}|null>(null);
let drag:{x:number;y:number;original:CanvasNote;resize:boolean;pointer:number}|undefined;
const title=computed(()=>props.note.kind.startsWith('title'));
const style=computed(()=>({left:((moving.value?.x??props.note.x)+props.offsetX)+'px',top:((moving.value?.y??props.note.y)+props.offsetY)+'px',width:(moving.value?.w??props.note.w)+'px',maxHeight:props.note.maxH+'px'}));
async function edit(){if(!props.writable)return;emit('select');text.value=props.note.text;editing.value=true;await nextTick();input.value?.focus();input.value?.select();}
function finish(cancel=false){if(!editing.value)return;editing.value=false;if(!cancel&&text.value!==props.note.text)emit('patch',{text:text.value});}
watch(()=>props.note.text,()=>{if(editing.value)finish(true);});
watch(()=>props.writable,v=>{if(!v){drag=undefined;moving.value=null;}});
function down(e:PointerEvent,resize=false){if(e.button!==0||!props.writable||editing.value)return;emit('select');drag={x:e.clientX,y:e.clientY,original:{...props.note},resize,pointer:e.pointerId};(e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);e.preventDefault();}
function move(e:PointerEvent){if(!drag||drag.pointer!==e.pointerId)return;const dx=(e.clientX-drag.x)/props.scale,dy=(e.clientY-drag.y)/props.scale;moving.value=drag.resize?{x:drag.original.x,y:drag.original.y,w:Math.max(120,Math.min(16000,Math.round(drag.original.w+dx)))}:{x:Math.round(drag.original.x+dx),y:Math.round(drag.original.y+dy),w:drag.original.w};}
function up(cancel=false){if(!drag)return;const original=drag.original;drag=undefined;const patch=moving.value;moving.value=null;if(!cancel&&patch&&(patch.x!==original.x||patch.y!==original.y||patch.w!==original.w))emit('patch',patch);}
function key(e:KeyboardEvent){
 if(e.isComposing)return;
 if(editing.value){if(e.key==='Escape'){e.preventDefault();finish(true);}else if(e.key==='Tab'||e.key==='Enter'&&(e.metaKey||e.ctrlKey)){e.preventDefault();finish();}return;}
 if(!props.writable)return;
 if(e.key==='Enter'){e.preventDefault();void edit();}
 else if(e.key==='Delete'||e.key==='Backspace'){e.preventDefault();emit('remove');}
 else if((e.metaKey||e.ctrlKey)&&e.key.toLowerCase()==='d'){e.preventDefault();emit('duplicate');}
 else if(e.key.startsWith('Arrow')){e.preventDefault();const d=e.shiftKey?10:1;emit('patch',{x:props.note.x+(e.key==='ArrowLeft'?-d:e.key==='ArrowRight'?d:0),y:props.note.y+(e.key==='ArrowUp'?-d:e.key==='ArrowDown'?d:0)});}
}
</script>
<template>
<article class="canvas-note" :class="{selected,title,readonly:!writable}" :style="style" tabindex="0" :aria-label="(title?'画布标题：':'画布便签：')+note.text.slice(0,80)" @click.stop="emit('select')" @dblclick.stop="edit" @keydown.stop="key" @pointerdown.stop="down" @pointermove.stop="move" @pointerup.stop="up()" @pointercancel.stop="up(true)">
 <textarea v-if="editing" ref="input" v-model="text" aria-label="编辑画布便签" maxlength="50000" @pointerdown.stop @blur="finish()"/>
 <div v-else class="note-text">{{note.text}}</div>
 <button v-if="selected&&writable&&!editing" class="note-resize" aria-label="调整便签宽度" @pointerdown.stop="down($event,true)" @pointermove.stop="move" @pointerup.stop="up()" @pointercancel.stop="up(true)"/>
</article>
</template>
<style scoped>
.canvas-note{position:absolute;box-sizing:border-box;padding:20px;background:var(--ui-soft);color:var(--ui-text);border:2px solid transparent;border-radius:8px;cursor:move;font:16px/1.6 system-ui;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;touch-action:none}.canvas-note.readonly{pointer-events:none}.canvas-note.selected{border-color:var(--ui-btn)}.canvas-note.title{padding:0;background:transparent;border-radius:0;font-size:28px;font-weight:650}.canvas-note:focus-visible{outline:2px solid var(--ui-btn);outline-offset:3px}.canvas-note textarea{width:100%;min-height:160px;box-sizing:border-box;background:transparent;color:inherit;font:inherit;resize:vertical;border:0;outline:none;cursor:text}.note-resize{position:absolute;right:0;bottom:0;width:12px;height:12px;border:1px solid var(--ui-btn);background:var(--ui-surface);cursor:ew-resize;padding:0}
</style>
