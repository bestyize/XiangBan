<script setup lang="ts">
import { ref, watch } from 'vue';
import { emptyCountdown, validateCountdown } from './content.ts';
import type { CountdownData, WidgetContent } from './model.ts';
const props=defineProps<{data?:CountdownData;save:(content:WidgetContent)=>Promise<void>}>();
const draft=ref<CountdownData>(JSON.parse(JSON.stringify(props.data??emptyCountdown()))),dirty=ref(false),busy=ref(false),error=ref('');
watch(()=>props.data,value=>{if(!dirty.value&&!busy.value)draft.value=JSON.parse(JSON.stringify(value??emptyCountdown()));},{deep:true});
function mode(){if(draft.value.mode==='elapsed')draft.value.repeat='none';dirty.value=true;}
async function commit(){error.value='';try{validateCountdown(draft.value);busy.value=true;await props.save({countdown:{...draft.value}});dirty.value=false;}catch(e){error.value=String(e);}finally{busy.value=false;}}
</script>
<template>
  <form class="countdown-editor" @submit.prevent="commit"><fieldset :disabled="busy"><label>事件名称<input v-model="draft.title" aria-label="事件名称" maxlength="40" required @input="dirty=true"/></label><label>日期<input v-model="draft.date" aria-label="事件日期" type="date" min="1900-01-01" max="2100-12-31" required @input="dirty=true"/></label><label>计算方式<select v-model="draft.mode" aria-label="计算方式" @change="mode"><option value="countdown">倒数日 · 距离这一天</option><option value="elapsed">纪念日 · 从这一天开始</option></select></label><label v-if="draft.mode==='countdown'">重复<select v-model="draft.repeat" aria-label="重复方式" @change="dirty=true"><option value="none">一次性</option><option value="yearly">每年重复</option></select></label><p v-if="draft.repeat==='yearly'" class="editor-hint">2 月 29 日在平年按 2 月 28 日纪念。</p><button :disabled="!dirty||busy">{{ busy?'保存中…':'保存日期设置' }}</button></fieldset><p v-if="error" role="alert">{{ error }}</p></form>
</template>
<style scoped>
.countdown-editor{margin-top:18px;font-size:12px}.countdown-editor fieldset{border:0;padding:0;margin:0;display:grid;gap:15px}.countdown-editor label{display:grid;gap:8px}.countdown-editor input,.countdown-editor select{box-sizing:border-box;width:100%;border:1px solid var(--ui-border,#cbbba4);background:var(--ui-surface,#fff9ee);color:inherit;border-radius:6px;padding:8px;font:inherit}.countdown-editor button{border:1px solid var(--ui-border,#cbbba4);background:var(--ui-surface-alt,#eee2cb);color:inherit;padding:9px;border-radius:6px;cursor:pointer}.countdown-editor button:disabled{opacity:.4;cursor:default}.countdown-editor p{font-size:11px;line-height:1.7;color:#b26450;margin:0}.countdown-editor .editor-hint{color:var(--ui-text-tertiary,#8f826d)}
</style>
