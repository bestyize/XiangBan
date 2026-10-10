<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { EffectsLayer } from '../../effects/layer.ts';
import type { EffectPack } from '../../effects/types.ts';
const props = defineProps<{ pack: EffectPack; paused?: boolean; fps?: 30 | 60 }>();
const host = ref<HTMLElement>();
let layer: EffectsLayer | undefined;
/** A card is small and its particles are counted by area, so the preview is denser and a little faster; otherwise an effect that comes in turns, such as meteors, would often look empty. */
const params = { density: 2, speed: 1.4, size: 1 };
const config = () => ({ paused: props.paused ?? false, fps: props.fps ?? 60 });
// Sky colours were checked as hex when the pack was read, so they are safe to put into CSS.
const sky = computed(() => `linear-gradient(${props.pack.sky.join(',')})`);
onMounted(() => { if (host.value) layer = new EffectsLayer(host.value, props.pack, params, config()); });
watch(() => [props.pack, props.paused, props.fps] as const, () => layer?.update(props.pack, params, config()));
onBeforeUnmount(() => { layer?.destroy(); layer = undefined; });
</script>
<template><div ref="host" class="effect-preview" :style="{ background: sky }" aria-hidden="true" /></template>
<style scoped>
.effect-preview{width:100%;height:100%}
</style>
