<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { PetView } from '../../pet.ts';
import type { Settings } from '../../bridge.ts';
const props = defineProps<{ settings: Settings }>();
const host = ref<HTMLElement>();
let pet: PetView | null = null;
const previewSettings = () => ({ ...props.settings, size: 250, paused: props.settings.paused });
onMounted(() => { if (host.value) pet = new PetView(host.value, previewSettings()); });
watch(() => [props.settings.expression, props.settings.paused, props.settings.theme, props.settings.companionSkin],() => pet?.update(previewSettings()));
onBeforeUnmount(() => { pet?.destroy(); pet = null; });
defineExpose({ poke: () => pet?.poke() });
</script>
<template><div ref="host" class="control-pet-preview" aria-label="小伴互动预览" @pointermove="pet?.look($event.clientX, $event.clientY)" @pointerdown="pet?.poke()"></div></template>
