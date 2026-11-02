<script setup lang="ts">
defineProps<{ items: readonly { key: string; label: string }[]; active: string; label: string }>()
defineEmits<{ change: [key: string] }>()
</script>
<template><nav class="community-tabs" :aria-label="label"><button v-for="item in items" :key="item.key" type="button" :class="{ on: active === item.key }" :aria-current="active === item.key ? 'page' : undefined" @click="$emit('change', item.key)">{{ item.label }}</button><div v-if="$slots.actions" class="community-tab-actions"><slot name="actions" /></div></nav></template>
<style scoped>
.community-tabs{display:flex;justify-content:safe center;gap:clamp(14px,4vw,24px);overflow-x:auto;padding:0 12px;margin-bottom:12px;background:var(--surface);scrollbar-width:none;border-bottom:1px solid color-mix(in srgb,var(--line) 65%,transparent)}
button{position:relative;flex:none;min-height:44px;padding:8px 0 10px;border:0;background:none;color:var(--ink-2);font-size:14px;cursor:pointer}button.on{color:var(--ink);font-weight:650}button.on:after{content:'';position:absolute;left:50%;bottom:3px;transform:translateX(-50%);height:3px;width:18px;border-radius:2px;background:var(--brand)}
.community-tab-actions{display:flex;align-items:center;gap:12px;margin-left:auto;flex:none}
@media(min-width:1100px){
.community-tabs{justify-content:flex-start;gap:28px;background:none;padding:0 12px;margin-bottom:18px;border-bottom:1px solid var(--line-strong)}button{font-size:15px;min-height:48px;padding-bottom:14px}button.on:after{bottom:0;width:24px}
}
</style>
