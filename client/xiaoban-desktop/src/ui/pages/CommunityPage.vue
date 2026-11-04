<script setup lang="ts">
import { nextTick, onMounted, onActivated, onBeforeUnmount, onDeactivated, ref, watch } from 'vue';
import ChatPage from './ChatPage.vue';
import CommunityMediaAssistant from '../community/CommunityMediaAssistant.vue';
import { NAlert, NButton } from 'naive-ui';
import { invoke, listen } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { applyFlow, trackEvent } from '../community/apply.ts';

defineOptions({ name: 'CommunityPage' });
const props = defineProps<{ store: ControlStore }>();
const webHost = ref<HTMLElement | null>(null);
const webError = ref('');
const assistantOpen = ref(false);
const mediaPage = ref(false);

let stopOpenChat: (() => void) | undefined;
let disposed = false;

onMounted(async () => {
  if (!props.store.desktop) return;
  const stop = await listen('community-open-chat', () => {
    if (!active) return;
    assistantOpen.value = true;
    mediaPage.value = false;
    void invoke<boolean>('community_media_page').then(recognized => {
      if (active && assistantOpen.value) mediaPage.value = recognized;
    }).catch(() => {});
  });
  if (disposed) stop();
  else stopOpenChat = stop;
});

function closeAssistant() {
  if (!assistantOpen.value) return;
  assistantOpen.value = false;
  mediaPage.value = false;
  props.store.live?.end();
  props.store.asr.deactivate();
  void props.store.speech.stop();
}

function text(error: unknown) {
  return error instanceof Error ? error.message : typeof error === 'string' ? error : '社区暂时无法访问';
}

let active = false;
let accountChanged = false;
let pending = Promise.resolve();
let observer: ResizeObserver | undefined;

function syncWebsite() {
  pending = pending.then(async () => {
    const rect = active && props.store.page === 'community' && applyFlow.state.phase === 'idle' ? webHost.value?.getBoundingClientRect() : undefined;
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
  closeAssistant();
  window.removeEventListener('resize', syncWebsite);
  syncWebsite();
}
onDeactivated(deactivate);
onBeforeUnmount(() => {
  disposed = true;
  stopOpenChat?.();
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
    <template v-else>
      <div ref="webHost" class="community-web-host" aria-label="小伴社区网站" />
      <div v-if="assistantOpen" class="community-assistant-dock">
        <CommunityMediaAssistant v-if="mediaPage" recognized />
        <ChatPage :store="store" embedded @close="closeAssistant" />
      </div>
    </template>
  </section>
</template>

<style scoped>
.community-page {
  display: flex;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
}
.community-web-host { flex: 1; min-width: 0; }
.community-assistant-dock {
  display: flex;
  flex: 0 0 min(440px, 55%);
  flex-direction: column;
  align-items: stretch;
  min-width: 0;
  box-sizing: border-box;
  border-inline-start: 1px solid var(--ui-line);
}
</style>
