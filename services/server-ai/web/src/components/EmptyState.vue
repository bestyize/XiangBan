<script setup lang="ts">
import Icon, { type IconName } from './Icon.vue';

withDefaults(defineProps<{ icon?: IconName; title: string; desc?: string }>(), { icon: 'layers' });
</script>

<template>
  <div class="empty">
    <span class="badge"><Icon :name="icon" :size="20" /></span>
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
  padding: 40px 24px;
}
.badge {
  display: grid;
  place-items: center;
  width: 44px;
  height: 44px;
  border-radius: var(--r-md);
  background: var(--subtle);
  color: var(--ink-2);
  margin-bottom: 6px;
}
h3 {
  font-size: 15px;
  font-weight: 650;
}
p {
  color: var(--ink-3);
  font-size: 12.5px;
  max-width: 44ch;
}
.act {
  margin-top: 12px;
}
</style>
