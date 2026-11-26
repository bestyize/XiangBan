<script setup lang="ts">
import { ref, watch } from 'vue';
import { emptyCountdown, validateCountdown } from './content.ts';
import type { CountdownData, CountdownIcon, WidgetContent } from './model.ts';
const props=defineProps<{data?:CountdownData;save:(content:WidgetContent)=>Promise<void>}>();
const normalize=(value?:CountdownData):CountdownData=>({quote:'',icon:'',...emptyCountdown(),...(value??{})});
const draft=ref<CountdownData>(normalize(props.data)),dirty=ref(false),busy=ref(false),error=ref('');
watch(()=>props.data,value=>{if(!dirty.value&&!busy.value)draft.value=normalize(value);},{deep:true});
const icons:Array<{value:CountdownIcon;label:string}>=[{value:'heart',label:'心形'},{value:'flag',label:'目标'},{value:'sparkle',label:'星光'},{value:'calendar',label:'日程'}];
function mode(){if(draft.value.mode==='elapsed')draft.value.repeat='none';dirty.value=true;}
async function commit(){
  error.value='';
  const next:CountdownData={title:draft.value.title,date:draft.value.date,mode:draft.value.mode,repeat:draft.value.repeat};
  if(draft.value.quote?.trim())next.quote=draft.value.quote.trim();
  if(draft.value.icon)next.icon=draft.value.icon;
  try{validateCountdown(next);busy.value=true;await props.save({countdown:next});dirty.value=false;}
  catch(e){error.value=String(e);}
  finally{busy.value=false;}
}
</script>
<template>
  <form class="countdown-editor" @submit.prevent="commit">
    <fieldset :disabled="busy">
      <label>事件名称<input v-model="draft.title" aria-label="事件名称" maxlength="40" required @input="dirty=true"/></label>
      <label>日期<input v-model="draft.date" aria-label="事件日期" type="date" min="1900-01-01" max="2100-12-31" required @input="dirty=true"/></label>
      <label>计算方式<select v-model="draft.mode" aria-label="计算方式" @change="mode"><option value="countdown">倒数日 · 距离这一天</option><option value="elapsed">纪念日 · 从这一天开始</option></select></label>
      <label v-if="draft.mode==='countdown'">重复<select v-model="draft.repeat" aria-label="重复方式" @change="dirty=true"><option value="none">一次性</option><option value="yearly">每年重复</option></select></label>
      <p v-if="draft.repeat==='yearly'" class="editor-hint">2 月 29 日在平年按 2 月 28 日纪念。</p>
      <div class="icon-picker">
        <span>图标</span>
        <div class="icon-options" role="group" aria-label="纪念日图标">
          <button v-for="ic in icons" :key="ic.value" type="button" :class="{'is-on':(draft.icon||'heart')===ic.value}" @click="draft.icon=ic.value;dirty=true">{{ ic.label }}</button>
        </div>
      </div>
      <label>底部短句（留空隐藏）<input v-model="draft.quote" aria-label="底部短句" maxlength="60" placeholder="例如：每一次倒数，都是奔赴期待的开始" @input="dirty=true"/></label>
      <button type="submit" class="save-btn" :disabled="!dirty||busy">{{ busy?'保存中…':'保存日期设置' }}</button>
    </fieldset>
    <p v-if="error" role="alert">{{ error }}</p>
  </form>
</template>
<style scoped>
.countdown-editor{margin-top:8px;font-size:12.5px}
.countdown-editor fieldset{border:0;padding:0;margin:0;display:grid;gap:12px}
.countdown-editor label,.icon-picker{display:grid;gap:5px;color:var(--ui-text-secondary,#6e6e73);font-size:12px}
.countdown-editor input,.countdown-editor select{box-sizing:border-box;width:100%;border:0;background:var(--ui-control,#f2f2f4);color:var(--ui-text-primary,#1d1d1f);border-radius:8px;padding:8px 10px;font:inherit;font-size:12.5px}
.icon-options{display:grid;grid-template-columns:repeat(4,1fr);gap:6px}
.icon-options button{border:0;border-radius:8px;height:28px;background:var(--ui-control,#f2f2f4);color:var(--ui-text-secondary,#6e6e73);font:inherit;font-size:12px;cursor:pointer}
.icon-options button.is-on{background:var(--ui-soft,#faeedb);color:var(--ui-ink,#87500a);font-weight:600}
.save-btn{border:0;background:var(--ui-btn,#a8640c);color:#fff;padding:8px 12px;border-radius:9px;font-weight:500;cursor:pointer}
.save-btn:disabled{opacity:.4;cursor:default}
.countdown-editor p{font-size:11.5px;line-height:1.6;color:var(--ui-danger,#d93a44);margin:0}
.countdown-editor .editor-hint{color:var(--ui-text-tertiary,#8e8e93)}
</style>
