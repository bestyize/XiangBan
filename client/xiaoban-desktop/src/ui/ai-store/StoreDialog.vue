<script setup lang="ts">
import { watch } from 'vue';
import { NModal } from 'naive-ui';
import { closeConfigurationWindow } from '../configuration/windows.ts';
import { storeWindowScene } from './store-window.ts';
defineOptions({ inheritAttrs: false });
const props = withDefaults(defineProps<{ show: boolean; title?: string; closable?: boolean; maskClosable?: boolean; hideTitle?: boolean }>(), { closable: true, maskClosable: true });
const emit = defineEmits<{ 'update:show': [value: boolean] }>();
const detached = storeWindowScene();
let opened = props.show;
watch(() => props.show, show => {
  if (show) opened = true;
  else if (detached && opened) void closeConfigurationWindow();
});
</script>
<template>
  <section v-if="detached && show" v-bind="$attrs" class="store-detached-dialog" aria-label="插件详情">
    <header v-if="!hideTitle" class="store-detail-title"><h1>{{ title }}</h1></header>
    <div class="store-detail-content"><slot /></div>
  </section>
  <NModal v-else-if="!detached" v-bind="$attrs" :show="show" preset="card" :title="hideTitle ? '' : title" :closable="closable" :mask-closable="maskClosable" @update:show="emit('update:show', $event)"><slot /></NModal>
</template>
<style>
.store-detached-dialog{display:flex;flex-direction:column;width:100%!important;max-width:none!important;height:100%;min-height:0;background:var(--ui-surface)}
.store-detail-title{padding:14px 28px 20px;flex:none}.store-detail-title h1{margin:0;font-size:22px;font-weight:600;letter-spacing:-.025em;color:var(--ui-text-primary)}
.store-detail-content{padding:0 28px 28px;flex:1;min-height:0;overflow:auto;overscroll-behavior:none}
@media(max-width:500px){.store-detail-title{padding:12px 18px 18px}.store-detail-content{padding:0 18px 18px}}
</style>
