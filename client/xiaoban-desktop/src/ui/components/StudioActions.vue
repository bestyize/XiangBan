<script setup lang="ts">
/**
 * Puts a studio page's header buttons (导入 / 让小伴创作 …) into StudioPage's `#studio-actions` slot, next to the large tabs.
 * Pages live under <KeepAlive>, so the buttons leave while the page is inactive. Without that target
 * (the page mounted on its own, e.g. in tests) they simply render in place.
 */
import { onActivated, onDeactivated, onMounted, ref } from 'vue';
const target = ref<HTMLElement | null>(typeof document === 'undefined' ? null : document.getElementById('studio-actions'));
const shown = ref(true);
onMounted(() => { target.value = document.getElementById('studio-actions'); });
onActivated(() => { target.value = document.getElementById('studio-actions'); shown.value = true; });
onDeactivated(() => { shown.value = false; });
</script>
<template>
  <Teleport v-if="target && shown" :to="target"><slot /></Teleport>
  <slot v-else-if="!target" />
</template>
