<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ name: string; src?: string | null; size?: number }>()

/** 没有头像时用昵称首字，黑白灰底色跟随当前主题。 */
const initial = computed(() => Array.from(props.name.trim())[0] ?? '?')
</script>

<template>
  <span class="avatar" :style="{ width: `${size ?? 32}px`, height: `${size ?? 32}px` }" aria-hidden="true">
    <img v-if="src" :src="src" alt="" loading="lazy" />
    <span v-else :style="{ fontSize: `${Math.round((size ?? 32) * 0.46)}px` }">{{ initial }}</span>
  </span>
</template>

<style scoped>
.avatar {
  flex: none;
  display: inline-grid;
  place-items: center;
  border-radius: 50%;
  overflow: hidden;
  background: var(--surface-2);
  color: var(--ink-2);
  border: 1px solid var(--line);
  font-weight: 700;
  line-height: 1;
}
.avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
</style>
