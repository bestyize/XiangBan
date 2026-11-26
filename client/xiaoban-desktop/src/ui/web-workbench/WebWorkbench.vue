<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { NConfigProvider, darkTheme } from 'naive-ui';
import { createControlStore } from '../control-store.ts';
import ChatWorkbench from '../pages/ChatWorkbench.vue';
import { applyUiTokens } from '../design-tokens.ts';
import { accents, DEFAULT_ACCENT, type AccentId } from '../appearance.ts';
import { themeFor } from '../theme.ts';
import { createCloudChat, type CloudRequest, type WebTier } from './cloud-chat.ts';
import '../control-utilities.css';

const props = defineProps<{ owner: string; request: CloudRequest; tiers: WebTier[] }>();
const box = ref<HTMLElement>();
const dark = ref(false);
const accent = ref<AccentId>(DEFAULT_ACCENT);
const notice = ref('');
const notify = (message: unknown) => { notice.value = message instanceof Error ? message.message : String(message); };
const cloud = createCloudChat({ owner: props.owner, request: props.request, notify });
const { chat, state } = cloud;
const store = createControlStore(notify, { web: { chat, state } });
store.page = 'chat';
const theme = computed(() => themeFor(dark.value, accent.value));
watch(() => props.tiers, tiers => {
  state.tiers = tiers;
  if (!tiers.some(t => t.id === state.tier)) state.tier = (tiers.find(t => t.default) ?? tiers[0])?.id ?? '';
  chat.configured = Boolean(state.tier);
}, { immediate: true });
function applyTheme() {
  dark.value = document.documentElement.dataset.theme === 'dark';
  accent.value = accents.find(a => a.id === document.documentElement.dataset.accent)?.id ?? DEFAULT_ACCENT;
  if (box.value) applyUiTokens(dark.value, accent.value, box.value);
}
function select(event: Event) { void chat.select((event.target as HTMLSelectElement).value || null); }
function refresh() { void chat.refresh(); }
function retry() { refresh(); cloud.reconnect(); }
const observer = new MutationObserver(applyTheme);
onMounted(() => {
  applyTheme(); observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme', 'data-accent'] });
  window.addEventListener('focus', refresh);
});
onBeforeUnmount(() => { observer.disconnect(); window.removeEventListener('focus', refresh); chat.dispose(); });
</script>
<template>
  <section ref="box" class="web-workbench" aria-label="小伴对话工作台">
    <NConfigProvider abstract :theme="dark ? darkTheme : null" :theme-overrides="theme">
      <nav class="sessions" aria-label="对话历史">
        <select :value="chat.currentId ?? ''" aria-label="选择对话" @change="select">
          <option value="">新对话</option>
          <option v-for="item in chat.sessions" :key="item.id" :value="item.id">{{ item.title }}</option>
        </select>
        <button type="button" class="xb-btn" @click="chat.select(null)">新建对话</button>
      </nav>
      <p v-if="chat.loadError || state.connectionError" class="feedback" role="status">
        {{ chat.loadError || state.connectionError }} <button class="xb-link" @click="retry">重新连接</button>
      </p>
      <p v-if="notice" class="feedback" role="status">{{ notice }} <button class="xb-link" @click="notice = ''">关闭</button></p>
      <ChatWorkbench :store="store" />
    </NConfigProvider>
  </section>
</template>
<style scoped>
.web-workbench{display:flex;flex-direction:column;position:relative;isolation:isolate;min-width:0;height:clamp(360px,calc(100dvh - 166px),980px);background:var(--ui-surface);color:var(--ui-text-primary);border-radius:18px;overflow:hidden;font:14px/1.5 var(--ui-font-family);--ui-text:var(--ui-text-primary);--ui-muted:var(--ui-text-secondary)}
.web-workbench :deep(*){box-sizing:border-box}
.web-workbench :deep(.chat-page){flex:1;min-height:0}
.sessions{display:flex;justify-content:space-between;align-items:center;gap:12px;padding:10px 18px;background:var(--ui-surface-alt)}
.sessions select{min-width:0;max-width:70%;border:0;background:transparent;color:var(--ui-text-primary);font:inherit;cursor:pointer}
.feedback{margin:0;padding:7px 16px;color:var(--ui-text-secondary)}
@media(max-width:650px){.web-workbench{height:calc(100dvh - 170px);min-height:310px}.sessions{padding:7px 12px}}
</style>
