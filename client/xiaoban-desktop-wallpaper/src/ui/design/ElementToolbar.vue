<script setup lang="ts">
import {Copy,Trash2,Group,Lock,Type,MousePointer2} from 'lucide-vue-next';
defineProps<{count:number;disabled:boolean;readonly?:boolean}>();
const emit=defineEmits<{command:[command:string]}>();
</script>
<template><div class="element-toolbar" role="toolbar" aria-label="选中元素操作">
<span>{{count}} 个元素</span>
<button :disabled="disabled||readonly" title="编辑文字 (Enter)" aria-label="编辑文字" @click="emit('command','editText')"><Type :size="14"/></button>
<button :disabled="disabled||readonly" title="复制元素 (⌘/Ctrl+D)" aria-label="复制元素" @click="emit('command','duplicate')"><Copy :size="14"/></button>
<button :disabled="disabled||readonly||count<2" title="编组 (⌘/Ctrl+G)" aria-label="编组元素" @click="emit('command','group')"><Group :size="14"/></button>
<select :disabled="disabled||readonly" aria-label="对齐元素" value="" @change="emit('command','align:'+($event.target as HTMLSelectElement).value);($event.target as HTMLSelectElement).value=''">
<option value="" disabled>对齐 / 分布</option><option value="left">左对齐</option><option value="center-x">水平居中</option><option value="right">右对齐</option><option value="top">顶对齐</option><option value="center-y">垂直居中</option><option value="bottom">底对齐</option><option value="distribute-x">水平等距</option><option value="distribute-y">垂直等距</option></select>
<button :disabled="disabled||readonly" title="锁定元素" aria-label="锁定元素" @click="emit('command','lock')"><Lock :size="14"/></button>
<button :disabled="disabled||readonly" title="删除元素 (Delete)" aria-label="删除元素" @click="emit('command','delete')"><Trash2 :size="14"/></button>
<button title="取消选择 (Esc)" aria-label="取消选择" @click="emit('command','clear')"><MousePointer2 :size="14"/></button>
</div></template>
<style scoped>.element-toolbar{position:absolute;top:8px;left:144px;z-index:8;display:flex;align-items:center;gap:4px;padding:4px 6px;border:1px solid var(--ui-line);border-radius:8px;background:var(--ui-surface);box-shadow:var(--ui-shadow-menu);font-size:10px;max-width:calc(100% - 156px);overflow-x:auto}.element-toolbar span{white-space:nowrap;color:var(--ui-text-secondary)}.element-toolbar select{font:inherit;color:inherit;background:var(--ui-control);border:1px solid var(--ui-line);border-radius:5px;max-width:100px}.element-toolbar button{display:inline-flex;align-items:center;justify-content:center;flex:none;border:0;background:none;color:inherit;font:inherit;min-width:25px;height:25px;padding:4px;border-radius:5px;cursor:pointer}.element-toolbar button:hover{background:var(--ui-control)}.element-toolbar button:disabled{opacity:.35;cursor:default}</style>
