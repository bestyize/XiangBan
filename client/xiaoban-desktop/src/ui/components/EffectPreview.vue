<script setup lang="ts">
import { computed, onActivated, onBeforeUnmount, onDeactivated, onMounted, ref, watch } from 'vue';
import { EffectsLayer } from '../../effects/layer.ts';
import { arowanaCover, isArowana, type EffectPack, type EffectParams } from '../../effects/types.ts';
import { previewParams, subscribeEffectStill } from './effect-stills.ts';

const props = defineProps<{ pack: EffectPack; playing?: boolean; paused?: boolean; fps?: 30 | 60; tuning?: EffectParams }>();
const host = ref<HTMLElement>();
const stillUrl = ref('');
let layer: EffectsLayer | undefined;
let active = true;
let mounted = false;
let unsubscribe: (() => void) | undefined;
const sky = computed(() => `linear-gradient(${props.pack.sky.join(',')})`);

function destroyLayer() { layer?.destroy(); layer = undefined; }
function syncLayer() {
  if (!mounted || !active || !props.playing || (props.paused && !props.tuning) || !host.value) { destroyLayer(); return; }
  const config = { paused: props.paused ?? false, fps: props.fps ?? 60 };
  const params = props.tuning ?? previewParams;
  if (layer) layer.update(props.pack, params, config);
  else layer = new EffectsLayer(host.value, props.pack, params, config);
}
function loadStill() {
  unsubscribe?.(); unsubscribe = undefined;
  if (!active) return;
  if (isArowana(props.pack)) stillUrl.value = arowanaCover;
  else unsubscribe = subscribeEffectStill(props.pack, url => { stillUrl.value = url; });
}

onMounted(() => { mounted = true; loadStill(); syncLayer(); });
watch(() => props.pack, () => { stillUrl.value = ''; destroyLayer(); loadStill(); syncLayer(); }, { deep: true });
watch(() => [props.playing, props.paused, props.fps, props.tuning], syncLayer);
onActivated(() => {
  // The first activation follows onMounted; don't create/update a layer twice.
  if (active) return;
  active = true; loadStill(); syncLayer();
});
onDeactivated(() => {
  active = false; unsubscribe?.(); unsubscribe = undefined; destroyLayer();
});
onBeforeUnmount(() => {
  mounted = false; active = false; unsubscribe?.(); destroyLayer();
});
</script>
<template>
  <div ref="host" class="effect-preview" :style="{ background: sky }" aria-hidden="true">
    <img v-if="stillUrl" class="effect-still" :src="stillUrl" alt="" draggable="false" />
  </div>
</template>
<style scoped>
.effect-preview{position:relative;width:100%;height:100%}
.effect-still{display:block;position:absolute;inset:0;width:100%;height:100%;object-fit:fill;pointer-events:none}
.effects-host:not(.effects-loading) .effect-still{visibility:hidden}
</style>
