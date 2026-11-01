<script setup lang="ts">
import {computed,ref,watch} from 'vue';
import {elementCode,type ElementAction} from './element-commands.ts';
const props=defineProps<{source:string;nodeId:string;writable:boolean;readonly?:boolean}>();
const emit=defineEmits<{save:[payload:{expectedSource:string;action:ElementAction}];notice:[message:string]}>();
const css=ref(''),overrides=ref(''),attributes=ref('{}'),original=ref(''),loaded=ref(''),error=ref('');
const dirty=computed(()=>JSON.stringify([css.value,overrides.value,attributes.value])!==loaded.value);
function read(){try{const value=elementCode(props.source,props.nodeId);css.value=value.style;overrides.value=value.overrides;attributes.value=JSON.stringify(value.attributes,null,2);original.value=props.source;loaded.value=JSON.stringify([css.value,overrides.value,attributes.value]);error.value='';}catch{error.value='此元素由组件生成，请修改组件源文件。';}}
watch(()=>props.nodeId,read,{immediate:true});
watch(()=>props.source,()=>{if(!dirty.value){read();return;}try{const value=elementCode(props.source,props.nodeId);if(value.style===css.value&&value.overrides===overrides.value&&JSON.stringify(value.attributes)===JSON.stringify(JSON.parse(attributes.value)))read();}catch{}});
function save(){if(props.readonly||!props.writable)return;if(original.value!==props.source){emit('notice','源文件已有新版本，请先复制当前草稿再重新读取');return;}try{const attrs=JSON.parse(attributes.value);if(!attrs||Array.isArray(attrs)||typeof attrs!=='object')throw Error('属性必须是 JSON 对象');emit('save',{expectedSource:original.value,action:{type:'code',id:props.nodeId,style:css.value,overrides:overrides.value,attributes:attrs}});}catch{emit('notice','属性必须是有效的 JSON 对象');}}
</script>
<template>
<form class="element-code" @submit.prevent="save">
  <p v-if="error" role="status">{{error}}</p>
  <template v-else>
    <label>CSS<textarea v-model="css" aria-label="元素 CSS" spellcheck="false" :readonly="readonly||!writable"/></label>
    <label v-if="overrides">样式覆盖<textarea v-model="overrides" aria-label="元素样式覆盖" spellcheck="false" :readonly="readonly||!writable"/></label>
    <label>HTML 属性<textarea v-model="attributes" aria-label="元素 HTML 属性" spellcheck="false" :readonly="readonly||!writable"/></label>
    <p v-if="original!==source" role="alert">源文件已更新，当前草稿尚未保存。</p>
    <div class="code-actions"><button type="button" @click="read">重新读取</button><button type="submit" :disabled="readonly||!writable||!dirty">保存修改</button></div>
  </template>
</form>
</template>
<style scoped>
.element-code label{display:block!important}.element-code textarea{display:block;box-sizing:border-box;width:100%;min-height:140px;margin-top:6px;padding:8px;resize:vertical;border:1px solid var(--ui-line);border-radius:6px;background:var(--ui-control);color:var(--ui-text-primary);font:11px/1.6 monospace}.code-actions{display:flex;justify-content:flex-end;gap:8px}.element-code p{color:var(--ui-text-secondary);font-size:11px}
</style>
