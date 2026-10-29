<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
const props=defineProps<{scope?:HTMLElement|null}>();
const emit=defineEmits<{visibilityChange:[]}>();
const opened=ref(false),x=ref(0),y=ref(0),failed=ref(false),copying=ref(false),menu=ref<HTMLElement>();
let selected='',range:Range|null=null,disposed=false,restoringSelection=false;
function close(){if(!opened.value)return;opened.value=false;selected='';range=null;void nextTick(()=>emit('visibilityChange'));}
function context(event:MouseEvent){
  const scope=props.scope,selection=window.getSelection();
  if(!scope||!(event.target instanceof Node)||!scope.contains(event.target)||!selection?.rangeCount||selection.isCollapsed){close();return;}
  const chosen=selection.getRangeAt(0);
  if(!scope.contains(chosen.startContainer)||!scope.contains(chosen.endContainer)||!selection.toString().trim()){close();return;}
  event.preventDefault();selected=selection.toString();range=chosen.cloneRange();failed.value=false;
  x.value=Math.max(6,Math.min(event.clientX,window.innerWidth-154));y.value=Math.max(6,Math.min(event.clientY,window.innerHeight-46));
  opened.value=true;void nextTick(()=>emit('visibilityChange'));
}
async function copy(){
  if(copying.value||!selected)return;
  copying.value=true;const value=selected,saved=range?.cloneRange();
  try{
    try{if(!navigator.clipboard?.writeText)throw new Error('Clipboard unavailable');await navigator.clipboard.writeText(value);}
    catch(error){
      if(!saved)throw error;
      restoringSelection=true;
      try{const selection=window.getSelection();if(selection?.toString()!==value){selection?.removeAllRanges();selection?.addRange(saved);}
        if(!document.execCommand?.('copy'))throw error;
      }finally{restoringSelection=false;}
    }
    if(!disposed)close();
  }catch{if(!disposed)failed.value=true;}
  finally{copying.value=false;}
}
function outside(event:PointerEvent){if(!menu.value?.contains(event.target as Node))close();}
function keyboard(event:KeyboardEvent){
  if(!opened.value)return;
  if(event.key==='Escape'){event.preventDefault();event.stopPropagation();close();}
  if(event.key==='Enter'){event.preventDefault();event.stopPropagation();void copy();}
}
function selectionChanged(){if(opened.value&&!restoringSelection&&window.getSelection()?.toString()!==selected)close();}
onMounted(()=>{document.addEventListener('contextmenu',context);document.addEventListener('pointerdown',outside,true);document.addEventListener('keydown',keyboard,true);document.addEventListener('selectionchange',selectionChanged);window.addEventListener('blur',close);window.addEventListener('resize',close);document.addEventListener('scroll',close,true);});
onBeforeUnmount(()=>{disposed=true;document.removeEventListener('contextmenu',context);document.removeEventListener('pointerdown',outside,true);document.removeEventListener('keydown',keyboard,true);document.removeEventListener('selectionchange',selectionChanged);window.removeEventListener('blur',close);window.removeEventListener('resize',close);document.removeEventListener('scroll',close,true);});
</script>
<template>
  <Teleport to="body"><div v-if="opened" ref="menu" class="selection-copy-menu" role="menu" aria-label="选中文字操作" :style="{left:x+'px',top:y+'px'}" data-no-drag @pointerdown.prevent.stop @contextmenu.prevent.stop><button type="button" role="menuitem" :disabled="copying" @click="copy">{{failed?'复制失败，重试':'复制'}}</button></div></Teleport>
</template>
<style scoped>
.selection-copy-menu{position:fixed;z-index:10000;width:148px;padding:4px;border-radius:9px;border:1px solid var(--ui-border,var(--line,#e2e4e0));background:var(--ui-surface,var(--card,#fff));color:var(--ui-text-primary,var(--text,#343832));box-shadow:0 4px 16px #0002;pointer-events:auto;cursor:default;user-select:none;box-sizing:border-box}
.selection-copy-menu button{display:block;width:100%;height:30px;padding:0 10px;text-align:left;border:0;border-radius:5px;background:transparent;color:inherit;font:13px/1.4 -apple-system,"PingFang SC","Microsoft YaHei",sans-serif;cursor:pointer}
.selection-copy-menu button:hover,.selection-copy-menu button:focus-visible{background:var(--ui-surface-alt,#8882);outline:none}.selection-copy-menu button:disabled{opacity:.5}
</style>
