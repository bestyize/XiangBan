<script setup lang="ts">
import Icon, { type IconName } from './Icon.vue';

withDefaults(defineProps<{ icon?: IconName; title: string; desc?: string }>(), { icon: 'layers' });
</script>

<template>
  <div class="empty">
    <span class="badge"><Icon :name="icon" :size="22" /></span>
    <h3>{{ title }}</h3>
    <p v-if="desc">{{ desc }}</p>
    <div v-if="$slots.default" class="act"><slot /></div>
  </div>
</template>

<style scoped>
.empty { display: flex; flex-direction: column; align-items: center; text-align: center; gap: 6px; padding: 48px 24px; }
.badge { display: grid; place-items: center; width: 52px; height: 52px; border-radius: 16px; background: var(--tide-soft); color: var(--tide); margin-bottom: 8px; }
h3 { font-size: 15.5px; }
p { color: var(--muted); max-width: 46ch; }
.act { margin-top: 12px; }
</style>
