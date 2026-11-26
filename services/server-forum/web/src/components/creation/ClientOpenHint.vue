<script setup lang="ts">
defineProps<{ download: string }>()
defineEmits<{ dismiss: [] }>()
</script>

<template>
  <div class="client-hint">
    <span role="status">没有打开小伴？</span>
    <a :href="download" target="_blank" rel="noopener noreferrer">下载客户端</a>
    <button type="button" @click="$emit('dismiss')">关闭</button>
  </div>
</template>

<style scoped>
.client-hint { position:absolute; z-index:5; inset:auto 12px max(16px, env(safe-area-inset-bottom)) auto; display:flex; align-items:center; gap:10px; padding:6px 12px; border:0; border-radius:10px; background:#292929; color:white; box-shadow:0 5px 20px #0003; font-size:13px; }
.client-hint a,.client-hint button { display:inline-flex; align-items:center; min-height:32px; padding:0 3px; border:0; background:transparent; color:white; font:inherit; cursor:pointer; }
.client-hint a { text-decoration:underline; text-underline-offset:3px; }
.client-hint :is(a,button):focus-visible { outline:0; background:#fff; color:#222; }
</style>
