<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{ seed: string; size?: number }>(), { size: 34 });

// 同一个账号永远是同一种颜色，一眼能在列表里认出来；饱和度压低，不抢内容的风头。
const hue = computed(() => [...props.seed].reduce((h, c) => (h * 31 + c.charCodeAt(0)) % 360, 7));
const initials = computed(() => props.seed.replace(/[^0-9a-z]/gi, '').slice(0, 2).toUpperCase() || '?');
</script>

<template>
  <span
    class="avatar"
    :style="{ width: `${size}px`, height: `${size}px`, fontSize: `${Math.round(size * 0.37)}px`, background: `hsl(${hue} 42% 91%)`, color: `hsl(${hue} 48% 26%)` }"
    aria-hidden="true"
  >{{ initials }}</span>
</template>

<style scoped>
.avatar { display: inline-grid; place-items: center; border-radius: 50%; font-weight: 700; letter-spacing: 0.02em; flex: none; user-select: none; }
</style>
