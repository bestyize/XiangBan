<script setup lang="ts">
import {ref} from 'vue';
import {dateKey} from './calendar.ts';
import {repeats,validateTodo,validateReminder} from './content.ts';
import type {TodoItem,ReminderItem,ItemRepeat} from './model.ts';
const props=defineProps<{mode:'todo'|'agenda';item?:TodoItem|ReminderItem;busy?:boolean}>();
const emit=defineEmits<{save:[item:TodoItem|ReminderItem];cancel:[]}>();
const agenda=props.mode==='agenda',old=props.item;
const title=ref(old?('text'in old?old.text:old.title):''),date=ref(old?('at'in old?old.at.slice(0,10):old.date||dateKey(new Date())):dateKey(new Date()));
const time=ref(old?('at'in old?old.at.slice(11):old.time||''):agenda?'09:00':'');
const end=ref(old&&'at'in old?old.end||'10:00':'10:00'),location=ref(old&&'at'in old?old.location||'':'');
const repeat=ref<ItemRepeat>(old?.repeat||'none'),notify=ref(old?.notify??(old?agenda:true)),error=ref('');
function submit(){
 error.value='';const id=old?.id||crypto.randomUUID().replaceAll('-','').slice(0,16);
 const changed=old&&(('at'in old?old.at:old.date+'T'+old.time)!==date.value+'T'+time.value||old.repeat!==repeat.value);
 const schedule={repeat:repeat.value,notify:!!time.value&&notify.value,done:repeat.value==='none'?old?.done??false:false,completedOn:changed?'':old?.completedOn||''};
 const item:TodoItem|ReminderItem=agenda?{...schedule,id,title:title.value.trim(),at:date.value+'T'+time.value,end:end.value,location:location.value.trim()}:{...schedule,id,text:title.value.trim(),date:date.value,time:time.value};
 try{if(agenda)validateReminder(item as ReminderItem);else validateTodo(item as TodoItem);emit('save',item);}catch(e){error.value=String(e);}
}
</script>
<template><form class="xb-item-form" data-no-drag @submit.prevent="submit"><fieldset :disabled="busy"><header><strong>{{ item?'编辑':'添加' }}{{ agenda?'日程':'待办' }}</strong><button type="button" class="item-close" aria-label="关闭编辑" @click="emit('cancel')">×</button></header><label>事项名称<input v-model="title" :maxlength="agenda?60:120" placeholder="写下要做的事" aria-label="事项名称" required/></label><label>日期<input v-model="date" type="date" min="1900-01-01" max="2100-12-31" :aria-label="agenda?'日程日期':'待办日期'" required/></label><div class="item-times"><label>{{ agenda?'开始时间':'提醒时间（可选）' }}<input v-model="time" type="time" :aria-label="agenda?'开始时间':'待办时间'" :required="agenda"/></label><label v-if="agenda">结束时间<input v-model="end" type="time" aria-label="结束时间" required/></label></div><label v-if="agenda">地点<input v-model="location" maxlength="80" placeholder="地点（可选）" aria-label="日程地点"/></label><div class="item-repeat"><label>重复<select v-model="repeat" aria-label="重复方式"><option v-for="r in repeats" :key="r.value" :value="r.value">{{ r.label }}</option></select></label><label class="item-notify"><input v-model="notify" type="checkbox" :disabled="!time" aria-label="到点提醒"/>到点提醒</label></div><small>提醒在小伴运行时生效，遵循关怀提醒总开关和勿扰设置。</small><button class="item-submit" :disabled="busy">{{ busy?'保存中…':'保存' }}</button><p v-if="error" role="alert">{{ error }}</p></fieldset></form></template>
<style scoped>
.xb-item-form{font-size:14px;color:inherit}.xb-item-form fieldset{display:grid;gap:10px;border:0;padding:0;margin:0}.xb-item-form header{display:flex;align-items:center;justify-content:space-between;min-height:24px}.xb-item-form label{display:grid;gap:5px;font-size:13px}.xb-item-form input:not([type=checkbox]),.xb-item-form select{box-sizing:border-box;min-width:0;width:100%;border:1px solid var(--ui-border,#d8ccbe);background:var(--ui-surface,#fffaf3);color:inherit;border-radius:8px;padding:7px;font:inherit;touch-action:auto;user-select:text}.item-times,.item-repeat{display:flex;gap:12px}.item-times label,.item-repeat>label:first-child{flex:1;min-width:0}.item-notify{display:flex!important;align-items:center;gap:5px;white-space:nowrap}.item-notify input{accent-color:#b4987b}.xb-item-form button{font:inherit;cursor:pointer;border:0}.item-close{background:none;color:inherit;font-size:24px!important}.item-submit{padding:8px;border-radius:9px;background:#b4987b;color:#fff}.xb-item-form small{font-size:11px;line-height:1.5;color:var(--ui-text-tertiary,#9a8a78)}.xb-item-form p{font-size:12px;color:#bb6558;margin:0}
</style>
