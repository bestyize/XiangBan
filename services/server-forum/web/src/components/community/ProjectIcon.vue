<script setup lang="ts">
import { ref, watch } from 'vue'
const props = defineProps<{ name: string; src?: string; fit?: string }>()
const failed = ref(false)
watch(() => [props.name, props.src], () => { failed.value = false })
</script>
<template>
  <span class="project-icon" aria-hidden="true">
    <img v-if="src && !failed" :src="src" :class="{ mark: fit === 'mark-left' }" alt="" loading="lazy" @error="failed = true">
    <span v-else>{{ name.slice(0, 1).toUpperCase() }}</span>
  </span>
</template>
<style scoped>
.project-icon { display:grid;place-items:center;flex:none;width:40px;height:40px;overflow:hidden;border-radius:10px;background:var(--surface-2);color:var(--ink-2);font-size:18px; }
img { width:100%;height:100%;object-fit:contain; }
img.mark { object-fit:cover;object-position:left; }
</style>
