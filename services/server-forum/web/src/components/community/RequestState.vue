<script setup lang="ts">
import AppIcon from '@/components/AppIcon.vue'
defineProps<{ loading?: boolean; error?: string; empty?: boolean; message?: string }>()
defineEmits<{ retry: [] }>()
</script>
<template><div v-if="loading || error || empty" class="request-state" :class="{loading}" :role="error ? 'alert' : 'status'" :aria-busy="loading || undefined">
  <template v-if="loading"><div class="loading-lines" aria-hidden="true"><i/><i/><i/></div><span>正在加载…</span></template>
  <template v-else><span class="state-icon" aria-hidden="true"><AppIcon :name="error ? 'chat' : 'folder'" :size="24"/></span><p>{{ error || message || '这里还没有已发布内容。' }}</p><button v-if="error" type="button" @click="$emit('retry')">重新加载</button></template>
</div></template>
<style scoped>
.request-state{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;min-height:180px;padding:28px 20px;margin:10px 0;text-align:center;color:var(--ink-3);font-size:14px;line-height:1.7;background:var(--surface);border-radius:16px}.state-icon{display:grid;place-items:center;width:48px;height:48px;border-radius:16px;background:var(--surface-2);color:var(--ink-3)}p{margin:0;max-width:34em;overflow-wrap:anywhere}button{min-height:44px;padding:8px 22px;border:1px solid var(--line);border-radius:22px;background:var(--surface-2);color:var(--ink);font-size:13px;font-weight:550;cursor:pointer}.loading-lines{display:grid;gap:10px;width:min(280px,85%)}.loading-lines i{display:block;height:12px;border-radius:6px;background:var(--surface-2);animation:pulse 1.5s ease-in-out infinite}.loading-lines i:nth-child(2){width:88%;animation-delay:.15s}.loading-lines i:last-child{width:64%;animation-delay:.3s}@keyframes pulse{50%{opacity:.45}}@media(prefers-reduced-motion:reduce){.loading-lines i{animation:none}}
</style>
