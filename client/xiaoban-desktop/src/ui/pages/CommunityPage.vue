<script setup lang="ts">
import { nextTick, onActivated, onBeforeUnmount, onDeactivated, ref, watch } from 'vue';
import { NAlert, NButton } from 'naive-ui';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { applyFlow, trackEvent } from '../community/apply.ts';

defineOptions({ name: 'CommunityPage' });
const props = defineProps<{ store: ControlStore }>();
const webHost = ref<HTMLElement | null>(null);
const webError = ref('');

function text(error: unknown) {
  return error instanceof Error ? error.message : typeof error === 'string' ? error : '社区暂时无法访问';
}

let active = false;
let accountChanged = false;
let pending = Promise.resolve();
let observer: ResizeObserver | undefined;

function syncWebsite() {
  pending = pending.then(async () => {
    const rect = active && applyFlow.state.phase === 'idle' ? webHost.value?.getBoundingClientRect() : undefined;
    const bounds = rect && rect.width > 0 && rect.height > 0
      ? [rect.left, rect.top, rect.width, rect.height]
      : null;
    const syncAccount = bounds !== null && accountChanged;
    if (syncAccount) accountChanged = false;
    try {
      await invoke('community_tab_bounds', { bounds, syncAccount });
    } catch (error) {
      if (syncAccount) accountChanged = true;
      throw error;
    }
  }).catch(e => { webError.value = text(e); });
}

watch(webHost, (host, previous) => {
  if (previous) observer?.unobserve(previous);
  if (host && typeof ResizeObserver !== 'undefined') {
    observer ??= new ResizeObserver(syncWebsite);
    observer.observe(host);
  }
  if (props.store.desktop) syncWebsite();
}, { flush: 'post' });
watch(() => props.store.account?.user.uid, () => {
  accountChanged = true;
  if (active) syncWebsite();
});
watch(() => applyFlow.state.phase, () => {
  if (active && props.store.desktop) syncWebsite();
});
onActivated(() => {
  active = props.store.desktop;
  if (!active) return;
  trackEvent('desktop_community_open');
  window.addEventListener('resize', syncWebsite);
  void nextTick(syncWebsite);
});
function deactivate() {
  if (!active) return;
  active = false;
  window.removeEventListener('resize', syncWebsite);
  syncWebsite();
}
onDeactivated(deactivate);
onBeforeUnmount(() => {
  deactivate();
  observer?.disconnect();
});
</script>

<template>
  <section class="community-page">
    <NAlert v-if="!store.desktop" type="info" :show-icon="false">社区内容需要在桌面应用中浏览。</NAlert>
    <NAlert v-else-if="webError" type="warning" :show-icon="false">
      {{ webError }}
      <NButton size="tiny" @click="webError = ''">重试</NButton>
    </NAlert>
    <div v-else ref="webHost" class="community-web-host" aria-label="小伴社区网站" />
  </section>
</template>

<style scoped>
.community-page,
.community-web-host {
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
}
</style>
