<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { initialOf, tintFor } from '@/lib/format';

const props = withDefaults(defineProps<{ name: string; seed?: string; iconUrl?: string | null; size?: number }>(), { size: 56 });
const failed = ref(false);
watch(() => props.iconUrl, () => (failed.value = false));
const tint = computed(() => tintFor(props.seed || props.name));
const radius = computed(() => `${Math.round(props.size * 0.27)}px`);
</script>

<template>
  <img v-if="iconUrl && !failed" class="avatar" :src="iconUrl" alt="" :width="size" :height="size" :style="{ borderRadius: radius }" loading="lazy" @error="failed = true" />
  <div v-else class="avatar tile" :style="{ width: `${size}px`, height: `${size}px`, borderRadius: radius, background: tint.bg, color: tint.ink, fontSize: `${Math.round(size * 0.4)}px` }" aria-hidden="true">
    {{ initialOf(name) }}
  </div>
</template>

<style scoped>
.avatar { flex: 0 0 auto; box-shadow: inset 0 0 0 1px rgba(20, 20, 20, 0.05); object-fit: cover; background: #fff; }
.tile { display: flex; align-items: center; justify-content: center; font-weight: 600; }
</style>
