<script setup lang="ts">
import Icon, { type IconName } from './Icon.vue';

withDefaults(defineProps<{ icon?: IconName; title: string; desc?: string }>(), { icon: 'layers' });
</script>

<template>
  <div class="empty">
    <span class="badge"><Icon :name="icon" :size="21" /></span>
    <h3>{{ title }}</h3>
    <p v-if="desc">{{ desc }}</p>
    <div v-if="$slots.default" class="act"><slot /></div>
  </div>
</template>

<style scoped>
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 5px;
  padding: 44px 24px;
}
.badge {
  display: grid;
  place-items: center;
  width: 46px;
  height: 46px;
  border-radius: 13px;
  background: var(--canvas-soft);
  color: var(--ink-2);
  margin-bottom: 6px;
}
h3 {
  font-size: 15px;
  font-weight: 680;
}
p {
  color: var(--ink-3);
  font-size: 13px;
  max-width: 46ch;
}
.act {
  margin-top: 12px;
}
</style>
