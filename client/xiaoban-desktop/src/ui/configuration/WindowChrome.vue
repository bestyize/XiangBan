<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watchEffect } from 'vue';
import { Maximize2, Minimize2, Minus, X } from 'lucide-vue-next';
import { desktop, invoke, listen } from '../../bridge.ts';
import { closeConfigurationWindow } from './windows.ts';
const props = defineProps<{ title?: string; platform?: string }>();
const state = ref({ maximized: false, fullscreen: false, focused: true });
watchEffect(() => { if (desktop) document.documentElement.classList.toggle('configuration-expanded', state.value.maximized || state.value.fullscreen); });
const error = ref('');
const mac = computed(() => props.platform ? props.platform === 'macos' : /Mac/i.test(navigator.platform));
const expandLabel = computed(() => state.value.fullscreen ? '退出全屏' : mac.value ? '全屏' : state.value.maximized ? '还原窗口' : '最大化');
let disposed = false, off: (() => void) | undefined;
async function act(action: string) {
  if (!desktop) return;
  try {
    const next = await invoke<typeof state.value>('configuration_window_action', { action });
    if (!disposed && next) { state.value = next; error.value = ''; }
  } catch (e) { if (!disposed) error.value = String(e instanceof Error ? e.message : e); }
}
function drag(event: MouseEvent) {
  if (event.button !== 0 || event.detail > 1 || state.value.fullscreen || (event.target as HTMLElement).closest('button')) return;
  event.preventDefault(); void act('drag');
}
function zoom(event: MouseEvent) {
  if ((event.target as HTMLElement).closest('button') || state.value.fullscreen) return;
  void act('double-click');
}
function expand(event: MouseEvent) { void act(state.value.fullscreen || (mac.value && !event.altKey) ? 'fullscreen' : 'maximize'); }
function close() { if (desktop) void closeConfigurationWindow().catch(e => { error.value = String(e); }); }
function shortcut(event: KeyboardEvent) {
  if (event.defaultPrevented || !desktop) return;
  if (event.key === 'Escape' && state.value.fullscreen) {
    event.preventDefault(); event.stopImmediatePropagation(); void act('fullscreen');
  } else if ((mac.value ? event.metaKey : event.ctrlKey) && event.key.toLowerCase() === 'w') {
    event.preventDefault(); event.stopImmediatePropagation(); close();
  } else if (mac.value && event.metaKey && event.key.toLowerCase() === 'm') {
    event.preventDefault(); void act('minimize');
  } else if (!mac.value && event.key === 'F11') {
    event.preventDefault(); void act('fullscreen');
  }
}
onMounted(async () => {
  window.addEventListener('keydown', shortcut, true);
  if (!desktop) return;
  const stop = await listen<typeof state.value>('configuration-window-state', next => { state.value = next; });
  if (disposed) { stop(); return; } off = stop;
  await act('state');
});
onBeforeUnmount(() => { disposed = true; off?.(); window.removeEventListener('keydown', shortcut, true); });
</script>
<template>
  <header class="configuration-chrome" :class="{ 'is-inactive': !state.focused }" :title="error || undefined" aria-label="窗口工具栏" data-testid="window-chrome" @mousedown="drag" @dblclick="zoom">
    <div class="caption-controls" role="group" aria-label="窗口控制">
      <button type="button" class="caption-close" aria-label="关闭窗口" title="关闭窗口" data-testid="window-close" @click="close"><X :size="9" :stroke-width="2.6" aria-hidden="true" /></button>
      <button type="button" class="caption-minimize" aria-label="最小化" title="最小化" :disabled="state.fullscreen" data-testid="window-minimize" @click="act('minimize')"><Minus :size="9" :stroke-width="2.6" aria-hidden="true" /></button>
      <button type="button" class="caption-expand" :aria-label="expandLabel" :title="mac ? `${expandLabel}（Option 点击缩放）` : expandLabel" :aria-pressed="state.fullscreen || state.maximized" data-testid="window-expand" @click="expand"><Minimize2 v-if="state.fullscreen || state.maximized" :size="8" :stroke-width="2.5" aria-hidden="true" /><Maximize2 v-else :size="8" :stroke-width="2.5" aria-hidden="true" /></button>
    </div>
    <span v-if="title" class="chrome-title">{{ title }}</span>
  </header>
</template>
<style scoped>
.configuration-chrome{position:relative;display:flex;align-items:center;gap:18px;flex-shrink:0;min-height:46px;padding:0 20px;user-select:none;-webkit-user-select:none;cursor:default;color:var(--ui-text-secondary)}
.caption-controls{display:flex;align-items:center;gap:2px;flex-shrink:0;margin-left:-5px}
.caption-controls button{display:grid;place-items:center;position:relative;width:22px;height:30px;border:0;padding:0;border-radius:5px;background:transparent;color:var(--ui-text-primary);cursor:default;box-shadow:none}
.caption-controls button::before{content:'';position:absolute;width:12px;height:12px;border-radius:50%;background:var(--caption-color)}
.caption-controls .caption-close{--caption-color:var(--ui-window-close)}.caption-controls .caption-minimize{--caption-color:var(--ui-window-minimize)}.caption-controls .caption-expand{--caption-color:var(--ui-window-expand)}
.caption-controls button svg{position:relative;opacity:0;color:var(--ui-window-symbol)}.caption-controls:hover button:not(:disabled) svg,.caption-controls button:focus-visible svg{opacity:1}
.caption-controls button:focus-visible{outline:2px solid var(--ui-btn);outline-offset:1px}
.caption-controls button:active::before{filter:brightness(.88)}.is-inactive .caption-controls:not(:hover) button::before,.caption-controls button:disabled::before{background:var(--ui-text-disabled);opacity:.5}
.chrome-title{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px;font-weight:500;pointer-events:none}
</style>
