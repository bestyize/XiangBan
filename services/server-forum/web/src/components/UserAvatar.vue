<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ name: string; src?: string | null; size?: number }>()

/** 没有头像时用昵称首字，底色按昵称稳定取一个。 */
const hue = computed(() => {
  let h = 0
  for (const ch of props.name) h = (h * 31 + ch.charCodeAt(0)) % 360
  return h
})
const initial = computed(() => Array.from(props.name.trim())[0] ?? '?')
</script>

<template>
  <span class="avatar" :style="{ width: `${size ?? 32}px`, height: `${size ?? 32}px`, '--h': hue }" aria-hidden="true">
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
  background: hsl(var(--h) 42% 88%);
  color: hsl(var(--h) 45% 28%);
  font-weight: 700;
  line-height: 1;
}
@media (prefers-color-scheme: dark) {
  .avatar {
    background: hsl(var(--h) 28% 26%);
    color: hsl(var(--h) 50% 82%);
  }
}
.avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
</style>
