<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { MessageCircleMore } from 'lucide-vue-next';

const props = defineProps<{ disabled?: boolean; busy?: boolean; nearComposer?: boolean }>();
const emit = defineEmits<{ open: [] }>();
const button = ref<HTMLButtonElement>();
const position = ref<{ x: number; y: number }>();
const dragging = ref(false);
const style = computed(() => position.value ? { left: `${position.value.x}px`, top: `${position.value.y}px`, right: 'auto', bottom: 'auto' } : undefined);
let gesture: { id: number; x: number; y: number; left: number; top: number } | undefined;
let suppressClick = false;
let observer: ResizeObserver | undefined;

function clamp(x: number, y: number) {
  const element = button.value!, bounds = element.parentElement!.getBoundingClientRect();
  const rect = element.getBoundingClientRect();
  const limit = (value: number, available: number) => {
    const max = Math.max(0, available), inset = Math.min(12, max / 2);
    return Math.max(inset, Math.min(max - inset, value));
  };
  return { x: limit(x, bounds.width - rect.width), y: limit(y, bounds.height - rect.height) };
}
function fit() { if (position.value) position.value = clamp(position.value.x, position.value.y); }
function down(event: PointerEvent) {
  if (props.disabled || props.busy || event.button !== 0 || gesture) return;
  const element = button.value!, rect = element.getBoundingClientRect(), parent = element.parentElement!.getBoundingClientRect();
  gesture = { id: event.pointerId, x: event.clientX, y: event.clientY, left: rect.left - parent.left, top: rect.top - parent.top };
  suppressClick = false;
  element.setPointerCapture(event.pointerId);
}
function move(event: PointerEvent) {
  if (!gesture || event.pointerId !== gesture.id) return;
  const dx = event.clientX - gesture.x, dy = event.clientY - gesture.y;
  if (!dragging.value && Math.hypot(dx, dy) <= 5) return;
  dragging.value = true; suppressClick = true;
  event.preventDefault();
  position.value = clamp(gesture.left + dx, gesture.top + dy);
}
function end(event: PointerEvent) {
  if (!gesture || event.pointerId !== gesture.id) return;
  if (event.type === 'pointercancel') suppressClick = true;
  const id = gesture.id;
  gesture = undefined; dragging.value = false;
  if (button.value?.hasPointerCapture(id)) button.value.releasePointerCapture(id);
}
function click() {
  if (suppressClick) { suppressClick = false; return; }
  if (!props.disabled && !props.busy) emit('open');
}
function keydown(event: KeyboardEvent) { if (event.key === 'Enter' || event.key === ' ') suppressClick = false; }
onMounted(() => {
  observer = new ResizeObserver(fit);
  observer.observe(button.value!.parentElement!);
});
onBeforeUnmount(() => observer?.disconnect());
</script>

<template>
  <button ref="button" type="button" class="assistant-launcher" :class="{ dragging, 'near-composer': nearComposer }" :style="style" data-testid="open-assistant" aria-label="打开超级小伴对话框" title="超级小伴 · 点击对话，按住拖动" :disabled="disabled || busy" :aria-busy="busy" @pointerdown="down" @pointermove="move" @pointerup="end" @pointercancel="end" @lostpointercapture="end" @keydown="keydown" @click="click">
    <MessageCircleMore :size="26" :stroke-width="1.8" aria-hidden="true" />
  </button>
</template>

<style scoped>
.assistant-launcher{position:absolute;right:24px;bottom:48px;z-index:30;display:grid;place-items:center;width:54px;height:54px;padding:0;border:1px solid #ffffff30;border-radius:18px;background:var(--ui-primary);color:#fff;box-shadow:0 5px 18px var(--ui-shadow-hover);cursor:grab;touch-action:none;user-select:none;transition:background-color .15s,box-shadow .15s}
.assistant-launcher.near-composer{bottom:188px}
.assistant-launcher:hover:not(:disabled){background:var(--ui-primary-hover);box-shadow:0 7px 22px var(--ui-shadow-pop)}
.assistant-launcher.dragging{cursor:grabbing}
.assistant-launcher:focus-visible{outline:3px solid var(--ui-primary-soft);outline-offset:4px}
.assistant-launcher:disabled{opacity:.5;cursor:default}
@media(prefers-reduced-motion:reduce){.assistant-launcher{transition:none}}
</style>
